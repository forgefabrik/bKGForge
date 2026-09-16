//! Provider-neutral browser contracts. Core remains independent of CDP and Ego Lite.
use bkgforge_http::{CaptureClient, Method, Request};
use bkgforge_security::NetworkPolicy;
use std::collections::BTreeMap;
use std::process::{Child, Command, Stdio};
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
    BrowserLaunch,
    BrowserConnect,
    TargetDiscovery,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeMode {
    Launched,
    Connected,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserVersion {
    pub product: String,
    pub websocket_debugger_url: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredTarget {
    pub target_id: String,
    pub kind: String,
    pub url: String,
    pub title: String,
}
pub struct BrowserRuntime {
    mode: RuntimeMode,
    endpoint: String,
    version: BrowserVersion,
    child: Option<Child>,
}
impl BrowserRuntime {
    /// Launches Chromium with CDP enabled, then verifies the live debugging endpoint.
    ///
    /// # Errors
    /// Returns an error when Chromium cannot be started or does not expose CDP.
    pub fn launch(executable: &str, port: u16) -> Result<Self, String> {
        let child = Command::new(executable)
            .args([
                format!("--remote-debugging-port={port}"),
                "--no-first-run".into(),
                "--no-default-browser-check".into(),
                "about:blank".into(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("unable to launch Chromium: {error}"))?;
        let endpoint = format!("http://127.0.0.1:{port}");
        let version = fetch_version(&endpoint)?;
        Ok(Self {
            mode: RuntimeMode::Launched,
            endpoint,
            version,
            child: Some(child),
        })
    }
    /// Connects to a live Chromium CDP HTTP endpoint and verifies its version response.
    ///
    /// # Errors
    /// Returns an error when the endpoint is unavailable or is not Chromium CDP.
    pub fn connect(endpoint: impl Into<String>) -> Result<Self, String> {
        let endpoint = endpoint.into().trim_end_matches('/').to_owned();
        let version = fetch_version(&endpoint)?;
        Ok(Self {
            mode: RuntimeMode::Connected,
            endpoint,
            version,
            child: None,
        })
    }
    #[must_use]
    pub const fn mode(&self) -> RuntimeMode {
        self.mode
    }
    #[must_use]
    pub fn version(&self) -> &BrowserVersion {
        &self.version
    }
    /// Discovers live page targets through Chromium's CDP HTTP endpoint.
    ///
    /// # Errors
    /// Returns an error when target discovery fails or the response is malformed.
    pub fn targets(&self) -> Result<Vec<DiscoveredTarget>, String> {
        let body = cdp_get(&self.endpoint, "/json/list")?;
        Ok(parse_targets(&body))
    }
    #[must_use]
    pub fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::BrowserConnect,
            Capability::TargetDiscovery,
            if self.mode == RuntimeMode::Launched {
                Capability::BrowserLaunch
            } else {
                Capability::BrowserConnect
            },
        ]
    }
    /// Terminates only a Chromium process created by this runtime.
    ///
    /// # Errors
    /// Returns an error if the owned Chromium process cannot be terminated.
    pub fn shutdown(&mut self) -> Result<(), String> {
        if let Some(child) = &mut self.child {
            child
                .kill()
                .map_err(|error| format!("unable to stop Chromium: {error}"))?;
            let _ = child.wait();
        }
        self.child = None;
        Ok(())
    }
}
impl Drop for BrowserRuntime {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
fn cdp_get(endpoint: &str, path: &str) -> Result<String, String> {
    let exchange = CaptureClient::new(NetworkPolicy {
        allow_private: true,
    })
    .execute(Request {
        method: Method::Get,
        url: format!("{endpoint}{path}"),
        headers: BTreeMap::default(),
        body: vec![],
    })?;
    if exchange.response.status != 200 {
        return Err(format!(
            "CDP endpoint returned HTTP {}",
            exchange.response.status
        ));
    }
    String::from_utf8(exchange.response.body).map_err(|_| "CDP response was not UTF-8 JSON".into())
}
fn fetch_version(endpoint: &str) -> Result<BrowserVersion, String> {
    let json = cdp_get(endpoint, "/json/version")?;
    Ok(BrowserVersion {
        product: json_field(&json, "Browser").ok_or("CDP version response has no Browser field")?,
        websocket_debugger_url: json_field(&json, "webSocketDebuggerUrl")
            .ok_or("CDP version response has no webSocketDebuggerUrl field")?,
    })
}
fn json_field(value: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = value.find(&marker)? + marker.len();
    let tail = &value[start..];
    let end = tail.find('"')?;
    Some(tail[..end].replace("\\/", "/"))
}
fn parse_targets(json: &str) -> Vec<DiscoveredTarget> {
    json.split("{\"")
        .filter_map(|object| {
            let value = format!("{{\"{object}");
            Some(DiscoveredTarget {
                target_id: json_field(&value, "id")?,
                kind: json_field(&value, "type")?,
                url: json_field(&value, "url")?,
                title: json_field(&value, "title").unwrap_or_default(),
            })
        })
        .collect()
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
