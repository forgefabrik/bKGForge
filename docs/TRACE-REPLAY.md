# Trace and replay

No trace format is implemented in M1. The only current prerequisite is that every `DiscoveryRequest` preserves its `ExecutionMode`, including the distinct `Replay` mode, and `DiscoveryResult` reserves a `TraceId` reference.

M2 will introduce a versioned, serializable trace with navigation, request/response exchanges, events, sensitive-header sanitization, persistence, and a replay port. Until that milestone, no trace is written and no replay command is exposed.
