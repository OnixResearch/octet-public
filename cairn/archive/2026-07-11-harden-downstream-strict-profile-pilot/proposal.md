## Why

The first Valence `deterministic-core` downstream pilot exposed two blockers in Octet rather than in Valence domain logic. The packaged runner lacks a linker in its standalone runtime closure, so a proc-macro dependency fails before linting unless the consumer enters its own dev shell. Once linked, `concrete_construction_in_core` emits repeated diagnostics for nested parent expressions and local domain DTO constructors even though the accepted contract limits it to external, non-domain, non-`std` construction.

These defects make strict-profile evidence depend on ambient shell state and encourage broad source suppressions for valid functional-core code.

## What Changes

- Add the Nix C/C++ linker toolchain to the packaged `cargo-octet` runtime path so ordinary downstream crates with proc macros compile from a standalone invocation.
- Replace snippet-substring construction detection with direct HIR call classification.
- Exclude local-domain and standard-library value construction while preserving diagnostics for external concrete constructors.
- Add positive and negative UI fixtures plus a real Valence downstream packaging smoke.

## Impact

- **Files**: `flake.nix`, the concrete-construction lint core/shell, UI fixtures/snapshots, strict-profile documentation, and lifecycle evidence.
- **Compatibility**: removes false-positive diagnostics; genuine external constructor diagnostics and no-scope behavior remain.
- **Testing**: focused lint UI tests, positive local/std cases, negative external case, packaged Valence proc-macro smoke, profile catalog checks, Rust tests, and Cairn gates.
