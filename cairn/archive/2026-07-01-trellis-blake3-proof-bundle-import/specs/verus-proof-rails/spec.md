## ADDED Requirements

### Requirement: Trellis BLAKE3 proof-bundle import

r[octet.verus_proof_rails.trellis_blake3_import] Octet SHOULD support imported Trellis proof-bundle references that bind external proof artifacts by BLAKE3 identity while keeping imported references separate from Octet-owned `formal_proof` provenance until rerun and receipt validation succeed.

#### Scenario: Imported bundle declares required identity fields

r[octet.verus_proof_rails.trellis_blake3_import.fields]

- GIVEN a maintainer declares an imported Trellis proof bundle
- WHEN import validation runs
- THEN the bundle MUST name the repository label, repository revision, source path, proof path or proof module, verifier command when known, hash algorithm, source artifact hash, proof artifact hash, bounded stdout/stderr hashes when logs are present, receipt hash when a receipt is present, evidence origin, intended evidence tier, and trust-boundary notes.
- AND paths MUST be normalized before hash comparison and diagnostics.

#### Scenario: BLAKE3 identity is not proof correctness

r[octet.verus_proof_rails.trellis_blake3_import.identity_boundary]

- GIVEN an imported Trellis bundle has matching BLAKE3 source, proof, log, or receipt hashes
- WHEN Octet renders a proof card, proof-path explainer, provenance summary, or artifact-verifier output for that bundle
- THEN the output MUST state or encode that the hashes identify exact artifact bytes.
- AND the output MUST NOT claim those hashes prove Verus success, solver soundness, model adequacy, Octet receipt validity, whole-program correctness, certification, semantic equivalence, business-domain validation, full safety, or a verified runtime.

#### Scenario: Imported bundle stays external until Octet rerun

r[octet.verus_proof_rails.trellis_blake3_import.promotion]

- GIVEN an imported Trellis bundle has not been rerun by Octet
- WHEN evidence-tier assignment runs
- THEN the bundle MUST remain external evidence or `manual_review_required`.
- AND Octet MUST NOT emit `formal_proof` provenance for the imported bundle.
- WHEN Octet reruns the referenced proof with pinned verifier, solver, toolchain, command argv, source/proof bindings, and validates an Octet-owned structured receipt
- THEN Octet MAY emit scoped `formal_proof` provenance for the Octet-owned receipt only.

#### Scenario: Malformed or stale import fails closed

r[octet.verus_proof_rails.trellis_blake3_import.fail_closed]

- GIVEN an imported Trellis bundle omits a required field, declares an unsupported hash algorithm, has a malformed digest, has a stale source/proof/log/receipt hash, duplicates an artifact identity where uniqueness is required, omits trust-boundary notes, or requests `formal_proof` without an Octet-owned validated receipt
- WHEN import validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the rejected field or artifact.
- AND Octet MUST NOT render the bundle as valid proof-card evidence.

#### Scenario: Import manifest ownership is registered

r[octet.verus_proof_rails.trellis_blake3_import.registry]

- GIVEN Octet maintains human-authored Trellis imported-proof bundle declarations
- WHEN policy freshness and registry validation run
- THEN the import manifest MUST be typed Nickel and registered as Nickel-authored.
- AND generated JSON imports MUST be covered by freshness checks.
- AND Rust-derived import validation receipts, if emitted, MUST be registered as Rust-derived receipt families.
- AND raw Trellis logs, verifier internals, environment state, and secrets MUST remain non-candidate surfaces except as bounded hashes, paths, labels, allowlists, or explicit rationale.
