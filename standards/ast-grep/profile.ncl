let contracts = import "contracts.ncl" in
{
  schema_version = "octet-ast-grep-structural-profile/v1",
  profile_id = "octet-advisory-source-shape",
  tool = {
    name = "ast-grep",
    version = "0.41.1",
  },
  sgconfig_path = "standards/ast-grep/sgconfig.yml",
  bundle_inputs = [
    "standards/ast-grep/sgconfig.yml",
    "standards/ast-grep/rules/unsafe-block-review.yml",
    "standards/ast-grep/rule-tests/unsafe-block-review-test.yml",
    "standards/ast-grep/rule-tests/__snapshots__/unsafe-block-review-snapshot.yml",
  ],
  bundle_identity = "b3:d19d96313ce51c8d388036e2097e4546bc9db13f3707de87c607456145f6d016",
  scan_scope = {
    paths = ["src", "cargo-octet/src", "crates"],
    globs = ["*.rs", "!**/target/**", "!**/tests/fixtures/**"],
  },
  severity_posture = "advisory",
  promotion_lanes = [
    { lane = "inventory", blocking = false },
    { lane = "advisory", blocking = false },
    { lane = "warning", blocking = false },
    { lane = "error", blocking = true },
    { lane = "migration", blocking = false },
  ],
  rule_coverage = [
    {
      rule_id = "unsafe-block-review",
      lane = "advisory",
      positive_cases = 2,
      negative_cases = 2,
    },
  ],
  codemod_policy = {
    mode = "dry-run-only",
    preserve_proposed_diff = true,
    validation_commands = [
      "ast-grep test --config standards/ast-grep/sgconfig.yml",
      "cargo test -p cargo-octet ast_grep",
    ],
  },
  non_claims = [
    "ast-grep findings are syntax-only candidate evidence and do not prove behavioral correctness",
    "ast-grep findings do not prove Rust type correctness, panic freedom, memory safety, proof discharge, release eligibility, or whole-program safety",
    "ast-grep codemod output is a proposed diff until focused validation accepts the declared migration scope",
  ],
} | contracts.Profile
