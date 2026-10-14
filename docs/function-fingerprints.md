# Function fingerprints

Function fingerprints are selective review gates for functions whose exact body was
reviewed, verified, or certified. They are not a workspace-wide style rule and they
do not prove correctness.

Use fingerprints when a small number of functions carry review evidence that should
fail loudly if their signature, attributes, or body drift after review. Do not use
them as a replacement for tests, proofs, or normal code review.

## Marker forms

Comment markers work in stable Rust without adding an attribute macro:

```rust
// octet-fingerprint: b3:<hash>; review=safety-critical; evidence=openspec/.../evidence/check.txt
fn checked_transition(input: State) -> State { /* ... */ }
```

Attribute markers are also recognized by the scanner:

```rust
#[fingerprint("b3:<hash>", review = "safety-critical", evidence = "openspec/.../evidence/check.txt")]
fn checked_transition(input: State) -> State { /* ... */ }
```

The implementation hashes normalized function tokens and ignores the marker itself.
Formatting and comments do not change the hash; signature, attribute, visibility,
or body token changes do. Macro calls produce a review warning unless the marker
records an explicit macro review allowance such as `macro=allow`.

## Operator workflow

Start in a scratch copy or review branch. `--write` and `--update` rewrite source
markers, so do not run them against a production checkout unless you intend to keep
those marker changes.

1. Add fingerprints to selected reviewed functions:

   ```bash
   cargo octet fingerprint --write path/to/file.rs
   ```

   Use this only for initial marker creation. It discovers functions in the target
   file or directory and writes markers for functions that do not already have one.

2. Gate reviewed functions in CI or during review:

   ```bash
   cargo octet fingerprint --check path/to/file-or-directory
   ```

   `--check` inspects marked functions only. It exits nonzero when any marked
   function is missing a hash, mismatches its stored hash, lacks required metadata,
   or has an unallowed macro warning.

3. Preview drift without rewriting markers:

   ```bash
   cargo octet fingerprint --diff path/to/file-or-directory
   ```

   `--diff` has the same failure conditions as `--check`, but is intended for
   operator diagnosis before deciding whether the source change is legitimate.

4. Repair accepted drift after re-review:

   ```bash
   cargo octet fingerprint --update path/to/file-or-directory
   ```

   `--update` rewrites only stale or missing marker hashes. Treat each `written`
   record as a review decision: the new function body is now the reviewed body.

## Human output

Human output is the default and is designed for terminals, PR comments, and simple
CI logs:

```text
octet fingerprint check
src/lib.rs checked_transition mismatch stored=b3:old computed=b3:new review=safety-critical evidence=openspec/.../evidence/check.txt macro_warning=false
summary total=6 match=5 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

Use the summary line for quick operator decisions:

- `mismatch > 0`: a reviewed function changed; inspect the record line and rerun
  `--diff` or re-review before `--update`.
- `missing > 0`: a marker is incomplete or a write/update flow did not finish.
- `written > 0`: `--write` or `--update` changed marker hashes; review and commit
  the source diff with the evidence that justified it.
- `macro_warnings > 0`: a marked function uses macro calls without explicit review
  allowance; either avoid fingerprinting that function or add a deliberate
  `macro=allow` marker after reviewing the macro expansion risk.
- `metadata_missing > 0`: add `review=...` and `evidence=...` metadata before using
  the marker as a review gate.

A clean check has every marked function in `match` status and a summary like:

```text
summary total=12 match=12 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

For complete checked-in transcripts, see:

- [`docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md`](examples/function-fingerprint-lifecycle/lifecycle-transcript.md),
  which shows initial `--update`, clean `--check`, drift, failing `--diff`,
  accepted `--update`, and final clean `--check`.
- [`docs/examples/function-fingerprint-human-summary/transcript.md`](examples/function-fingerprint-human-summary/transcript.md),
  which shows human summary counts for two marked functions, one-function drift,
  non-zero check status, one-marker repair, and final clean check.
- [`docs/examples/function-fingerprint-warning-summary/transcript.md`](examples/function-fingerprint-warning-summary/transcript.md),
  which shows `macro_warnings` and `metadata_missing` counts, including the
  non-zero clean-check status for unresolved operator warnings.
- [`docs/examples/function-fingerprint-json-output/transcript.md`](examples/function-fingerprint-json-output/transcript.md),
  which shows script-facing JSON payloads for update, clean check, drift check,
  accepted update, and final clean check.
- [`docs/examples/function-fingerprint-source-scanner/transcript.md`](examples/function-fingerprint-source-scanner/transcript.md),
  which shows the source-scanner boundary for `macro_rules!` generated functions
  and the explicit wrapper pattern for reviewed generated-code edges.
- [`docs/examples/function-fingerprint-downstream-smoke.md`](examples/function-fingerprint-downstream-smoke.md),
  which documents the flake-backed scratch-consumer smoke over all checked
  fingerprint docs fixtures.

These examples are part of the contributor verification surface. When the
fingerprint CLI output, marker rewrite behavior, human summary counts, JSON
fields, source-scanner boundary behavior, downstream scratch-smoke behavior, or
example transcripts change, use this single CI/operator review gate:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

That `docs-examples` Nix check is part of `nix flake check` and is the aggregate
home for executable docs/examples regressions. Treat it as the review command;
use the narrower checks below only to debug a failing slice. It currently replays
the documented fingerprint lifecycle, human-summary, warning-summary,
JSON-output, and source-scanner-boundary flows; runs the checked fixtures through
a scratch consumer crate via `fingerprint-docs-downstream-smoke`; asserts
downstream human/JSON `--diff` parity before repair; asserts the observed stdout
appears in the checked-in transcripts; parses JSON payloads for script-facing
field guarantees; and confirms repaired or updated scratch sources exactly match
the checked-in `reviewed.rs` fixtures.

Focused local debug commands:

```bash
nix build .#checks.x86_64-linux.fingerprint-lifecycle-docs --no-link -L
nix build .#checks.x86_64-linux.fingerprint-human-summary-docs --no-link -L
nix build .#checks.x86_64-linux.fingerprint-warning-summary-docs --no-link -L
nix build .#checks.x86_64-linux.fingerprint-json-output-docs --no-link -L
nix build .#checks.x86_64-linux.fingerprint-source-scanner-docs --no-link -L
nix build .#checks.x86_64-linux.fingerprint-docs-downstream-smoke --no-link -L
nix develop -c bash -c 'cargo test -p cargo-octet --test fingerprint_consumer fingerprint_lifecycle_docs_fixture_matches_transcript -- --nocapture'
nix develop -c bash -c 'cargo test -p cargo-octet --test fingerprint_consumer fingerprint_human_summary_docs_fixture_matches_transcript -- --nocapture'
nix develop -c bash -c 'cargo test -p cargo-octet --test fingerprint_consumer fingerprint_warning_summary_docs_fixture_matches_transcript -- --nocapture'
nix develop -c bash -c 'cargo test -p cargo-octet --test fingerprint_consumer fingerprint_json_output_docs_fixture_matches_transcript -- --nocapture'
nix develop -c bash -c 'cargo test -p cargo-octet --test fingerprint_consumer fingerprint_source_scanner_docs_fixture_matches_transcript -- --nocapture'
```

## JSON output

Use JSON for scripts that need exact counts or per-record assertions:

```bash
cargo octet fingerprint --check path/to/file-or-directory --output-format=json
```

The JSON report contains the mode and one record per marked function, including the
path, function name, stored hash, computed hash, status, review metadata, evidence
metadata, and macro-warning flag. Common script checks are:

```bash
# Count mismatched functions.
cargo octet fingerprint --check src --output-format=json \
  | jq '[.records[] | select(.status == "mismatch")] | length'

# List changed functions by path/name.
cargo octet fingerprint --diff src --output-format=json \
  | jq -r '.records[] | select(.status != "match") | "\(.path) \(.name) \(.status)"'

# Assert an update repaired exactly one accepted drift.
cargo octet fingerprint --update src --output-format=json \
  | jq '[.records[] | select(.status == "written")] | length == 1'
```

## Review manifests

Use `--review-manifest <path>` when a CI job or human reviewer needs a stable
review plan alongside normal output:

```bash
cargo octet fingerprint --check src --review-manifest artifacts/fingerprint-review.json
```

The manifest records the tool name/version, input paths, mode, a BLAKE3
`manifest_hash`, and sorted records. Each record includes a stable content ID,
path, function name, computed/stored hash, review/evidence metadata, macro
warning state, and `requires_review` boolean. Sorting is by path, function name,
and stable ID so two runs over the same source produce comparable manifests.

## Source-scanner boundaries

The fingerprint command is a source scanner, not a macro-expansion analyzer.
It intentionally skips `fn` tokens inside `macro_rules!` definitions and does not
claim to fingerprint functions generated after macro expansion. Fingerprint explicit
source wrappers around generated code when a reviewed boundary is needed. The
checked [`source-scanner example`](examples/function-fingerprint-source-scanner/transcript.md)
keeps that contract executable.

Directory inputs are traversed recursively across Rust files. Non-write modes gate
only marked functions, so large `#[cfg(test)]` modules or helper functions are not
pulled into the gate unless they have markers.
