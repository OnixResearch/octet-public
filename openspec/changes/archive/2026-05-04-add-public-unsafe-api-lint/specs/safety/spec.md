## ADDED Requirements

### Requirement: Public Unsafe API Lint [r[public-unsafe-api-lint]]

The lint MUST be able to warn when normal Rust public APIs expose `unsafe fn` directly instead of containing the unsafe operation behind a safe public wrapper and a small private unsafe helper.

The first implementation MUST cover public free functions and public inherent methods declared `unsafe fn` when the signature uses the normal Rust ABI.

The lint MUST NOT warn on private unsafe functions, trait-dictated methods, FFI or exported `extern` shims, macro-generated signatures, or test-context fixtures in the first implementation scope.

The lint MUST emit advisory or warning-level diagnostics without a machine-applicable suggestion because removing `unsafe` or wrapping the API requires human review of the safety contract.

#### Scenario: Public unsafe free function warns [r[public-unsafe-api-lint.scenario.public-free-fn]]

- GIVEN `pub unsafe fn read_raw(ptr: *const u8) -> u8 { ... }`
- WHEN the lint runs
- THEN it warns that the public API leaks unsafe caller obligations instead of containing the boundary

#### Scenario: Public unsafe inherent method warns [r[public-unsafe-api-lint.scenario.public-inherent-method]]

- GIVEN `impl Buffer { pub unsafe fn from_raw(ptr: *mut u8) -> Self { ... } }`
- WHEN the lint runs
- THEN it warns at the public unsafe method boundary

#### Scenario: Private unsafe helper is accepted [r[public-unsafe-api-lint.scenario.private-helper-accepted]]

- GIVEN a safe public wrapper that delegates to a private `unsafe fn` helper
- WHEN the lint runs
- THEN it does NOT warn because the unsafe obligation is contained inside the module boundary

#### Scenario: FFI shim is accepted [r[public-unsafe-api-lint.scenario.ffi-shim-accepted]]

- GIVEN `pub unsafe extern "C" fn exported(ptr: *const u8) -> u8 { ... }`
- WHEN the lint runs
- THEN it does NOT warn in the first implementation scope because FFI boundary policy belongs to a separate wrapper/error contract

#### Scenario: Trait-dictated unsafe method is accepted [r[public-unsafe-api-lint.scenario.trait-method-accepted]]

- GIVEN an implementation method whose unsafe signature is dictated by a trait
- WHEN the lint runs
- THEN it does NOT warn in the first implementation scope
