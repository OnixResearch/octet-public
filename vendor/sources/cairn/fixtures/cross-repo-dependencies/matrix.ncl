let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let Hash = std.contract.from_predicate (fun value => std.is_string value && std.string.length value == 64) in
let Dependency = {
  id | NonEmpty,
  repo | NonEmpty,
  change | NonEmpty,
  requires | NonEmpty,
  expected_status | NonEmpty,
  expected_policy_hash | Hash,
  expected_subject_hash | Hash,
  expected_receipt_hash | Hash,
  non_claims | Array NonEmpty,
} in
let Summary = {
  schema | NonEmpty,
  repo | NonEmpty,
  change | NonEmpty,
  receipt_kind | NonEmpty,
  status | NonEmpty,
  policy_hash | Hash,
  subject_hash | Hash,
  receipt_hash | Hash,
  diagnostic_summary | NonEmpty,
} in
let Matrix = {
  schema | NonEmpty,
  positive | Array { dependency | Dependency, evidence | Summary },
  non_pass_statuses | Array NonEmpty,
  negative_classes | Array NonEmpty,
} in
let fixture_policy_hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
let fixture_subject_hash = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" in
let fixture_receipt_hash = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" in
let fixture_non_claims = [
  "dependency satisfaction proves presented receipt linkage only",
  "dependency satisfaction does not prove upstream correctness or remote trust",
  "dependency satisfaction does not prove downstream integration correctness or release eligibility",
] in
let positive_rows = std.array.map (fun kind => {
  dependency = {
    id = "valence-" ++ kind,
    repo = "valence",
    change = "canonical-evidence",
    requires = kind,
    expected_status = "pass",
    expected_policy_hash = fixture_policy_hash,
    expected_subject_hash = fixture_subject_hash,
    expected_receipt_hash = fixture_receipt_hash,
    non_claims = fixture_non_claims,
  },
  evidence = {
    schema = "cairn.cross-repo-receipt-summary.v1",
    repo = "valence",
    change = "canonical-evidence",
    receipt_kind = kind,
    status = "pass",
    policy_hash = fixture_policy_hash,
    subject_hash = fixture_subject_hash,
    receipt_hash = fixture_receipt_hash,
    diagnostic_summary = "current passing upstream receipt",
  },
}) [
  "proposal_gate",
  "design_gate",
  "tasks_gate",
  "validation_receipt",
  "sync_receipt",
  "archive_receipt",
  "radicle-issue.v1",
] in
({
  schema = "cairn.cross-repo-dependency-fixture-matrix.v1",
  positive = positive_rows,
  non_pass_statuses = [
    "failed",
    "blocked",
    "skipped",
    "deferred",
    "timed_out",
    "dry_run",
    "cancelled",
    "infrastructure_failed",
    "malformed",
    "unsupported",
    "unknown",
    "stale",
  ],
  negative_classes = [
    "missing",
    "stale_policy",
    "stale_subject",
    "wrong_repo",
    "wrong_change",
    "wrong_kind",
    "unsupported_kind",
    "unsafe_diagnostic",
    "receipt_hash_mismatch",
  ],
} | Matrix)
