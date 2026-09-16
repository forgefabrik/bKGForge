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
/// Shared outbound-network policy for HTTP and browser adapters.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NetworkPolicy {
    pub allow_private: bool,
}
impl NetworkPolicy {
    /// Checks a parsed host before connecting. Hostname resolution must be performed by adapters.
    ///
    /// # Errors
    ///
    /// Returns an error when the target is local and private targets are not allowed.
    pub fn check_host(&self, host: &str) -> Result<(), String> {
        if self.allow_private {
            return Ok(());
        }
        let blocked = host == "localhost"
            || host == "::1"
            || host.starts_with("127.")
            || host.starts_with("10.")
            || host.starts_with("192.168.")
            || host.starts_with("169.254.")
            || host.starts_with("fc")
            || host.starts_with("fd")
            || host.starts_with("fe80:")
            || host.split('.').nth(1) == Some("16") && host.starts_with("172.")
            || (host.starts_with("172.")
                && matches!(
                    host.split('.').nth(1).and_then(|n| n.parse::<u8>().ok()),
                    Some(16..=31)
                ));
        if blocked {
            Err(format!("SSRF policy rejected host {host}"))
        } else {
            Ok(())
        }
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
