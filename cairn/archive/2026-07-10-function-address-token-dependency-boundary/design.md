## Design

Octet keeps the existing dependency hash algorithm but makes its kind explicit in reports and baselines. The function-address report should expose an identity kind such as `source-token-set` alongside the hash and status. Any text claiming resolved dependencies, semantic call graph, or whole closure is rejected.

### Decisions

1. **Identity kind is explicit.** Dependency hash records carry a kind that names source-token identity.
2. **Compatibility is additive.** Existing `dependency_set_hash` remains readable while new receipts can prefer the explicit kind.
3. **Overclaims fail closed.** Baselines and downstream projections must not describe the token set as semantic dependency closure.
4. **Algorithm stays deterministic.** The source-token hash remains pure and independent of compiler execution.

### Validation shape

Positive fixtures show token-set dependency identities for simple calls. Negative fixtures include overclaiming labels, missing identity kind, and text describing the token set as complete semantic closure.
