## Phase 1: Pilot selection and harness convention

- [x] [serial] r[octet.kani_panic_pilots.selection] Select pilot surfaces using documented criteria for functional-core, arithmetic/encoding, unsafe-boundary, and compact state-transition code.
- [x] [serial] r[octet.kani_panic_pilots.harness_convention] Define a harness convention for property class, input admission, assumptions, Kani mode, replay, and receipt linkage.
- [x] [serial] r[octet.kani_panic_pilots.assumption_review] Add review guidance that treats `kani::assume` and safety-invariant admission as proof-critical inputs.

## Phase 2: Pilot evidence

- [x] [parallel] r[octet.kani_panic_pilots.positive_evidence] Add passing pilot harnesses for selected panic-freedom or boundary-safety properties.
- [x] [parallel] r[octet.kani_panic_pilots.negative_evidence] Add negative pilot fixtures or replay cases for malformed inputs, invalid assumptions, or expected failing assertions.
- [x] [serial] r[octet.kani_panic_pilots.receipt_linkage] Link pilot harness outputs to Kani rail receipts without promoting them to whole-workspace claims.

## Phase 3: Documentation and validation

- [x] [serial] r[octet.kani_panic_pilots.boundary_docs] Document pilot scope, non-claims, and how consumers should graduate from panic-freedom to contracts.
- [x] [serial] r[octet.kani_panic_pilots.selection] Run pilot checks, negative fixtures, artifact verification, and Cairn validation/gates before archive.

## Verification Coverage

| Scenario | Planned Evidence |
|---|---|
| Pilot surface selection is documented | Guide check |
| Passing panic-freedom harness records evidence | Positive Kani pilot receipt |
| Assumption review is visible | Checklist or docs fixture |
| Malformed input is rejected | Negative fixture or replay |
| Invalid proof promotion is blocked | Artifact verification fixture |
| Pilot boundary is documented | Docs check or transcript |
