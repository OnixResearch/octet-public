use super::*;

// r[verify octet.external_lint.sarif_intake_verification]
const CODEQL_REPORT: &str = r#"{
  "version":"2.1.0",
  "runs":[{
    "tool":{"driver":{"name":"CodeQL","rules":[{"id":"rust/sql-injection"}]}},
    "results":[{
      "ruleId":"rust/sql-injection",
      "level":"warning",
      "message":{"text":"query includes untrusted input"},
      "locations":[{"physicalLocation":{"artifactLocation":{"uri":"src/query.rs"},"region":{"startLine":12,"startColumn":5}}}]
    }]
  }]
}"#;

const SEMGREP_REPORT: &str = r#"{
  "version":"2.1.0",
  "runs":[{
    "tool":{"driver":{"name":"Semgrep","rules":[{"id":"rust.lang.security.audit"}]}},
    "artifacts":[{"location":{"uri":"src/lib.rs"}}],
    "results":[{
      "ruleId":"rust.lang.security.audit",
      "level":"error",
      "message":{"text":"unsafe operation"},
      "locations":[{"physicalLocation":{"artifactLocation":{"index":0},"region":{"startLine":7,"startColumn":3}}}]
    }]
  }]
}"#;

#[test]
fn checked_in_fixtures_cover_distinct_producers_and_unsafe_paths() {
    let codeql = include_str!("../../../tests/fixtures/evidence-rails/sarif-codeql.json");
    let semgrep = include_str!("../../../tests/fixtures/evidence-rails/sarif-semgrep.json");
    let unsafe_report = include_str!("../../../tests/fixtures/evidence-rails/sarif-unsafe.json");
    assert!(evaluate(codeql, "codeql-local").details.is_some());
    assert!(evaluate(semgrep, "semgrep-local").details.is_some());
    assert!(evaluate(unsafe_report, "codeql-local").details.is_none());
}

// The `sarif-codeql-cli*` fixtures are CodeQL CLI 2.27.2 `database analyze --format=sarif-latest`
// output for a two-file Rust crate, without and with `--sarif-group-rules-by-pack` (the layout
// `github/codeql-action` uploads), trimmed of notifications and pack locations.
#[test]
fn genuine_codeql_cli_reports_import_with_source_root_and_pack_rules() {
    for report in [
        include_str!("../../../tests/fixtures/evidence-rails/sarif-codeql-cli.json"),
        include_str!("../../../tests/fixtures/evidence-rails/sarif-codeql-cli-grouped.json"),
    ] {
        let evaluation = evaluate(report, "codeql-local");
        assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
        let details = evaluation.details.expect("CodeQL CLI details");
        let locations = details
            .findings
            .iter()
            .map(|finding| {
                (finding.source_rule_id.as_str(), finding.source_level.as_str(), finding.file.as_str(), finding.line)
            })
            .collect::<std::collections::BTreeSet<_>>();
        let expected = [
            ("rust/demo-let-binding", "warning", "src/lib.rs", 5),
            ("rust/demo-let-binding", "warning", "src/my module.rs", 2),
        ];
        assert_eq!(locations, expected.into_iter().collect());
    }

    let grouped = include_str!("../../../tests/fixtures/evidence-rails/sarif-codeql-cli-grouped.json");
    // CodeQL omits `startColumn` when it is 1, the SARIF default; an explicit 0 stays invalid.
    let column_one = grouped.replace("\"startColumn\":5,", "");
    let details = evaluate(&column_one, "codeql-local").details.expect("default-column details");
    assert!(details.findings.iter().all(|finding| finding.column == 1));
    assert!(
        evaluate(&grouped.replace("\"startColumn\":5,", "\"startColumn\":0,"), "codeql-local")
            .details
            .is_none()
    );

    for invalid in [
        grouped.replace("\"uriBaseId\":\"%SRCROOT%\",\"index\":0}},", "\"uriBaseId\":\"%OTHER%\",\"index\":0}},"),
        grouped.replace("\"toolComponent\":{\"index\":0}", "\"toolComponent\":{\"index\":1}"),
        grouped.replace("\"toolComponent\":{\"index\":0}", "\"toolComponent\":{\"index\":7}"),
        grouped.replace("\"toolComponent\":{\"index\":0}", "\"toolComponent\":{\"name\":\"octet/demo-queries\"}"),
        grouped.replace("\"rule\":{\"id\":\"rust/demo-let-binding\"", "\"rule\":{\"id\":\"rust/other\""),
        grouped
            .replace("\"ruleId\":\"rust/demo-let-binding\",", "\"ruleId\":\"rust/demo-let-binding\",\"ruleIndex\":1,"),
    ] {
        let evaluation = evaluate(&invalid, "codeql-local");
        assert!(evaluation.details.is_none(), "invalid CodeQL report passed: {invalid}");
        assert!(!evaluation.diagnostics.is_empty());
    }
}

#[test]
fn distinct_profiles_preserve_source_identity_and_determinism() {
    let first = evaluate(CODEQL_REPORT, "codeql-local");
    let second = evaluate(CODEQL_REPORT, "codeql-local");
    assert_eq!(first, second);
    let codeql = first.details.expect("CodeQL details");
    assert!(first.diagnostics.is_empty());
    assert_eq!(codeql.source_driver, "CodeQL");
    assert_eq!(codeql.findings[0].source_level, "warning");
    assert_eq!(codeql.findings[0].gate_severity, "warning");
    assert_eq!(codeql.findings[0].file, "src/query.rs");

    let semgrep = evaluate(SEMGREP_REPORT, "semgrep-local");
    let details = semgrep.details.expect("Semgrep details");
    assert!(semgrep.diagnostics.is_empty());
    assert_eq!(details.source_driver, "Semgrep");
    assert_eq!(details.findings[0].source_rule_id, "rust.lang.security.audit");
    assert_eq!(details.findings[0].file, "src/lib.rs");
}

#[test]
fn malformed_profile_version_driver_rule_and_location_fail_closed() {
    assert!(evaluate("not-json", "codeql-local").details.is_none());
    assert!(evaluate(CODEQL_REPORT, "unknown").details.is_none());

    for invalid in [
        CODEQL_REPORT.replace("2.1.0", "2.0.0"),
        CODEQL_REPORT.replace("CodeQL", "Other"),
        CODEQL_REPORT.replace("rust/sql-injection\"}", "other\"}"),
        CODEQL_REPORT.replace("src/query.rs", "../escape.rs"),
        CODEQL_REPORT.replace("src/query.rs", "https://example.invalid/source.rs"),
        CODEQL_REPORT.replace("src/query.rs", "C:/escape.rs"),
        CODEQL_REPORT.replace("src/query.rs", "https:source.rs"),
        CODEQL_REPORT.replace("physicalLocation", "missingPhysicalLocation"),
        CODEQL_REPORT.replace("artifactLocation", "missingArtifactLocation"),
        CODEQL_REPORT.replace("region", "missingRegion"),
    ] {
        let result = evaluate(&invalid, "codeql-local");
        assert!(result.details.is_none(), "invalid report passed: {invalid}");
        assert!(!result.diagnostics.is_empty());
    }
}

#[test]
fn duplicate_overlimit_and_unsupported_severity_fail_closed() {
    let duplicate = CODEQL_REPORT
        .replace("{\"id\":\"rust/sql-injection\"}", "{\"id\":\"rust/sql-injection\"},{\"id\":\"rust/sql-injection\"}");
    assert!(evaluate(&duplicate, "codeql-local").details.is_none());
    assert!(evaluate(&CODEQL_REPORT.replace("warning", "critical"), "codeql-local").details.is_none());

    let profile = selected_profile("codeql-local").expect("CodeQL profile");
    let mut artifact_heavy: serde_json::Value = serde_json::from_str(CODEQL_REPORT).expect("parse report fixture");
    artifact_heavy["runs"][0]["artifacts"] =
        serde_json::Value::Array(vec![serde_json::json!({"location":{"uri":"src/lib.rs"}}); profile.max_artifacts + 1]);
    let artifact_heavy = serde_json::to_string(&artifact_heavy).expect("serialize artifact-limit fixture");
    assert!(evaluate(&artifact_heavy, "codeql-local").details.is_none());

    let mut oversized = CODEQL_REPORT.to_owned();
    oversized.push_str(&" ".repeat(1_048_577));
    assert!(evaluate(&oversized, "codeql-local").details.is_none());
}

#[test]
fn missing_result_level_uses_rule_default_configuration() {
    let error_default = CODEQL_REPORT
        .replace(
            "{\"id\":\"rust/sql-injection\"}",
            "{\"id\":\"rust/sql-injection\",\"defaultConfiguration\":{\"level\":\"error\"}}",
        )
        .replace("\"level\":\"warning\",", "");
    let evaluation = evaluate(&error_default, "codeql-local");
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let details = evaluation.details.expect("rule-default details");
    assert_eq!(details.findings[0].source_level, "error");
    assert_eq!(details.findings[0].gate_severity, "error");

    let explicit_wins = error_default.replace("\"ruleId\"", "\"level\":\"note\",\"ruleId\"");
    let details = evaluate(&explicit_wins, "codeql-local").details.expect("explicit-level details");
    assert_eq!(details.findings[0].source_level, "note");

    let no_default = CODEQL_REPORT.replace("\"level\":\"warning\",", "");
    let details = evaluate(&no_default, "codeql-local").details.expect("fallback details");
    assert_eq!(details.findings[0].source_level, "warning");

    let unsupported_default = error_default.replace("\"level\":\"error\"", "\"level\":\"critical\"");
    assert!(evaluate(&unsupported_default, "codeql-local").details.is_none());
}

#[test]
fn non_fail_result_kinds_do_not_gate_and_undecided_kinds_fail_closed() {
    let with_kind = |kind: &str| {
        CODEQL_REPORT.replace("\"level\":\"warning\",", &format!("\"kind\":\"{kind}\",")).replace(
            "{\"id\":\"rust/sql-injection\"}",
            "{\"id\":\"rust/sql-injection\",\"defaultConfiguration\":{\"level\":\"error\"}}",
        )
    };
    for kind in ["pass", "notApplicable", "informational"] {
        let evaluation = evaluate(&with_kind(kind), "codeql-local");
        assert!(evaluation.diagnostics.is_empty(), "{kind}: {:?}", evaluation.diagnostics);
        assert!(evaluation.details.expect("non-gating details").findings.is_empty(), "{kind} result was imported");
    }
    let explicit_none = with_kind("pass").replace("\"kind\"", "\"level\":\"none\",\"kind\"");
    assert!(evaluate(&explicit_none, "codeql-local").details.expect("pass details").findings.is_empty());

    let fail = evaluate(&with_kind("fail"), "codeql-local").details.expect("fail details");
    assert_eq!(fail.findings[0].gate_severity, "error");

    for invalid in [
        with_kind("review"),
        with_kind("open"),
        with_kind("unexpected"),
        with_kind("pass").replace("\"kind\"", "\"level\":\"error\",\"kind\""),
    ] {
        let evaluation = evaluate(&invalid, "codeql-local");
        assert!(evaluation.details.is_none(), "invalid report passed: {invalid}");
        assert!(!evaluation.diagnostics.is_empty());
    }
}

#[test]
fn missing_null_or_non_array_run_tables_fail_closed() {
    let mut empty_results: serde_json::Value = serde_json::from_str(CODEQL_REPORT).expect("parse report fixture");
    empty_results["runs"][0]["results"] = serde_json::json!([]);
    let empty_results = serde_json::to_string(&empty_results).expect("serialize empty-results fixture");
    let evaluation = evaluate(&empty_results, "codeql-local");
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    assert!(evaluation.details.expect("clean run details").findings.is_empty());

    for (pointer, field, value) in [
        ("/runs/0", "results", None),
        ("/runs/0", "results", Some(serde_json::Value::Null)),
        ("/runs/0", "results", Some(serde_json::json!({}))),
        ("/runs/0/tool/driver", "rules", None),
        ("/runs/0/tool/driver", "rules", Some(serde_json::Value::Null)),
        ("/runs/0/tool/driver", "rules", Some(serde_json::json!("rust/sql-injection"))),
        ("/runs/0", "artifacts", Some(serde_json::Value::Null)),
        ("/runs/0", "artifacts", Some(serde_json::json!({"uri":"src/query.rs"}))),
        ("/runs/0", "invocations", Some(serde_json::json!([{"executionSuccessful":false}]))),
        ("/runs/0", "invocations", Some(serde_json::json!({"executionSuccessful":true}))),
    ] {
        let mut report: serde_json::Value = serde_json::from_str(CODEQL_REPORT).expect("parse report fixture");
        let object = report.pointer_mut(pointer).and_then(serde_json::Value::as_object_mut).expect("fixture object");
        match value {
            Some(value) => object.insert(field.to_owned(), value),
            None => object.remove(field),
        };
        let report = serde_json::to_string(&report).expect("serialize structural fixture");
        let evaluation = evaluate(&report, "codeql-local");
        assert!(evaluation.details.is_none(), "unsupported run structure passed: {report}");
        assert!(!evaluation.diagnostics.is_empty());
    }

    let mut successful: serde_json::Value = serde_json::from_str(CODEQL_REPORT).expect("parse report fixture");
    successful["runs"][0]["invocations"] = serde_json::json!([{"executionSuccessful":true}]);
    let successful = serde_json::to_string(&successful).expect("serialize successful-invocation fixture");
    assert!(evaluate(&successful, "codeql-local").details.is_some());
}

#[test]
fn percent_encoded_locations_decode_to_workspace_paths() {
    for (uri, expected) in [
        ("src/query.rs", "src/query.rs"),
        ("src/my%20query.rs", "src/my query.rs"),
        ("src/100%25.rs", "src/100%.rs"),
        ("src/caf%C3%A9.rs", "src/caf\u{e9}.rs"),
        ("src/a:b.rs", "src/a:b.rs"),
        ("src/a%23b.rs", "src/a#b.rs"),
    ] {
        let report = CODEQL_REPORT.replace("src/query.rs", uri);
        let evaluation = evaluate(&report, "codeql-local");
        assert!(evaluation.diagnostics.is_empty(), "{uri}: {:?}", evaluation.diagnostics);
        assert_eq!(evaluation.details.expect("decoded details").findings[0].file, expected);
    }
}

#[test]
fn octet_sarif_artifact_uris_round_trip_through_intake() {
    for path in [
        "src/lib.rs",
        "src/my module.rs",
        "src/100%.rs",
        "src/a#b?.rs",
        "src/caf\u{e9}.rs",
        "src/a:b.rs",
    ] {
        let report = CODEQL_REPORT.replace("src/query.rs", &crate::output::sarif_artifact_uri(path));
        let evaluation = evaluate(&report, "codeql-local");
        assert!(evaluation.diagnostics.is_empty(), "{path}: {:?}", evaluation.diagnostics);
        assert_eq!(evaluation.details.expect("round-trip details").findings[0].file, path);
    }
}

#[test]
fn percent_encoded_traversal_and_ambiguous_locations_fail_closed() {
    for uri in [
        "src/%2e%2e/%2E%2E/etc/passwd",
        "%2e%2e/escape.rs",
        "src/%2E/query.rs",
        "%2Fetc/passwd",
        "src%2F..%2F..%2Fescape.rs",
        "%5C%5Cserver/share.rs",
        "src/..%5Cescape.rs",
        "C%3A/escape.rs",
        "C:/escape.rs",
        "c:escape.rs",
        "file:/escape.rs",
        "file%3A/escape.rs",
        "src/%00.rs",
        "src/%zz.rs",
        "src/%4.rs",
        "src/%",
        "src/%FF.rs",
        "src/query.rs?x=1",
        "src/query.rs#L1",
    ] {
        let report = CODEQL_REPORT.replace("src/query.rs", uri);
        let evaluation = evaluate(&report, "codeql-local");
        assert!(evaluation.details.is_none(), "unsafe location passed: {uri}");
        assert!(!evaluation.diagnostics.is_empty());
    }
}

#[test]
fn artifact_and_rule_indexes_must_resolve_and_agree() {
    let with_location = |location: &str| {
        SEMGREP_REPORT.replace("\"artifactLocation\":{\"index\":0}", &format!("\"artifactLocation\":{location}"))
    };
    let with_rule_index = |index: &str| {
        SEMGREP_REPORT.replace(
            "\"ruleId\":\"rust.lang.security.audit\",",
            &format!("\"ruleId\":\"rust.lang.security.audit\",\"ruleIndex\":{index},"),
        )
    };
    // SARIF 2.1.0 uses `-1` as the "no index" default for both `index` and `ruleIndex`.
    for valid in [
        with_location("{\"uri\":\"src/lib.rs\",\"index\":0}"),
        with_location("{\"uri\":\"src/li%62.rs\",\"index\":0}"),
        with_location("{\"uri\":\"src/lib.rs\"}"),
        with_location("{\"uri\":\"src/lib.rs\",\"index\":-1}"),
        with_rule_index("0"),
        with_rule_index("-1"),
    ] {
        let evaluation = evaluate(&valid, "semgrep-local");
        assert!(evaluation.diagnostics.is_empty(), "{valid}: {:?}", evaluation.diagnostics);
        assert_eq!(evaluation.details.expect("indexed details").findings[0].file, "src/lib.rs");
    }

    let two_rules = SEMGREP_REPORT.replace(
        "[{\"id\":\"rust.lang.security.audit\"}]",
        "[{\"id\":\"rust.lang.security.other\"},{\"id\":\"rust.lang.security.audit\"}]",
    );
    for invalid in [
        with_location("{\"uri\":\"src/other.rs\",\"index\":0}"),
        with_location("{\"uri\":\"src/lib.rs\",\"index\":1}"),
        with_location("{\"uri\":\"src/lib.rs\",\"index\":-2}"),
        with_location("{\"index\":-1}"),
        with_location("{\"uri\":\"src/lib.rs\",\"index\":\"0\"}"),
        with_location("{\"index\":1}"),
        with_rule_index("1"),
        with_rule_index("-2"),
        with_rule_index("\"0\""),
        two_rules.replace(
            "\"ruleId\":\"rust.lang.security.audit\",",
            "\"ruleId\":\"rust.lang.security.audit\",\"ruleIndex\":0,",
        ),
    ] {
        let evaluation = evaluate(&invalid, "semgrep-local");
        assert!(evaluation.details.is_none(), "bad index passed: {invalid}");
        assert!(!evaluation.diagnostics.is_empty());
    }
    let two_rules_valid = two_rules.replace(
        "\"ruleId\":\"rust.lang.security.audit\",",
        "\"ruleId\":\"rust.lang.security.audit\",\"ruleIndex\":1,",
    );
    assert!(evaluate(&two_rules_valid, "semgrep-local").details.is_some());
}

#[test]
fn profile_registry_and_projection_identity_are_fresh() {
    assert_eq!(prefixed_identity(PROFILE_JSON.as_bytes()), PROFILE_IDENTITY);
    for id in ["codeql-local", "semgrep-local"] {
        let profile = selected_profile(id).expect("known profile");
        assert_eq!(profile.supported_version, SUPPORTED_VERSION);
        assert!(profile.require_rule_id);
        assert!(profile.workspace_relative_only);
        assert_eq!(profile.evidence_tier, "static_lint");
    }
    let details = evaluate(CODEQL_REPORT, "codeql-local").details.expect("details");
    assert_eq!(prefixed_identity(&projection_bytes(&details).expect("projection")), details.projection_identity);
}
