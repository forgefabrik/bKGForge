//! Normalized security-event decisions, usable for real, replay, and synthetic runs.

use std::net::{IpAddr, ToSocketAddrs};

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
    /// Resolves and checks every destination address before connecting.
    ///
    /// # Errors
    ///
    /// Returns an error when resolution fails, yields no address, or an address is local/private.
    pub fn check_host(&self, host: &str) -> Result<(), String> {
        if self.allow_private {
            return Ok(());
        }
        if host.eq_ignore_ascii_case("localhost") {
            return Err("SSRF policy rejected host localhost".into());
        }
        if let Ok(ip) = host.parse::<IpAddr>() {
            return self.check_ip(ip);
        }
        let addresses = (host, 0)
            .to_socket_addrs()
            .map_err(|error| format!("unable to resolve {host}: {error}"))?;
        let mut found = false;
        for address in addresses {
            found = true;
            self.check_ip(address.ip())?;
        }
        if found {
            Ok(())
        } else {
            Err(format!("DNS resolution returned no addresses for {host}"))
        }
    }

    /// Checks a resolved address. Adapters should call this again after every redirect.
    ///
    /// # Errors
    ///
    /// Returns an error when the address is non-public and private targets are not allowed.
    pub fn check_ip(&self, ip: IpAddr) -> Result<(), String> {
        if self.allow_private {
            return Ok(());
        }
        let blocked = match ip {
            IpAddr::V4(ip) => {
                ip.is_private()
                    || ip.is_loopback()
                    || ip.is_link_local()
                    || ip.is_unspecified()
                    || ip.is_multicast()
                    || ip.is_broadcast()
            }
            IpAddr::V6(ip) => {
                ip.is_loopback()
                    || ip.is_unspecified()
                    || ip.is_unique_local()
                    || ip.is_unicast_link_local()
                    || ip.is_multicast()
            }
        };
        if blocked {
            Err(format!("SSRF policy rejected address {ip}"))
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
    #[test]
    fn rejects_private_address_ranges() {
        let policy = NetworkPolicy::default();
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.1.1",
            "::1",
            "fd00::1",
        ] {
            assert!(policy.check_host(ip).is_err(), "{ip} should be rejected");
        }
    }
}
