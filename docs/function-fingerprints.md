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

Attribute markers are also recognized by the scanner. The attribute must fit on
one line; it is not hashed as one of the function's attributes:

```rust
#[fingerprint("b3:<hash>", review = "safety-critical", evidence = "openspec/.../evidence/check.txt")]
fn checked_transition(input: State) -> State { /* ... */ }
```

A marker sits on the line directly above the function's first outer attribute,
or above `fn` when there is none. Fingerprint markers and `// octet-object:`
markers can share that position: a contiguous block of marker lines may hold one
of each, in either order, and each command reads its own kind from the block.
`fingerprint --write` and `object update` insert a new marker at the bottom of
the block, so markers already there stay bound.

Any other line between a marker and its function, including a blank line or a
doc comment, leaves the marker bound to no function; put doc comments above the
marker. A fingerprint marker between attributes and `fn`, inside a function
body, or stacked over another fingerprint marker is also unbound. `--check` and
`--diff` report each unbound fingerprint marker with its file and line and exit
nonzero. `--write` and `--update` refuse to rewrite any source while one exists,
so they never add a second marker next to a misplaced one; move or delete it
first.

An object marker whose hash is empty or not hexadecimal, such as `b3:` or
`b3:TODO`, still binds: `object check` reports `missing`, and `object update`
replaces it in place and keeps its `review` and `evidence`.

Marker metadata is a list of `key=value` items separated by `;` or `,`. Keys
match exactly: `preview=` does not supply `review`, and `review=macro=allow`
records the review class `macro=allow` without granting a macro allowance.
Spaces around keys and values and quotes around values are ignored. A `TODO`
or empty value counts as missing metadata.

The implementation hashes canonical function tokens and ignores the marker itself.
Ordinary whitespace and non-doc comments that leave Rust TokenStream unchanged
do not change the hash; doc comments can become compiler-visible attributes
and change identity, particularly inside macro inputs. Signature, attribute,
visibility, or body token changes also change the hash. String,
raw-string, byte, and character literals are hashed whole, so any change inside a
literal changes the hash, and a `//` inside a literal does not hide the code after
it. Line endings inside literals and doc comments are read as LF, as rustc reads
them, so a CRLF checkout, for example with `core.autocrlf=true`, hashes the same as
an LF one. Macro calls anywhere in the item, including attributes such as
`#[doc = include_str!("f.md")]` and the signature, produce a review warning unless
the marker records an explicit macro review allowance such as `macro=allow`.

Marker hashes are version 2: a version tag is hashed ahead of the tokens. Markers
written before version 2 never match, even for an unchanged function, so they fail
closed with `mismatch` until the function is re-reviewed and `--update` rewrites
them.

## Operator workflow

Start in a scratch copy or review branch. `--write` and `--update` rewrite source
markers, so do not run them against a production checkout unless you intend to keep
those marker changes. Give at most one mode flag (`--check`, `--write`, `--update`,
or `--diff`) per command; with none, the command runs `--check`.

1. Add fingerprints to selected reviewed functions:

   ```bash
   cargo octet fingerprint --write path/to/file.rs
   ```

   It discovers functions in the target file or directory, writes markers for
   functions that do not already have one, and also rewrites any existing marker
   whose hash is stale or missing. Like `--update`, that rewrite re-approves the
   current body of every stale marked function, so run `--check` or `--diff`
   first when the target already contains markers.

2. Gate reviewed functions in CI or during review:

   ```bash
   cargo octet fingerprint --check path/to/file-or-directory
   ```

   `--check` inspects marked functions only. It exits nonzero when any marked
   function is missing a hash, mismatches its stored hash, lacks required metadata,
   or has an unallowed macro warning, and when any fingerprint marker binds to no
   function. A marker whose hash is empty or not hexadecimal, such as `b3:` or
   `b3:TODO`, still marks its function and reports `missing`.

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
src/lib.rs checked_transition mismatch semantic_id=b3:cf0af9c656f20f31 stored=9a1cd5ce932bd24379e84878bd357b352388e38e8dde2082855e592f5cd6afa7 computed=b3:dc0e7855981b557572c35b946241cc138ec9e17896196e75ec5a3fd1ccc3a769 normalized_body_hash=b3:87c7e8e7b2a1005418e42da07b9adb703d5625993b8c191b2b8bbc5bfda8f473 dependency_identity_status=source-scanner-token-set review=safety-critical evidence=openspec/.../evidence/check.txt macro_warning=false
src/lib.rs bounded_retry match semantic_id=b3:ec59f5695ff884e4 stored=83edea8b1e4f02ee331fca0471de323c7a4fd4396e1e9a4bcfb1d22a889f565e computed=b3:83edea8b1e4f02ee331fca0471de323c7a4fd4396e1e9a4bcfb1d22a889f565e normalized_body_hash=b3:3fd231f58701d9d8272d9111712aa44364d66a7104c96c12be535741e77e5c89 dependency_identity_status=source-scanner-token-set review=safety-critical evidence=openspec/.../evidence/check.txt macro_warning=false
summary total=2 match=1 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0
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
- A `<path>:<line> unbound-marker` line: a fingerprint marker binds to no function
  and is not counted in the summary. Move it directly above its function's first
  outer attribute, or delete it.

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
```

## JSON output

Use JSON for scripts that need exact counts or per-record assertions:

```bash
cargo octet fingerprint --check path/to/file-or-directory --output-format=json
```

The JSON report contains the mode and one record per marked function, including the
path, function name, stored hash, computed hash, status, review metadata, evidence
metadata, and macro-warning flag. When a fingerprint marker binds to no function,
the report also has an `unbound_markers` array of `{"path", "line"}` entries;
the key is absent otherwise. Record paths are relative to the current
directory with `/` separators, even when the command names an absolute path, so
run the command from a directory that contains every scanned path; a path
outside it is rejected before any marker is read or written. Common script
checks are:

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
`manifest_hash`, and sorted records. Input and record paths are relative to the
current directory, like JSON record paths. Each record includes a stable content ID,
path, function name, computed/stored hash, review/evidence metadata, macro
warning state, and `requires_review` boolean. Sorting is by path, function name,
and stable ID so two runs over the same source produce comparable manifests.

## Source-scanner boundaries

The fingerprint command is a source scanner, not a macro-expansion analyzer.
It intentionally skips `fn` tokens inside `macro_rules!` definitions, whether the
definition body is delimited by braces, parentheses, or brackets, and does not
claim to fingerprint functions generated after macro expansion. Fingerprint explicit
source wrappers around generated code when a reviewed boundary is needed. The
checked [`source-scanner example`](examples/function-fingerprint-source-scanner/transcript.md)
shows that contract, and the `fingerprint-source-scanner-docs` check replays it.

The scanner ignores comments (line, doc, and nested block comments) and string,
raw-string, byte, and character literals when it finds functions and their braces.
Documentation examples and quoted source text are therefore not functions, and a
quoted or commented brace does not end a function early.
Macro-call detection uses the same view: a macro call is a name or path followed by
`!` and a `(`, `[`, or `{` group. A keyword before a unary `!`, as in `if !(ready)`,
and macro-like text inside comments or literals are not macro calls.

A function signature ends at its first `{` or `;` outside parentheses, brackets,
braces, and generic arguments. Array types such as `[u8; 2]` in parameters or
return types therefore keep the function and its body; a trailing `;` still marks
a bodiless trait or foreign declaration.

A function is found when `fn` is followed by whitespace, including a tab or a
newline, or by a comment. Unicode identifiers and raw identifiers such as `r#type`
are full function names and locators. Unicode identifiers stay whole in canonical
tokens and dependency sets, and a Unicode-named macro call counts as macro context.
These rules changed the names, locators, and hashes of affected functions, so
markers written before them must be re-reviewed; nothing rebinds them. The scanner
does not claim complete Rust syntax coverage.

Directory inputs are traversed recursively across Rust files. Non-write modes gate
only marked functions, so large `#[cfg(test)]` modules or helper functions are not
pulled into the gate unless they have markers.

Every path named on the command line must exist and be a directory or a `.rs`
file; otherwise the command fails before scanning, so a mistyped scope cannot pass
as an empty one. The same rule applies to `cargo octet object` source paths and to
`--function-address-path` scopes. Non-Rust files found inside a directory are
skipped.
Function-address collection also refuses, rather than skips, a symlinked
directory or `.rs` file inside a declared scope (`--function-address-path` or
`core_source_scopes`) whose target resolves outside the workspace root. Without
a declared scope the whole workspace root is scanned; there a symlinked
directory that resolves outside the root, such as a Nix `result` link, a
nix-direnv `.direnv` input, or a crate linked in from elsewhere in a monorepo, is
skipped without being followed, while a symlinked `.rs` file that resolves
outside the root is still refused because it is Rust source in the tree. The
workspace-relative path of each skipped link is listed, sorted, in the
function-address artifact's `skipped_directory_links` array, such as
`["crates/shared", "result"]`. The array is empty when nothing was skipped,
which is always the case under a declared scope. The link target is not
recorded: it is machine-specific, such as a home directory or a Nix `result`
store path that changes on every rebuild, and would make the artifact and its
recorded identity non-reproducible. `cargo octet check` instead prints a
`note:` line on stderr for each skipped link, naming the path and its target.
The list is reporting only: it does not enter the evidence-chain roots, and a
skipped link's Rust source still gets no function addresses; declare
function-address paths to refuse such a link instead. Fingerprint and object
inputs keep following such links.
