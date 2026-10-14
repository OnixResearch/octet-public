## ADDED Requirements

### Requirement: Rollout Kit Covers Low-Noise New-Lint Promotion

The downstream rollout kit MUST document how a lint family graduates from
`starter-candidate` classification into the copied starter profile.

The guidance MUST include:

- which low-noise new lint families are currently promoted into the starter
  profile
- which pending families remain inventory-first and therefore stay disabled in
  copied starter examples, README rollout text, and consumer template comments
- the documented starter package set that remains in scope for this promotion:
  `aspen-time`, `aspen-hlc`, and `aspen-core`
- saved Aspen evidence for the promoted starter profile, not only the earlier
  classification pass
- the first-class `cargo-tigerstyle` replay path used for that evidence
- deterministic evidence artifacts under the change `evidence/` directory:
  `replay-aspen-promoted-starter.sh`, `check-aspen-promoted-starter.py`,
  `aspen-promoted-starter-config.toml`,
  `aspen-promoted-starter-default.txt`,
  `aspen-promoted-starter-coordination.txt`, and
  `aspen-promoted-starter-counts.txt`
- separate saved proof for both the promoted starter scope and a focused
  `aspen-coordination` lib-only spotcheck that uses the same promoted lint set
- the exact promoted starter-profile configuration shape consumers should copy

#### Scenario: Maintainer copies the promoted starter profile

- GIVEN `explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
  `negated_predicate` have already met the saved low-noise rule
- WHEN a maintainer copies the starter / boundary-first rollout example
- THEN those four families are enabled in that copied profile
- AND `unbounded_collection_growth` plus `too_many_parameters` remain disabled
  until a later promotion review lands

#### Scenario: Maintainer reviews promotion proof

- GIVEN a maintainer wants to confirm the promoted starter profile before
  copying it into a consumer workspace
- WHEN they follow the rollout kit
- THEN they can find the exact `cargo-tigerstyle` replay path
- AND separate saved transcripts for the promoted starter scope and the focused
  `aspen-coordination` lib-only spotcheck
- AND the checker/count artifact sits next to those transcripts under the
  change `evidence/` directory
- AND the proof still pins the starter package set to `aspen-time`,
  `aspen-hlc`, and `aspen-core`
