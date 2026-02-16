use nexum::identity::{browser_launch_command, profile_dir_for_capsule};

#[test]
fn domain_isolation_uses_default_browser_command_when_no_collision() {
    let launch = browser_launch_command("https://alpha.nexum.local", "cap-alpha", false);
    assert_eq!(launch, "xdg-open https://alpha.nexum.local");
}

#[test]
fn collision_activates_profile_fallback_command() {
    let profile = profile_dir_for_capsule("cap-collision");
    let launch = browser_launch_command("https://alpha.nexum.local", "cap-collision", true);

    assert!(launch.contains("firefox --profile"));
    assert!(launch.contains(profile.to_str().unwrap()));
    assert!(launch.contains("https://alpha.nexum.local"));
}
