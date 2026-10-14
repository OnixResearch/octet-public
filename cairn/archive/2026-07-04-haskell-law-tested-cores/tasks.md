## Phase 1: Implementation

- [x] [serial] r[octet.law_tested_deterministic_cores.catalog] Inventory pure cores that need law coverage, prioritizing config precedence, diagnostic ordering, artifact/receipt identity, boundary scans, and lint classifier helpers.
- [x] [serial] r[octet.law_tested_deterministic_cores.catalog] Add a checked-in law catalog or test module comments that name each adopted law and its owning pure core.
- [x] [serial] r[octet.law_tested_deterministic_cores.reproducible] Add reproducible property-test configuration with named case budgets and replay guidance for failing generated inputs.
- [x] [serial] r[octet.law_tested_deterministic_cores.negative_cases] Add negative property or generated-case tests for malformed config, contradictory state, duplicate identities, unstable ordering, and invalid classifier facts.
- [x] [serial] r[octet.law_tested_deterministic_cores.fixture_complement] Keep existing UI, fixture, golden-output, and transcript rails intact for user-visible behavior.

## Phase 2: Verification

- [x] [serial] r[octet.law_tested_deterministic_cores.catalog.config_precedence] Verify config merge/disable/level precedence laws with generated valid configs and concrete examples.
- [x] [serial] r[octet.law_tested_deterministic_cores.catalog.deterministic_identity] Verify repeated artifact, receipt, diagnostic, or manifest construction is byte-stable for equivalent inputs.
- [x] [serial] r[octet.law_tested_deterministic_cores.reproducible.failure_replay] Verify a failing generated case can be reproduced from saved output or a checked-in regression input.
- [x] [serial] r[octet.law_tested_deterministic_cores.negative_cases.fail_closed] Verify invalid generated inputs return typed errors or deterministic diagnostics instead of panicking.
- [x] [serial] Run focused law suites, affected existing fixture/UI/runner tests, and Cairn validate/gates before closeout. [depends:cairn-gates]
