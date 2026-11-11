# Three-route u64 pilot comparison

The comparison model reports `incomparable_claims` for the current state:

| Route | Outcome | Executable refinement bridge |
| --- | --- | --- |
| Aeneas/Charon/Lean | supported | checked: extraction + universal model theorem |
| Vermilion/Verus/Lean | supported | open: the Verus-syntax source is a parallel specification |
| Verus | supported | open: the u64 contract model is checked, but the plain-Rust refinement theorem is not |

The Aeneas row is comparable; the Vermilion and Verus rows are explicitly
non-comparable because their executable refinement bridges remain open
obligations. No provider evidence authority is merged, and no agreement or
release claim follows.

This is exercised by
`crates/octet-standards/src/vermilion/tests.rs::u64_pilot_routes_remain_incomparable_until_executable_bridges_close`.
