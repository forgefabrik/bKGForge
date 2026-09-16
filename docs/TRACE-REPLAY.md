# Trace and replay

M2 defines `bkgforge-trace/1`: metadata, ordered events, request/response exchanges, a `TraceStore`, `TraceReplaySource`, `ReplayInput`, and `InMemoryTraceStore`. Header names are sanitized case-insensitively for `Authorization`, `Cookie`, `Set-Cookie`, and `X-API-Key`; values become `[REDACTED]` before capture records are created.

M3 HTTP exchanges use this same model. Browser navigation and network events are designed to use it as well once a runtime backend is configured; no competing browser trace format exists.
