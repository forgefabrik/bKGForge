//! Normalized security-event decisions, usable for real, replay, and synthetic runs.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityDecision {
    Allow,
    Monitor,
    Challenge,
    Pause,
    Block,
    HumanReview,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityEvent {
    pub event_type: String,
    pub identity_fingerprint: Option<String>,
    pub workflow_fingerprint: String,
    pub repeated_failures: u32,
    pub velocity_per_minute: u32,
}
#[must_use]
pub fn decide(event: &SecurityEvent) -> SecurityDecision {
    if event.repeated_failures >= 5 {
        SecurityDecision::Block
    } else if event.velocity_per_minute >= 20 {
        SecurityDecision::HumanReview
    } else if event.repeated_failures >= 3 {
        SecurityDecision::Pause
    } else if event.velocity_per_minute >= 10 {
        SecurityDecision::Monitor
    } else {
        SecurityDecision::Allow
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_failure_blocks() {
        assert_eq!(
            decide(&SecurityEvent {
                event_type: "authentication".into(),
                identity_fingerprint: None,
                workflow_fingerprint: "x".into(),
                repeated_failures: 5,
                velocity_per_minute: 1
            }),
            SecurityDecision::Block
        );
    }
}
