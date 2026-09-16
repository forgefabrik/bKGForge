# Architecture

## Status

This document records the repository state after M1, **Discovery Domain Foundation**. It is not a claim that HTTP, browser discovery, traces, schemas, or generators exist.

## Reviewed workspace

| Component | Actual responsibility | Tests |
| --- | --- | --- |
| `bkgforge-core` | Provider-neutral identifiers, resource/lane/execution/discovery lifecycles, domain errors, ports. | Resource matching. |
| `bkgforge-workflow` | Validated workflow IR and execution-path selection. | Validation gate and direct-API preference. |
| `bkgforge-resources` | Generic ready-resource allocation, release, and degradation. | Release makes a resource reusable. |
| `bkgforge-lanes` | Logical lanes and resource rebinding. | A lane retains identity across a rebind. |
| `bkgforge-security` | Normalized event decision policy. | Repeated failures block. |
| `bkgforge-discovery` | Discovery request/job/result contracts and lifecycle transitions. | Completion, failure, and invalid transition paths. |
| `bkgforge-runtime` | Composition root, security gates, lane and discovery registration. | Blocked lane and approved discovery registration. |
| `bkgforge-cli` | Native `status` and `doctor` commands. | Exercised through command execution; no CLI unit test yet. |

## Dependency direction

```text
bkgforge-core
  ↑        ↑         ↑        ↑         ↑
workflow resources lanes security discovery
  ↑        ↑         ↑        ↑         ↑
  └──────────── bkgforge-runtime ──────┘
                        ↑
                 bkgforge-cli
```

`bkgforge-lanes` also uses `bkgforge-resources` to bind generic resources. The runtime alone composes the lower layers. There are no reverse dependencies into the runtime or apps.

## M1 contract

`bkgforge-core` owns generic stable identifiers and the discovery state machine. `bkgforge-discovery` owns the discovery-specific request, job, and result contracts. The runtime accepts a newly-created `DiscoveryJob`, applies the existing normalized security decision, and advances an accepted job to `Validating`.

This preserves the established architecture: the domain does not instantiate a browser, perform HTTP I/O, store cookies, or retain secret values. A future discovery executor must use the request's declared execution mode, workflow identity, and generic resource requirements rather than bypassing runtime/resource policy.

## Site2CLI comparison boundary

The project is using site2cli only as a functional reference, not as a source implementation. At M1, only the job/lifecycle prerequisite is present. API discovery, extraction, crawling, OpenAPI, client generation, CLI generation, MCP, sessions, screenshots, proxies, and replay processing are explicitly future milestones.

## Known limitations

- `target` is an opaque input string at M1; URL parsing and SSRF enforcement belong to M3's HTTP boundary.
- No trace format or serialization exists before M2.
- A registered job remains at `Validating`; no executor is exposed before M7 orchestration.
