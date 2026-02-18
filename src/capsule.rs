use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleMode {
    HostDefault,
    IsolatedNixShell,
}

impl FromStr for CapsuleMode {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "host_default" => Ok(Self::HostDefault),
            "isolated_nix_shell" => Ok(Self::IsolatedNixShell),
            _ => Err(format!("invalid mode: {input}")),
        }
    }
}

impl fmt::Display for CapsuleMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(mode_to_str(*self))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleState {
    Creating,
    Ready,
    Restoring,
    Degraded,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capsule {
    pub capsule_id: String,
    pub slug: String,
    pub display_name: String,
    pub repo_path: String,
    pub mode: CapsuleMode,
    pub state: CapsuleState,
    pub workspace: u16,
}

impl Capsule {
    pub fn new(capsule_id: &str, display_name: &str, mode: CapsuleMode, workspace: u16) -> Self {
        Self {
            capsule_id: capsule_id.to_string(),
            slug: normalize_slug(display_name),
            display_name: display_name.to_string(),
            repo_path: String::new(),
            mode,
            state: CapsuleState::Ready,
            workspace,
        }
    }

    pub fn rename_display_name(&mut self, display_name: &str) {
        self.display_name = display_name.to_string();
    }

    pub fn domain(&self) -> String {
        format!("{}.nexum.local", self.slug)
    }

    pub fn transition_state(&mut self, state: CapsuleState) {
        self.state = state;
    }

    pub fn with_repo_path(mut self, repo_path: &str) -> Self {
        self.repo_path = repo_path.to_string();
        self
    }

    pub fn set_repo_path(&mut self, repo_path: &str) {
        self.repo_path = repo_path.to_string();
    }
}

pub fn normalize_slug(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut previous_dash = false;

    for ch in input.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_lowercase() || lower.is_ascii_digit() {
            slug.push(lower);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    slug = slug.trim_matches('-').to_string();

    if slug.is_empty() {
        return "capsule".to_string();
    }

    slug
}

pub fn parse_state(input: &str) -> Option<CapsuleState> {
    match input {
        "creating" => Some(CapsuleState::Creating),
        "ready" => Some(CapsuleState::Ready),
        "restoring" => Some(CapsuleState::Restoring),
        "degraded" => Some(CapsuleState::Degraded),
        "archived" => Some(CapsuleState::Archived),
        _ => None,
    }
}

pub fn mode_to_str(mode: CapsuleMode) -> &'static str {
    match mode {
        CapsuleMode::HostDefault => "host_default",
        CapsuleMode::IsolatedNixShell => "isolated_nix_shell",
    }
}

pub fn state_to_str(state: CapsuleState) -> &'static str {
    match state {
        CapsuleState::Creating => "creating",
        CapsuleState::Ready => "ready",
        CapsuleState::Restoring => "restoring",
        CapsuleState::Degraded => "degraded",
        CapsuleState::Archived => "archived",
    }
}
