#!/bin/bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf '%s\n' 'usage: tests/fixtures/openspec-task-governance/run-fixture-gate.sh <fixture-name>' >&2
  exit 2
fi

script_rel='tests/fixtures/openspec-task-governance/run-fixture-gate.sh'
fixture_name="$1"
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
fixture_dir="$repo_root/tests/fixtures/openspec-task-governance/$fixture_name"
active_dir="$repo_root/openspec/changes/$fixture_name"
checker="$repo_root/tests/fixtures/openspec-task-governance/check-change-status.py"
extension="$HOME/.pi/agent/extensions/openspec-gates.ts"
prompt="Call the openspec_gate tool with stage=tasks and change=$fixture_name. Print the full result."

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
}
trap cleanup EXIT

printf '%s %s\n' "$script_rel" "$fixture_name"
cp -R -- "$fixture_dir" "$active_dir"
cd "$repo_root"
pi --no-session --thinking minimal -e "$extension" -p "$prompt"
python3 "$checker" "$active_dir/tasks.md"
