## ADDED Requirements

### Requirement: Usize In Public API

The lint MUST warn on inherent `pub fn` parameters and return types that use
`usize` or `isize`, including nested occurrences inside references, slices,
arrays, tuples, and generic arguments such as `Vec<usize>`.

The lint MUST suggest using fixed-width integer types such as `u32`, `u64`,
`i32`, or `i64`.

The lint MUST NOT fire on trait declarations or trait implementations where the
signature is dictated by the trait, including but not limited to `Index`,
`IndexMut`, `Iterator::nth`, and `Read::read`.

The lint MUST NOT fire on private functions or on restricted-visibility items
such as `pub(crate)` and `pub(super)`.

#### Scenario: Public function with usize parameter

- GIVEN `pub fn get_item(index: usize) -> &Item`
- WHEN the lint runs
- THEN it warns that `usize` in a public API harms cross-platform consistency

#### Scenario: Trait-directed signature

- GIVEN `impl Index<usize> for MyVec { fn index(&self, idx: usize) -> &T { ... } }`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Too Many Parameters

The lint MUST warn on named function items with more than a configurable
number of parameters. The default threshold MUST be 5.

`self`, `&self`, and `&mut self` receivers MUST NOT count toward the threshold.

The lint MUST suggest grouping parameters into a named struct.

The lint MUST evaluate free functions, inherent methods, and trait method
signatures. Closures are out of scope.

The lint MUST NOT fire on trait implementations.

The lint MUST NOT fire in the repo's supported test contexts, including
`#[test]` functions and `#[cfg(test)]` items detected by the shared
test-context helper.

The threshold MUST be configurable via `[tigerstyle].max_parameters` in
`dylint.toml`, using the same repo-owned config loading path as other numeric
thresholds.

#### Scenario: Six parameters at default threshold

- GIVEN `fn connect(host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool, proxy: &str)`
- WHEN the lint runs
- THEN it warns that the function exceeds the default limit of 5 parameters

#### Scenario: Five parameters at default threshold

- GIVEN a function with exactly 5 counted parameters
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Trait declaration still warns

- GIVEN `trait Connector { fn connect(&self, a: u8, b: u8, c: u8, d: u8, e: u8, f: u8); }`
- WHEN the lint runs
- THEN it warns because the declaration still exposes a long positional API

#### Scenario: Custom threshold

- GIVEN `[tigerstyle] max_parameters = 7` in `dylint.toml`
- WHEN the lint runs on a function with 6 counted parameters
- THEN it does NOT fire
