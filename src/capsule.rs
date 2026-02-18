use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleMode {
    HostDefault,
    IsolatedNixShell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capsule {
    pub capsule_id: String,
    pub slug: String,
    pub display_name: String,
    pub mode: CapsuleMode,
    pub workspace: u16,
}

impl Capsule {
    pub fn new(capsule_id: &str, display_name: &str, mode: CapsuleMode, workspace: u16) -> Self {
        Self {
            capsule_id: capsule_id.to_string(),
            slug: normalize_slug(display_name),
            display_name: display_name.to_string(),
            mode,
            workspace,
        }
    }

    pub fn rename_display_name(&mut self, display_name: &str) {
        self.display_name = display_name.to_string();
    }

    pub fn domain(&self) -> String {
        format!("{}.nexum.local", self.slug)
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
