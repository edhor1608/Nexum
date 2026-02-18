use serde::{Deserialize, Serialize};

use crate::flags::{CutoverFlags, FlagName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Routing,
    Restore,
    Attention,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CutoverInput {
    pub capability: Capability,
    pub parity_score: f64,
    pub min_parity_score: f64,
    pub critical_events: u32,
    pub max_critical_events: u32,
    pub shadow_mode_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CutoverDecision {
    pub capability: Capability,
    pub allowed: bool,
    pub reasons: Vec<String>,
    pub flag_to_enable: Option<String>,
}

pub fn evaluate_cutover(input: &CutoverInput) -> CutoverDecision {
    let mut reasons = Vec::new();

    if !input.shadow_mode_enabled {
        reasons.push("shadow_mode must be enabled".to_string());
    }
    if !input.parity_score.is_finite() || !input.min_parity_score.is_finite() {
        reasons.push("parity values must be finite".to_string());
    } else if !(0.0..=1.0).contains(&input.parity_score)
        || !(0.0..=1.0).contains(&input.min_parity_score)
    {
        reasons.push("parity values must be between 0 and 1".to_string());
    } else if input.parity_score < input.min_parity_score {
        reasons.push(format!(
            "parity below threshold: {} < {}",
            input.parity_score, input.min_parity_score
        ));
    }
    if input.critical_events > input.max_critical_events {
        reasons.push(format!(
            "critical events exceeded: {} > {}",
            input.critical_events, input.max_critical_events
        ));
    }

    let allowed = reasons.is_empty();
    let flag_to_enable = if allowed {
        Some(capability_flag(input.capability).to_string())
    } else {
        None
    };

    CutoverDecision {
        capability: input.capability,
        allowed,
        reasons,
        flag_to_enable,
    }
}

pub fn apply_cutover(flags: &mut CutoverFlags, decision: &CutoverDecision) {
    if !decision.allowed {
        return;
    }

    match decision.capability {
        Capability::Routing => flags.set(FlagName::RoutingControlPlane, true),
        Capability::Restore => flags.set(FlagName::RestoreControlPlane, true),
        Capability::Attention => flags.set(FlagName::AttentionControlPlane, true),
    }
}

pub fn parse_capability(input: &str) -> Option<Capability> {
    match input {
        "routing" => Some(Capability::Routing),
        "restore" => Some(Capability::Restore),
        "attention" => Some(Capability::Attention),
        _ => None,
    }
}

fn capability_flag(capability: Capability) -> &'static str {
    match capability {
        Capability::Routing => "routing_control_plane",
        Capability::Restore => "restore_control_plane",
        Capability::Attention => "attention_control_plane",
    }
}
