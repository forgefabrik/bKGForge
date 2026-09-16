# bkgForge

**Build · Konstrukt · Generate · Forge**

bkgForge is a provider-neutral Rust workspace for constructing reproducible workflows and operating them through replaceable resources. The current delivery includes the M1 discovery-domain foundation; it does not yet fetch websites, capture traffic, or generate artifacts.

## Architecture

```text
core ← workflow, resources, lanes, security, discovery
                         ↑
                      runtime ← CLI
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
- Discovery jobs have a tested lifecycle contract and are security-gated by the runtime before they enter validation.

See [the architecture record](docs/ARCHITECTURE.md), [discovery lifecycle](docs/DISCOVERY.md), and [milestone plan](docs/TASKS.md) for the implemented boundary and planned work.
