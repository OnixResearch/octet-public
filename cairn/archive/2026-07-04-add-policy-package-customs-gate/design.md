## Context

Octet already has a registry of config/receipt families and multiple Nickel-authored standards inputs. As the number of policy, proof, evidence, and template packages grows, reviewers need a repeatable gate that catches missing metadata before package content is promoted.

`nickel-customs` shows a small sanity-checker model for Nickel package submissions. Octet can adapt the concept locally with stronger evidence-chain constraints: every package should identify ownership, generated artifacts, fixtures, provenance, and explicit non-claims.

## Decisions

### 1. Use a local package manifest

**Choice:** Each customs-checked package declares a Nickel-authored or Rust-derived manifest with package ID, kind, owner, source paths, generated artifact paths, config/receipt family IDs, generation commands, validation commands, positive fixtures, negative fixtures, provenance, license notes, and non-claims.

**Rationale:** A manifest lets the gate validate package completeness without inferring review intent from filenames.

### 2. Split pure validation core from shell

**Choice:** The customs validation core receives an in-memory package model, registry model, fixture inventory, and artifact manifest model, then returns deterministic diagnostics. The shell owns filesystem reads, Nickel export invocation, and report writing.

**Rationale:** Functional-core validation keeps policy logic testable with positive and negative fixtures and avoids hidden filesystem state.

### 3. Require both positive and negative fixtures

**Choice:** A package that introduces contracts, profiles, templates, or policy rules must include positive fixtures and negative fixtures or explicitly classify itself as a non-candidate with rationale.

**Rationale:** Octet's evidence model treats sad-path behavior as part of the contract. A package that only proves acceptance is incomplete.

### 4. Report review actions instead of granting trust

**Choice:** Passing customs emits a deterministic report that lists checked surfaces, accepted non-claims, and any manual-review actions; it does not mark external package authors or upstream data as trusted.

**Rationale:** The gate supports local review. It should not become a package trust network.

## Risks / Trade-offs

- **Manifest burden:** Contributors must write more metadata. Mitigate with templates and actionable diagnostics.
- **Overblocking experiments:** Early policy drafts may lack full fixtures. Mitigate by allowing explicit non-candidate or draft status while blocking promotion.
- **Secret scanning limits:** Automated checks can miss secrets. Mitigate with fail-closed raw secret patterns and required human review actions.
- **Trust overclaim:** A passing gate might be treated as safety proof. Mitigate with non-claims in reports and docs.

## Validation

- A complete package fixture passes and emits deterministic report rows.
- Missing registry entry, missing negative fixture, stale generated artifact, unknown source path, or missing provenance fails with field-identifying diagnostics.
- Secret-bearing package fixtures fail or require non-candidate exclusion before promotion.
- The customs report records manual-review actions and non-claims.
- Cairn validate and gates pass for this change.
