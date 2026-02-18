use nexum::routing::{RouteCommand, RouterState};

#[test]
fn snapshot_list_routes_contract() {
    let mut state = RouterState::default();
    state.handle(RouteCommand::Register {
        capsule_id: "cap-a".into(),
        domain: "alpha.nexum.local".into(),
        upstream: "127.0.0.1:4301".into(),
    });
    state.handle(RouteCommand::Register {
        capsule_id: "cap-b".into(),
        domain: "beta.nexum.local".into(),
        upstream: "127.0.0.1:4302".into(),
    });

    let listed = state.handle(RouteCommand::List);
    insta::assert_yaml_snapshot!("routing_list_contract", listed);
}
