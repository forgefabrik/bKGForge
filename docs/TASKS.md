# Milestone plan

## Current delivery: M2 — Trace Foundation

**Status: complete.** The current branch adds a versioned, serialized trace model with capture records, sensitive-header redaction, persistence and replay ports, and an in-memory reference store. It deliberately performs no HTTP request, browser interaction, endpoint detection, schema inference, or artifact generation.

| Milestone | Scope | Status |
| --- | --- | --- |
| M0 | Inventory and architecture freeze | Complete; recorded in `ARCHITECTURE.md`. |
| M1 | Discovery domain foundation | Complete. |
| M2 | Versioned trace, sanitization, persistence and replay contracts | Complete. |
| M3 | Real HTTP capture, URL validation/SSRF guard, redirects | Planned. |
| M4 | Endpoint discovery | Planned. |
| M5 | Neutral schema inference | Planned. |
| M6 | Site model | Planned. |
| M7–M8 | Browser resource and browser network capture | Planned. |
| M9 | Discovery orchestration from URL to site model | Planned. |
| M10–M15 | Artifact foundation and OpenAPI/Rust/CLI/MCP/TypeScript generators | Planned. |
| M16–M18 | Content extraction, crawl, and composed pipeline | Planned. |
| M19–M20 | Runtime API and dashboard | Planned. |
| M21–M24 | Full replay, hardening, capability audit, release gate | Planned. |

## Ordered next work

The next milestone is M3. Trace must precede real replay consumers; real HTTP capture must precede endpoint discovery; endpoint discovery must precede schema/site-model inference; the site model must precede artifact generators. This is the declared dependency chain, not a claim that these features exist now.

## M2 definition-of-done record

- Code: `bkgforge-trace` provides versioned trace/capture records, sanitization, documented text serialization, persistence, and replay contracts.
- Architecture: trace depends only on core; no existing crate was restructured.
- Tests: exchange → trace → serialized text → trace → replay input, unsupported version rejection, sensitive-header redaction, store/replay contract, and execution-mode provenance are present.
- Non-goals: HTTP, browser, endpoint detection, schemas, generators, API, and dashboard.
- Known limits: M2 does not sanitize opaque payload bodies or execute full discovery replay; those boundaries require later capture/orchestration milestones.
