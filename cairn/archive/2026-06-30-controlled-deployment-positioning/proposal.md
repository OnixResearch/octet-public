## Why

external evidence tooling's deployment page emphasizes running where customer models already live, source visibility, controlled supply chain, and no hosted dependency. Octet has a similar deployment advantage: it runs inside the consumer's Rust/Nix/CI boundary and emits local artifacts. The docs should say that clearly without implying Octet controls what consumers do with artifacts after generation.

This positioning matters for teams that cannot send proprietary source, build paths, or review artifacts to external services.

## What Changes

- Document Octet's controlled deployment posture: local or CI execution, no required hosted service, pinned Nix/cargo runner path, source-visible toolchain, and local artifact directories.
- Add a supply-chain checklist for consumers: pinned Octet revision, checked-in config, deterministic artifact directory, and optional pre-commit gate.
- Clarify the boundary: Octet itself does not require uploading source or artifacts, but consumers remain responsible for their own CI storage and integrations.
- Add positive and negative documentation checks around local-execution and no-required-hosted-service claims.

## Impact

- **Files**: `README.md`, `docs/consumer-guide.md`, `docs/downstream-rollout.md`, `docs/release-candidate.md`, and templates if a checklist is added.
- **Testing**: documentation check for controlled-deployment bullets; negative check preventing absolute claims such as "source never leaves" without the consumer-owned boundary caveat.
