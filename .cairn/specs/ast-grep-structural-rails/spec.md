# Ast-grep Structural Rails Specification

## Purpose

Define Octet's ownership model for ast-grep syntax-aware rule packs, candidate findings, and scoped codemods.

## Requirements

### Requirement: Structural rail profile
r[octet.ast_grep_structural_rails.profile] Octet MUST define an ast-grep structural rail profile that declares rule-pack layout, scan scope, severity posture, promotion lanes, and non-claim boundaries.

#### Scenario: Advisory rule pack is discovered
- GIVEN an Octet checkout contains an ast-grep rule pack with declared scan scope and advisory posture
- WHEN the structural rail enumerates available rules
- THEN Octet MUST report the rule pack as syntax-aware candidate evidence and MUST NOT promote it to proof or release evidence by default.

### Requirement: Positive and negative rule tests
r[octet.ast_grep_structural_rails.rule_tests] Every ast-grep rule that participates in advisory, warning, error, or codemod posture MUST have positive and negative rule tests before promotion.

#### Scenario: Rule without negative tests stays inventory-only
- GIVEN an ast-grep rule has matching examples but no negative examples
- WHEN Octet evaluates rule promotion
- THEN Octet MUST keep the rule inventory-only and MUST report the missing negative coverage deterministically.

### Requirement: Rule bundle identity
r[octet.ast_grep_structural_rails.bundle_identity] Octet MUST compute or record a BLAKE3 identity for the ast-grep config, rules, tests, and declared scan scope used by a structural rail run.

#### Scenario: Rule bundle changes after evidence capture
- GIVEN a stored structural rail receipt names a rule bundle identity
- WHEN the checked-in rule files or scan scope change
- THEN Octet MUST treat previous findings as stale for the changed bundle until a fresh scan receipt is captured.

### Requirement: Findings are candidate evidence
r[octet.ast_grep_structural_rails.findings] Octet MUST preserve ast-grep findings as candidate source-shape evidence with file spans, rule ids, severities, metavariables when available, diagnostics, and explicit non-claims.

#### Scenario: Finding reports an unsafe block
- GIVEN ast-grep reports an `unsafe` block source shape
- WHEN Octet records the finding
- THEN Octet MAY route it for trust-boundary review but MUST NOT claim unsafety, memory unsoundness, or proof failure solely from the match.

### Requirement: Codemods stay dry-run-first
r[octet.ast_grep_structural_rails.codemods] Octet MUST keep ast-grep rewrites dry-run-first, interactive, or scoped to an explicit migration session unless validated rule tests and focused checks justify automatic application.

#### Scenario: Rewrite recipe proposes a migration
- GIVEN an ast-grep rewrite has a tested rule and a declared target scope
- WHEN a migration session runs the recipe
- THEN Octet MUST preserve the proposed diff, validation commands, and non-claims before treating the codemod as accepted.

### Requirement: Valence export boundary
r[octet.ast_grep_structural_rails.valence_export] Octet MUST export ast-grep structural findings to Valence only as recorded-only or boundary evidence unless a stronger Octet rail separately promotes the claim.

#### Scenario: Candidate finding becomes Evidence IR
- GIVEN an ast-grep finding is exported through the Valence adapter
- WHEN Valence validates the artifact
- THEN the artifact MUST retain source span identity, rule bundle identity, verification role, and non-claims without claiming behavioral correctness.

### Requirement: Structural rail validation
r[octet.ast_grep_structural_rails.validation] Octet MUST validate structural rail changes with ast-grep rule tests, focused scan fixtures, artifact verification, Cairn gates, and the smallest relevant Octet checks.

#### Scenario: Rail validation runs
- GIVEN a change modifies the ast-grep structural rail profile or promoted rules
- WHEN validation executes
- THEN the evidence MUST include rule-test results, representative scan output, receipt/artifact checks, and Cairn gate results for the changed lifecycle package.
