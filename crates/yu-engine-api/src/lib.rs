use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineProvider {
    BuiltIn,
    Managed,
    System,
}

impl fmt::Display for EngineProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::BuiltIn => "built_in",
            Self::Managed => "managed",
            Self::System => "system",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineState {
    Ready,
    NotInstalled,
    Broken,
    Incompatible,
    Disabled,
}

impl fmt::Display for EngineState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Ready => "ready",
            Self::NotInstalled => "not_installed",
            Self::Broken => "broken",
            Self::Incompatible => "incompatible",
            Self::Disabled => "disabled",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineDescriptor {
    pub id: String,
    pub display_name: String,
    pub provider: EngineProvider,
    pub state: EngineState,
    pub version: Option<String>,
    pub capabilities: Vec<String>,
}
