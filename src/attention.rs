use serde::{Deserialize, Serialize};

use crate::restore::SignalType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionPriority {
    Blocking,
    Active,
    Passive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionChannel {
    BannerAndSound,
    Banner,
    Feed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttentionEvent {
    pub capsule_id: String,
    pub signal: SignalType,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutedAttention {
    pub capsule_id: String,
    pub priority: AttentionPriority,
    pub channel: AttentionChannel,
    pub requires_ack: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct AttentionPolicy;

impl AttentionPolicy {
    pub fn route(&self, event: &AttentionEvent) -> RoutedAttention {
        let (priority, channel, requires_ack) = match event.signal {
            SignalType::CriticalFailure => (
                AttentionPriority::Blocking,
                AttentionChannel::BannerAndSound,
                true,
            ),
            SignalType::NeedsDecision => {
                (AttentionPriority::Active, AttentionChannel::Banner, true)
            }
            SignalType::PassiveCompletion => {
                (AttentionPriority::Passive, AttentionChannel::Feed, false)
            }
        };

        RoutedAttention {
            capsule_id: event.capsule_id.clone(),
            priority,
            channel,
            requires_ack,
            summary: event.summary.clone(),
        }
    }
}
