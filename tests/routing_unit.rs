use nexum::routing::{RouteCommand, RouteOutcome, RouterState};

#[test]
fn register_and_resolve_roundtrip() {
    let mut state = RouterState::default();

    let register = state.handle(RouteCommand::Register {
        capsule_id: "cap-a".into(),
        domain: "cap-a.nexum.local".into(),
        upstream: "127.0.0.1:4302".into(),
    });
    assert!(
        matches!(register, RouteOutcome::Registered { domain } if domain == "cap-a.nexum.local")
    );

    let resolved = state.handle(RouteCommand::Resolve {
        domain: "cap-a.nexum.local".into(),
    });

    match resolved {
        RouteOutcome::Resolved { route } => {
            let route = route.expect("route should exist");
            assert_eq!(route.capsule_id, "cap-a");
            assert_eq!(route.upstream, "127.0.0.1:4302");
            assert_eq!(route.tls_mode, "self_signed");
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn duplicate_domain_from_different_capsule_is_rejected() {
    let mut state = RouterState::default();
    state.handle(RouteCommand::Register {
        capsule_id: "cap-a".into(),
        domain: "shared.nexum.local".into(),
        upstream: "127.0.0.1:4302".into(),
    });

    let result = state.handle(RouteCommand::Register {
        capsule_id: "cap-b".into(),
        domain: "shared.nexum.local".into(),
        upstream: "127.0.0.1:4303".into(),
    });

    assert!(matches!(result, RouteOutcome::Error { code, .. } if code == "domain_conflict"));
}

#[test]
fn list_routes_is_sorted_by_domain() {
    let mut state = RouterState::default();
    state.handle(RouteCommand::Register {
        capsule_id: "cap-z".into(),
        domain: "zeta.nexum.local".into(),
        upstream: "127.0.0.1:4310".into(),
    });
    state.handle(RouteCommand::Register {
        capsule_id: "cap-a".into(),
        domain: "alpha.nexum.local".into(),
        upstream: "127.0.0.1:4301".into(),
    });

    let result = state.handle(RouteCommand::List);
    match result {
        RouteOutcome::Listed { routes } => {
            let domains = routes.iter().map(|r| r.domain.as_str()).collect::<Vec<_>>();
            assert_eq!(domains, vec!["alpha.nexum.local", "zeta.nexum.local"]);
        }
        other => panic!("unexpected outcome: {other:?}"),
    }
}
