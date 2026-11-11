# SpaceWasm Pilot ABI Admission Specification

## Purpose

Admit exact WebAssembly core-module bytes against the Molten actor ABI and the separate observe/pause diagnostic ABI by resolved static function signatures, bounded parsing, and the reviewed SpaceWasm support intersection, without implying runtime semantics, hardware behavior, authenticated provenance, safety, or release eligibility.

## Requirements

### Requirement: Core function ABI facts are resolved and identity-bound

r[octet.spacewasm_pilot.resolved_abi] Octet MUST resolve imported and exported
function signatures through function/type indices, emit deterministic ABI facts,
and bind them into normalized interface identity separately from exact-byte identity.
The hostless SpaceWasm run export MUST have signature `()->()`.

#### Scenario: Export name matches but signature does not
- GIVEN a core module exporting run with an i32 result
- WHEN analyzed with spacewasm-mvp
- THEN an abi-mismatch error MUST deny conformance.

#### Scenario: Imported function shifts exported function index
- GIVEN distinct imported and defined function types
- WHEN the defined function is exported
- THEN the exported ABI MUST resolve its actual defined type, not the import type.

### Requirement: Actual Molten actor ABI has a dedicated static profile

r[octet.spacewasm_pilot.molten_abi] Octet MUST provide spacewasm-molten-abi-pilot
requiring memory, molten_alloc(i32)->i32, molten_dealloc(i32,i32)->(), and
molten_run(i32,i32)->i64. Only optional molten:hostcall assert/send imports with
signature (i32,i32)->i64 MAY be statically admitted. Memory MUST be unshared wasm32,
initially at least one page, with explicit maximum at most 256 pages. Tables,
start functions, memory growth, WASI, components, and unsupported proposals MUST deny.

#### Scenario: Real actor shape is admitted
- GIVEN exact module bytes satisfying the declared actor ABI and resource limits
- WHEN the actual producer collects spacewasm-molten-abi-pilot evidence
- THEN static conformance MUST pass with resolved ABI and resource facts.

#### Scenario: Memory or host contract differs
- GIVEN no memory maximum, excessive maximum, wrong allocator signature, or a forbidden hostcall
- WHEN the actor profile evaluates the bytes
- THEN conformance MUST deny with error findings.

### Requirement: Diagnostic host ABI remains distinct

r[octet.spacewasm_pilot.diagnostic_abi] Octet MUST keep spacewasm-mvp-host-pilot
separate, requiring molten.observe(i32)->(), molten.pause()->(), run()->(), and an
exported mutable i32 total; a diagnostic receipt MUST NOT imply full actor admission.

#### Scenario: Diagnostic import or global differs
- GIVEN wrong observe signature, missing pause, or immutable total
- WHEN the diagnostic profile evaluates the bytes
- THEN ABI conformance MUST deny.

#### Scenario: Core type order differs
- GIVEN observe and pause imports with the required signatures declared at different core type indices
- WHEN the diagnostic profile evaluates the bytes
- THEN the imports MUST be admitted by name and resolved signature, not by type index.

### Requirement: Parsing is bounded and features intersect reviewed support

r[octet.spacewasm_pilot.bounded_intersection] Octet MUST bound artifact reads and
reject oversized artifacts before parser/validator construction, MUST NOT emit
truncated-byte identities, and MUST reject profile proposals outside reviewed
runtime support or within its denied proposals.

#### Scenario: Artifact exceeds selected byte ceiling
- GIVEN artifact bytes exceeding the selected ceiling
- WHEN CLI intake or pure analysis runs
- THEN parsing MUST NOT proceed and the input MUST be denied.

#### Scenario: A profile widens beyond reviewed runtime support
- GIVEN a profile allowing a proposal absent from the reviewed support projection
- WHEN registry validation runs
- THEN the profile MUST be rejected.

### Requirement: Consumers verify actual persisted static receipts

r[octet.spacewasm_pilot.receipt_boundary] The producer MUST retain the existing
persisted evidence-rail receipt schema and bind exact module, registry, profile,
parser cohort, immutable NASA source/runtime cohort, facts, findings, and nonclaims.
Actual intake MUST use artifact verification and candidate-byte equality rather
than accepting a fabricated or mismatched receipt. Static conformance MUST NOT
claim runtime semantics, hardware, authenticated compiler provenance, safety, or
release eligibility. Runtime reports MUST remain recorded-only and cannot override denial.

#### Scenario: Candidate bytes change after receipt generation
- GIVEN an otherwise passing bundle whose bundled module is altered
- WHEN the actual pinned producer verifies artifacts
- THEN verification MUST fail.

#### Scenario: External evidence mismatches or overclaims
- GIVEN a report with wrong module/cohort identity or runtime-proof authority
- WHEN external report intake evaluates it
- THEN it MUST deny rather than override static policy.

### Requirement: Real producer positive and negative smoke is required

r[octet.spacewasm_pilot.operator_smoke] Final acceptance MUST execute focused
SpaceWasm tests and the checked operator smoke with actual pinned producer/parser
executables. It MUST observe positive actor admission plus artifact verification,
and negative unbounded-memory, wrong-host, wrong-allocator, WASI, component,
unsupported-proposal, and exact-byte-tamper denials. Command output MUST be retained
by the final integration owner; unexecuted fixtures MUST NOT be described as passing.

#### Scenario: Operator replays the actual producer
- GIVEN pinned cargo-octet and wasm-tools executable paths
- WHEN smoke.py runs
- THEN every declared positive/negative outcome MUST match or the smoke MUST fail.
