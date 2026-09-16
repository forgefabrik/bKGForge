# Milestone plan

## Current delivery: M3 — HTTP Capture + Browser Resource Foundation

**Status: complete.** The current branch adds the discovery identifiers, lifecycle, request/job/result contracts, and runtime security-gated registration. It deliberately contains no website access or fabricated discovery result.

| Milestone | Scope | Status |
| --- | --- | --- |
| M0 | Inventory and architecture freeze | Complete; recorded in `ARCHITECTURE.md`. |
| M1 | Discovery domain foundation | Complete. |
| M2 | Versioned trace, sanitization, persistence and replay contracts | Complete. |
| M3 | HTTP capture, common SSRF policy, and browser contracts | In progress: HTTP transport and contracts are present; a pinned mini-browser protocol/runtime is still required. |
| M4 | Neutral schemas and site model | Planned. |
| M5 | Artifact model and OpenAPI 3.1 | Planned. |
| M6 | Browser resource and browser network capture | Planned. |
| M7 | Discovery orchestration from URL to site model | Planned. |
| M8–M11 | Rust, CLI, MCP, and TypeScript generators | Planned. |
| M12–M14 | Content extraction, crawl, and composed pipeline | Planned. |
| M15–M16 | Dashboard API and dashboard | Planned. |
| M17–M20 | Replay E2E, hardening, parity audit, release gate | Planned. |

## Ordered next work

Trace precedes browser integration. Endpoint discovery, schemas, generators, crawling, and content extraction remain explicitly out of scope.

## M1 definition-of-done record

- Code: Discovery types and lifecycle implementation added in `bkgforge-core` and `bkgforge-discovery`.
- Architecture: runtime composes discovery; discovery depends only on core.
- Tests: lifecycle completion, failure, invalid transition, and runtime security-gated admission are present.
- Non-goals: HTTP, browser, trace serialization, schemas, generators, API, and dashboard.
- Known limits: URL validation and SSRF prevention cannot be truthfully implemented until M3's HTTP boundary.
