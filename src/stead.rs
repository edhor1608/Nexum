use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::restore::SignalType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchEvent {
    pub capsule_id: String,
    pub signal: SignalType,
    pub upstream: String,
    #[serde(default)]
    pub identity_collision: bool,
    #[serde(default)]
    pub high_risk_secret_workflow: bool,
    #[serde(default)]
    pub force_isolated_mode: bool,
}

#[derive(Debug, Error)]
pub enum SteadError {
    #[error("invalid --event-json: {0}")]
    ParseJson(String),
    #[error("invalid --events-json: {0}")]
    ParseJsonBatch(String),
}

pub fn parse_dispatch_event(value: &str) -> Result<DispatchEvent, SteadError> {
    serde_json::from_str::<DispatchEvent>(value)
        .map_err(|error| SteadError::ParseJson(error.to_string()))
}

pub fn parse_dispatch_events(value: &str) -> Result<Vec<DispatchEvent>, SteadError> {
    serde_json::from_str::<Vec<DispatchEvent>>(value)
        .map_err(|error| SteadError::ParseJsonBatch(error.to_string()))
}
