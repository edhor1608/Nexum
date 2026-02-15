use std::path::PathBuf;

use nexum::{
    capsule::{Capsule, CapsuleMode},
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

    println!("{}", serde_json::to_string_pretty(&payload)?);
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
    Ok(value.to_string())
}

fn parse_mode(input: &str) -> Result<CapsuleMode, Box<dyn std::error::Error>> {
    match input {
        "host_default" => Ok(CapsuleMode::HostDefault),
        "isolated_nix_shell" => Ok(CapsuleMode::IsolatedNixShell),
        _ => Err(format!("invalid mode: {input}").into()),
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
}
