## Phase 1: Baseline and policy

- [ ] [serial] Record current `bool_naming`, parameter-count, Clippy co-reporting, and public API behavior before changes. r[octet.structure.raw_bool_parameter.compatibility]
- [ ] [serial] Add the Radiance style reference, exact revision, MIT role, selected guidance, and rejected guidance to documentation. r[octet.structure.raw_bool_parameter.reference_boundary]
- [ ] [serial] Define the public API, private-count, generated-code, trait-ownership, exception, and non-claim policy. r[octet.structure.raw_bool_parameter.policy]
- [ ] [serial] Add a named constant for the minimum ambiguous private boolean-parameter count. r[octet.structure.raw_bool_parameter.policy]

## Phase 2: Lint and metadata

- [ ] [serial] Implement `raw_bool_parameter` for public free functions, inherent methods, and crate-owned trait declarations. r[octet.structure.raw_bool_parameter.public_api]
- [ ] [serial] Report non-public callables at the named direct-boolean parameter count. r[octet.structure.raw_bool_parameter.private_api]
- [ ] [serial] Skip generated expansion output and externally owned trait implementation signatures. r[octet.structure.raw_bool_parameter.compatibility]
- [ ] [serial] Emit stable diagnostics and domain-modeling help without an automatic type rewrite. r[octet.structure.raw_bool_parameter.diagnostics]
- [ ] [serial] Register lint identity, structure group, metadata, source path, examples, docs, and strict-profile membership. r[octet.structure.raw_bool_parameter.catalog]
- [ ] [serial] Add `clippy::fn_params_excessive_bools` to the reviewed co-reporting profile without merging finding identities. r[octet.structure.raw_bool_parameter.clippy_evidence]

## Phase 3: Positive and negative verification

- [ ] [parallel] Add negative UI fixtures for public free functions, inherent methods, crate-owned trait methods, and multiple private booleans. r[octet.structure.raw_bool_parameter.verification]
- [ ] [parallel] Add positive fixtures for private single flags, boolean returns, fields, enums, option records, wrappers, generated code, and external trait implementations. r[octet.structure.raw_bool_parameter.verification]
- [ ] [parallel] Add narrow expectation fixtures with required specific reason and named owner. r[octet.structure.raw_bool_parameter.exceptions]
- [ ] [parallel] Reject crate, module, workspace, baseline, and warning-budget suppression. r[octet.structure.raw_bool_parameter.exceptions]
- [ ] [parallel] Add catalog, profile, group, alias, metadata, docs, and snapshot freshness tests. r[octet.structure.raw_bool_parameter.catalog]

## Phase 4: Migration and deny-all activation

- [ ] [serial] Produce a deterministic Octet self-inventory and selected-consumer inventory before activation. r[octet.structure.raw_bool_parameter.migration]
- [ ] [serial] Replace covered Octet public booleans with reviewed domain types or narrow justified expectations. r[octet.structure.raw_bool_parameter.migration]
- [ ] [serial] Migrate selected consumers and record compatibility changes, rejections, and remaining external-signature exceptions. r[octet.structure.raw_bool_parameter.migration]
- [ ] [serial] Enable the lint in the full pinned deny-all catalog only after the declared migration set is clean. r[octet.structure.raw_bool_parameter.catalog]

## Phase 5: Documentation and closeout

- [ ] [parallel] Document design alternatives, examples, migration patterns, exceptions, Clippy relation, and non-claims. r[octet.structure.raw_bool_parameter.diagnostics]
- [ ] [serial] Add the Radiance repository to README references with its exact style-reference role and claim boundary. r[octet.structure.raw_bool_parameter.reference_boundary]
- [ ] [serial] Run focused unit, UI, snapshot, metadata, profile, Clippy co-reporting, and migration checks. r[octet.structure.raw_bool_parameter.verification]
- [ ] [serial] Run formatting, workspace tests, Clippy, Octet deny-all, Cairn validation, traceability, and `nix flake check -L`. r[octet.structure.raw_bool_parameter.catalog]
- [ ] [serial] Run all Cairn gates. Sync and archive only after deny-all activation has no warning budget or baseline. r[octet.structure.raw_bool_parameter.migration]
