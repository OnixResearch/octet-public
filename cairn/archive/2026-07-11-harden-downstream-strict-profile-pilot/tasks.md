## Phase 1: Lint precision

- [x] [serial] Extract pure direct-construction classification from the HIR/source-map shell. r[octet.strict_rust_profiles.core_construction_precision]
- [x] [serial] Restrict findings to direct external ADT `new`/`default` calls and exclude local plus `core`/`alloc`/`std` result types. r[octet.strict_rust_profiles.core_construction_precision.external] r[octet.strict_rust_profiles.core_construction_precision.local] r[octet.strict_rust_profiles.core_construction_precision.no_duplicates]
- [x] [serial] Add positive and negative classifier/UI fixtures covering external, local, standard, no-scope, and nested-parent shapes. r[octet.strict_rust_profiles.core_construction_precision.external] r[octet.strict_rust_profiles.core_construction_precision.local] r[octet.strict_rust_profiles.core_construction_precision.no_duplicates]

## Phase 2: Packaged downstream execution

- [x] [serial] Add the linker toolchain to the packaged runner runtime closure. r[octet.strict_rust_profiles.downstream_linker]
- [x] [serial] Add a dependency-bearing standalone package smoke that reaches Octet lint/function-address processing without a consumer dev shell. r[octet.strict_rust_profiles.downstream_linker.proc_macro]
- [x] [serial] Document the bounded downstream packaging and source-shape claim. r[octet.strict_rust_profiles.downstream_hardening_boundaries] r[octet.strict_rust_profiles.downstream_hardening_boundaries.non_claims]

## Phase 3: Validation and lifecycle

- [x] [serial] Run focused unit/UI tests, packaged downstream smoke, profile catalog/policy checks, formatting, Clippy, and supported Cairn gates; record exact evidence before sync and archive. r[octet.strict_rust_profiles.downstream_hardening_boundaries.non_claims]
