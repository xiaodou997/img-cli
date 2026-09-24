use serde::Serialize;

pub use yu_engine_api::{EngineDescriptor, EngineProvider, EngineState};

pub const SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityDescriptor {
    pub id: String,
    pub summary: String,
    pub engines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EngineSummary {
    pub total: usize,
    pub ready: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorReport {
    pub version: String,
    pub healthy: bool,
    pub platform: PlatformInfo,
    pub capabilities: usize,
    pub engines: EngineSummary,
}

#[derive(Debug, Clone)]
pub struct RuntimeRegistry {
    capabilities: Vec<CapabilityDescriptor>,
    engines: Vec<EngineDescriptor>,
}

impl RuntimeRegistry {
    pub fn bootstrap() -> Self {
        let capabilities = vec![
            CapabilityDescriptor {
                id: "runtime.doctor".to_owned(),
                summary: "Diagnose YuTool runtime and engine health.".to_owned(),
                engines: vec!["yu-runtime".to_owned()],
            },
            CapabilityDescriptor {
                id: "runtime.capabilities".to_owned(),
                summary: "List capabilities available on this machine.".to_owned(),
                engines: vec!["yu-runtime".to_owned()],
            },
            CapabilityDescriptor {
                id: "engine.list".to_owned(),
                summary: "List known engines and their current state.".to_owned(),
                engines: vec!["yu-runtime".to_owned()],
            },
        ];

        let engines = vec![EngineDescriptor {
            id: "yu-runtime".to_owned(),
            display_name: "YuTool Runtime".to_owned(),
            provider: EngineProvider::BuiltIn,
            state: EngineState::Ready,
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            capabilities: capabilities
                .iter()
                .map(|capability| capability.id.clone())
                .collect(),
        }];

        Self {
            capabilities,
            engines,
        }
    }

    pub fn capabilities(&self) -> &[CapabilityDescriptor] {
        &self.capabilities
    }

    pub fn engines(&self) -> &[EngineDescriptor] {
        &self.engines
    }

    pub fn doctor_report(&self) -> DoctorReport {
        let ready = self
            .engines
            .iter()
            .filter(|engine| engine.state == EngineState::Ready)
            .count();

        DoctorReport {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            healthy: ready == self.engines.len(),
            platform: PlatformInfo {
                os: std::env::consts::OS.to_owned(),
                arch: std::env::consts::ARCH.to_owned(),
            },
            capabilities: self.capabilities.len(),
            engines: EngineSummary {
                total: self.engines.len(),
                ready,
            },
        }
    }
}

impl Default for RuntimeRegistry {
    fn default() -> Self {
        Self::bootstrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_registry_is_healthy() {
        let registry = RuntimeRegistry::bootstrap();
        let report = registry.doctor_report();

        assert!(report.healthy);
        assert_eq!(report.engines.total, 1);
        assert_eq!(report.engines.ready, 1);
    }

    #[test]
    fn bootstrap_registry_exposes_runtime_capabilities() {
        let registry = RuntimeRegistry::bootstrap();
        let ids: Vec<_> = registry
            .capabilities()
            .iter()
            .map(|capability| capability.id.as_str())
            .collect();

        assert_eq!(
            ids,
            vec!["runtime.doctor", "runtime.capabilities", "engine.list"]
        );
    }

    #[test]
    fn runtime_engine_is_built_in_and_ready() {
        let registry = RuntimeRegistry::bootstrap();
        let engine = &registry.engines()[0];

        assert_eq!(engine.id, "yu-runtime");
        assert_eq!(engine.provider, EngineProvider::BuiltIn);
        assert_eq!(engine.state, EngineState::Ready);
    }
}
