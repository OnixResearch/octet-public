## ADDED Requirements

### Requirement: Pure Result Cache [r[pure-result-cache]]

The system MUST support deterministic pure-result caching for eligible lint, check, documentation, and receipt-generation outputs. Cache keys MUST include command identity, normalized input hashes, configuration hash, tool revision, and an explicit environment allowlist. Commands with undeclared network, secret, clock, randomness, or broad filesystem effects MUST be uncached or partitioned by declared effect identity.

#### Scenario: Pure cache hit reuses matching result [r[pure-result-cache.scenario.cache-hit-reuses-matching-result]]

- GIVEN a deterministic command has the same command identity, input hashes, config hash, tool revision, and environment allowlist as a previous run
- WHEN the pure-result cache is queried
- THEN the cache may reuse the previous result
- AND the hit receipt names the cache key components and stored output hash

#### Scenario: Environment drift invalidates cache [r[pure-result-cache.scenario.environment-drift-invalidates-cache]]

- GIVEN a deterministic command's input hashes are unchanged but the declared environment allowlist or tool revision changes
- WHEN the pure-result cache is queried
- THEN the previous result is not reused as a valid hit
- AND the miss receipt identifies the changed key component without dumping the full environment
