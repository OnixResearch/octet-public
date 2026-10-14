# Integration Triage Results

Static analysis scan of three target repos using pattern matching
against the 24 tigerstyle lint patterns. Counts are match occurrences
in non-test .rs files.

## verified-logic

| Lint | Count | Assessment |
|------|------:|------------|
| no_unwrap (.unwrap) | 12 files | Low — most are in conversion helpers with documented safety |
| no_panic | 1 | Low — clean |
| compound_assertion | 26 | Medium — these are the known compound asserts from the proposal |
| platform_dependent_cast | 11 | Low |
| ignored_result (let _ =) | 0 | Low — exemplary |
| unbounded_loop | 1 | Low |
| unchecked_narrowing | 11 | Low — most are in the conversion module with Verus proofs |

**Verdict:** Near-zero false positive risk. The few .unwrap() calls are
in conversion.rs (the module that proves narrowing casts safe). Compound
assertions are the main gap — matches the proposal's finding.
Recommended allowlist: `#![allow(tigerstyle::unchecked_narrowing)]` on
`conversion.rs` (Verus-verified casts).

## aspen

| Lint | Count | Assessment |
|------|------:|------------|
| no_unwrap (.unwrap) | 305 | High — largest violation category |
| no_unwrap (.expect) | 116 | High |
| no_panic | 323 | High — mix of todo!, panic!, unreachable! |
| compound_assertion | 73 | Medium |
| platform_dependent_cast | 344 | High — heavy as usize usage |
| ignored_result (let _ =) | 299 | High |
| unbounded_loop | 173 | High — many event loops (select!) may be false positives |
| unchecked_narrowing | 510 | High — most common cast pattern |

**Verdict:** Large codebase with significant violation counts. Priority
triage order:
1. **ignored_result** (299) — highest silent-failure risk
2. **no_unwrap** (305+116) — panics in production
3. **no_panic** (323) — many may be legitimate unreachable!/todo!
4. **unchecked_narrowing** (510) — highest count but many may be safe
5. **unbounded_loop** (173) — need to filter select! false positives

Recommended initial config:
```toml
[tigerstyle]
function_length_max = 100  # aspen has larger handler functions
```

Recommended phased rollout: start with `ignored_result` and
`compound_assertion` (low false positive rate), then add `no_unwrap`
after fixing the worst offenders.

## clankers

| Lint | Count | Assessment |
|------|------:|------------|
| no_unwrap (.unwrap) | 62 | Medium |
| no_unwrap (.expect) | 72 | Medium |
| no_panic | 107 | Medium |
| compound_assertion | 18 | Low |
| platform_dependent_cast | 92 | Medium |
| ignored_result (let _ =) | 450 | High — worst category |
| unbounded_loop | 68 | Medium |
| unchecked_narrowing | 63 | Medium |

**Verdict:** `let _ =` is the standout issue (450 matches). Likely
many are fire-and-forget channel sends or log calls, but some will be
silenced Results. The `float_for_currency` lint was designed for this
codebase (CostTracker uses f64).

Recommended initial config:
```toml
[tigerstyle]
acronym_allowlist = ["IO"]
```

## Cross-Repo Summary

| Lint | verified-logic | aspen | clankers |
|------|---------------:|------:|---------:|
| no_unwrap | 12 | 421 | 134 |
| no_panic | 1 | 323 | 107 |
| compound_assertion | 26 | 73 | 18 |
| platform_dependent_cast | 11 | 344 | 92 |
| ignored_result | 0 | 299 | 450 |
| unbounded_loop | 1 | 173 | 68 |
| unchecked_narrowing | 11 | 510 | 63 |

verified-logic is the cleanest by far — it was built with Tiger Style
from the start. aspen has the most work ahead. clankers has a
concentrated problem in ignored results.
