# Unison-inspired Rust patterns for Tiger Style

Tiger Style imports Unison's discipline, not its syntax. Rust code keeps ordinary
traits, enums, modules, crates, tests, and Git history; the standard asks for the
same architectural properties Unison makes visible: explicit effects, replayable
handlers, content-addressed evidence, executable examples, and identity-preserving
refactors.

## Effect capability boundaries

Pure core code should not read clocks, random sources, environment variables,
files, sockets, process state, secret stores, remote executors, unsafe escape
hatches, mutable global state, or concurrency primitives directly. Octet's
approved capability families are `time`, `randomness`, `filesystem`, `network`,
`process`, `environment`, `secrets`, `remote-execution`, `unsafe-code`,
`global-state`, `concurrency`, and explicit `unknown`. Core code should either:

- return an effect description as data, such as an enum command for the shell to
  interpret; or
- accept a narrowly named capability trait or token, such as `Clock`,
  `ArtifactStore`, `RemoteEndpoint`, or `CredentialVault`.

The shell owns interpretation. That shell may use `std`, `tokio`, `tower`,
`cap-std`, or platform APIs, but the core remains deterministic and testable.
Capability names must expose the authority they grant: `ReadConfigDir` is better
than `Context`, and `PublishReceipt` is better than `Client`. Each project should
state which capabilities are allowed in core, shell, tests, and remote adapters.
Shell-allowed uses are interpreter responsibilities, while functional-core uses
are prohibited unless represented as explicit data or named capability traits.

## Delayed effects and top-level purity

Constructors, builders, statics, and lazy initializers should validate and assemble
values only. They must not hide IO, time, networking, process spawning, random
state, or global mutation. If work must happen later, represent it as data and run
it through an explicit interpreter.

## Deterministic handlers

Every effect family that matters to safety or review should have a deterministic
handler used in tests. The handler records requested effects, returns scripted
responses, and proves failure cleanup paths without ambient wall-clock, network,
filesystem, or environment dependencies.

## Content-addressed evidence

Tiger Style does not replace Git with a content-addressed codebase. It uses
content addressing for review identity:

- finding hashes tie diagnostics to lint/config/tool/source snippets;
- evidence headers record BLAKE3 hashes for fixtures and transcripts;
- function fingerprints prove that a reviewed function body is unchanged; and
- refactor records map old symbol identity to new symbol identity.

A hash is identity evidence, not a correctness proof.

## Live examples

Examples in standards docs should link to fixtures, doctests, trybuild cases, or
saved transcripts when possible. Prose-only examples are allowed only when they are
explicitly marked as illustrative and not verification evidence.

## Nominal and structural types

Use nominal wrappers for identities, units, authority tokens, secrets, protocol
states, and remote idempotency keys. Use structural records for plain DTOs and
configuration snapshots where field names carry the meaning and no unique authority
or identity is implied.
