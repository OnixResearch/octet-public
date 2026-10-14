## Design: default-deny gate posture

### Decision: cargo alias, not build.rs

A `build.rs` that runs the gate on every `cargo build` would add latency to
every incremental build and would couple the gate to the Rust build graph in a
way that breaks the functional-core / imperative-shell boundary. The cargo
alias (`cargo octet-check = "!cargo octet check -- && cargo check"`) is the
lighter-weight alternative: it runs the gate first, and if it fails, the build
does not proceed. Consumers who want the default-on posture use
`cargo octet-check` as their primary build command.

### Decision: templates, not enforcement

The templates are copyable; adoption is the consumer's decision. The posture
doc makes the recommendation explicit ("CI must not pass bypass flags") but
does not enforce it. This matches external reference's posture: the bypass flag exists
for emergencies, but the docs say "CI must not."

### What is explicitly NOT provided

- A `build.rs` auto-gate (latency + coupling concerns, see above).
- Enforcement that consumers adopt the posture (templates are copyable).
- A correctness proof (the gate is operational gating, not formal proof).
