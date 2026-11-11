# valence-core

`valence-core` is the reusable, deterministic core of the Valence Evidence IR.
It validates canonical evidence identity, typed linkage, provenance, roles, and
non-claim boundaries over already-loaded values. Filesystem access, process
execution, source discovery, artifact measurement, publication, and operator
policy remain in shells or producer repositories.

## Distribution

The package is distributed from reviewed Git revisions and is not published to
crates.io. Pin an exact public revision:

```toml
[dependencies]
valence-core = {
  git = "https://github.com/OnixResearch/valence",
  rev = "<reviewed-commit>",
  package = "valence-core",
}
```

Rust code imports the crate as `valence_core`.

## Public modules

Identity and envelope primitives:

- `artifact_ref`
- `evidence_ir`
- `evidence_chain`
- `evidence_model`
- `preserves_evidence`
- `provenance`
- `shared_identity`
- `stack_role_registry`

Evidence adapters and profiles:

- `ast_grep_evidence`
- `evidence_adapters`
- `formal_proof_chain`
- `function_address_preserves`
- `function_address_profile`
- `function_object`
- `oxide_primitives`
- `proof_carrying_ir`
- `stack_provenance_adapter`
- `trellis_proof_evidence`
- `wasm_component_evidence`

Indexes, policy models, and deterministic harnesses:

- `capability_plan`
- `evidence_codebase_index`
- `evidence_harness`
- `evidence_index`
- `hegel_property`

The crate-level documentation in `src/lib.rs` describes each module's bounded
claim surface.

## Functional-core boundary

Production code in this crate must remain testable without filesystem,
environment, process, clock, network, or terminal dependencies. Octet and other
producers may construct Valence inputs, but they do not inherit Valence's
identity or linkage claims without rerunning the applicable validators.

## Evidence limits

Passing validation establishes only the specific syntax, canonical identity,
typed linkage, role, digest-domain, or non-claim facts named by that validator.
It does not establish evidence truth, behavioral correctness, semantic
equivalence, verifier soundness, requirement satisfaction, runtime authority,
whole-program safety, or release eligibility.

## License

`valence-core` is licensed under `MPL-2.0`. The complete package-local text is
in [LICENSE](LICENSE); the repository-wide package map is in
[../../docs/licensing.md](../../docs/licensing.md).
