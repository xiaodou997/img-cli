use serde::Serialize;
use std::{error::Error, fmt};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveErrorKind {
    UnknownEngine,
    EngineUnavailable,
    EngineIncompatible,
    NoCompatibleEngine,
}

#[derive(Debug, Clone)]
pub struct ResolveError {
    pub kind: ResolveErrorKind,
    pub message: String,
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ResolveError {}

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
            CapabilityDescriptor {
                id: "image.info".to_owned(),
                summary: "Inspect a raster image.".to_owned(),
                engines: vec!["raster-rs".to_owned()],
            },
            CapabilityDescriptor {
                id: "image.resize".to_owned(),
                summary: "Resize a raster image.".to_owned(),
                engines: vec!["raster-rs".to_owned()],
            },
        ];

        let engines = vec![
            EngineDescriptor {
                id: "yu-runtime".to_owned(),
                display_name: "YuTool Runtime".to_owned(),
                provider: EngineProvider::BuiltIn,
                state: EngineState::Ready,
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                capabilities: vec![
                    "runtime.doctor".to_owned(),
                    "runtime.capabilities".to_owned(),
                    "engine.list".to_owned(),
                ],
            },
            EngineDescriptor {
                id: "raster-rs".to_owned(),
                display_name: "YuTool Rust Raster Engine".to_owned(),
                provider: EngineProvider::BuiltIn,
                state: EngineState::Ready,
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                capabilities: vec!["image.info".to_owned(), "image.resize".to_owned()],
            },
        ];

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

    pub fn resolve_engine(
        &self,
        capability: &str,
        requested_engine: Option<&str>,
    ) -> Result<&EngineDescriptor, ResolveError> {
        if let Some(requested_engine) = requested_engine {
            let engine = self
                .engines
                .iter()
                .find(|engine| engine.id == requested_engine)
                .ok_or_else(|| ResolveError {
                    kind: ResolveErrorKind::UnknownEngine,
                    message: format!("unknown engine: {requested_engine}"),
                })?;

            if engine.state != EngineState::Ready {
                return Err(ResolveError {
                    kind: ResolveErrorKind::EngineUnavailable,
                    message: format!(
                        "engine {} is not ready (state: {})",
                        engine.id, engine.state
                    ),
                });
            }

            if !engine.capabilities.iter().any(|item| item == capability) {
                return Err(ResolveError {
                    kind: ResolveErrorKind::EngineIncompatible,
                    message: format!(
                        "engine {} does not provide capability {capability}",
                        engine.id
                    ),
                });
            }

            return Ok(engine);
        }

        self.engines
            .iter()
            .filter(|engine| {
                engine.state == EngineState::Ready
                    && engine.capabilities.iter().any(|item| item == capability)
            })
            .min_by_key(|engine| provider_priority(engine.provider))
            .ok_or_else(|| ResolveError {
                kind: ResolveErrorKind::NoCompatibleEngine,
                message: format!("no ready engine provides capability {capability}"),
            })
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

fn provider_priority(provider: EngineProvider) -> u8 {
    match provider {
        EngineProvider::BuiltIn => 0,
        EngineProvider::Managed => 1,
        EngineProvider::System => 2,
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
        assert_eq!(report.engines.total, 2);
        assert_eq!(report.engines.ready, 2);
    }

    #[test]
    fn bootstrap_registry_exposes_runtime_and_image_capabilities() {
        let registry = RuntimeRegistry::bootstrap();
        let ids: Vec<_> = registry
            .capabilities()
            .iter()
            .map(|capability| capability.id.as_str())
            .collect();

        assert_eq!(
            ids,
            vec![
                "runtime.doctor",
                "runtime.capabilities",
                "engine.list",
                "image.info",
                "image.resize"
            ]
        );
    }

    #[test]
    fn automatic_resolution_selects_built_in_raster_engine() {
        let registry = RuntimeRegistry::bootstrap();
        let engine = registry.resolve_engine("image.info", None).unwrap();

        assert_eq!(engine.id, "raster-rs");
        assert_eq!(engine.provider, EngineProvider::BuiltIn);
    }

    #[test]
    fn explicit_incompatible_engine_is_rejected() {
        let registry = RuntimeRegistry::bootstrap();
        let error = registry
            .resolve_engine("image.info", Some("yu-runtime"))
            .unwrap_err();

        assert_eq!(error.kind, ResolveErrorKind::EngineIncompatible);
    }

    #[test]
    fn unknown_engine_is_rejected() {
        let registry = RuntimeRegistry::bootstrap();
        let error = registry
            .resolve_engine("image.info", Some("does-not-exist"))
            .unwrap_err();

        assert_eq!(error.kind, ResolveErrorKind::UnknownEngine);
    }
}
