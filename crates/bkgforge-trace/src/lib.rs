//! Versioned, sanitized discovery trace contracts.
//!
//! The documented line format is deliberately dependency-free and deterministic.
//! This crate performs no HTTP or browser I/O.

use bkgforge_core::{ExecutionMode, TraceId};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Write};

pub const SUPPORTED_TRACE_VERSION: u16 = 1;
const FORMAT_HEADER: &str = "bkgforge-trace/1";
const REDACTED_HEADER_VALUE: &str = "<redacted>";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceMetadata {
    pub trace_version: u16,
    pub trace_id: TraceId,
    pub created_at: String,
    pub source_url: String,
    pub execution_mode: TraceExecutionMode,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceExecutionMode {
    Real,
    Simulation,
}
impl TraceExecutionMode {
    fn encoded(self) -> &'static str {
        match self {
            Self::Real => "real",
            Self::Simulation => "simulation",
        }
    }
    fn decode(value: &str) -> Result<Self, TraceError> {
        match value {
            "real" => Ok(Self::Real),
            "simulation" => Ok(Self::Simulation),
            _ => Err(TraceError::Format("unknown trace execution mode".into())),
        }
    }
}
impl TryFrom<ExecutionMode> for TraceExecutionMode {
    type Error = TraceError;
    fn try_from(value: ExecutionMode) -> Result<Self, Self::Error> {
        match value {
            ExecutionMode::Real => Ok(Self::Real),
            ExecutionMode::Simulation => Ok(Self::Simulation),
            ExecutionMode::Replay => Err(TraceError::ReplayCannotProduceTrace),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}
impl HttpMethod {
    fn encoded(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }
    fn decode(value: &str) -> Result<Self, TraceError> {
        match value {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "PATCH" => Ok(Self::Patch),
            "DELETE" => Ok(Self::Delete),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            _ => Err(TraceError::Format("unknown HTTP method".into())),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedRequest {
    pub method: HttpMethod,
    pub url: String,
    pub normalized_url: String,
    pub path: String,
    pub query: BTreeMap<String, Vec<String>>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Option<String>,
    pub content_type: Option<String>,
    pub duration_ms: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedExchange {
    pub request: CapturedRequest,
    pub response: CapturedResponse,
    pub timestamp: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceNavigation {
    pub sequence: u64,
    pub target: String,
    pub phase: NavigationPhase,
    pub timestamp: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationPhase {
    Started,
    Completed,
    Failed,
}
impl NavigationPhase {
    fn encoded(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
    fn decode(value: &str) -> Result<Self, TraceError> {
        match value {
            "started" => Ok(Self::Started),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(TraceError::Format("unknown navigation phase".into())),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceEvent {
    pub sequence: u64,
    pub kind: String,
    pub timestamp: String,
    pub message: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trace {
    pub metadata: TraceMetadata,
    pub navigation: Vec<TraceNavigation>,
    pub exchanges: Vec<CapturedExchange>,
    pub events: Vec<TraceEvent>,
}

impl Trace {
    /// Validates the trace version.
    ///
    /// # Errors
    /// Returns [`TraceError::UnsupportedVersion`] for an unknown format version.
    pub fn validate(&self) -> Result<(), TraceError> {
        if self.metadata.trace_version != SUPPORTED_TRACE_VERSION {
            return Err(TraceError::UnsupportedVersion {
                found: self.metadata.trace_version,
                supported: SUPPORTED_TRACE_VERSION,
            });
        }
        Ok(())
    }
    /// Produces a trace suitable for persistence by redacting sensitive headers.
    #[must_use]
    pub fn sanitized_for_persistence(&self) -> Self {
        let mut trace = self.clone();
        for exchange in &mut trace.exchanges {
            sanitize_headers(&mut exchange.request.headers);
            sanitize_headers(&mut exchange.response.headers);
        }
        trace
    }
    /// Serializes a validated, sanitized trace as the documented `bkgforge-trace/1` line format.
    ///
    /// # Errors
    /// Returns an error for an unsupported trace version.
    pub fn to_persisted_text(&self) -> Result<String, TraceError> {
        self.validate()?;
        let trace = self.sanitized_for_persistence();
        let mut lines = vec![FORMAT_HEADER.into()];
        lines.push(format!(
            "M|{}|{}|{}|{}",
            hex(&trace.metadata.trace_id.0),
            hex(&trace.metadata.created_at),
            hex(&trace.metadata.source_url),
            trace.metadata.execution_mode.encoded()
        ));
        for navigation in &trace.navigation {
            lines.push(format!(
                "N|{}|{}|{}|{}",
                navigation.sequence,
                navigation.phase.encoded(),
                hex(&navigation.target),
                hex(&navigation.timestamp)
            ));
        }
        for exchange in &trace.exchanges {
            lines.push(format!(
                "X|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                hex(&exchange.timestamp),
                exchange.request.method.encoded(),
                hex(&exchange.request.url),
                hex(&exchange.request.normalized_url),
                hex(&exchange.request.path),
                encode_query(&exchange.request.query),
                encode_map(&exchange.request.headers),
                encode_option(exchange.request.body.as_ref()),
                exchange.response.status,
                encode_map(&exchange.response.headers),
                encode_option(exchange.response.body.as_ref()),
                encode_option(exchange.response.content_type.as_ref()),
                exchange.response.duration_ms
            ));
        }
        for event in &trace.events {
            lines.push(format!(
                "E|{}|{}|{}|{}",
                event.sequence,
                hex(&event.kind),
                hex(&event.timestamp),
                encode_option(event.message.as_ref())
            ));
        }
        Ok(lines.join("\n") + "\n")
    }
    /// Parses the documented `bkgforge-trace/1` line format and defensively redacts headers.
    ///
    /// # Errors
    /// Returns an error for malformed content or unsupported trace versions.
    pub fn from_persisted_text(input: &str) -> Result<Self, TraceError> {
        let mut lines = input.lines();
        if lines.next() != Some(FORMAT_HEADER) {
            return Err(TraceError::Format("unsupported trace format header".into()));
        }
        let metadata_line = lines
            .next()
            .ok_or_else(|| TraceError::Format("missing metadata".into()))?;
        let metadata_fields = fields(metadata_line, "M", 4)?;
        let mut trace = Self {
            metadata: TraceMetadata {
                trace_version: SUPPORTED_TRACE_VERSION,
                trace_id: TraceId(unhex(metadata_fields[0])?),
                created_at: unhex(metadata_fields[1])?,
                source_url: unhex(metadata_fields[2])?,
                execution_mode: TraceExecutionMode::decode(metadata_fields[3])?,
            },
            navigation: Vec::new(),
            exchanges: Vec::new(),
            events: Vec::new(),
        };
        for line in lines {
            let kind = line.split('|').next().unwrap_or_default();
            match kind {
                "N" => {
                    let f = fields(line, "N", 4)?;
                    trace.navigation.push(TraceNavigation {
                        sequence: number(f[0])?,
                        phase: NavigationPhase::decode(f[1])?,
                        target: unhex(f[2])?,
                        timestamp: unhex(f[3])?,
                    });
                }
                "X" => {
                    let f = fields(line, "X", 13)?;
                    trace.exchanges.push(CapturedExchange {
                        timestamp: unhex(f[0])?,
                        request: CapturedRequest {
                            method: HttpMethod::decode(f[1])?,
                            url: unhex(f[2])?,
                            normalized_url: unhex(f[3])?,
                            path: unhex(f[4])?,
                            query: decode_query(f[5])?,
                            headers: decode_map(f[6])?,
                            body: decode_option(f[7])?,
                        },
                        response: CapturedResponse {
                            status: number(f[8])?,
                            headers: decode_map(f[9])?,
                            body: decode_option(f[10])?,
                            content_type: decode_option(f[11])?,
                            duration_ms: number(f[12])?,
                        },
                    });
                }
                "E" => {
                    let f = fields(line, "E", 4)?;
                    trace.events.push(TraceEvent {
                        sequence: number(f[0])?,
                        kind: unhex(f[1])?,
                        timestamp: unhex(f[2])?,
                        message: decode_option(f[3])?,
                    });
                }
                _ => return Err(TraceError::Format("unknown trace record".into())),
            }
        }
        trace.validate()?;
        Ok(trace.sanitized_for_persistence())
    }
    /// Creates immutable replay input for later discovery orchestration.
    ///
    /// # Errors
    /// Returns an error for unsupported trace versions.
    pub fn replay_input(&self) -> Result<ReplayInput, TraceError> {
        self.validate()?;
        Ok(ReplayInput {
            trace_id: self.metadata.trace_id.clone(),
            source_url: self.metadata.source_url.clone(),
            navigation: self.navigation.clone(),
            exchanges: self.sanitized_for_persistence().exchanges,
            events: self.events.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayInput {
    pub trace_id: TraceId,
    pub source_url: String,
    pub navigation: Vec<TraceNavigation>,
    pub exchanges: Vec<CapturedExchange>,
    pub events: Vec<TraceEvent>,
}
/// Persistence boundary for sanitized trace records.
pub trait TraceStore: Send + Sync {
    /// Saves a validated, sanitized trace.
    ///
    /// # Errors
    ///
    /// Returns a validation or storage error.
    fn save(&mut self, trace: &Trace) -> Result<(), TraceError>;
    /// Loads a trace by its immutable identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the trace does not exist or cannot be loaded.
    fn load(&self, trace_id: &TraceId) -> Result<Trace, TraceError>;
}
/// Replay boundary consumed by later discovery orchestration.
pub trait TraceReplaySource: Send + Sync {
    /// Loads a trace as replay input without executing I/O.
    ///
    /// # Errors
    ///
    /// Returns an error when the trace cannot be loaded or validated.
    fn load_replay_input(&self, trace_id: &TraceId) -> Result<ReplayInput, TraceError>;
}
#[derive(Default)]
pub struct InMemoryTraceStore {
    traces: BTreeMap<TraceId, Trace>,
}
impl TraceStore for InMemoryTraceStore {
    fn save(&mut self, trace: &Trace) -> Result<(), TraceError> {
        trace.validate()?;
        self.traces.insert(
            trace.metadata.trace_id.clone(),
            trace.sanitized_for_persistence(),
        );
        Ok(())
    }
    fn load(&self, trace_id: &TraceId) -> Result<Trace, TraceError> {
        self.traces
            .get(trace_id)
            .cloned()
            .ok_or_else(|| TraceError::NotFound(trace_id.clone()))
    }
}
impl TraceReplaySource for InMemoryTraceStore {
    fn load_replay_input(&self, trace_id: &TraceId) -> Result<ReplayInput, TraceError> {
        self.load(trace_id)?.replay_input()
    }
}
#[derive(Debug)]
pub enum TraceError {
    UnsupportedVersion { found: u16, supported: u16 },
    ReplayCannotProduceTrace,
    Format(String),
    NotFound(TraceId),
}
impl Display for TraceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion { found, supported } => write!(
                f,
                "unsupported trace version {found}; supported version is {supported}"
            ),
            Self::ReplayCannotProduceTrace => f.write_str("replay mode cannot produce a new trace"),
            Self::Format(error) => write!(f, "invalid trace format: {error}"),
            Self::NotFound(id) => write!(f, "trace not found: {}", id.0),
        }
    }
}
impl std::error::Error for TraceError {}

fn fields<'a>(line: &'a str, record: &str, count: usize) -> Result<Vec<&'a str>, TraceError> {
    let fields: Vec<_> = line.split('|').collect();
    if fields.len() != count + 1 || fields[0] != record {
        return Err(TraceError::Format(format!("invalid {record} record")));
    }
    Ok(fields[1..].to_vec())
}
fn number<T: std::str::FromStr>(value: &str) -> Result<T, TraceError> {
    value
        .parse()
        .map_err(|_| TraceError::Format("invalid number".into()))
}
fn hex(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.as_bytes() {
        write!(encoded, "{byte:02x}").expect("writing to a string cannot fail");
    }
    encoded
}
fn unhex(value: &str) -> Result<String, TraceError> {
    if !value.len().is_multiple_of(2) {
        return Err(TraceError::Format("invalid hexadecimal string".into()));
    }
    let bytes: Result<Vec<_>, _> = (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16))
        .collect();
    String::from_utf8(bytes.map_err(|_| TraceError::Format("invalid hexadecimal string".into()))?)
        .map_err(|_| TraceError::Format("invalid UTF-8 string".into()))
}
fn encode_option(value: Option<&String>) -> String {
    value.map_or_else(|| "-".into(), |value| hex(value))
}
fn decode_option(value: &str) -> Result<Option<String>, TraceError> {
    if value == "-" {
        Ok(None)
    } else {
        Ok(Some(unhex(value)?))
    }
}
fn encode_map(map: &BTreeMap<String, String>) -> String {
    map.iter()
        .map(|(key, value)| format!("{}={}", hex(key), hex(value)))
        .collect::<Vec<_>>()
        .join(";")
}
fn decode_map(value: &str) -> Result<BTreeMap<String, String>, TraceError> {
    if value.is_empty() {
        return Ok(BTreeMap::new());
    }
    value
        .split(';')
        .map(|entry| {
            let (key, value) = entry
                .split_once('=')
                .ok_or_else(|| TraceError::Format("invalid map entry".into()))?;
            Ok((unhex(key)?, unhex(value)?))
        })
        .collect()
}
fn encode_query(query: &BTreeMap<String, Vec<String>>) -> String {
    query
        .iter()
        .map(|(key, values)| {
            format!(
                "{}={}",
                hex(key),
                values
                    .iter()
                    .map(|value| hex(value))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
fn decode_query(value: &str) -> Result<BTreeMap<String, Vec<String>>, TraceError> {
    if value.is_empty() {
        return Ok(BTreeMap::new());
    }
    value
        .split(';')
        .map(|entry| {
            let (key, values) = entry
                .split_once('=')
                .ok_or_else(|| TraceError::Format("invalid query entry".into()))?;
            Ok((
                unhex(key)?,
                if values.is_empty() {
                    Vec::new()
                } else {
                    values.split(',').map(unhex).collect::<Result<_, _>>()?
                },
            ))
        })
        .collect()
}
fn sanitize_headers(headers: &mut BTreeMap<String, String>) {
    for (name, value) in headers {
        if matches!(
            name.to_ascii_lowercase().as_str(),
            "authorization" | "cookie" | "set-cookie" | "x-api-key"
        ) {
            *value = REDACTED_HEADER_VALUE.into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn trace() -> Trace {
        Trace {
            metadata: TraceMetadata {
                trace_version: SUPPORTED_TRACE_VERSION,
                trace_id: TraceId("trace-1".into()),
                created_at: "2026-09-16T00:00:00Z".into(),
                source_url: "https://example.invalid".into(),
                execution_mode: TraceExecutionMode::Real,
            },
            navigation: vec![TraceNavigation {
                sequence: 1,
                target: "https://example.invalid".into(),
                phase: NavigationPhase::Completed,
                timestamp: "2026-09-16T00:00:01Z".into(),
            }],
            exchanges: vec![CapturedExchange {
                request: CapturedRequest {
                    method: HttpMethod::Get,
                    url: "https://example.invalid/api/items?limit=2".into(),
                    normalized_url: "https://example.invalid/api/items?limit=2".into(),
                    path: "/api/items".into(),
                    query: BTreeMap::from([("limit".into(), vec!["2".into()])]),
                    headers: BTreeMap::from([("Accept".into(), "application/json".into())]),
                    body: None,
                },
                response: CapturedResponse {
                    status: 200,
                    headers: BTreeMap::from([("Content-Type".into(), "application/json".into())]),
                    body: Some("{\"items\":[]}".into()),
                    content_type: Some("application/json".into()),
                    duration_ms: 12,
                },
                timestamp: "2026-09-16T00:00:01Z".into(),
            }],
            events: vec![TraceEvent {
                sequence: 1,
                kind: "navigation_completed".into(),
                timestamp: "2026-09-16T00:00:01Z".into(),
                message: None,
            }],
        }
    }
    #[test]
    fn exchange_trace_serialization_deserialization_and_replay_preserve_semantics() {
        let original = trace();
        let encoded = original.to_persisted_text().unwrap();
        let decoded = Trace::from_persisted_text(&encoded).unwrap();
        let replay = decoded.replay_input().unwrap();
        assert_eq!(decoded, original);
        assert_eq!(replay.exchanges, original.exchanges);
        assert_eq!(replay.trace_id, original.metadata.trace_id);
    }
    #[test]
    fn unsupported_trace_version_is_rejected() {
        let mut invalid = trace();
        invalid.metadata.trace_version = SUPPORTED_TRACE_VERSION + 1;
        assert!(matches!(
            invalid.to_persisted_text(),
            Err(TraceError::UnsupportedVersion { .. })
        ));
    }
    #[test]
    fn sensitive_headers_are_redacted_before_persistence_and_replay() {
        let mut sensitive = trace();
        sensitive.exchanges[0].request.headers.extend([
            ("Authorization".into(), "Bearer top-secret".into()),
            ("cookie".into(), "session=top-secret".into()),
            ("X-API-Key".into(), "top-secret".into()),
        ]);
        sensitive.exchanges[0]
            .response
            .headers
            .insert("Set-Cookie".into(), "session=top-secret".into());
        let encoded = sensitive.to_persisted_text().unwrap();
        let decoded = Trace::from_persisted_text(&encoded).unwrap();
        assert!(!encoded.contains("top-secret"));
        assert_eq!(
            decoded.exchanges[0].request.headers["Authorization"],
            REDACTED_HEADER_VALUE
        );
        assert_eq!(
            decoded.exchanges[0].request.headers["cookie"],
            REDACTED_HEADER_VALUE
        );
        assert_eq!(
            decoded.exchanges[0].request.headers["X-API-Key"],
            REDACTED_HEADER_VALUE
        );
        assert_eq!(
            decoded.exchanges[0].response.headers["Set-Cookie"],
            REDACTED_HEADER_VALUE
        );
    }
    #[test]
    fn trace_store_persists_sanitized_trace_and_exposes_replay_contract() {
        let trace = trace();
        let id = trace.metadata.trace_id.clone();
        let mut store = InMemoryTraceStore::default();
        store.save(&trace).unwrap();
        assert_eq!(store.load_replay_input(&id).unwrap().exchanges.len(), 1);
    }
    #[test]
    fn replay_mode_cannot_be_used_to_label_a_new_trace() {
        assert!(matches!(
            TraceExecutionMode::try_from(ExecutionMode::Replay),
            Err(TraceError::ReplayCannotProduceTrace)
        ));
    }
}
