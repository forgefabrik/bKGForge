# M3 — HTTP capture and browser foundation

`bkgforge-http` is a real dependency-free HTTP/1.1 capture transport for `http://` targets. It captures all standard methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS), request/response headers and raw binary bodies, and produces the M2 `Exchange` record. HTTPS and automatic redirect following are intentionally not claimed by this transport yet.

`NetworkPolicy` is shared by HTTP and browser adapters and rejects localhost, loopback, private IPv4, link-local, and local IPv6 by default. Every redirect implementation must call this policy again after parsing and resolving its destination. Local fixture testing may explicitly set `allow_private`.

`bkgforge-browser` keeps browser/CDP types outside core: stable session and tab identities, structured accessibility nodes, observations, capability declarations, and network-event contracts. `MiniBrowserBackend` does not fabricate a protocol. The upstream mini-browser repository could not be fetched in this environment, so no upstream command/API is pinned or executed. It reports no capabilities and returns an explicit runtime/protocol error. A future implementation must integrate the real upstream client/CLI, apply `NetworkPolicy` before each navigation and redirect, and persist sanitized browser network events through `bkgforge-trace`.

Browser integration tests require a real Chrome/CDP and mini-browser runtime. They are not represented by mocks or fake success tests here. Endpoint discovery, schema inference, OpenAPI, generators, crawler, RAG/search, and dashboard work are out of scope.
