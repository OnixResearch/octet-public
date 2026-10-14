# Evidence Provenance Specification Delta

## ADDED Requirements

### Requirement: Binary debug-map profiles are explicit and bounded

r[octet.evidence_provenance.binary_debug_map_profile]

Octet MUST define an opt-in binary debug-map profile with exact object, debug-format, source-mapping, producer, and resource-limit policy.

#### Scenario: Admitted ELF and DWARF profile loads

- GIVEN a valid `elf-dwarf-v1` profile with every required limit and source-root mapping
- WHEN Octet loads the profile for binary debug-map collection
- THEN profile validation MUST pass with one deterministic profile identity
- AND the identity MUST bind every admitted format and limit field

#### Scenario: Missing or unsupported profile field fails

- GIVEN a profile omits a required limit or selects an unsupported debug format
- WHEN Octet validates the profile
- THEN validation MUST fail before binary parsing
- AND no default limit or format MUST be inferred from ambient state

### Requirement: Binary debug-map evidence links exact bytes to source spans

r[octet.evidence_provenance.binary_debug_map_linkage]

Octet MUST bind normalized symbol and source-span rows to exact binary, debug-companion, profile, and producer identities.

#### Scenario: Inline debug data maps successfully

- GIVEN an admitted binary contains valid DWARF and maps to a verified Octet function-address artifact
- WHEN the binary debug-map core evaluates supplied bytes
- THEN it MUST emit deterministic bounded rows with exact binary and source identities
- AND demangled names MUST remain display metadata rather than canonical function identities

#### Scenario: Split debug companion maps successfully

- GIVEN an admitted stripped binary and its exact matching split-debug companion
- WHEN the evidence rail evaluates both artifacts
- THEN each emitted row MUST bind both BLAKE3 identities
- AND the receipt MUST distinguish binary-local addresses from stable source identities

#### Scenario: Ambiguous function mapping fails

- GIVEN one binary row matches several function-address candidates without exact source and span agreement
- WHEN strict linkage validation runs
- THEN validation MUST fail with an ambiguity issue
- AND symbol-name similarity MUST NOT select a candidate

### Requirement: Binary debug-map failures remain fail-closed and bounded

r[octet.evidence_provenance.binary_debug_map_failure_boundary]

Octet MUST reject malformed, stale, unsupported, unsafe, or over-limit binary debug-map inputs without publishing a successful receipt.

#### Scenario: Wrong debug companion fails

- GIVEN a split-debug companion does not match the selected binary identity or build identity
- WHEN collection runs
- THEN collection MUST fail before successful bundle publication
- AND diagnostics MUST identify the mismatched role without printing private paths

#### Scenario: Absolute source path is unmapped

- GIVEN DWARF contains an absolute source path outside every admitted mapping
- WHEN path normalization runs
- THEN the row MUST fail or remain excluded according to explicit profile policy
- AND the receipt MUST NOT disclose the absolute build root

#### Scenario: Resource limit is exceeded

- GIVEN a binary exceeds a profile limit for bytes, sections, units, symbols, ranges, lines, frames, paths, or rows
- WHEN parsing reaches that limit
- THEN collection MUST stop with a deterministic limit issue
- AND truncation MUST NOT be reported as complete evidence

### Requirement: Binary debug-map handoff preserves evidence roles

r[octet.evidence_provenance.binary_debug_map_handoff]

Octet MUST export binary debug-map receipts as bounded recorded linkage evidence with typed Valence and release handoff fields.

#### Scenario: Valid typed requirement link is preserved

- GIVEN a debug-map row carries a current typed Valence requirement reference
- WHEN Octet exports the receipt downstream
- THEN the handoff MUST preserve repository, revision, registry, requirement, binary, and source identities
- AND downstream consumers MUST be able to reject stale identity domains

#### Scenario: Claim promotion fails

- GIVEN a consumer labels binary debug-map evidence as formal proof, semantic equivalence, or requirement satisfaction
- WHEN the handoff validator evaluates that role
- THEN validation MUST fail with a role-promotion issue
- AND the original recorded-only evidence identity MUST remain unchanged
