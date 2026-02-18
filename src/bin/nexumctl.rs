use std::path::PathBuf;

use nexum::{
    capsule::{Capsule, CapsuleMode},
    flags::{CutoverFlags, FlagName},
    shadow::{ExecutionResult, compare_execution},
    shell::{NiriShellCommand, NiriShellPlan, render_shell_script},
    store::CapsuleStore,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "capsule" => capsule_command(&args[1..])?,
        "flags" => flags_command(&args[1..])?,
        "parity" => parity_command(&args[1..])?,
        "shell" => shell_command(&args[1..])?,
        _ => {
            usage();
            std::process::exit(2);
        }
    }

    Ok(())
}

fn capsule_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "create" => capsule_create(&args[1..]),
        "list" => capsule_list(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn capsule_create(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let id = required_arg(args, "--id")?;
    let name = required_arg(args, "--name")?;
    let workspace = required_arg(args, "--workspace")?.parse::<u16>()?;
    let mode = required_arg(args, "--mode")?;

    let mode = parse_mode(&mode)?;

    let mut store = CapsuleStore::open(&PathBuf::from(db))?;
    let capsule = Capsule::new(&id, &name, mode, workspace);
    store.upsert(capsule)?;

    println!("created {}", id);
    Ok(())
}

fn capsule_list(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let store = CapsuleStore::open(&PathBuf::from(db))?;
    let listed = store.list()?;

    let payload = listed
        .into_iter()
        .map(|capsule| {
            serde_json::json!({
                "capsule_id": capsule.capsule_id,
                "slug": capsule.slug,
                "domain": capsule.domain(),
                "display_name": capsule.display_name,
                "mode": mode_to_str(capsule.mode),
                "workspace": capsule.workspace,
            })
        })
        .collect::<Vec<_>>();

    println!("{}", serde_json::to_string(&payload)?);
    Ok(())
}

fn flags_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "set" => flags_set(&args[1..]),
        "show" => flags_show(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn flags_set(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let file = required_arg(args, "--file")?;
    let path = PathBuf::from(file);

    let mut flags = CutoverFlags::load_or_default(&path)?;
    if let Some(value) = optional_arg(args, "--shadow") {
        flags.set(FlagName::ShadowMode, parse_bool(&value)?);
    }
    if let Some(value) = optional_arg(args, "--routing") {
        flags.set(FlagName::RoutingControlPlane, parse_bool(&value)?);
    }
    if let Some(value) = optional_arg(args, "--restore") {
        flags.set(FlagName::RestoreControlPlane, parse_bool(&value)?);
    }
    if let Some(value) = optional_arg(args, "--attention") {
        flags.set(FlagName::AttentionControlPlane, parse_bool(&value)?);
    }
    flags.save(&path)?;

    println!("{}", serde_json::to_string(&flags)?);
    Ok(())
}

fn flags_show(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let file = required_arg(args, "--file")?;
    let flags = CutoverFlags::load_or_default(&PathBuf::from(file))?;
    println!("{}", serde_json::to_string(&flags)?);
    Ok(())
}

fn parity_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "compare" => parity_compare(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn parity_compare(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let primary_json = arg_or_file(args, "--primary-json", "--primary-file")?;
    let candidate_json = arg_or_file(args, "--candidate-json", "--candidate-file")?;

    let primary: ExecutionResult = serde_json::from_str(&primary_json)?;
    let candidate: ExecutionResult = serde_json::from_str(&candidate_json)?;

    let report = compare_execution(&primary, &candidate);
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn shell_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "render" => shell_render(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn shell_render(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = required_arg(args, "--workspace")?.parse::<u16>()?;
    let terminal = required_arg(args, "--terminal")?;
    let editor = required_arg(args, "--editor")?;
    let browser = required_arg(args, "--browser")?;
    let attention = required_arg(args, "--attention")?;

    let plan = NiriShellPlan {
        workspace,
        commands: vec![
            NiriShellCommand::FocusWorkspace(workspace),
            NiriShellCommand::SpawnTerminal(terminal),
            NiriShellCommand::SpawnEditor(editor),
            NiriShellCommand::SpawnBrowser(browser),
            NiriShellCommand::RaiseAttention(attention),
        ],
    };

    println!("{}", render_shell_script(&plan));
    Ok(())
}

fn required_arg(args: &[String], key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let pos = args
        .iter()
        .position(|arg| arg == key)
        .ok_or_else(|| format!("missing arg {key}"))?;
    let value = args
        .get(pos + 1)
        .ok_or_else(|| format!("missing value for {key}"))?;
    if value.starts_with('-') {
        return Err(format!("missing value for {key}").into());
    }
    Ok(value.to_string())
}

fn optional_arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == key)
        .and_then(|pos| args.get(pos + 1))
        .map(ToString::to_string)
}

fn arg_or_file(
    args: &[String],
    json_key: &str,
    file_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(path) = optional_arg(args, file_key) {
        return Ok(std::fs::read_to_string(path)?);
    }
    required_arg(args, json_key)
}

fn parse_mode(input: &str) -> Result<CapsuleMode, Box<dyn std::error::Error>> {
    match input {
        "host_default" => Ok(CapsuleMode::HostDefault),
        "isolated_nix_shell" => Ok(CapsuleMode::IsolatedNixShell),
        _ => Err(format!("invalid mode: {input}").into()),
    }
}

fn parse_bool(input: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match input {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid bool: {input}").into()),
    }
}

fn mode_to_str(mode: CapsuleMode) -> &'static str {
    match mode {
        CapsuleMode::HostDefault => "host_default",
        CapsuleMode::IsolatedNixShell => "isolated_nix_shell",
    }
}

fn usage() {
    eprintln!(
        "nexumctl capsule create --db <path> --id <id> --name <name> --workspace <n> --mode <host_default|isolated_nix_shell>"
    );
    eprintln!("nexumctl capsule list --db <path>");
    eprintln!(
        "nexumctl flags set --file <path> [--shadow true|false] [--routing true|false] [--restore true|false] [--attention true|false]"
    );
    eprintln!("nexumctl flags show --file <path>");
    eprintln!(
        "nexumctl parity compare (--primary-json <json> | --primary-file <path>) (--candidate-json <json> | --candidate-file <path>)"
    );
    eprintln!(
        "nexumctl shell render --workspace <n> --terminal <cmd> --editor <path> --browser <url> --attention <level>"
    );
}
