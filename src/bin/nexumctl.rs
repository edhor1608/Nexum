use std::path::PathBuf;

use nexum::{
    capsule::{Capsule, CapsuleMode, CapsuleState, parse_state, state_to_str},
    cutover::{CutoverInput, apply_cutover, evaluate_cutover, parse_capability},
    events::EventStore,
    flags::{CutoverFlags, FlagName},
    restore::SignalType,
    routing::{RouteCommand, default_socket_path, send_command},
    runflow::{RestoreRunInput, run_restore_flow},
    shadow::{ExecutionResult, compare_execution},
    shell::{NiriShellCommand, NiriShellPlan, render_shell_script},
    stead::{DispatchEvent, parse_dispatch_event, parse_dispatch_events},
    store::CapsuleStore,
    tls::{ensure_self_signed_cert, rotate_if_expiring},
};
use serde::Serialize;

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
        "events" => events_command(&args[1..])?,
        "routing" => routing_command(&args[1..])?,
        "shell" => shell_command(&args[1..])?,
        "stead" => stead_command(&args[1..])?,
        "supervisor" => supervisor_command(&args[1..])?,
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

#[derive(Debug, Serialize)]
struct SupervisorCapsuleStatus {
    capsule_id: String,
    display_name: String,
    repo_path: String,
    mode: String,
    state: String,
    workspace: u16,
    critical_events: u32,
    last_event_level: Option<String>,
    last_event_message: Option<String>,
    last_event_ts_unix_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
struct SupervisorStatusReport {
    flags: CutoverFlags,
    total_capsules: u32,
    degraded_capsules: u32,
    archived_capsules: u32,
    critical_events: u32,
    capsules: Vec<SupervisorCapsuleStatus>,
}

#[derive(Debug, Serialize)]
struct SupervisorBlocker {
    capsule_id: String,
    state: String,
    critical_events: u32,
    reasons: Vec<String>,
}

fn supervisor_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "status" => supervisor_status(&args[1..]),
        "blockers" => supervisor_blockers(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn supervisor_status(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let capsule_db = PathBuf::from(required_arg(args, "--capsule-db")?);
    let events_db = PathBuf::from(required_arg(args, "--events-db")?);
    let flags_file = PathBuf::from(required_arg(args, "--flags-file")?);

    let store = CapsuleStore::open(&capsule_db)?;
    let events = EventStore::open(&events_db)?;
    let flags = CutoverFlags::load_or_default(&flags_file)?;
    let listed = store.list()?;

    let mut degraded_capsules = 0u32;
    let mut archived_capsules = 0u32;
    let mut critical_events = 0u32;
    let mut capsules = Vec::with_capacity(listed.len());

    for capsule in listed {
        if capsule.state == CapsuleState::Degraded {
            degraded_capsules += 1;
        }
        if capsule.state == CapsuleState::Archived {
            archived_capsules += 1;
        }

        let capsule_critical = events.count_for_capsule_level(&capsule.capsule_id, "critical")?;
        critical_events += capsule_critical;

        let last = events
            .list_recent(Some(&capsule.capsule_id), None, Some(1))?
            .into_iter()
            .next();

        capsules.push(SupervisorCapsuleStatus {
            capsule_id: capsule.capsule_id,
            display_name: capsule.display_name,
            repo_path: capsule.repo_path,
            mode: mode_to_str(capsule.mode).to_string(),
            state: state_to_str(capsule.state).to_string(),
            workspace: capsule.workspace,
            critical_events: capsule_critical,
            last_event_level: last.as_ref().map(|value| value.level.clone()),
            last_event_message: last.as_ref().map(|value| value.message.clone()),
            last_event_ts_unix_ms: last.as_ref().map(|value| value.ts_unix_ms),
        });
    }

    let report = SupervisorStatusReport {
        flags,
        total_capsules: capsules.len() as u32,
        degraded_capsules,
        archived_capsules,
        critical_events,
        capsules,
    };

    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn supervisor_blockers(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let capsule_db = PathBuf::from(required_arg(args, "--capsule-db")?);
    let events_db = PathBuf::from(required_arg(args, "--events-db")?);
    let critical_threshold = optional_arg(args, "--critical-threshold")
        .map(|value| value.parse::<u32>())
        .transpose()?
        .unwrap_or(1);

    let store = CapsuleStore::open(&capsule_db)?;
    let events = EventStore::open(&events_db)?;
    let listed = store.list()?;

    let mut blockers = Vec::new();
    for capsule in listed {
        let critical_events = events.count_for_capsule_level(&capsule.capsule_id, "critical")?;
        let mut reasons = Vec::new();
        if capsule.state == CapsuleState::Degraded {
            reasons.push("state_degraded".to_string());
        }
        if critical_events >= critical_threshold {
            reasons.push("critical_events_threshold".to_string());
        }
        if reasons.is_empty() {
            continue;
        }

        blockers.push(SupervisorBlocker {
            capsule_id: capsule.capsule_id,
            state: state_to_str(capsule.state).to_string(),
            critical_events,
            reasons,
        });
    }

    println!("{}", serde_json::to_string(&blockers)?);
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
        "export" => capsule_export(&args[1..]),
        "rename" => capsule_rename(&args[1..]),
        "set-repo" => capsule_set_repo(&args[1..]),
        "set-state" => capsule_set_state(&args[1..]),
        "allocate-port" => capsule_allocate_port(&args[1..]),
        "release-ports" => capsule_release_ports(&args[1..]),
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
    let repo_path = optional_arg(args, "--repo-path").unwrap_or_default();

    let mode = parse_mode(&mode)?;

    let mut store = CapsuleStore::open(&PathBuf::from(db))?;
    let capsule = Capsule::new(&id, &name, mode, workspace).with_repo_path(&repo_path);
    store.upsert(capsule)?;

    println!("created {}", id);
    Ok(())
}

fn capsule_list(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let store = CapsuleStore::open(&PathBuf::from(db))?;
    let listed = store.list()?;

    let mut payload = Vec::with_capacity(listed.len());
    for capsule in listed {
        let allocated_ports = store.list_ports(&capsule.capsule_id)?;
        payload.push(serde_json::json!({
            "capsule_id": capsule.capsule_id,
            "slug": capsule.slug,
            "domain": capsule.domain(),
            "display_name": capsule.display_name,
            "repo_path": capsule.repo_path,
            "mode": mode_to_str(capsule.mode),
            "state": state_to_str(capsule.state),
            "workspace": capsule.workspace,
            "allocated_ports": allocated_ports,
        }));
    }

    println!("{}", serde_json::to_string(&payload)?);
    Ok(())
}

fn capsule_export(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let format = required_arg(args, "--format")?;
    let store = CapsuleStore::open(&PathBuf::from(db))?;

    match format.as_str() {
        "yaml" => {
            println!("{}", store.export_yaml()?);
            Ok(())
        }
        _ => Err(format!("unsupported export format: {format}").into()),
    }
}

fn capsule_rename(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let id = required_arg(args, "--id")?;
    let name = required_arg(args, "--name")?;

    let mut store = CapsuleStore::open(&PathBuf::from(db))?;
    store.rename_display_name(&id, &name)?;

    println!("renamed {}", id);
    Ok(())
}

fn capsule_set_state(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let id = required_arg(args, "--id")?;
    let state =
        parse_state(&required_arg(args, "--state")?).ok_or_else(|| "invalid state".to_string())?;

    let mut store = CapsuleStore::open(&PathBuf::from(db))?;
    store.transition_state(&id, state)?;

    println!("state_updated {}", id);
    Ok(())
}

fn capsule_set_repo(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let id = required_arg(args, "--id")?;
    let repo_path = required_arg(args, "--repo-path")?;

    let mut store = CapsuleStore::open(&PathBuf::from(db))?;
    store.set_repo_path(&id, &repo_path)?;

    println!("repo_updated {}", id);
    Ok(())
}

fn capsule_allocate_port(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let id = required_arg(args, "--id")?;
    let start = required_arg(args, "--start")?.parse::<u16>()?;
    let end = required_arg(args, "--end")?.parse::<u16>()?;

    let mut store = CapsuleStore::open(&PathBuf::from(db))?;
    let port = store.allocate_port(&id, start, end)?;
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "capsule_id": id,
            "port": port,
        }))?
    );
    Ok(())
}

fn capsule_release_ports(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let id = required_arg(args, "--id")?;

    let mut store = CapsuleStore::open(&PathBuf::from(db))?;
    let released = store.release_ports(&id)?;
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "capsule_id": id,
            "released": released,
        }))?
    );
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

fn events_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "summary" => events_summary(&args[1..]),
        "list" => events_list(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn events_summary(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let store = EventStore::open(&PathBuf::from(db))?;
    let summary = store.summary()?;
    println!("{}", serde_json::to_string(&summary)?);
    Ok(())
}

fn events_list(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = required_arg(args, "--db")?;
    let capsule_id = optional_arg(args, "--capsule-id");
    let level = optional_arg(args, "--level");
    let limit = optional_arg(args, "--limit")
        .map(|value| value.parse::<u32>())
        .transpose()?;

    let store = EventStore::open(&PathBuf::from(db))?;
    let listed = store.list_recent(capsule_id.as_deref(), level.as_deref(), limit)?;
    println!("{}", serde_json::to_string(&listed)?);
    Ok(())
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
        "apply-from-events" => cutover_apply_from_events(&args[1..]),
        "apply-from-summary" => cutover_apply_from_summary(&args[1..]),
        "rollback" => cutover_rollback(&args[1..]),
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

fn cutover_apply_from_events(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let file = PathBuf::from(required_arg(args, "--file")?);
    let capability = parse_capability(&required_arg(args, "--capability")?)
        .ok_or_else(|| "invalid capability".to_string())?;
    let parity_score = required_arg(args, "--parity-score")?.parse::<f64>()?;
    let min_parity_score = required_arg(args, "--min-parity-score")?.parse::<f64>()?;
    let events_db = PathBuf::from(required_arg(args, "--events-db")?);
    let capsule_id = required_arg(args, "--capsule-id")?;
    let max_critical_events = required_arg(args, "--max-critical-events")?.parse::<u32>()?;
    let shadow_mode_enabled = parse_bool(&required_arg(args, "--shadow-mode")?)?;

    let events = EventStore::open(&events_db)?;
    let critical_events = events.count_for_capsule_level(&capsule_id, "critical")?;

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

fn cutover_apply_from_summary(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let file = PathBuf::from(required_arg(args, "--file")?);
    let capability = parse_capability(&required_arg(args, "--capability")?)
        .ok_or_else(|| "invalid capability".to_string())?;
    let parity_score = required_arg(args, "--parity-score")?.parse::<f64>()?;
    let min_parity_score = required_arg(args, "--min-parity-score")?.parse::<f64>()?;
    let events_db = PathBuf::from(required_arg(args, "--events-db")?);
    let max_critical_events = required_arg(args, "--max-critical-events")?.parse::<u32>()?;
    let shadow_mode_enabled = parse_bool(&required_arg(args, "--shadow-mode")?)?;

    let events = EventStore::open(&events_db)?;
    let critical_events = events.summary()?.critical_events;

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

fn cutover_rollback(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let file = PathBuf::from(required_arg(args, "--file")?);
    let capability_input = required_arg(args, "--capability")?;
    let capability =
        parse_capability(&capability_input).ok_or_else(|| "invalid capability".to_string())?;

    let mut flags = CutoverFlags::load_or_default(&file)?;
    let (flag, name) = match capability {
        nexum::cutover::Capability::Routing => {
            (FlagName::RoutingControlPlane, "routing_control_plane")
        }
        nexum::cutover::Capability::Restore => {
            (FlagName::RestoreControlPlane, "restore_control_plane")
        }
        nexum::cutover::Capability::Attention => {
            (FlagName::AttentionControlPlane, "attention_control_plane")
        }
    };
    flags.set(flag, false);
    flags.save(&file)?;

    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "capability": capability_input,
            "flag": name,
            "rolled_back": true,
        }))?
    );
    Ok(())
}

fn run_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "restore" => run_restore(&args[1..]),
        "restore-capsule" => run_restore_capsule(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn stead_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        usage();
        std::process::exit(2);
    }

    match args[0].as_str() {
        "dispatch" => stead_dispatch(&args[1..]),
        "dispatch-batch" => stead_dispatch_batch(&args[1..]),
        "validate-events" => stead_validate_events(&args[1..]),
        _ => {
            usage();
            std::process::exit(2);
        }
    }
}

fn stead_dispatch(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let capsule_db = PathBuf::from(required_arg(args, "--capsule-db")?);
    let event = parse_dispatch_event(&required_arg(args, "--event-json")?)
        .map_err(|error| error.to_string())?;
    let summary = dispatch_stead_event(
        &capsule_db,
        event,
        optional_arg(args, "--terminal"),
        optional_arg(args, "--editor"),
        optional_arg(args, "--browser"),
        optional_arg(args, "--routing-socket").map(PathBuf::from),
        PathBuf::from(required_arg(args, "--tls-dir")?),
        PathBuf::from(required_arg(args, "--events-db")?),
    )?;

    println!("{}", serde_json::to_string(&summary)?);
    Ok(())
}

#[derive(Debug, Serialize)]
struct SteadBatchResult {
    capsule_id: String,
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SteadBatchReport {
    processed: u32,
    succeeded: u32,
    failed: u32,
    results: Vec<SteadBatchResult>,
}

fn stead_dispatch_batch(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let capsule_db = PathBuf::from(required_arg(args, "--capsule-db")?);
    let events = parse_dispatch_events(&required_arg(args, "--events-json")?)
        .map_err(|error| error.to_string())?;
    let terminal = optional_arg(args, "--terminal");
    let editor = optional_arg(args, "--editor");
    let browser = optional_arg(args, "--browser");
    let routing_socket = optional_arg(args, "--routing-socket").map(PathBuf::from);
    let tls_dir = PathBuf::from(required_arg(args, "--tls-dir")?);
    let events_db = PathBuf::from(required_arg(args, "--events-db")?);

    let mut succeeded = 0u32;
    let mut failed = 0u32;
    let mut results = Vec::with_capacity(events.len());

    for event in events {
        let capsule_id = event.capsule_id.clone();
        match dispatch_stead_event(
            &capsule_db,
            event,
            terminal.clone(),
            editor.clone(),
            browser.clone(),
            routing_socket.clone(),
            tls_dir.clone(),
            events_db.clone(),
        ) {
            Ok(_) => {
                succeeded += 1;
                results.push(SteadBatchResult {
                    capsule_id,
                    ok: true,
                    error: None,
                });
            }
            Err(error) => {
                failed += 1;
                results.push(SteadBatchResult {
                    capsule_id,
                    ok: false,
                    error: Some(error.to_string()),
                });
            }
        }
    }

    let report = SteadBatchReport {
        processed: (succeeded + failed),
        succeeded,
        failed,
        results,
    };
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn dispatch_stead_event(
    capsule_db: &PathBuf,
    event: DispatchEvent,
    terminal_override: Option<String>,
    editor_override: Option<String>,
    browser_override: Option<String>,
    routing_socket: Option<PathBuf>,
    tls_dir: PathBuf,
    events_db: PathBuf,
) -> Result<nexum::runflow::RestoreRunSummary, Box<dyn std::error::Error>> {
    let store = CapsuleStore::open(capsule_db)?;
    let capsule = store
        .get(&event.capsule_id)?
        .ok_or_else(|| format!("unknown capsule: {}", event.capsule_id))?;

    let terminal_cmd = if let Some(terminal) = terminal_override {
        terminal
    } else if !capsule.repo_path.is_empty() {
        format!("cd {} && nix develop", capsule.repo_path)
    } else {
        return Err(
            "missing restore surfaces: provide --terminal and --editor or set capsule repo_path"
                .into(),
        );
    };

    let editor_target = if let Some(editor) = editor_override {
        editor
    } else if !capsule.repo_path.is_empty() {
        capsule.repo_path.clone()
    } else {
        return Err(
            "missing restore surfaces: provide --terminal and --editor or set capsule repo_path"
                .into(),
        );
    };

    let browser_url = browser_override.unwrap_or_else(|| format!("https://{}", capsule.domain()));

    run_restore_flow(RestoreRunInput {
        capsule_id: capsule.capsule_id,
        display_name: capsule.display_name,
        workspace: capsule.workspace,
        signal: event.signal,
        terminal_cmd,
        editor_target,
        browser_url,
        route_upstream: event.upstream,
        routing_socket,
        identity_collision: event.identity_collision,
        high_risk_secret_workflow: event.high_risk_secret_workflow,
        force_isolated_mode: event.force_isolated_mode,
        capsule_db: Some(capsule_db.to_path_buf()),
        tls_dir,
        events_db,
    })
    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })
}

fn stead_validate_events(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let events = parse_dispatch_events(&required_arg(args, "--events-json")?)
        .map_err(|error| error.to_string())?;
    let capsule_db = optional_arg(args, "--capsule-db").map(PathBuf::from);
    let event_count = events.len();

    let mut capsule_ids = events
        .iter()
        .map(|event| event.capsule_id.clone())
        .collect::<Vec<_>>();
    capsule_ids.sort();
    capsule_ids.dedup();

    let mut missing_capsule_ids = Vec::new();
    if let Some(capsule_db) = capsule_db {
        let store = CapsuleStore::open(&capsule_db)?;
        for capsule_id in &capsule_ids {
            if store.get(capsule_id)?.is_none() {
                missing_capsule_ids.push(capsule_id.clone());
            }
        }
    }
    let valid = missing_capsule_ids.is_empty();

    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "valid": valid,
            "event_count": event_count,
            "capsule_ids": capsule_ids,
            "missing_capsule_ids": missing_capsule_ids,
        }))?
    );
    Ok(())
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
        capsule_db: optional_arg(args, "--capsule-db").map(PathBuf::from),
        tls_dir: PathBuf::from(required_arg(args, "--tls-dir")?),
        events_db: PathBuf::from(required_arg(args, "--events-db")?),
    })?;

    println!("{}", serde_json::to_string(&summary)?);
    Ok(())
}

fn run_restore_capsule(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let capsule_db = PathBuf::from(required_arg(args, "--capsule-db")?);
    let capsule_id = required_arg(args, "--capsule-id")?;
    let signal = parse_signal(&required_arg(args, "--signal")?)?;
    let route_upstream = required_arg(args, "--upstream")?;
    let routing_socket = optional_arg(args, "--routing-socket").map(PathBuf::from);
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

    let store = CapsuleStore::open(&capsule_db)?;
    let capsule = store
        .get(&capsule_id)?
        .ok_or_else(|| format!("unknown capsule: {capsule_id}"))?;

    let terminal_cmd = if let Some(terminal) = optional_arg(args, "--terminal") {
        terminal
    } else if !capsule.repo_path.is_empty() {
        format!("cd {} && nix develop", capsule.repo_path)
    } else {
        return Err(
            "missing restore surfaces: provide --terminal and --editor or set capsule repo_path"
                .into(),
        );
    };

    let editor_target = if let Some(editor) = optional_arg(args, "--editor") {
        editor
    } else if !capsule.repo_path.is_empty() {
        capsule.repo_path.clone()
    } else {
        return Err(
            "missing restore surfaces: provide --terminal and --editor or set capsule repo_path"
                .into(),
        );
    };

    let browser_url =
        optional_arg(args, "--browser").unwrap_or_else(|| format!("https://{}", capsule.domain()));

    let summary = run_restore_flow(RestoreRunInput {
        capsule_id: capsule.capsule_id,
        display_name: capsule.display_name,
        workspace: capsule.workspace,
        signal,
        terminal_cmd,
        editor_target,
        browser_url,
        route_upstream,
        routing_socket,
        identity_collision,
        high_risk_secret_workflow,
        force_isolated_mode,
        capsule_db: Some(capsule_db),
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
        "nexumctl capsule create --db <path> --id <id> --name <name> --workspace <n> --mode <host_default|isolated_nix_shell> [--repo-path <path>]"
    );
    eprintln!("nexumctl capsule list --db <path>");
    eprintln!("nexumctl capsule export --db <path> --format <yaml>");
    eprintln!("nexumctl capsule rename --db <path> --id <id> --name <name>");
    eprintln!("nexumctl capsule set-repo --db <path> --id <id> --repo-path <path>");
    eprintln!(
        "nexumctl capsule set-state --db <path> --id <id> --state <creating|ready|restoring|degraded|archived>"
    );
    eprintln!("nexumctl capsule allocate-port --db <path> --id <id> --start <u16> --end <u16>");
    eprintln!("nexumctl capsule release-ports --db <path> --id <id>");
    eprintln!(
        "nexumctl flags set --file <path> [--shadow true|false] [--routing true|false] [--restore true|false] [--attention true|false]"
    );
    eprintln!("nexumctl flags show --file <path>");
    eprintln!("nexumctl parity compare --primary-json <json> --candidate-json <json>");
    eprintln!("nexumctl events summary --db <path>");
    eprintln!(
        "nexumctl events list --db <path> [--capsule-id <id>] [--level <level>] [--limit <n>]"
    );
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
    eprintln!(
        "nexumctl stead dispatch --capsule-db <path> --event-json <json> [--terminal <cmd>] [--editor <path>] [--browser <url>] [--routing-socket <path>] --tls-dir <path> --events-db <path>"
    );
    eprintln!(
        "nexumctl stead dispatch-batch --capsule-db <path> --events-json <json-array> [--terminal <cmd>] [--editor <path>] [--browser <url>] [--routing-socket <path>] --tls-dir <path> --events-db <path>"
    );
    eprintln!("nexumctl stead validate-events --events-json <json-array> [--capsule-db <path>]");
    eprintln!(
        "nexumctl supervisor status --capsule-db <path> --events-db <path> --flags-file <path>"
    );
    eprintln!(
        "nexumctl supervisor blockers --capsule-db <path> --events-db <path> [--critical-threshold <u32>]"
    );
    eprintln!("nexumctl tls ensure --dir <path> --domain <domain> [--validity-days <days>]");
    eprintln!("nexumctl tls rotate --dir <path> --domain <domain> --threshold-days <days>");
    eprintln!(
        "nexumctl cutover apply --file <path> --capability <routing|restore|attention> --parity-score <f64> --min-parity-score <f64> --critical-events <u32> --max-critical-events <u32> --shadow-mode <true|false>"
    );
    eprintln!(
        "nexumctl cutover apply-from-events --file <path> --capability <routing|restore|attention> --parity-score <f64> --min-parity-score <f64> --events-db <path> --capsule-id <id> --max-critical-events <u32> --shadow-mode <true|false>"
    );
    eprintln!(
        "nexumctl cutover apply-from-summary --file <path> --capability <routing|restore|attention> --parity-score <f64> --min-parity-score <f64> --events-db <path> --max-critical-events <u32> --shadow-mode <true|false>"
    );
    eprintln!("nexumctl cutover rollback --file <path> --capability <routing|restore|attention>");
    eprintln!(
        "nexumctl run restore --capsule-id <id> --name <name> --workspace <n> --signal <needs_decision|critical_failure|passive_completion> --terminal <cmd> --editor <path> --browser <url> --upstream <host:port> [--routing-socket <path>] [--identity-collision true|false] [--high-risk-secret true|false] [--force-isolated true|false] [--capsule-db <path>] --tls-dir <path> --events-db <path>"
    );
    eprintln!(
        "nexumctl run restore-capsule --capsule-db <path> --capsule-id <id> --signal <needs_decision|critical_failure|passive_completion> --upstream <host:port> [--terminal <cmd>] [--editor <path>] [--browser <url>] [--routing-socket <path>] [--identity-collision true|false] [--high-risk-secret true|false] [--force-isolated true|false] --tls-dir <path> --events-db <path>"
    );
}
