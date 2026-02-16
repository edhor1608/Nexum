use std::path::PathBuf;

use nexum::{
    capsule::{Capsule, CapsuleMode},
    cutover::{CutoverInput, apply_cutover, evaluate_cutover, parse_capability},
    flags::{CutoverFlags, FlagName},
    restore::SignalType,
    routing::{RouteCommand, default_socket_path, send_command},
    runflow::{RestoreRunInput, run_restore_flow},
    shadow::{ExecutionResult, compare_execution},
    shell::{NiriShellCommand, NiriShellPlan, render_shell_script},
    store::CapsuleStore,
    tls::{ensure_self_signed_cert, rotate_if_expiring},
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
        "routing" => routing_command(&args[1..])?,
        "shell" => shell_command(&args[1..])?,
        "tls" => tls_command(&args[1..])?,
        "cutover" => cutover_command(&args[1..])?,
        "run" => run_command(&args[1..])?,
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
    let primary_json = required_arg(args, "--primary-json")?;
    let candidate_json = required_arg(args, "--candidate-json")?;

    let primary: ExecutionResult = serde_json::from_str(&primary_json)?;
    let candidate: ExecutionResult = serde_json::from_str(&candidate_json)?;

    let report = compare_execution(&primary, &candidate);
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn routing_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "health" => routing_health(&args[1..]),
        "register" => routing_register(&args[1..]),
        "resolve" => routing_resolve(&args[1..]),
        "remove" => routing_remove(&args[1..]),
        "list" => routing_list(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn routing_health(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let outcome = route_request(socket_arg_or_default(args), RouteCommand::Health)?;
    println!("{}", serde_json::to_string(&outcome)?);
    Ok(())
}

fn routing_register(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let outcome = route_request(
        socket_arg_or_default(args),
        RouteCommand::Register {
            capsule_id: required_arg(args, "--capsule-id")?,
            domain: required_arg(args, "--domain")?,
            upstream: required_arg(args, "--upstream")?,
        },
    )?;
    println!("{}", serde_json::to_string(&outcome)?);
    Ok(())
}

fn routing_resolve(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let outcome = route_request(
        socket_arg_or_default(args),
        RouteCommand::Resolve {
            domain: required_arg(args, "--domain")?,
        },
    )?;
    println!("{}", serde_json::to_string(&outcome)?);
    Ok(())
}

fn routing_remove(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let outcome = route_request(
        socket_arg_or_default(args),
        RouteCommand::Remove {
            domain: required_arg(args, "--domain")?,
        },
    )?;
    println!("{}", serde_json::to_string(&outcome)?);
    Ok(())
}

fn routing_list(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let outcome = route_request(socket_arg_or_default(args), RouteCommand::List)?;
    println!("{}", serde_json::to_string(&outcome)?);
    Ok(())
}

fn socket_arg_or_default(args: &[String]) -> PathBuf {
    optional_arg(args, "--socket")
        .map(PathBuf::from)
        .unwrap_or_else(default_socket_path)
}

fn route_request(
    socket: PathBuf,
    command: RouteCommand,
) -> Result<nexum::routing::RouteOutcome, Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()?;
    Ok(runtime.block_on(send_command(&socket, command))?)
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

fn tls_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "ensure" => tls_ensure(&args[1..]),
        "rotate" => tls_rotate(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn tls_ensure(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(required_arg(args, "--dir")?);
    let domain = required_arg(args, "--domain")?;
    let validity_days = optional_arg(args, "--validity-days")
        .unwrap_or_else(|| "30".to_string())
        .parse::<u64>()?;

    let record = ensure_self_signed_cert(&dir, &domain, validity_days)?;
    println!("{}", serde_json::to_string(&record)?);
    Ok(())
}

fn tls_rotate(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(required_arg(args, "--dir")?);
    let domain = required_arg(args, "--domain")?;
    let threshold_days = required_arg(args, "--threshold-days")?.parse::<u64>()?;

    let outcome = rotate_if_expiring(&dir, &domain, threshold_days)?;
    println!("{}", serde_json::to_string(&outcome)?);
    Ok(())
}

fn cutover_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "apply" => cutover_apply(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn cutover_apply(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let file = PathBuf::from(required_arg(args, "--file")?);
    let capability = parse_capability(&required_arg(args, "--capability")?)
        .ok_or_else(|| "invalid capability".to_string())?;
    let parity_score = required_arg(args, "--parity-score")?.parse::<f64>()?;
    let min_parity_score = required_arg(args, "--min-parity-score")?.parse::<f64>()?;
    let critical_events = required_arg(args, "--critical-events")?.parse::<u32>()?;
    let max_critical_events = required_arg(args, "--max-critical-events")?.parse::<u32>()?;
    let shadow_mode_enabled = parse_bool(&required_arg(args, "--shadow-mode")?)?;

    let decision = evaluate_cutover(&CutoverInput {
        capability,
        parity_score,
        min_parity_score,
        critical_events,
        max_critical_events,
        shadow_mode_enabled,
    });

    let mut flags = CutoverFlags::load_or_default(&file)?;
    apply_cutover(&mut flags, &decision, capability);
    flags.save(&file)?;

    println!("{}", serde_json::to_string(&decision)?);
    Ok(())
}

fn run_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "restore" => run_restore(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn run_restore(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let signal = parse_signal(&required_arg(args, "--signal")?)?;
    let identity_collision = optional_arg(args, "--identity-collision")
        .map(|value| parse_bool(&value))
        .transpose()?
        .unwrap_or(false);
    let high_risk_secret_workflow = optional_arg(args, "--high-risk-secret")
        .map(|value| parse_bool(&value))
        .transpose()?
        .unwrap_or(false);
    let force_isolated_mode = optional_arg(args, "--force-isolated")
        .map(|value| parse_bool(&value))
        .transpose()?
        .unwrap_or(false);

    let summary = run_restore_flow(RestoreRunInput {
        capsule_id: required_arg(args, "--capsule-id")?,
        display_name: required_arg(args, "--name")?,
        workspace: required_arg(args, "--workspace")?.parse::<u16>()?,
        signal,
        terminal_cmd: required_arg(args, "--terminal")?,
        editor_target: required_arg(args, "--editor")?,
        browser_url: required_arg(args, "--browser")?,
        route_upstream: required_arg(args, "--upstream")?,
        routing_socket: optional_arg(args, "--routing-socket").map(PathBuf::from),
        identity_collision,
        high_risk_secret_workflow,
        force_isolated_mode,
        tls_dir: PathBuf::from(required_arg(args, "--tls-dir")?),
        events_db: PathBuf::from(required_arg(args, "--events-db")?),
    })?;

    println!("{}", serde_json::to_string(&summary)?);
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

fn optional_arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == key)
        .and_then(|pos| args.get(pos + 1))
        .map(ToString::to_string)
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

fn parse_signal(input: &str) -> Result<SignalType, Box<dyn std::error::Error>> {
    match input {
        "needs_decision" => Ok(SignalType::NeedsDecision),
        "critical_failure" => Ok(SignalType::CriticalFailure),
        "passive_completion" => Ok(SignalType::PassiveCompletion),
        _ => Err(format!("invalid signal: {input}").into()),
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
    eprintln!("nexumctl parity compare --primary-json <json> --candidate-json <json>");
    eprintln!("nexumctl routing health [--socket <path>]");
    eprintln!(
        "nexumctl routing register --capsule-id <id> --domain <domain> --upstream <host:port> [--socket <path>]"
    );
    eprintln!("nexumctl routing resolve --domain <domain> [--socket <path>]");
    eprintln!("nexumctl routing remove --domain <domain> [--socket <path>]");
    eprintln!("nexumctl routing list [--socket <path>]");
    eprintln!(
        "nexumctl shell render --workspace <n> --terminal <cmd> --editor <path> --browser <url> --attention <level>"
    );
    eprintln!("nexumctl tls ensure --dir <path> --domain <domain> [--validity-days <days>]");
    eprintln!("nexumctl tls rotate --dir <path> --domain <domain> --threshold-days <days>");
    eprintln!(
        "nexumctl cutover apply --file <path> --capability <routing|restore|attention> --parity-score <f64> --min-parity-score <f64> --critical-events <u32> --max-critical-events <u32> --shadow-mode <true|false>"
    );
    eprintln!(
        "nexumctl run restore --capsule-id <id> --name <name> --workspace <n> --signal <needs_decision|critical_failure|passive_completion> --terminal <cmd> --editor <path> --browser <url> --upstream <host:port> [--routing-socket <path>] [--identity-collision true|false] [--high-risk-secret true|false] [--force-isolated true|false] --tls-dir <path> --events-db <path>"
    );
}
