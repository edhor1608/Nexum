use std::process::Command;

#[test]
fn nexumctl_can_render_shell_script() {
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let output = Command::new(nexumctl)
        .arg("shell")
        .arg("render")
        .arg("--workspace")
        .arg("5")
        .arg("--terminal")
        .arg("cd /workspace/ui && nix develop")
        .arg("--editor")
        .arg("/workspace/ui")
        .arg("--browser")
        .arg("https://ui.nexum.local")
        .arg("--attention")
        .arg("needs_decision")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("niri msg action focus-workspace 5"));
    assert!(stdout.contains("xdg-open 'https://ui.nexum.local'"));
    assert!(stdout.contains("notify-send 'Nexum Attention' 'needs_decision'"));
}
