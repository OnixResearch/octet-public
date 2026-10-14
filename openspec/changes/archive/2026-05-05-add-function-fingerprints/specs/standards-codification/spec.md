## ADDED Requirements

### Requirement: Function Fingerprint Policy [r[function-fingerprint-policy]]

The system MUST define a Tigerstyle policy for selective content-addressed function fingerprints that bind safety-critical Rust functions to exact reviewed or verified source-token bodies.

The policy MUST state that fingerprints are identity and review gates, not correctness proofs. It MUST recommend fingerprints for verified primitives, protocol transitions, security-sensitive functions, capability/token constructors, safety-critical arithmetic, and fixture-backed documentation examples. It MUST NOT require blanket fingerprints for every function in a workspace.

#### Scenario: Critical function categories are documented [r[function-fingerprint-policy.scenario.critical-categories]]

- GIVEN a maintainer reads the function fingerprint guidance
- WHEN they decide whether a function should be fingerprinted
- THEN the guidance names verified primitives, protocol transitions, security-sensitive functions, capability/token constructors, safety-critical arithmetic, and fixture-backed examples as recommended categories
- AND the guidance says ordinary functions do not need fingerprints by default

#### Scenario: Fingerprints do not claim correctness [r[function-fingerprint-policy.scenario.identity-not-correctness]]

- GIVEN a fingerprinted function matches its stored hash
- WHEN the tool reports the fingerprint as valid
- THEN the documentation and diagnostic language state that the exact source body matches the reviewed body
- AND they do not claim the function is semantically correct, safe, or verified solely because the hash matches

### Requirement: Function Fingerprint Metadata [r[function-fingerprint-metadata]]

The system MUST require review metadata for function fingerprints so each selected function records why content identity matters.

At minimum, each fingerprint MUST name the hash value and review class. Supported review classes MUST include `verified`, `protocol-transition`, `security`, `capability-token`, `safety-arithmetic`, and `example-evidence`. Optional metadata MAY include owner, spec ID, evidence path, and macro allowance.

#### Scenario: Missing review class is rejected [r[function-fingerprint-metadata.scenario.missing-review-class]]

- GIVEN a function carries a fingerprint hash but omits review class metadata
- WHEN fingerprint validation runs in strict mode
- THEN validation reports the function as incomplete
- AND the diagnostic names the missing review class field

#### Scenario: Review metadata links to evidence [r[function-fingerprint-metadata.scenario.links-to-evidence]]

- GIVEN a fingerprinted verified primitive or protocol transition names a spec ID or evidence path
- WHEN fingerprint inventory output is generated
- THEN the output preserves the function name, hash, review class, and evidence reference
- AND reviewers can trace the function identity to the relevant OpenSpec or verification artifact

### Requirement: Function Fingerprint Hash Semantics [r[function-fingerprint-hash-semantics]]

The system MUST compute function fingerprints from a versioned, domain-separated, normalized representation of Rust function source tokens.

The hash input MUST include normalized visibility, signature, generic parameters, where clauses, and body tokens. It MUST exclude the fingerprint attribute or marker itself, whitespace, comments, file paths, line numbers, timestamps, local usernames, and absolute paths. The hash algorithm MUST be identified with a `b3:` prefix for BLAKE3 values.

#### Scenario: Formatting and comments do not change hash [r[function-fingerprint-hash-semantics.scenario.formatting-stable]]

- GIVEN a fingerprinted function whose executable tokens are unchanged
- WHEN only rustfmt layout, comments, or line numbers change
- THEN recomputing the fingerprint yields the same `b3:` value

#### Scenario: Signature or body token changes update hash [r[function-fingerprint-hash-semantics.scenario.token-change-detected]]

- GIVEN a fingerprinted function with a stored hash
- WHEN its signature, generics, where clause, visibility, or body tokens change
- THEN fingerprint validation reports a mismatch with expected and actual `b3:` values

### Requirement: Function Fingerprint Macro Boundary [r[function-fingerprint-macro-boundary]]

The system MUST make macro-call limitations explicit for fingerprinted functions.

If a fingerprinted function contains source macro invocations, validation MUST report a macro-boundary warning unless the fingerprint metadata explicitly allows macros. The warning MUST explain that source-token fingerprints do not cover macro definition changes or expansion semantics.

#### Scenario: Macro call warns by default [r[function-fingerprint-macro-boundary.scenario.warns-by-default]]

- GIVEN a fingerprinted function body contains a macro invocation
- WHEN fingerprint validation runs
- THEN validation emits a macro-boundary diagnostic
- AND the diagnostic explains that macro definitions are not covered by the function source-token hash

#### Scenario: Explicit macro allowance suppresses caveat [r[function-fingerprint-macro-boundary.scenario.explicit-allowance]]

- GIVEN a fingerprinted function body contains a macro invocation and metadata explicitly allows macros
- WHEN fingerprint validation runs
- THEN validation does not emit the default macro-boundary warning
- AND inventory output preserves that macro allowance for review
