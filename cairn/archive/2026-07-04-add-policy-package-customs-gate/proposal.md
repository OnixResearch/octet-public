## Why

Octet policy, profile, proof-rail, template, and generated-contract contributions need the same review questions every time: is the family registered, are fixtures present, are generated artifacts fresh, are secrets excluded, does provenance identify the source, and does the package state its non-claims? The Nickel ecosystem's `nickel-customs` project is a useful reference for a small automated gate that sanity-checks package submissions before they enter a package index.

Octet should adapt that idea as a local customs gate for contributed policy/config packages. The gate should produce deterministic review reports, not operate a public registry or trust service.

## What Changes

- Add a local `policy package customs` gate for proposed Octet policy/profile/template/contract packages.
- Define a package manifest shape with package ID, owner, scope, source paths, generated artifacts, config or receipt family IDs, fixtures, provenance, license/review notes, and explicit non-claims.
- Validate that each contributed package has registry coverage, positive and negative fixtures, deterministic export or generation commands, artifact manifest freshness, and secret exclusion.
- Emit a deterministic report with pass/fail status, blocking diagnostics, and manual-review actions.
- Keep the gate local and offline; package promotion remains a repository review decision.

## Non-Goals

- Do not create a hosted Octet package registry.
- Do not trust third-party packages automatically.
- Do not fetch remote packages or schemas during the gate.
- Do not claim that passing customs proves a package is safe, correct, vulnerability-free, or production-ready.

## Impact

- **Files**: standards package manifest contract, customs gate core/shell, policy registry integration, fixtures, docs, and Cairn specs.
- **Behavior**: contributed policy/config packages fail early when they omit fixture coverage, generated-artifact freshness, registry ownership, provenance, or secret-exclusion rationale.
- **Testing**: valid package fixture, missing registration fixture, missing negative fixture, stale generated artifact fixture, secret-bearing fixture, unknown source/provenance fixture, and Cairn validation/gates.
