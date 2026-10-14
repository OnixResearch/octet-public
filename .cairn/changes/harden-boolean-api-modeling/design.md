## Context

The existing `bool_naming` lint checks `let` bindings. It does not inspect function parameters.

Clippy provides `fn_params_excessive_bools`, but that rule targets multiple booleans. One public boolean can still hide API meaning.

A fully name-based rule would create unstable semantic guesses. A public-boundary rule has a clear syntactic contract.

## Completion Contract

Completion requires a strict lint that reports every direct `bool` parameter on public crate-owned callable APIs. It also reports non-public callables with multiple direct boolean parameters.

A warning-only rollout, baseline suppression, generated-code finding, external-trait break, or machine-applicable type rewrite is false completion.

## Decisions

### Decision: Check public direct boolean parameters

**Choice:** Report direct `bool` parameters on public free functions, inherent methods, and crate-owned trait method declarations.

**Rationale:** Public call sites carry no parameter names. A domain type makes the meaning visible and prevents unrelated values from mixing.

### Decision: Check excessive private boolean parameters

**Choice:** Report non-public functions, methods, and closures when their direct boolean-parameter count reaches a named constant.

The initial constant represents two parameters. The implementation must name it and document its role.

**Rationale:** Multiple private booleans create combinatorial and call-site ambiguity. One private predicate flag can remain practical.

### Decision: Limit the first rule to direct `bool`

**Choice:** The first version does not inspect `Option<bool>`, generic aliases, fields, return values, or transitive wrapper contents.

**Rationale:** Direct HIR types provide a precise first boundary. Broader semantic type analysis needs separate evidence.

### Decision: Preserve externally owned signatures

**Choice:** Skip methods that implement an externally owned trait signature. Keep crate-owned trait declarations in scope.

Generated expansion output also remains outside the lint.

**Rationale:** Consumers cannot redesign an external signature. Crate-owned traits define local API policy.

### Decision: Do not provide an automatic rewrite

**Choice:** Emit help that names enums, option records, typestate, or nominal wrappers. Do not emit a machine-applicable suggestion.

**Rationale:** The correct replacement depends on domain meaning and compatibility policy.

### Decision: Reuse Clippy as independent evidence

**Choice:** Add `clippy::fn_params_excessive_bools` to the reviewed co-reporting profile. Keep Octet and Clippy finding identities separate.

**Rationale:** Independent agreement is useful. Clippy does not replace the public single-boolean rule.

### Decision: Migrate before deny-all activation

**Choice:** Add the lint, metadata, fixtures, and migration report first. Fix Octet and selected consumer APIs without warning budgets or baselines.

Activate the lint in the full deny-all catalog only when the declared migration set is clean.

**Rationale:** Full-catalog activation must remain an error gate. A warning budget would weaken the project rule.

## Lint Boundary

The lint operates in the Dylint structure domain. It inspects callable visibility, owner, trait context, expansion status, and direct parameter types.

It performs no filesystem, process, environment, clock, or network effects.

The lint message identifies the parameter and callable. Help text explains that boolean returns and predicate values remain valid.

## Exceptions

An unavoidable public compatibility surface can use one narrow source expectation. The expectation must include a specific reason and named owner.

Crate-wide, module-wide, workspace-wide, generated baseline, and warning-budget suppression remain forbidden.

## Positive and Negative Matrix

Positive fixtures include private single predicate flags, boolean returns, boolean fields, domain enums, option records, nominal wrappers, generated code, and external trait implementations.

Negative fixtures include public free functions, inherent methods, crate-owned trait methods, and private callables with multiple direct booleans.

Migration fixtures cover API changes and narrow justified compatibility expectations.

## Risks and Trade-offs

- New wrappers can add type volume. Review must choose the smallest domain type that improves meaning.
- Public API migrations can break consumers. Release notes and compatibility tests remain required.
- Visibility and trait ownership rules can be subtle in HIR. UI fixtures must cover each boundary.

## Claim Boundary

A clean result proves only that covered callable shapes avoid direct raw boolean parameters under this rule. It does not prove valid state modeling or API correctness.
