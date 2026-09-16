# Milestone plan

## Current delivery: M1 — Discovery Domain Foundation

**Status: complete.** The current branch adds the discovery identifiers, lifecycle, request/job/result contracts, and runtime security-gated registration. It deliberately contains no website access or fabricated discovery result.

| Milestone | Scope | Status |
| --- | --- | --- |
| M0 | Inventory and architecture freeze | Complete; recorded in `ARCHITECTURE.md`. |
| M1 | Discovery domain foundation | Complete. |
| M2 | Versioned trace, sanitization, persistence and replay contracts | Planned. |
| M3 | Real HTTP capture, URL validation/SSRF guard, redirects, endpoint candidates | Planned. |
| M4 | Neutral schemas and site model | Planned. |
| M5 | Artifact model and OpenAPI 3.1 | Planned. |
| M6 | Browser resource and browser network capture | Planned. |
| M7 | Discovery orchestration from URL to site model | Planned. |
| M8–M11 | Rust, CLI, MCP, and TypeScript generators | Planned. |
| M12–M14 | Content extraction, crawl, and composed pipeline | Planned. |
| M15–M16 | Dashboard API and dashboard | Planned. |
| M17–M20 | Replay E2E, hardening, parity audit, release gate | Planned. |

## Ordered next work

The order remains M2, M3, M4, M5, M6, and M7. Trace must precede replayable browser integration; real HTTP capture must precede schema/site-model inference; the site model must precede artifact generators. This is the declared dependency chain, not a claim that these features exist now.

## M1 definition-of-done record

- Code: Discovery types and lifecycle implementation added in `bkgforge-core` and `bkgforge-discovery`.
- Architecture: runtime composes discovery; discovery depends only on core.
- Tests: lifecycle completion, failure, invalid transition, and runtime security-gated admission are present.
- Non-goals: HTTP, browser, trace serialization, schemas, generators, API, and dashboard.
- Known limits: URL validation and SSRF prevention cannot be truthfully implemented until M3's HTTP boundary.
