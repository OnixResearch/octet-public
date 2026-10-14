## Why

Octet documentation already names Miri and Loom as optional dynamic evidence for unsafe, FFI, and concurrency review. The docs also say ordinary consumer lint runs must not require heavy dynamic tools. The missing piece is a small runner contract that records selected dynamic evidence without making those tools part of the starter path.

Formalizing dynamic evidence rails gives reviewers deterministic receipts for scoped Miri, Loom, sanitizer, or profiler runs while preserving Octet's evidence-tier boundaries.

## What Changes

- Add a dynamic-evidence profile registry for opt-in rails such as unsafe-boundary/Miri and concurrency/Loom.
- Add a command surface that records selected tool, version, command, fixture/workload, acceptance signal, result, and bounded log identity.
- Emit `transcript_assertion`, `manual_review_required`, or other explicit non-proof tiers, never `formal_proof`.
- Keep starter `cargo octet check` free of dynamic tool requirements.

## Non-Goals

- Do not require Miri, Loom, sanitizers, Valgrind, or profilers for ordinary lint runs.
- Do not claim dynamic execution proves whole-program correctness or soundness.
- Do not invent Octet-specific replacements for Miri or Loom.
- Do not run arbitrary unconfigured commands from consumer config.

## Impact

- **Files**: runner command code, dynamic profile config/docs, receipt schemas, artifact verifier, tests, and Cairn specs.
- **Behavior**: optional dynamic evidence command writes deterministic receipts and bounded logs for selected profiles.
- **Testing**: positive profile receipt fixture, missing tool/config failure, bad acceptance signal failure, non-default starter check proof, artifact verification, and Cairn gates.
