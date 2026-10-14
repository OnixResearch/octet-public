#!/bin/bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf '%s\n' 'usage: tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh <fixture-name>' >&2
  exit 2
fi

script_rel='tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh'
fixture_name="$1"
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
fixture_dir="$repo_root/tests/fixtures/openspec-task-governance/$fixture_name"
active_dir="$repo_root/openspec/changes/$fixture_name"
extension="$HOME/.pi/agent/extensions/openspec-gates.ts"
prompt="Call the openspec_gate tool with stage=tasks and change=$fixture_name. Print the full result."
expected=$(tr -d '[:space:]' < "$fixture_dir/expected_gate.txt")

if [[ ! -d "$fixture_dir" ]]; then
  printf 'missing fixture: %s\n' "$fixture_dir" >&2
  exit 1
fi
if [[ -e "$active_dir" ]]; then
  printf 'active change already exists: %s\n' "$active_dir" >&2
  exit 1
fi

cleanup() {
  rm -rf -- "$active_dir"
  rm -f -- "$tmp_log"
}
tmp_log=$(mktemp)
trap cleanup EXIT

printf '%s %s\n' "$script_rel" "$fixture_name"
cp -R -- "$fixture_dir" "$active_dir"
rm -f -- "$active_dir/expected_gate.txt"
cd "$repo_root"
set +e
pi --no-session --thinking minimal -e "$extension" -p "$prompt" 2>&1 | tee "$tmp_log"
status=${PIPESTATUS[0]}
set -e

if [[ "$expected" == 'blocked' ]]; then
  if ([[ $status -ne 0 ]] || grep -Eiq 'VERDICT:[[:space:]]*FAIL|blocked' "$tmp_log") && grep -Eiq 'omission|missing|misses' "$tmp_log" && grep -Eiq 'tasks|task|verification|scenario|fixture' "$tmp_log"; then
    printf 'PASS expected blocked fixture: %s\n' "$fixture_name"
    exit 0
  fi
  printf 'FAIL expected blocked fixture but target tasks omission/missing finding was absent: %s\n' "$fixture_name" >&2
  exit 1
fi

if [[ "$expected" == 'pass' ]]; then
  if grep -Eiq 'VERDICT:[[:space:]]*PASS' "$tmp_log" && ! grep -Eiq 'VERDICT:[[:space:]]*FAIL' "$tmp_log"; then
    printf 'PASS expected passing fixture: %s\n' "$fixture_name"
    exit 0
  fi
  printf 'FAIL expected passing fixture but gate blocked: %s\n' "$fixture_name" >&2
  exit 1
fi

printf 'unknown expected gate result: %s\n' "$expected" >&2
exit 2
