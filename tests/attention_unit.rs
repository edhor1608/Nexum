use nexum::{
    attention::{AttentionChannel, AttentionEvent, AttentionPolicy, AttentionPriority},
    restore::SignalType,
};

#[test]
fn maps_critical_failure_to_blocking_banner_and_sound() {
    let policy = AttentionPolicy::default();
    let routed = policy.route(&AttentionEvent {
        capsule_id: "cap-a".into(),
        signal: SignalType::CriticalFailure,
        summary: "Build crashed".into(),
    });

    assert_eq!(routed.priority, AttentionPriority::Blocking);
    assert_eq!(routed.channel, AttentionChannel::BannerAndSound);
    assert!(routed.requires_ack);
}

#[test]
fn maps_needs_decision_to_banner_only() {
    let policy = AttentionPolicy::default();
    let routed = policy.route(&AttentionEvent {
        capsule_id: "cap-b".into(),
        signal: SignalType::NeedsDecision,
        summary: "Approve migration".into(),
    });

    assert_eq!(routed.priority, AttentionPriority::Active);
    assert_eq!(routed.channel, AttentionChannel::Banner);
    assert!(routed.requires_ack);
}

#[test]
fn maps_passive_completion_to_feed_without_ack() {
    let policy = AttentionPolicy::default();
    let routed = policy.route(&AttentionEvent {
        capsule_id: "cap-c".into(),
        signal: SignalType::PassiveCompletion,
        summary: "Task done".into(),
    });

    assert_eq!(routed.priority, AttentionPriority::Passive);
    assert_eq!(routed.channel, AttentionChannel::Feed);
    assert!(!routed.requires_ack);
}
