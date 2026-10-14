#!/bin/bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf '%s
' 'usage: tests/fixtures/openspec-proposal-governance/run-proposal-fixture-gate.sh <fixture-name>' >&2
  exit 2
fi

script_rel='tests/fixtures/openspec-proposal-governance/run-proposal-fixture-gate.sh'
fixture_name="$1"
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
fixture_dir="$repo_root/tests/fixtures/openspec-proposal-governance/$fixture_name"
active_dir="$repo_root/openspec/changes/$fixture_name"
extension="$HOME/.pi/agent/extensions/openspec-gates.ts"
prompt="Call the openspec_gate tool with stage=proposal and change=$fixture_name. Print the full result."
expected=$(tr -d '[:space:]' < "$fixture_dir/expected_gate.txt")

if [[ ! -d "$fixture_dir" ]]; then
  printf 'missing fixture: %s
' "$fixture_dir" >&2
  exit 1
fi
if [[ -e "$active_dir" ]]; then
  printf 'active change already exists: %s
' "$active_dir" >&2
  exit 1
fi

cleanup() {
  rm -rf -- "$active_dir"
  rm -f -- "$tmp_log"
}
tmp_log=$(mktemp)
trap cleanup EXIT

printf '%s %s
' "$script_rel" "$fixture_name"
cp -R -- "$fixture_dir" "$active_dir"
rm -f -- "$active_dir/expected_gate.txt"
cd "$repo_root"
set +e
pi --no-session --thinking minimal -e "$extension" -p "$prompt" 2>&1 | tee "$tmp_log"
status=${PIPESTATUS[0]}
set -e

if [[ "$expected" == 'blocked' ]]; then
  if ([[ $status -ne 0 ]] || grep -Eiq 'VERDICT:[[:space:]]*FAIL|blocked' "$tmp_log") && grep -Eiq 'omission' "$tmp_log" && grep -Eiq 'proposal|missing|scope|evidence|artifact' "$tmp_log"; then
    printf 'PASS expected blocked fixture: %s\n' "$fixture_name"
    exit 0
  fi
  printf 'FAIL expected blocked fixture but target proposal omission finding was absent: %s\n' "$fixture_name" >&2
  exit 1
fi

if [[ "$expected" == 'pass' ]]; then
  if [[ $status -eq 0 ]] && ! grep -Eiq 'VERDICT:[[:space:]]*FAIL|blocked' "$tmp_log"; then
    printf 'PASS expected passing fixture: %s
' "$fixture_name"
    exit 0
  fi
  printf 'FAIL expected passing fixture but gate blocked: %s
' "$fixture_name" >&2
  exit 1
fi

printf 'unknown expected gate result: %s
' "$expected" >&2
exit 2
