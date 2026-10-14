## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]
Verified proof rails MUST bind proof receipts to the configured verifier identity, toolchain identity, executable identity, build closure identity, and Nix closure graph evidence.

#### Scenario: Receipt records generated Nix closure graph digest [r[verified-enforcement-rails.nix-closure-graph-digest]]

- GIVEN a verified proof rail configured with `verifier_nix_closure_graph_paths` and `verifier_nix_closure_graph_digest`
- WHEN the checker validates a passing proof receipt
- THEN the checker MUST recompute a BLAKE3 digest from the configured generated Nix `exportReferencesGraph` artifacts
- AND the checker MUST require a matching `proof-verifier-nix-closure-graph-blake3` receipt line

#### Scenario: Generated artifacts expose Nix closure graph obligations [r[verified-enforcement-rails.nix-closure-graph-artifacts]]

- GIVEN policy artifacts are exported for a verified proof rail with Nix closure graph obligations
- WHEN downstream checkers consume the generated artifact bundle
- THEN the artifact bundle MUST include the configured Nix closure graph digest and graph artifact path obligations
