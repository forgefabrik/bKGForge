//! Provider-neutral browser contracts. Core remains independent of CDP and Ego Lite.
use bkgforge_security::NetworkPolicy;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tab {
    pub id: String,
    pub url: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserSession {
    pub session_id: String,
    pub active_tab: String,
    pub tabs: Vec<Tab>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessibilityNode {
    pub role: String,
    pub name: String,
    pub coordinates: Option<(i32, i32)>,
    pub state: Vec<String>,
    pub children: Vec<AccessibilityNode>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkEvent {
    pub kind: String,
    pub url: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    Navigate,
    CurrentUrl,
    Snapshot,
    Text,
    Screenshot,
    Click,
    Fill,
    Type,
    Key,
    Scroll,
    Wait,
    Tabs,
    NetworkEvents,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Key {
    Enter,
    Escape,
    Tab,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}
pub trait BrowserBackend {
    /// # Errors
    /// Returns an adapter/runtime connection error.
    fn connect(&mut self) -> Result<BrowserSession, String>;
    fn capabilities(&self) -> Vec<Capability>;
    /// # Errors
    /// Returns an SSRF-policy or adapter navigation error.
    fn navigate(&mut self, url: &str) -> Result<(), String>;
    /// # Errors
    /// Returns an adapter observation error.
    fn current_url(&self) -> Result<String, String>;
    /// # Errors
    /// Returns an adapter observation error.
    fn snapshot(&self) -> Result<AccessibilityNode, String>;
    /// # Errors
    /// Returns an adapter observation error.
    fn text(&self) -> Result<String, String>;
    /// # Errors
    /// Returns an adapter observation error.
    fn screenshot(&self) -> Result<Vec<u8>, String>;
    /// # Errors
    /// Returns an adapter interaction error.
    fn click(&mut self, target: &str) -> Result<(), String>;
    /// # Errors
    /// Returns an adapter interaction error.
    fn fill(&mut self, target: &str, value: &str) -> Result<(), String>;
    /// # Errors
    /// Returns an adapter interaction error.
    fn type_text(&mut self, value: &str) -> Result<(), String>;
    /// # Errors
    /// Returns an adapter interaction error.
    fn key(&mut self, key: Key) -> Result<(), String>;
    /// # Errors
    /// Returns an adapter interaction error.
    fn scroll(&mut self, x: i32, y: i32) -> Result<(), String>;
    /// # Errors
    /// Returns an adapter wait error.
    fn wait(&mut self, milliseconds: u64) -> Result<(), String>;
    /// # Errors
    /// Returns an adapter tab query error.
    fn tabs(&self) -> Result<Vec<Tab>, String>;
    /// # Errors
    /// Returns an adapter network-event query error.
    fn network_events(&self) -> Result<Vec<NetworkEvent>, String>;
}
/// A clean-room Rust backend boundary informed only by public browser-automation ideas.
///
/// This type contains no Ego Lite source code and does not execute a command/protocol until a
/// real Rust CDP transport is supplied. Keeping the unavailable state explicit prevents callers
/// from treating a design-time adapter as a running browser.
pub struct EgoLiteBackend {
    pub cdp_endpoint: String,
    pub policy: NetworkPolicy,
}
impl EgoLiteBackend {
    #[must_use]
    pub fn new(cdp_endpoint: impl Into<String>, policy: NetworkPolicy) -> Self {
        Self {
            cdp_endpoint: cdp_endpoint.into(),
            policy,
        }
    }
    fn unavailable(&self) -> String {
        format!(
            "Ego Lite Rust backend requires a configured CDP transport for {}",
            self.cdp_endpoint
        )
    }
}
impl BrowserBackend for EgoLiteBackend {
    fn connect(&mut self) -> Result<BrowserSession, String> {
        Err(self.unavailable())
    }
    fn capabilities(&self) -> Vec<Capability> {
        vec![]
    }
    fn navigate(&mut self, url: &str) -> Result<(), String> {
        let host = url
            .strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .and_then(|u| u.split('/').next())
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        self.policy.check_host(host)?;
        Err(self.unavailable())
    }
    fn current_url(&self) -> Result<String, String> {
        Err(self.unavailable())
    }
    fn snapshot(&self) -> Result<AccessibilityNode, String> {
        Err(self.unavailable())
    }
    fn text(&self) -> Result<String, String> {
        Err(self.unavailable())
    }
    fn screenshot(&self) -> Result<Vec<u8>, String> {
        Err(self.unavailable())
    }
    fn click(&mut self, _target: &str) -> Result<(), String> {
        Err(self.unavailable())
    }
    fn fill(&mut self, _target: &str, _value: &str) -> Result<(), String> {
        Err(self.unavailable())
    }
    fn type_text(&mut self, _value: &str) -> Result<(), String> {
        Err(self.unavailable())
    }
    fn key(&mut self, _key: Key) -> Result<(), String> {
        Err(self.unavailable())
    }
    fn scroll(&mut self, _x: i32, _y: i32) -> Result<(), String> {
        Err(self.unavailable())
    }
    fn wait(&mut self, _milliseconds: u64) -> Result<(), String> {
        Err(self.unavailable())
    }
    fn tabs(&self) -> Result<Vec<Tab>, String> {
        Err(self.unavailable())
    }
    fn network_events(&self) -> Result<Vec<NetworkEvent>, String> {
        Err(self.unavailable())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_applies_shared_policy() {
        let mut b = EgoLiteBackend::new("ws://browser:9222", NetworkPolicy::default());
        assert!(
            b.navigate("http://127.0.0.1/")
                .unwrap_err()
                .contains("SSRF")
        );
    }
}
