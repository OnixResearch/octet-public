## Context

The policy registry already identifies `cargo-deny` as the checker for dependency audit policy, but Octet does not yet produce a dependency-audit receipt that artifact verification can reason about.

## Approach

1. **Thin shell around cargo-deny.** Runner code resolves workspace root, config path, lockfile, output paths, and command argv; pure core validates requested mode and turns command results into deterministic receipt data.
2. **Tool-native config.** Use `deny.toml` because `cargo-deny` owns that format. Octet can provide a template, but it should not translate Nickel into a new cargo-deny policy language in this slice.
3. **Locked evidence mode.** Add an explicit switch or metadata field for database update behavior. Release evidence must record whether inputs were updated by a prior explicit step or reused as-is.
4. **Receipt and provenance.** Write a small Rust-owned DTO for dependency-audit receipts and register the receipt family in the policy/artifact manifest registry.

## Decisions

### Preserve cargo-deny categories

**Choice:** Keep categories such as advisories, licenses, bans, sources, and duplicate versions as cargo-deny categories.

**Rationale:** Octet should make the result reviewable, not obscure the checker that made the decision.

### Keep starter path light

**Choice:** Do not add dependency-audit execution to default `cargo octet check` yet.

**Rationale:** Consumers need a checked-in policy and advisory database/update posture before dependency findings can be treated as stable gates.

## Risks / Trade-offs

- **Policy churn:** License/advisory data changes over time. Mitigate by recording input/update identity and providing locked mode.
- **Legal overclaim:** License pass/fail can be mistaken for legal advice. Mitigate with receipt non-claims and docs.
- **Config absence:** Existing consumers may not have `deny.toml`. Mitigate with clear missing-config diagnostics and a template.

## Validation

- Unit tests for pure mode/config validation and receipt rendering.
- Fixture tests for clean and denied dependency-policy outputs.
- Artifact verification covers dependency-audit receipt provenance.
- Cairn validate and gates pass for this change.
