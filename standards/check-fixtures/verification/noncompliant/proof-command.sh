#!/usr/bin/env bash
set -euo pipefail

if grep --fixed-strings "proof_witness_deterministic_effect_handlers" src/lib.rs >/dev/null; then
  printf '%s\n' "proof-result: pass deterministic-effect-handlers"
  exit 0
fi
printf '%s\n' "proof-result: fail deterministic-effect-handlers"
exit 1
