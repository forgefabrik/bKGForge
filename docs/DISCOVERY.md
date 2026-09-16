# Discovery lifecycle

## M1 scope

Discovery is currently a **domain workflow contract**, not an I/O implementation. A request supplies a source identifier, execution mode, preferred execution path, workflow ID, and resource requirements. Runtime admission evaluates the existing security policy; an allowed job changes from `Created` to `Validating`.

## State machine

```text
Created → Validating → Provisioning → Capturing → Analyzing
        → Modeling → Generating → Completed
```

Every active state may move to `Failed` or `Cancelled` where declared by the state machine. Terminal jobs cannot be advanced. Invalid transitions return `DomainError::InvalidDiscoveryTransition`.

## Execution-mode boundary

`ExecutionMode::Real`, `Simulation`, and `Replay` are carried in `DiscoveryRequest`; they are not interchangeable. M1 does not execute any of these modes. M2 will define what a replay consumes, and M3 will define real HTTP capture. This prevents M1 from representing a simulated result as a real discovery.

## Resource and security boundary

Resource requirements remain generic (`kind` plus labels), allowing a later browser integration to request `browser` through the existing pool rather than constructing one in discovery code. Runtime applies `bkgforge-security` decisions before registration. URL validation, private-network prevention, redirect validation, and DNS considerations need a concrete HTTP resolution boundary and are therefore scheduled for M3.
