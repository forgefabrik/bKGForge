//! Versioned, deterministic and sanitized capture records.
use std::collections::BTreeMap;

pub const TRACE_FORMAT: &str = "bkgforge-trace/1";
pub type Headers = BTreeMap<String, String>;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceMetadata {
    pub id: String,
    pub source: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Exchange {
    pub request: HttpMessage,
    pub response: HttpResponse,
    pub redirect_chain: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpMessage {
    pub method: String,
    pub url: String,
    pub headers: Headers,
    pub body: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub final_url: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceEvent {
    Navigation { url: String },
    Interaction { action: String, target: String },
    NetworkRequest(HttpMessage),
    NetworkResponse(HttpResponse),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trace {
    pub format: String,
    pub metadata: TraceMetadata,
    pub exchanges: Vec<Exchange>,
    pub events: Vec<TraceEvent>,
}
impl Trace {
    #[must_use]
    pub fn new(metadata: TraceMetadata) -> Self {
        Self {
            format: TRACE_FORMAT.into(),
            metadata,
            exchanges: vec![],
            events: vec![],
        }
    }
}
#[must_use]
pub fn sanitize_headers(headers: &Headers) -> Headers {
    headers
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                if ["authorization", "cookie", "set-cookie", "x-api-key"]
                    .contains(&k.to_ascii_lowercase().as_str())
                {
                    "[REDACTED]".into()
                } else {
                    v.clone()
                },
            )
        })
        .collect()
}
pub trait TraceStore {
    fn put(&mut self, trace: Trace);
    fn get(&self, id: &str) -> Option<&Trace>;
}
pub trait TraceReplaySource {
    fn replay_input(&self, id: &str) -> Option<ReplayInput>;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayInput {
    pub trace: Trace,
}
#[derive(Default)]
pub struct InMemoryTraceStore {
    traces: BTreeMap<String, Trace>,
}
impl TraceStore for InMemoryTraceStore {
    fn put(&mut self, trace: Trace) {
        self.traces.insert(trace.metadata.id.clone(), trace);
    }
    fn get(&self, id: &str) -> Option<&Trace> {
        self.traces.get(id)
    }
}
impl TraceReplaySource for InMemoryTraceStore {
    fn replay_input(&self, id: &str) -> Option<ReplayInput> {
        self.get(id).cloned().map(|trace| ReplayInput { trace })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_sensitive_headers() {
        let mut h = Headers::new();
        h.insert("Authorization".into(), "secret".into());
        assert_eq!(sanitize_headers(&h)["Authorization"], "[REDACTED]");
    }
    #[test]
    fn stores_replay_consumer_input() {
        let mut s = InMemoryTraceStore::default();
        s.put(Trace::new(TraceMetadata {
            id: "t".into(),
            source: "test".into(),
        }));
        assert!(s.replay_input("t").is_some());
    }
}
