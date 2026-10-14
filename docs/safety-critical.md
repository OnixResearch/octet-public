# Safety-critical claims guide

Tiger Style helps teams make assumptions explicit, keep failure modes
reviewable, and demand named evidence for high-consequence work. Use this page
to keep safety-sensitive wording accurate. For Octet's product-level
scope-boundary and evidence path, see `trust-boundary-evidence.md#what-octet-does-not-claim`.

## What Tiger Style can claim

Tiger Style can claim that this repo ships reviewable engineering surfaces:
lints, guides, templates, and saved evidence paths. When those surfaces are
used well, they help teams surface assumptions, shrink classes of mistakes,
and make review notes concrete.

- Reviewer prompt: Which exact lint, guide, template, or saved artifact
  supports this sentence?
- Wording rule: Say `Tiger Style checks`, `Tiger Style reviews`, or
  `Tiger Style documents` only when you can name the shipped surface.
- Wording rule: Prefer `Tiger Style makes safety-related assumptions explicit
  and reviewable` over `Tiger Style makes the system safe`.

## What Tiger Style cannot claim

A clean Octet run is not a certification, regulator approval, or complete
safety case. Passing tests, green CI, or a finished review kit do not by
themselves prove that a product is safe for a high-consequence deployment.

- Reviewer prompt: Does this wording imply approval, certification, or
  completeness that the named evidence does not support?
- Wording rule: Do not write that Tiger Style `guarantees`, `certifies`, or
  `proves` overall safety unless a separate proof or certification process is
  named and scoped.
- Wording rule: Do not treat lint cleanliness, test passes, or review
  completion as the whole safety argument.

## Lints, tests, proof, assurance, and certification

Lints flag or forbid specific code patterns. Tests sample behavior under chosen
inputs. Formal proof establishes a stated property inside a stated model and
proof boundary. Assurance evidence collects multiple artifacts and judgments
into a broader argument. Certification is an external process with its own
authority, standards, and acceptance criteria.

- Reviewer prompt: Which of these surfaces is actually in hand, and which ones
  are only being implied?
- Wording rule: Name the exact surface: `linted`, `tested`, `proved for
  property X`, `reviewed with named evidence`, or `certified by Y`.
- Wording rule: Never collapse one surface into another; proof is not
  certification, tests are not proof, and lints are not a complete assurance
  case.

## Evidence and human review

High-consequence claims need named evidence and named human judgment. If a
README, rollout note, or review summary makes a strong safety-sensitive claim,
it should point to the exact artifacts and the human review step that support
that claim.

- Reviewer prompt: What named evidence artifacts support this claim, and who
  reviewed them?
- Reviewer prompt: If automation went green but no human read the argument,
  should this claim be softened?
- Wording rule: For high-consequence claims, cite the exact lint run, test
  report, proof artifact, design review, or hazard analysis rather than
  gesturing at `verification` in the abstract.
- Wording rule: Green automation is useful input, not a substitute for human
  review.

## Unsafe and FFI boundary evidence

For new or changed unsafe, raw-pointer, FFI, lifetime-erasing, foreign
allocation, or external ownership seams, use the
[unsafe/trusted-boundary record](rust-idiom-guidance.md#unsafe-trusted-boundary-records).
The record names the boundary, invariant, caller obligation, safe wrapper or
exception rationale, evidence tier, dynamic evidence selection, reviewer action,
and explicit non-claim. It is review evidence, not proof by itself.

## Responsible wording in docs and reviews

Prefer wording that names scope, evidence, and limits. Avoid wording that
sounds like certification, blanket assurance, or whole-system proof when the
repo only supports narrower claims.

- Reviewer prompt: If this sentence were quoted alone, would it still describe
  the evidence honestly?
- Preferred: `Tiger Style lints and review guides help make safety-related
  assumptions explicit and reviewable.`
- Preferred: `This change reduces one class of risk and adds named evidence
  for that boundary.`
- Preferred: `The repo has lint coverage, tests, and review artifacts for this
  claim; broader assurance still needs separate evidence.`
- Avoid: `Tiger Style certifies this system as safe.`
- Avoid: `A clean Octet run proves the system is safety-critical ready.`
- Avoid: `Reviewed and green, therefore safe.`
- Wording rule: In rollout notes and review summaries, state what improved,
  what evidence exists, and what still requires separate assurance work.
