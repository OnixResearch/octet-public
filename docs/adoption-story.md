# Octet adoption story

Octet is a Rust-compatible, evidence-backed checked path for systems code before
release. Existing Rust source enters Octet checks without a rewrite, and the
result is reviewable because every claim points to a command, checked-in policy,
receipt, transcript, or artifact.

## Checked path promise

The adoption path is deliberately concrete:

```text
Rust source
  -> cargo octet check
  -> checked-in config or policy (`dylint.toml`, `[workspace.metadata.octet]`, policy bundles)
  -> evidence artifacts (`status.json`, SARIF, JSONL findings, transcripts, provenance)
  -> CI or release gate
  -> artifact verification before promotion
```

That path covers lint findings, standards checks, optional scoped proof receipts,
provenance, and artifact verification. It is an incremental Rust adoption path,
not a new programming language and not a Rust replacement.

## Public story cards

A first-pass Octet story should fit on one skim surface. These cards are the
canonical inventory for a public landing page, README excerpt, or sales handoff;
longer reference docs should link here instead of repeating the whole story.

| Card | Reader promise | Concrete Octet surface | Boundary |
| --- | --- | --- | --- |
| Checked path card | Existing Rust moves through a local checked path before promotion. | `cargo octet check`, checked-in config, policy bundles, artifact directory, CI or release gate | Not a new language and not a Rust replacement. |
| How it works cards | Each layer names what it checks and what evidence it emits. | static lint edge, Nickel policy layer, evidence chain, optional scoped proof lane, artifact verification, controlled deployment | Validation evidence is not silently upgraded into formal proof. |
| Built with Octet evidence card | Case studies are reviewable because they name source, scope, command, config, artifact, outcome, and boundary. | Aspen phase-1 archive plus future case-study records | Historical rollout evidence is not a claim about every future consumer or Octet HEAD. |
| Before/after examples card | Ordinary Rust failure shapes map to bounded remediation paths. | existing lint families such as `unchecked_division`, `unchecked_narrowing`, `unbounded_channel`, `ambient_clock`, `public_unsafe_api`, and `ignored_result` | Lint examples are not solver models or formal counterexamples. |
| Quickstart card | A reader can copy commands from first check to saved artifacts and artifact verification. | `cargo octet check --artifact-dir artifacts/octet` and `cargo octet artifact verify --artifact-dir artifacts/octet` | Output is local review evidence; consumers still own CI storage and release decisions. |
| FAQ card | Comparison and boundary questions are answered before marketing claims expand. | Clippy comparison, proof scope, unsupported proof claims, Dylint/nightly tradeoff, source/artifact locality, starter-profile noise, and `cargo octet prove` usage | No mathematical certainty, whole-program proof, certification, C performance parity, or verified runtime claim. |

## How it works cards

| Surface | What the card should say | Evidence a reviewer can inspect |
| --- | --- | --- |
| Static lint edge | Octet adds Tiger Style Dylint checks for known source shapes that Rust and Clippy do not target as Octet policy. | stderr, SARIF, JSONL findings, lint names, stable IDs, and remediation text from `cargo octet check`. |
| Nickel policy layer | Human-authored policy and config live in typed Nickel or checked-in config, then export deterministic tool inputs. | `standards/policies/`, generated policy bundles, `octet-policy-validate`, and `octet-standards check`. |
| Evidence chain | A run is reviewable because sidecars and provenance record command, tool, evidence tier, artifact locator, BLAKE3 hash, and checked scope. | `status.json`, `summary.txt`, `results.jsonl`, `results.sarif`, transcripts, fingerprints, and `provenance.jsonl`. |
| Optional scoped proof lane | `cargo octet prove` is opt-in and records verifier-backed receipts only for declared properties and source/proof bindings. | proof receipts, proof cards, bounded logs, `formal_proof` provenance, and artifact verification. |
| Artifact verification | Saved evidence is checked before promotion instead of trusted by path name. | `cargo octet artifact verify --artifact-dir artifacts/octet` recomputes identities and fails closed on stale or malformed bundles. |
| Controlled deployment | Octet runs locally or inside consumer CI with a pinned runner path and local artifacts. | `cargo octet check`, `nix run ...#cargo-octet -- check`, `lib.mkConsumerCheck`, and `trust-boundary-evidence.md#controlled-deployment-posture`. |

## Built with Octet case-study format

A public "Built with Octet" case study is reviewable only when it names every
field below:

| Field | Required content |
| --- | --- |
| consumer or archive source | repository, commit, or archived evidence source |
| checked scope | package, crate, source path, policy family, or proof rail scope |
| command or gate | Octet command, CI check, transcript command, or release gate |
| configuration source | `dylint.toml`, `[workspace.metadata.octet]`, policy bundle, proof rail, or archived config snapshot |
| evidence artifact or archive path | transcript, receipt, sidecar, provenance file, policy artifact, or archive path |
| outcome summary | bounded findings, pass/fail status, suppression decision, or rollout status |
| scope boundary | what the evidence does not prove or what remains manual review |

### Built with Octet: Aspen phase-1 starter rollout

- consumer or archive source:
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`
- checked scope: Aspen commit `909e5e9fa`, starter packages `aspen-time`,
  `aspen-hlc`, and `aspen-core`
- command or gate: archived phase-1 default run captured in
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- configuration source:
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
  plus the archived wrapper/config snapshot named by
  `docs/downstream-rollout-evidence.md`
- evidence artifact or archive path:
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/validation-counts.txt`
  and the default-run transcript above
- outcome summary: the phase-1 starter run produced six findings, all
  `ambient_clock`, on the starter package set; that bounded signal informed the
  starter rollout profile
- scope boundary: this is historical rollout evidence for the archived starter
  scope, not Aspen HEAD, not whole-program proof, not certification, and not a
  claim that every later lint family is low-noise

The Aspen entry intentionally cites the canonical phase-1 archive. Mutable local
Aspen worktrees and later widened scripts are useful for follow-up experiments,
but they are not the source for the starter case study.

## Before and after examples

These examples are story cards, not a replacement for the full lint catalog. Each
one names the lint or policy surface, the deterministic failure shape, and the
bounded remediation path. For a complete first checked Rust patch walkthrough,
see [`docs/examples/first-checked-rust-patch/README.md`](examples/first-checked-rust-patch/README.md).
They are lint and policy examples, not SMT models, not solver models, not formal
counterexamples, not mathematical proof, and not solver results.

### `unchecked_division`: zero guard before division

Before:

```rust
fn average(total: u64, count: u64) -> u64 {
    total / count
}
```

After:

```rust
fn average(total: u64, count: u64) -> Result<u64, &'static str> {
    if count == 0 {
        return Err("count must be non-zero");
    }
    Ok(total / count)
}
```

Failure shape: a divisor can be zero. Remediation: add an explicit guard and
return a recoverable error instead of hiding a panic path.

### `unchecked_narrowing`: checked conversion at an API edge

Before:

```rust
fn encode_len(len: usize) -> u32 {
    len as u32
}
```

After:

```rust
fn encode_len(len: usize) -> Result<u32, std::num::TryFromIntError> {
    u32::try_from(len)
}
```

Failure shape: a platform-sized value can truncate. Remediation: use a checked
conversion or a prior range assertion tied to the same value.

### `unbounded_channel` and `unbounded_collection_growth`: resource boundary

Before:

```rust
let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
```

After:

```rust
let (tx, rx) = tokio::sync::mpsc::channel(config.command_queue_capacity);
```

Failure shape: producer pressure has no visible capacity bound. Remediation:
name the queue capacity in config, document backpressure, and save rollout
evidence when promoting the lint family.

### `ambient_clock`: shell/core split

Before:

```rust
fn expire_at(ttl: std::time::Duration) -> std::time::Instant {
    std::time::Instant::now() + ttl
}
```

After:

```rust
fn expire_at(now: std::time::Instant, ttl: std::time::Duration) -> std::time::Instant {
    now + ttl
}
```

Failure shape: core logic reads wall-clock time directly. Remediation: pass time
from the imperative shell into a deterministic core function.

### `public_unsafe_api`: safe wrapper or documented exception

Before:

```rust
pub unsafe fn read_word(ptr: *const u32) -> u32 {
    *ptr
}
```

After:

```rust
pub fn read_word(word: &u32) -> u32 {
    *word
}
```

Failure shape: a public API exports caller obligations that ordinary Rust call
sites cannot check. Remediation: expose a safe wrapper around a narrow private
unsafe helper, or document an explicit exception with caller obligation, reviewer
action, evidence tier, and non-claim.

### `ignored_result`: explicit error propagation

Before:

```rust
let _ = writer.flush();
```

After:

```rust
writer.flush()?;
```

Failure shape: a fallible result is silently dropped. Remediation: propagate the
error, handle it locally, or document why the result is intentionally irrelevant.

## Terminal-first quickstart

Use this quickstart when a reader wants one path from first check to reviewable
artifacts. Consumer repos should still keep their own `dylint.toml` and
`[workspace.metadata.octet]` runner scope checked in.

```bash
cp /path/to/octet/templates/dylint.toml dylint.toml
cat >> Cargo.toml <<'TOML'
[workspace.metadata.octet]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
TOML
cargo octet check --artifact-dir artifacts/octet --output-format sarif
cargo octet artifact verify --artifact-dir artifacts/octet
```

Expected reviewable outputs include:

- `artifacts/octet/command.txt` with the replayable command;
- `artifacts/octet/status.json` with exit status, counts, selected profile, and replay metadata;
- `artifacts/octet/summary.txt` with the human effect-summary output;
- `artifacts/octet/results.sarif` or `artifacts/octet/results.jsonl` for findings;
- `artifacts/octet/provenance.jsonl` with source IDs, checker IDs, evidence tiers, artifact locators, BLAKE3 hashes, and checked scopes.

A warning-only run can still exit successfully; teams decide when to promote
warning families to deny gates or pre-commit hooks after inventorying starter
noise.

## FAQ

### Why not just Clippy?

Use Clippy too. Octet targets Tiger Style and evidence-contract surfaces that are
specific to this project: checked artifact bundles, policy IDs, rollout profiles,
proof-card provenance, and lints such as ambient clock boundaries or public
unsafe API obligations. Clippy cleanliness and Octet cleanliness are both review
inputs, not whole-program proof.

### What does Octet prove?

The default `cargo octet check` path proves only that selected static and policy
checks ran and emitted reviewable evidence for their scopes. Optional
`cargo octet prove` rails can support `formal_proof` evidence only for a declared
property, verifier identity, source/proof binding, receipt, and artifact identity.

### What does Octet not prove?

Octet does not provide mathematical certainty, whole-program correctness,
certification, business-domain validation, semantic equivalence, C performance
parity, a verified runtime, or Salt-style language syntax such as `requires`,
`ensures`, or `@trusted`.

### Why Dylint or nightly?

Octet currently uses Dylint and rustc-private surfaces because several checks
need compiler internals that stable plugin APIs do not expose. The tradeoff is a
pinned runner/toolchain path and Nix wrappers for reproducibility; consumers pin
the Octet revision before using it as a gate.

### What data leaves my repository?

Octet itself runs locally or in consumer CI and writes local artifacts. It does
not require an Octet-operated upload service. Consumers still own what their CI
provider, log collector, artifact store, or integrations do with those local
outputs.

### How do we handle starter-profile noise?

Start with warning-level inventory, save artifacts, categorize findings, and
promote only lint families whose signal is understood. The Aspen phase-1 archive
is an example of this bounded starter rollout, not a universal noise guarantee.

### When should we use `cargo octet prove`?

Use proof rails when a scoped property already has a verifier configuration or a
proof card that reviewers need to validate before release. Do not use proof rails
to waive unrelated lint findings, certify the whole system, or replace human
review of business requirements.

## Methodology before public metrics

Public performance, cache, CI-latency, suppression-rate, false-positive, or
rollout-cost metrics should be published only with methodology. A metric is ready
for publication only when it names:

- command or gate that produced the measurement;
- input scope and configuration;
- environment or platform when it affects the result;
- raw artifact, transcript, or reproducible evidence path;
- interpretation boundary for the metric.

A candidate metric without those fields is held back until the raw command and
artifact path exist. For example, a cache-speed claim should name the exact
`cargo octet check` invocation, package scope, cache status fields, platform,
artifact directory or transcript, and the boundary that the value describes only
that run.

## Boundary-preserving comparison

Octet's adoption advantage is compatibility with existing Rust workspaces and
release gates. It can be compared with new-language or verifier-first tools only
by staying within supported evidence: Rust-compatible static checks, policy
checks, optional scoped proof receipts, provenance, and artifact verification.

Octet does not claim mathematical certainty, whole-program correctness,
certification, a verified runtime, language-level contract syntax, C performance
parity, or Salt-style trusted-code semantics unless a separate accepted
requirement and evidence artifact supports that exact claim.
