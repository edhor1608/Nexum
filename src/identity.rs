use std::path::PathBuf;

pub fn profile_dir_for_capsule(capsule_id: &str) -> PathBuf {
    PathBuf::from(format!("/tmp/nexum/profiles/{capsule_id}"))
}

pub fn browser_launch_command(url: &str, capsule_id: &str, collision_detected: bool) -> String {
    if collision_detected {
        return format!(
            "firefox --profile {} {url}",
            profile_dir_for_capsule(capsule_id).display()
        );
    }

    format!("xdg-open {url}")
}
