# Change: Kani property checklist

## Why

The Kani paper's Hifitime case study shows that contract-backed harnesses become most valuable when they target recurring property classes: normalization, algebraic laws, encode/decode identity, validity ranges, loop termination, and standard-library trait consistency. Octet should adapt those classes into a reusable checklist for Kani and property-test adoption.

## What Changes

- Add a Kani/property checklist for critical Rust modules.
- Classify harnesses by property class so receipts and reviews say what behavior was checked.
- Require both positive and negative evidence for each adopted checklist item.
- Include guidance for graduating from panic-freedom harnesses to contract or proof-rail evidence.

## Impact

- **Files**: checklist docs/templates, receipt property-class fields, examples, and fixture coverage.
- **Consumers**: teams get a concrete menu of high-value properties rather than generic "add formal methods" guidance.
- **Non-claims**: A checklist item records selected property coverage only; it does not prove total module correctness unless the checked property states and proves that scope.
