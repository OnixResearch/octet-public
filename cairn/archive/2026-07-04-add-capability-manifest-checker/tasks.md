## Phase 1: Implementation

- [x] [serial] r[octet.capability_manifest_checker.manifest] Add typed capability manifest policy contracts and deterministic runtime export for scope classes, capability families, injected traits/tokens, shell interpreters, and deterministic handlers.
- [x] [serial] r[octet.capability_manifest_checker.scope_resolution] Add pure scope-resolution logic that maps repo-relative paths or crates to declared scope classes and fails closed on malformed strict profiles.
- [x] [serial] r[octet.capability_manifest_checker.effect_matching] Add checker logic that compares direct platform-effect observations and declared injected capabilities against the manifest.
- [x] [serial] r[octet.capability_manifest_checker.handlers] Add deterministic handler evidence reporting for capabilities that declare test handlers.
- [x] [serial] r[octet.capability_manifest_checker.report] Add human/JSON capability reports with linked lint IDs, evidence tiers, replay artifacts, and explicit non-claims.

## Phase 2: Verification

- [x] [serial] r[octet.capability_manifest_checker.manifest.valid] Verify a valid Nickel-authored manifest exports deterministic runtime JSON and registry coverage.
- [x] [serial] r[octet.capability_manifest_checker.manifest.invalid] Verify malformed manifests, unknown capability families, unknown scope classes, and missing required fields fail closed.
- [x] [serial] r[octet.capability_manifest_checker.effect_matching.direct_effect] Verify undeclared filesystem/network/time/random/process/environment/secrets/global-state/concurrency effects in core scopes produce deterministic diagnostics.
- [x] [serial] r[octet.capability_manifest_checker.effect_matching.injected] Verify declared injected capability traits or tokens pass in core while concrete platform calls remain shell-owned.
- [x] [serial] r[octet.capability_manifest_checker.handlers.covered] Verify deterministic handler fixtures cover success and failure paths for a declared capability.
- [x] [serial] r[octet.capability_manifest_checker.boundaries.no_effect_system] Verify docs and reports avoid claims of Rust effect types, transitive purity proof, or whole-program effect absence.
- [x] [serial] Run Cairn validate plus proposal/design/tasks gates for this change package before implementation closeout. [depends:cairn-gates]
