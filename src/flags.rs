use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CutoverFlags {
    pub shadow_mode: bool,
    pub routing_control_plane: bool,
    pub restore_control_plane: bool,
    pub attention_control_plane: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagName {
    ShadowMode,
    RoutingControlPlane,
    RestoreControlPlane,
    AttentionControlPlane,
}

#[derive(Debug, Error)]
pub enum FlagError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml parse: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("toml serialize: {0}")]
    Serialize(#[from] toml::ser::Error),
}

impl Default for CutoverFlags {
    fn default() -> Self {
        Self {
            shadow_mode: true,
            routing_control_plane: false,
            restore_control_plane: false,
            attention_control_plane: false,
        }
    }
}

impl CutoverFlags {
    pub fn load_or_default(path: &Path) -> Result<Self, FlagError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, path: &Path) -> Result<(), FlagError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn set(&mut self, name: FlagName, enabled: bool) {
        match name {
            FlagName::ShadowMode => self.shadow_mode = enabled,
            FlagName::RoutingControlPlane => self.routing_control_plane = enabled,
            FlagName::RestoreControlPlane => self.restore_control_plane = enabled,
            FlagName::AttentionControlPlane => self.attention_control_plane = enabled,
        }
    }
}
