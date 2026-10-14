# Tasks

## Contract

- [x] [serial] Define the Octet compiler-policy provider manifest schema with provider, profile, rustc/toolchain, adapter, config, standards, waiver, claim, and non-claim fields. r[octet.compiler_policy_provider.manifest]
- [x] [serial] Define the artifact-acceptance profile semantics for deny/error lints, required standards checks, and fail-closed downstream consumption. r[octet.compiler_policy_provider.profile]
- [x] [serial] Define the structured waiver contract and exported waiver summary shape for accepted exceptions. r[octet.compiler_policy_provider.waivers]

## Implementation

- [x] [serial] Add manifest export plumbing through an Octet-owned command or Nix app without making downstream consumers import Octet internals. r[octet.compiler_policy_provider.manifest.export]
- [x] [serial] Bind Dylint driver, Octet lint library, lint config, rustc identity, and standards policy artifact identities into the exported manifest. r[octet.compiler_policy_provider.manifest.identity]
- [x] [serial] Add or tighten suppression validation so required profiles reject bare `allow` attributes, hidden required-lint disablement, and missing waiver fields. r[octet.compiler_policy_provider.waivers.validation]
- [x] [serial] Document the Mantle/downstream boundary: Octet owns rules and provider artifacts; downstream build systems own invocation, artifact acceptance, and receipts. r[octet.compiler_policy_provider.downstream_boundary]

## Verification

- [x] [serial] Add positive manifest fixture tests for a complete Dylint adapter profile with standards policy identity. r[octet.compiler_policy_provider.manifest]
- [x] [serial] Add negative fixture tests for missing rustc identity, missing driver, missing lint library, malformed digest, missing standards artifact, disabled required lint, and missing waiver metadata. r[octet.compiler_policy_provider.manifest.fail_closed]
- [x] [serial] Add downstream-consumer smoke coverage proving manifest hash changes when driver, lint library, config, or standards policy identity changes. r[octet.compiler_policy_provider.downstream_boundary]
- [x] [serial] Run focused provider-manifest tests plus `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.compiler_policy_provider.manifest]
