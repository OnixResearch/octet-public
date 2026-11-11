# Associated-item compiler regression

## Cause and correction

Both associated-item callbacks labeled every item as a function. The collector then requested `fn_sig` for associated types and constants. Rust rejected that query with an internal compiler error.

The callbacks now use the compiler definition kind. Associated types map to `TypeAlias`, constants map to `Constant`, and methods retain `Function`. Only methods carry signatures. No DTO schema or policy mode changed. This correction restores the existing compiler observation contract.

## Reproduction and controls

A dependency-free consumer reproduces the problem:

```rust
#![no_std]

pub trait Visitor {
    type Value;
}

pub struct ValueVisitor;

impl Visitor for ValueVisitor {
    type Value = u32;
}
```

Plain Cargo accepts the source. Published Octet `38b3d281` crashes on `Visitor::Value` during architecture collection. The same consumer completes compiler collection with the corrected package, and its artifact bundle verifies.

The repository fixture adds associated types, constants, and a default method. Assertions require the correct item kind and signature presence. The existing missing-shard control still requires incomplete coverage. Before the correction, the dedicated Nix check reproduced the compiler crash. After the correction, it covered all ten Cargo units and passed.

Exact owner checks:

```sh
nix build .#checks.x86_64-linux.compiler-architecture-targets .#checks.x86_64-linux.project-architecture-policy .#cargo-octet --no-link -L
cargo test -p octet -p cargo-octet -p octet-architecture-ir --lib -- --test-threads=1
cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings
cargo fmt --all -- --check
```

Cargo commands ran in the pinned base Nix shell with empty compiler wrappers and incremental compilation disabled. The suites passed 465 runner tests, 145 lint-library tests, and 23 IR tests. Eight existing runner tests remained ignored. The dedicated Nix check explicitly ran its ignored compiler control.

Private owner evidence is under `~/.local/state/workday-h7/admission-r12/`: `owner-red.log`, `owner-green.log`, `owner-scoped.log`, and `associated-fixed-verification.log`.

## Limits

This is not strict Workday acceptance. The minimal inventory policy still reports its missing composition root and a required compiler unknown. Complete collection and valid artifact integrity do not erase those findings. Changing the policy without regeneration still rejects the stale export before compilation. No tenant calls occurred.
