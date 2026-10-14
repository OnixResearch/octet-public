## Design

Octet command shells will open ambient workspace, source, artifact, proof, and cache directories as `cap_std::fs::Dir` roots. Runner modules that perform filesystem effects receive typed root wrappers; pure analysis modules receive loaded source text, artifact bytes, policy records, execution plans, or validated relative locators.

The first conversion targets are `cargo-octet` runner shells that handle `--artifact-dir`, lint cache directories, optional evidence rails, proof receipts, source acquisition, and artifact verification. Standards/lint cores should stay deterministic and independent of `cap-std`.

The conversion must preserve existing artifact bundle layouts and BLAKE3 provenance behavior for valid inputs. Negative diagnostics for traversal or root escape should avoid leaking absolute host paths while still naming the rejected relative locator or issue class.

Tests should cover valid artifact verification, valid cache hits/misses, valid source acquisition under a declared root, traversal, absolute paths, symlink escapes, missing roots, and cache/output escape attempts. Documentation should state that capability roots bound local filesystem access only; they do not prove lint soundness, source correctness, proof validity, or downstream release eligibility.
