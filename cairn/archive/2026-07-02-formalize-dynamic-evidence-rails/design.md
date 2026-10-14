## Context

Dynamic tools such as Miri and Loom are valuable for selected unsafe and concurrency evidence, but they are intentionally not part of the default Octet lint path. Octet needs a receipt-producing seam for selected dynamic evidence without overclaiming proof.

## Approach

1. **Profile registry.** Add a typed registry for dynamic profiles with profile ID, tool kind, allowed command, checked scope, workload/fixture, acceptance signal, and tier boundary.
2. **Command shell.** The CLI resolves the profile and executes only allowed commands. Pure core validates profile selection and converts observed command output into receipt diagnostics.
3. **Bounded logs.** Store bounded stdout/stderr identities and optional log excerpts rather than unbounded raw logs.
4. **Tier discipline.** Receipts use validation/review tiers and explicitly state non-claims for whole-program correctness, unsafe soundness, and exhaustive concurrency.

## Decisions

### Miri and Loom are initial named profiles

**Choice:** Document and test Miri/unsafe-boundary and Loom/concurrency profiles first.

**Rationale:** Existing Octet docs already name these tools and their review scopes.

### No arbitrary command execution

**Choice:** The runner executes only configured profile commands.

**Rationale:** Evidence commands must be auditable, reproducible, and safe to review.

## Risks / Trade-offs

- **Tool availability:** Consumers may not have Miri or Loom installed. Mitigate by keeping profiles opt-in and producing clear missing-tool diagnostics.
- **False confidence:** Passing dynamic tests can be misread as proof. Mitigate with mandatory tier boundaries and non-claims.
- **Log sensitivity:** Dynamic runs can print source paths or data. Mitigate with bounded logs and configurable artifact redaction policy later if needed.

## Validation

- Positive receipt fixture for a configured profile.
- Negative missing-tool or unknown-profile tests.
- Acceptance-signal mismatch fails closed.
- Starter check test proves dynamic tools are not required by default.
- Cairn validate and gates pass for this change.
