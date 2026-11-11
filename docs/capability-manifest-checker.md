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
commands, success-path coverage, or failure-path coverage are missing. Handler
artifacts and replay commands pair with deterministic handlers by position:
each handler needs its own entry in both lists, and a handler without one gets a
diagnostic naming it rather than borrowing another handler's command. A handler
artifact matches a checked source path exactly or on whole trailing path
components, so `fixtures/legacy_tests/fs-handler.txt` does not satisfy
`tests/fs-handler.txt`. Unknown scope classes and capability families, and
strict manifests whose handler, artifact, and replay-command lists differ in
length, are rejected by Nickel before exported JSON is consumed.

A handler artifact shows success-path coverage when it contains one of the whole
words or phrases `success`, `succeeds`, `ok`, or `happy path`, and failure-path
coverage when it contains `failure`, `fails`, `error`, or `sad path` (ASCII
case-insensitive). Markers inside longer words do not count, so `token` is not
`ok` and `terror` is not `error`. Labelled lines such as
`success path: fake clock returns deterministic millis` and
`failure path: fake clock returns configured error` are the recommended form.

## Boundaries

The checker validates declared policy and observed source shapes only. It does
not add Rust effect types or ability syntax, does not infer transitive effects,
does not prove whole-program purity, and does not prove complete runtime-effect
absence or semantic correctness.
