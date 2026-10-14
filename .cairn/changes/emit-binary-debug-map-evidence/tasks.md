# Tasks: Emit binary debug-map evidence

## Contracts and dependencies

- [ ] [serial] Define the Nickel-authored `elf-dwarf-v1` profile, explicit source-root mappings, and bounded resource limits r[octet.evidence_provenance.binary_debug_map_profile]
- [ ] [serial] Add pinned `object`, `gimli`, and `rustc-demangle` dependencies to a std evidence producer with license and source review r[octet.evidence_provenance.binary_debug_map_profile]
- [ ] [serial] Define Rust-owned receipt DTOs and generate their Nickel compatibility contract r[octet.evidence_provenance.binary_debug_map_profile]

## Core and shell

- [ ] [serial] Implement pure bounded object, DWARF, symbol, range, line, and source-map normalization over supplied bytes r[octet.evidence_provenance.binary_debug_map_linkage]
- [ ] [serial] Implement exact binary, debug companion, function address, and typed requirement linkage r[octet.evidence_provenance.binary_debug_map_linkage]
- [ ] [serial] Add the explicit `binary-debug-map` collection shell with capability-rooted reads and atomic bundle publication r[octet.evidence_provenance.binary_debug_map_handoff]

## Verification

- [ ] [parallel] Add positive inline-DWARF and split-debug fixtures with deterministic receipt goldens r[octet.evidence_provenance.binary_debug_map_linkage]
- [ ] [parallel] Add negative fixtures for malformed sections, wrong companions, unmapped paths, stale spans, ambiguity, truncation, and every resource-limit class r[octet.evidence_provenance.binary_debug_map_failure_boundary]
- [ ] [parallel] Add Valence handoff fixtures that preserve recorded-only roles and reject claim promotion r[octet.evidence_provenance.binary_debug_map_handoff]

## Documentation and closeout

- [ ] [parallel] Document supported formats, profile limits, stripped-binary behavior, privacy, and non-claims r[octet.evidence_provenance.binary_debug_map_failure_boundary]
- [ ] [serial] Run focused tests, generated-contract freshness, artifact verification, formatting, strict first-party lint rails, Cairn validation, and traceability coverage r[octet.evidence_provenance.binary_debug_map_profile]
- [ ] [serial] Run proposal, design, and tasks gates before sync and archive r[octet.evidence_provenance.binary_debug_map_handoff]
