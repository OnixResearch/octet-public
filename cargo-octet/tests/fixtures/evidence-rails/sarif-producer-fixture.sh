#!/bin/sh
# TEST-ONLY fixture. The pinned Semgrep and CodeQL executables are absent from this environment,
# so the producer tests run this script in their place. It never proves real scanner behavior: it
# writes fixed SARIF selected by SARIF_FIXTURE_MODE and exits with declared outcomes.
set -eu

mode="${SARIF_FIXTURE_MODE:-clean}"
if [ "${1:-}" = "--version" ]; then
  printf '%s\n' "${SARIF_FIXTURE_VERSION:-1.180.0}"
  exit 0
fi
if [ "${1:-}" = "version" ]; then
  printf '{"version":"%s"}\n' "${SARIF_FIXTURE_VERSION:-2.27.2}"
  exit 0
fi
if [ "${1:-}" = "database" ] && [ "${2:-}" = "create" ]; then
  mkdir -p "$3"
  exit 0
fi

out=""
previous=""
for argument in "$@"; do
  case "$argument" in
    --output=*) out="${argument#--output=}" ;;
  esac
  if [ "$previous" = "--output" ]; then
    out="$argument"
  fi
  previous="$argument"
done
if [ -z "$out" ]; then
  printf 'fixture: no --output argument\n' >&2
  exit 64
fi
if [ -n "${SARIF_FIXTURE_MUTATE:-}" ]; then
  printf '\n// mutated during scan\n' >> "$SARIF_FIXTURE_MUTATE"
fi
case "$mode" in
  clean|violation) printf '%s\n' "$SARIF_FIXTURE_REPORT" > "$out" ;;
  malformed) printf '%s\n' 'not-json' > "$out" ;;
  empty) : > "$out" ;;
  fail-no-report) exit 3 ;;
  fail-with-report) printf '%s\n' "$SARIF_FIXTURE_REPORT" > "$out"; exit 3 ;;
esac
exit 0
