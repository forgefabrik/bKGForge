# M2 handoff — Trace Foundation

- **Milestone:** M2 — Trace Foundation
- **Date:** 2026-09-16
- **Branch:** `work` (local working branch reported by Git)
- **Goal:** Provide a versioned, safe, reproducible trace contract without adding any HTTP or browser I/O.
- **Implementation:** New `bkgforge-trace` crate with metadata, navigation, typed captured request/response exchanges, events, version validation, sanitizing line-format serialization/deserialization, `TraceStore`, `TraceReplaySource`, `ReplayInput`, and `InMemoryTraceStore`.
- **Changed files:** Workspace manifest, new trace crate, README, architecture/trace/task documentation, and this handoff.
- **Tests:** Round trip and replay semantics; unsupported version rejection; sensitive header sanitization; sanitized store/replay contract; replay provenance rejection.
- **Executed commands:** `cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo run -p bkgforge-cli -- status`; `cargo run -p bkgforge-cli -- doctor`; `git diff --check`.
- **Testergebnisse:** All commands completed successfully. `cargo test --workspace` ran 16 unit tests successfully; the trace crate ran five focused M2 tests. The CLI reported one ready resource, zero active lanes, and zero discovery jobs.
- **Open points:** M3 must provide SSRF-safe real HTTP capture and define capture-time body-sanitization policy. M21 will orchestrate full trace-to-site-model replay.
- **Nicht-Ziele:** HTTP client, browser, network capture, endpoint discovery, schema inference, generators, API, dashboard, crawl, and search.
- **Next milestone:** M3 — HTTP Capture.
