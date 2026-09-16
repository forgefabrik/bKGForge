# bkgForge

**Build · Konstrukt · Generate · Forge**

bkgForge is a provider-neutral Rust workspace for constructing reproducible workflows and operating them through replaceable resources.

## Architecture

```text
core ← workflow, resources, lanes, security, discovery
                         ↑
              runtime ← CLI; trace, HTTP and browser are adapter crates
```

The core contains no provider URLs, selectors, credentials, sessions, or provider business rules. Concrete integrations belong in independent adapter crates under `providers/` and implement the core ports.

## Verify

```bash
cargo test --workspace
cargo run -p bkgforge-cli -- status
cargo run -p bkgforge-cli -- doctor
```

## Current guarantees

- Workflows can only be compiled after validation; experimental workflows must explicitly opt in.
- `REAL`, `SIMULATION`, and `REPLAY` are distinct execution modes.
- Resource allocation is generic and only allocates healthy, ready resources.
- A lane has its own identity and may be rebound without being recreated.
- Security evaluates normalized events and can pause or block execution.
- Secrets are represented by references in domain models; secret values are not stored by this workspace.
- M2 provides `bkgforge-trace/1`, deterministic ordered records, sensitive-header sanitization, in-memory storage and replay input.
- M3 provides an HTTP/1.1 capture foundation and a browser contract. Browser execution remains runtime-dependent; no browser availability is claimed without a configured backend.
