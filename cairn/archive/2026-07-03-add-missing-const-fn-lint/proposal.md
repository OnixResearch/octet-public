## Why

`const fn` is a compiler-checked subset that gives simple pure helpers a stronger contract. When a helper is pure-looking and const-compatible, making it `const fn` lets Rust reject future heap, IO, and other runtime-only behavior in that helper.

## What Changes

- Define a `missing_const_fn` advisory lint for functions that appear const-compatible but are not marked `const`.
- Start with conservative private/free-function shapes over primitive inputs and outputs.
- Require guardrails for trait methods, async functions, functions with allocations, calls that are not const-safe, and functions whose public API stability would be affected.

## Impact

- **Files**: `src/purity/missing_const_fn.rs`, registration, metadata, lint reference, templates, and fixtures.
- **Testing**: positive fixtures for simple arithmetic/match helpers and negative fixtures for non-const operations, trait methods, async, allocation, and shell calls.
