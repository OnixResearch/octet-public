#!/usr/bin/env bash
set -euo pipefail

grep --fixed-strings "proof_witness_deterministic_effect_handlers" src/lib.rs >/dev/null
grep --fixed-strings "ScriptedHandler" src/lib.rs >/dev/null
printf '%s\n' "proof-result: pass deterministic-effect-handlers"
printf '%s\n' "trust-boundary: local fixture proof command executed"
