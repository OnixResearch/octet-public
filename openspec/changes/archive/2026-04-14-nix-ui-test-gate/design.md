# Design: Prove UI Lint Fixtures Under Nix

## Context

The original gap had two parts:

- `tests/ui.rs` skipped when `rustup` was missing
- `checks.nextest` ran inside a crane/vendored sandbox where a nested driver
  build could not reliably pull in Dylint's extra crate graph

Making `nextest` build a temporary Dylint driver inside that sandbox turned out
fragile. Even after local devshell runs passed, the nix path still broke on
vendored dependency resolution and Dylint workspace assumptions.

## Goals / Non-Goals

**Goals:**

- make the nix validation path execute real checked-in UI fixtures
- fail loudly on fixture drift instead of silently skipping
- keep local developer runs available without requiring a separate bespoke tool
- isolate the Dylint driver build from `checks.nextest`

**Non-Goals:**

- exhaustive UI coverage for every lint in this change
- changing published lint behavior beyond whatever fixture text captures
- fixing unrelated repo-wide clippy churn discovered during validation

## Decisions

### 1. Split UI proof into dedicated `checks.ui`

**Choice:** Keep `checks.nextest` for ordinary Rust tests and add a dedicated
`checks.ui` derivation for the Dylint fixture harness.

**Rationale:** The UI harness needs a Dylint driver and lint library wired in a
very specific way. Giving that path its own derivation avoids nested cargo
builds inside the nextest sandbox and keeps the verification boundary explicit.

### 2. Let `tests/ui.rs` accept prebuilt artifacts

**Choice:** Teach `tests/ui.rs` to honor `TIGERSTYLE_UI_DRIVER` and
`TIGERSTYLE_UI_LIBRARY` when present, and fall back to local builds otherwise.

**Rationale:** Nix can prebuild the hard parts hermetically, while developers
can still run the same Rust test harness locally from the devshell.

### 3. Keep the UI test ignored by default

**Choice:** Mark the Rust UI test ignored, then run it explicitly in the
prebuilt nix path and with an explicit local cargo command.

**Rationale:** This keeps package builds and `checks.nextest` from accidentally
re-entering the old nested-build path while preserving one canonical harness.

### 4. Vendor a repo-local support driver

**Choice:** Add `tests/support/ui-driver/` plus a patched local copy of the
minimal Dylint driver/internal support crates under `tests/support/dylint/`.

**Rationale:** Dylint's stock driver build now wants extra generated symbol work
that reaches for networked Clippy metadata on newer nightlies. A local patched
support copy lets nix build the driver reproducibly without depending on that
network path.

## Risks / Trade-offs

**[Support-copy drift]** The local Dylint support copy can drift from upstream.
**Mitigation:** keep it limited to the driver/internal subset and pin the source
revision in the support crate lockfile.

**[Fixture stderr churn]** Warning text or spans can move when lint wording
changes. **Mitigation:** keep fixtures tiny and regenerate `.stderr` from real
runs.

**[Unrelated CI failures]** Other checks can still fail even after UI proof is
fixed. **Mitigation:** validate `checks.ui` and `checks.nextest` directly and
track unrelated failures separately.
