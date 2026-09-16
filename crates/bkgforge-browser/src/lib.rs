//! Provider-neutral browser contracts. Core remains independent of CDP and mini-browser.
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
}
/// Runtime adapter placeholder. It only reports availability by checking the configured binary;
/// no browser command is claimed until the upstream mini-browser protocol is pinned.
pub struct MiniBrowserBackend {
    pub executable: String,
    pub policy: NetworkPolicy,
}
impl MiniBrowserBackend {
    #[must_use]
    pub fn new(executable: impl Into<String>, policy: NetworkPolicy) -> Self {
        Self {
            executable: executable.into(),
            policy,
        }
    }
    fn unavailable(&self) -> String {
        format!(
            "mini-browser backend requires runtime executable {} and a pinned upstream protocol",
            self.executable
        )
    }
}
impl BrowserBackend for MiniBrowserBackend {
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
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_applies_shared_policy() {
        let mut b = MiniBrowserBackend::new("mini-browser", NetworkPolicy::default());
        assert!(
            b.navigate("http://127.0.0.1/")
                .unwrap_err()
                .contains("SSRF")
        );
    }
}
