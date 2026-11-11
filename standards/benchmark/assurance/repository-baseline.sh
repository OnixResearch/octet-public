#!/bin/sh
# Repository-script baseline for the benchmark assurance corpus.
#
# The script is deliberately small and dependency-free. It accepts one fixture
# path and exits 0 only when the retained evidence, checked policy, observed
# execution, and proof/source linkage agree. Exit 1 means the fixture was
# rejected. The benchmark records this exit status; it does not treat an
# unrun script as clean evidence.
set -eu

if [ "$#" -ne 1 ]; then
    printf 'usage: %s FIXTURE.json\n' "$0" >&2
    exit 2
fi

fixture=$1
if [ ! -f "$fixture" ]; then
    printf 'missing fixture: %s\n' "$fixture" >&2
    exit 2
fi

jq -e '
  (.schema_version == "octet-benchmark-assurance-fixture/v1")
  and (.checked_policy_identity == (.recorded_policy_identity // .checked_policy_identity))
  and (.evidence_identity == (.actual_evidence_identity // .evidence_identity))
  and (.execution.status == "succeeded")
  and (.proof.source_identity == (.proof.claimed_source_identity // .proof.source_identity))
' "$fixture" >/dev/null
