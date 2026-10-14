## Design

Observe mode remains a non-blocking adoption rail. The function-address collector produces a report first, then maps any missing-review debt into output-compatible advisory findings. The output-domain validator must accept the finding shape or the function-address rail must use an existing accepted advisory group/severity vocabulary.

### Decisions

1. **Artifact first.** The JSON report is the primary observe output and must be written when Cargo checking succeeds, even when advisory findings are present.
2. **Advisory findings are schema-valid.** Observe findings use a validator-accepted severity and group; they do not introduce an invalid `note`/group combination.
3. **Strict modes keep pressure.** Warn and deny continue to report stale or malformed baselines with warning/error severity.
4. **No semantic promotion.** Passing observe mode means the source-shape artifact was produced; it does not mean functions are reviewed or semantically correct.

### Validation shape

Positive fixtures cover an unbaselined Rust crate with functions and assert that `function-addresses.json` is written with zero process failure. Negative fixtures cover malformed BLAKE3 baseline entries and stale reviewed addresses in warn/deny modes.
