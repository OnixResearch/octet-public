## Context

Octet's dependency-trust rail focuses on human audit coverage, and dependency-audit rails cover advisories/licenses. Source acquisition is a separate question: how were stack-owned dependencies obtained, which credential or local-path assumptions exist, and whether the source is eligible for release evidence.

## Design

### Receipt shape

Add a receipt DTO and Nickel contract for source-acquisition rows:

- package or input label;
- source kind: `registry`, `git-https`, `git-ssh`, `local-path`, `vendored`, or `nix-flake`;
- URL or repository label;
- revision, tag, lock identity, or vendor manifest digest;
- credential boundary;
- local override policy;
- release eligibility;
- reviewer action when manual review is required;
- explicit non-claims.

### Checker behavior

The pure checker receives parsed dependency/source rows and policy. The shell reads Cargo/Nix metadata, lockfiles, and optional user-supplied manifests. The checker rejects unpinned Git sources, release-eligible path-only dependencies, private SSH dependencies without a credential boundary, vendored snapshots without source revision metadata, stale lock identities, and missing non-claims.

### Evidence separation

Source-acquisition receipts are not vulnerability scans and not cargo-vet audits. They may be parent artifacts for dependency-trust receipts, but the evidence tier remains policy/source validation unless another rail supplies audit or proof evidence.

## Alternatives

### Put this only in each consuming repo

Rejected. Consumers need local policy, but Octet can provide a common receipt family and artifact verifier so evidence shape stays consistent.

### Ban private SSH dependencies globally

Rejected. Some private stack development flows require SSH. The important boundary is to make credential and release assumptions explicit.

## Risks

- **False release confidence**: include non-claims that acquisition identity is not dependency audit, vulnerability freedom, or source correctness.
- **Metadata variance**: start with Cargo/Nix fixtures used by stack repos and keep parser cores small.
- **Consumer friction**: allow local overrides when they are marked non-release-eligible.
