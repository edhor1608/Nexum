use nexum::routing::RouteOutcome;

#[test]
fn snapshot_routing_cli_json_contract() {
    let outcome = RouteOutcome::Registered {
        domain: "alpha.nexum.local".into(),
    };

    let json = serde_json::to_string_pretty(&outcome).unwrap();
    insta::assert_snapshot!("routing_cli_json_contract", json);
}
