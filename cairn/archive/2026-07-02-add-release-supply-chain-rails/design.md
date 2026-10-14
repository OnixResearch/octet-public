## Context

Release operators need evidence about built artifacts in addition to source checks. Rust-specific metadata and general SBOM/vulnerability tools cover different parts of that release surface.

## Approach

1. **Rail group.** Add optional release supply-chain rails for auditable binary metadata, binary vulnerability audit, SBOM generation, and SBOM/container vulnerability scan.
2. **Tool adapters.** Keep each external tool behind a thin shell adapter. Pure core validates selected rail configuration and receipt data.
3. **Parent provenance.** Link release receipts back to source-check, proof, or artifact-verifier parents when they are derived from previous Octet artifacts.
4. **Attestation-shaped local receipts.** Include subject, materials/parents, runner identity, command argv, status, checked scope, and BLAKE3 identities so later SLSA/in-toto export can be added without changing core evidence fields.

## Decisions

### Optional and local-first

**Choice:** No hosted service, signing service, or transparency log is required in v1.

**Rationale:** This preserves Octet's controlled-deployment posture while leaving room for future attestation exports.

### SBOMs are evidence, not protection

**Choice:** Receipts explicitly state that SBOMs and vulnerability scans do not prevent supply-chain attacks.

**Rationale:** This aligns with Octet's scope-boundary discipline.

## Risks / Trade-offs

- **Toolchain size:** Syft/Grype and cargo-auditable add runtime dependencies. Mitigate by making rails opt-in and Nix-wrapped.
- **Database drift:** Vulnerability results change over time. Mitigate by recording database identity when the scanner exposes it.
- **Sensitive paths:** SBOMs and scans can expose path or package data. Mitigate by documenting release artifact boundaries and avoiding raw secret capture.

## Validation

- Auditable binary fixture passes when metadata is present.
- Missing auditable metadata fails selected rail.
- SBOM receipt and scan receipt link through parent provenance.
- Vulnerability finding fixture preserves severity/package/gate outcome.
- Artifact verification rejects missing parent or hash mismatch.
- Cairn validate and gates pass for this change.
