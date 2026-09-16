# M0 handoff — architecture freeze

- **Milestone:** M0 — Bestand und Architektur-Freeze
- **Date:** 2026-09-16
- **Branch:** `work` (local working branch reported by Git)
- **Goal:** Record the actual pre-extension workspace and retain its directed architecture.
- **Implemented components:** No new runtime capability; inventory is recorded in `docs/ARCHITECTURE.md`.
- **Changed files:** `docs/ARCHITECTURE.md`, `docs/TASKS.md`.
- **Tests / commands:** The M1 completion commands are recorded in `docs/bob/m1-discovery-domain.md` after they are executed.
- **Results:** The workspace has six original library crates and one CLI; runtime is the composition root; no discovery, trace, HTTP, browser, or generator crate existed before M1.
- **Open points:** M2 trace foundation is next.
- **Deliberate non-goals:** Browser, discovery I/O, generator, dashboard, API.
- **Next milestone:** M1 delivery is contained in this same change because the requested implementation starts immediately after the audit; future milestones must remain separate.
