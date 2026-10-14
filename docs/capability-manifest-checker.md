# Capability manifest checker

Octet capability manifests make effect authority reviewable without changing Rust syntax.
The manifest is Nickel-authored in `standards/policies/registry.ncl`, validated by
`standards/policies/contracts.ncl`, exported into deterministic checker artifacts,
and consumed by `octet-standards` architecture checks.

A capability-api mechanism declares:

- enforcement mode: `strict` or `advisory`
- scope classes: `core`, `shell`, `adapter`, `test`, `generated`, or `remote-execution`
- capability families: `time`, `randomness`, `filesystem`, `network`, `process`,
  `environment`, `secrets`, `remote-execution`, `unsafe-code`, `global-state`,
  `concurrency`, `observability`, or `unknown`
- direct platform-effect patterns forbidden in core scopes
- injected traits or authority tokens accepted by core scopes
- shell interpreters and deterministic handler artifacts with replay commands

`octet-standards check --family architecture` emits normal diagnostics plus a
`capability_reports` JSON field. Markdown output includes the same report under
`## Capability Manifest Reports`. Each observation names the resolved scope,
capability family, matched source shape, linked lint IDs, evidence tier, replay
artifact, and whether the observation was accepted or diagnostic.

Strict manifests fail closed when required core scopes, handler artifacts, replay
commands, success-path coverage, or failure-path coverage are missing. Unknown
scope classes and capability families are rejected by Nickel before exported JSON
is consumed.

## Boundaries

The checker validates declared policy and observed source shapes only. It does
not add Rust effect types or ability syntax, does not infer transitive effects,
does not prove whole-program purity, and does not prove complete runtime-effect
absence or semantic correctness.
