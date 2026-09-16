# Trace and replay

## M2 format

`bkgforge-trace` defines the supported trace format version as `1`. A trace is serialized as a documented line-oriented text format. The first line is exactly `bkgforge-trace/1`; each later record begins with `M` (metadata), `N` (navigation), `X` (exchange), or `E` (event). String fields are hexadecimal UTF-8 and maps are ordered, delimiter-safe key/value records. The logical structure is:

```text
metadata
navigation[]
exchanges[]
events[]
```

`metadata` contains `trace_version`, `trace_id`, `created_at`, `source_url`, and provenance (`real` or `simulation`). An exchange pairs a typed request and response. The request carries method, original and normalized URL, path, query, headers, and optional body; the response carries status, headers, optional body, content type, and duration. Navigation and events retain sequence numbers and execution-boundary timestamps.

The model is intentionally capture-technology-neutral. It performs no HTTP request and does not claim that an exchange came from a browser or network client.

## Version and persistence rules

Only version `1` is accepted. Both serialization and deserialization validate the version. Output is deterministic for a given trace because maps use `BTreeMap` and records are emitted in vector insertion order.

The `TraceStore` port stores a sanitized clone; `InMemoryTraceStore` is a tested reference implementation. A later durable adapter may implement the same port without changing the trace model.

## Secret boundary

Before persistence or replay input creation, the case-insensitive headers `Authorization`, `Cookie`, `Set-Cookie`, and `X-API-Key` are replaced with `<redacted>`. Trace bodies are opaque captured payloads; M2 does not attempt content-specific credential detection. Capture implementations in M3 and later must define body-sanitization policy before recording sensitive request or response bodies.

## Replay boundary

`TraceReplaySource` loads a `ReplayInput` containing the trace ID, source URL, navigation, exchanges, and events. The input has no I/O capability. `ExecutionMode::Replay` cannot label a newly produced trace; it is reserved for consumers of a prior `Real` or `Simulation` trace. M2 provides this contract only—full discovery replay belongs to M21.
