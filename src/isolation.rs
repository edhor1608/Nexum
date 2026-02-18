use crate::capsule::CapsuleMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsolationInput {
    pub identity_collision_detected: bool,
    pub high_risk_secret_workflow: bool,
    pub force_isolated_mode: bool,
}

pub fn select_capsule_mode(input: &IsolationInput) -> CapsuleMode {
    if input.identity_collision_detected
        || input.high_risk_secret_workflow
        || input.force_isolated_mode
    {
        return CapsuleMode::IsolatedNixShell;
    }

    CapsuleMode::HostDefault
}
