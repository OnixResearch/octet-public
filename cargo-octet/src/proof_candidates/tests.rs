use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde_json::json;

use super::*;

const PROPERTY_ID: &str = "octet.verus_proof_rails.addition-bound-production#rail-0";
const POLICY_ID: &str = "octet.verus_proof_rails.addition-bound-production";
const CHECKED_SCOPE: &str = "formal-proof:production:octet.verus_proof_rails.addition-bound-production#rail-0";
const RECEIPT: &str = "proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt";
const ANCHOR: &str =
    "octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus_proof_rails.addition-bound-production#rail-0";
const NON_CLAIMS: [&str; 5] = [
    "not whole-program correctness",
    "not certification",
    "not business-domain validation",
    "not semantic equivalence",
    "not full safety",
];

fn candidate_args(
    source: PathBuf,
    proof_cards: Option<PathBuf>,
    proof_refs: Vec<PathBuf>,
    trust_boundary: Option<PathBuf>,
) -> ProveArgs {
    ProveArgs {
        registry: None,
        policy_artifacts_dir: None,
        source: vec![source],
        family: Vec::new(),
        rail: Vec::new(),
        artifact_dir: PathBuf::from("artifacts/octet-proofs"),
        output_format: OutputFormat::Json,
        command_line: "cargo octet prove --candidates".to_owned(),
        list: false,
        explain: false,
        candidates: true,
        proof_cards,
        proof_refs,
        trust_boundary,
        negative_check: None,
    }
}

fn card_json(status: &str) -> serde_json::Value {
    json!({
        "proof_or_property_id": PROPERTY_ID,
        "source_requirement_or_policy_id": POLICY_ID,
        "checked_rust_source_scope": CHECKED_SCOPE,
        "evidence_tier": "formal_proof",
        "evidence_origin": "octet_rerun",
        "status": status,
        "artifact_blake3_identities": [{"artifact": RECEIPT, "blake3": "a".repeat(64)}],
        "non_claims": NON_CLAIMS,
        "diagnostics": [],
    })
}

fn deck_json(status: &str) -> serde_json::Value {
    json!({
        "schema_version": "octet-proof-cards/v1",
        "cards": [card_json(status)],
    })
}

fn write_deck(path: &Path, status: &str) {
    fs::write(path, serde_json::to_string_pretty(&deck_json(status)).expect("deck json")).expect("write deck");
}

fn reference_json(source_path: &str, source_text: &str, line: u32) -> serde_json::Value {
    json!({
        "schema_version": "octet-proof-evidence-ref/v1",
        "reference_id": "unchecked-narrowing-range-proof",
        "consumer_context": format!("allow tigerstyle::unchecked_narrowing at {source_path}:{line}"),
        "consumer_source_path": source_path,
        "consumer_source_blake3": blake3::hash(source_text.as_bytes()).to_hex().to_string(),
        "consumer_source_anchor": ANCHOR,
        "proof_or_property_id": PROPERTY_ID,
        "source_requirement_or_policy_id": POLICY_ID,
        "checked_rust_source_scope": CHECKED_SCOPE,
        "expected_evidence_tier": "formal_proof",
        "required_status": "passed",
        "required_artifacts": [RECEIPT],
        "non_claims": NON_CLAIMS,
    })
}

fn write_reference(path: &Path, value: &serde_json::Value) {
    fs::write(path, serde_json::to_string_pretty(value).expect("reference json")).expect("write reference");
}

fn trust_doc(prefixes: &[&str]) -> String {
    format!("# Trust boundary\n\n```text {TRUST_BOUNDARY_FENCE}\n{}\n```\n", prefixes.join("\n"))
}

#[test]
fn scanner_finds_tool_scoped_bare_multiline_and_expect_allowances() {
    let source = concat!(
        "// #[allow(tigerstyle::unchecked_narrowing)]\n",
        "/* #[allow(no_panic)] */\n",
        "const TEXT: &str = \"#[allow(tigerstyle::no_unwrap)]\";\n",
        "#[allow(tigerstyle::unchecked_narrowing)]\n",
        "fn first() {}\n",
        "#[allow(no_panic, reason = \"fixture\")]\n",
        "fn second() {}\n",
        "#[expect(\n",
        "    tigerstyle::unchecked_division,\n",
        "    reason = \"checked above\",\n",
        ")]\n",
        "fn third() {}\n",
    );

    let allowances = scan_allowances(source);

    assert_eq!(
        allowances
            .iter()
            .map(|allowance| (allowance.lint.as_str(), allowance.attribute.as_str(), allowance.line))
            .collect::<Vec<_>>(),
        vec![
            ("unchecked_narrowing", "allow", 4),
            ("no_panic", "allow", 6),
            ("unchecked_division", "expect", 8),
        ]
    );
}

#[test]
fn scanner_expands_group_allowances_to_member_lints() {
    let allowances = scan_allowances("#[allow(tigerstyle::safety)]\nfn guarded() {}\n");

    let lints = allowances.iter().map(|allowance| allowance.lint.as_str()).collect::<Vec<_>>();
    for member in lint_names_for_group("tigerstyle::safety") {
        assert!(lints.contains(&member), "missing safety member `{member}`");
    }
    assert!(lints.contains(&"unchecked_narrowing"));
}

#[test]
fn rank_score_weights_severity_then_trust_boundary() {
    assert!(rank_score("deny", true) > rank_score("deny", false));
    assert!(rank_score("deny", false) > rank_score("warn", true));
    assert!(rank_score("warn", true) > rank_score("warn", false));
    assert!(rank_score("warn", false) > rank_score("allow", true));
    assert_eq!(rank_score("warn", true), 5);
    assert_eq!(rank_score("allow", false), 2);
}

#[test]
fn candidate_is_listed_then_drops_after_validated_proof_ref() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(&source_dir).expect("source dir");
    let source_path = source_dir.join("codec.rs");
    let source_text = format!("#[allow(tigerstyle::unchecked_narrowing)]\n// {ANCHOR}\npub fn narrow() {{}}\n");
    fs::write(&source_path, &source_text).expect("source");
    let deck_path = temp.path().join("proof-cards.json");
    write_deck(&deck_path, "passed");
    let reference_path = temp.path().join("proof-ref.json");
    write_reference(&reference_path, &reference_json(source_path.to_string_lossy().as_ref(), &source_text, 1));
    let trust_path = temp.path().join("trust.md");
    let trust_prefix = format!("{}/", source_dir.display());
    fs::write(&trust_path, trust_doc(&[trust_prefix.as_str()])).expect("trust doc");

    let before = build_report(&candidate_args(source_dir.clone(), None, Vec::new(), Some(trust_path.clone())))
        .expect("before report");
    assert_eq!(before.candidates.len(), 1);
    assert_eq!(before.candidates[0].lint, "unchecked_narrowing");
    assert_eq!(before.candidates[0].proof_ref.status, STATUS_MISSING);
    assert!(before.candidates[0].trust_boundary);
    assert!(before.candidates[0].rail.is_some());
    assert_eq!(before.discharged, 0);

    let after = build_report(&candidate_args(source_dir, Some(deck_path), vec![reference_path], Some(trust_path)))
        .expect("after report");
    assert!(after.candidates.is_empty());
    assert_eq!(after.discharged, 1);
    assert!(!after.is_blocking());
}

#[test]
fn invalid_or_unbound_proof_ref_keeps_candidate_listed_and_blocks() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(&source_dir).expect("source dir");
    let source_path = source_dir.join("codec.rs");
    let source_text = format!("#[allow(tigerstyle::unchecked_narrowing)]\n// {ANCHOR}\npub fn narrow() {{}}\n");
    fs::write(&source_path, &source_text).expect("source");
    let deck_path = temp.path().join("proof-cards.json");
    write_deck(&deck_path, "passed");
    let trust_path = temp.path().join("trust.md");
    fs::write(&trust_path, trust_doc(&[])).expect("trust doc");

    let mut stale = reference_json(source_path.to_string_lossy().as_ref(), &source_text, 1);
    stale["consumer_source_blake3"] = json!("stale-digest");
    let stale_path = temp.path().join("stale-proof-ref.json");
    write_reference(&stale_path, &stale);
    let stale_report = build_report(&candidate_args(
        source_dir.clone(),
        Some(deck_path.clone()),
        vec![stale_path],
        Some(trust_path.clone()),
    ))
    .expect("stale report");
    assert_eq!(stale_report.candidates.len(), 1);
    assert_eq!(stale_report.candidates[0].proof_ref.status, STATUS_INVALID);
    assert!(stale_report.is_blocking());

    let mut unbound = reference_json(source_path.to_string_lossy().as_ref(), &source_text, 2);
    unbound["consumer_context"] = json!("allow tigerstyle::unchecked_narrowing at elsewhere.rs:7");
    let unbound_path = temp.path().join("unbound-proof-ref.json");
    write_reference(&unbound_path, &unbound);
    let unbound_report =
        build_report(&candidate_args(source_dir, Some(deck_path), vec![unbound_path], Some(trust_path)))
            .expect("unbound report");
    assert_eq!(unbound_report.candidates.len(), 1);
    assert_eq!(unbound_report.candidates[0].proof_ref.status, STATUS_UNBOUND);
    assert!(unbound_report.is_blocking());
}

#[test]
fn unsupported_lints_are_reported_without_claiming_a_rail() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(&source_dir).expect("source dir");
    fs::write(source_dir.join("panic.rs"), "#[allow(no_panic)]\nfn guarded() {}\n").expect("source");
    let trust_path = temp.path().join("trust.md");
    fs::write(&trust_path, trust_doc(&[])).expect("trust doc");

    let report = build_report(&candidate_args(source_dir, None, Vec::new(), Some(trust_path))).expect("report");

    assert_eq!(report.candidates.len(), 1);
    assert!(report.candidates[0].unsupported);
    assert!(report.candidates[0].rail.is_none());
    assert_eq!(report.unsupported_lints.len(), 1);
    assert_eq!(report.unsupported_lints[0].lint, "no_panic");
    assert_eq!(report.unsupported_lints[0].candidates, 1);
}

#[test]
fn candidate_json_is_stable_and_rank_sorted() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(source_dir.join("nested")).expect("source dir");
    fs::write(
        source_dir.join("a.rs"),
        "#[allow(no_panic)]\nfn first() {}\n#[allow(tigerstyle::unchecked_narrowing)]\nfn second() {}\n",
    )
    .expect("a");
    fs::write(source_dir.join("nested/b.rs"), "#[allow(tigerstyle::unchecked_division)]\nfn third() {}\n").expect("b");
    let trust_path = temp.path().join("trust.md");
    fs::write(&trust_path, trust_doc(&["src/nested/"])).expect("trust doc");
    let args = candidate_args(source_dir, None, Vec::new(), Some(trust_path));

    let first = build_report(&args).expect("first report");
    let second = build_report(&args).expect("second report");
    let first_json = serde_json::to_string(&first).expect("first json");
    let second_json = serde_json::to_string(&second).expect("second json");

    assert_eq!(first_json, second_json);
    assert_eq!(
        first.candidates.iter().map(|candidate| candidate.rank).collect::<Vec<_>>(),
        (1..=first.candidates.len()).collect::<Vec<_>>()
    );
    assert!(first.candidates.windows(2).all(|pair| {
        pair[0].rank_score > pair[1].rank_score
            || (pair[0].rank_score == pair[1].rank_score
                && (pair[0].lint.as_str(), pair[0].path.as_str(), pair[0].line)
                    <= (pair[1].lint.as_str(), pair[1].path.as_str(), pair[1].line))
    }));
    assert_eq!(first.schema_version, SCHEMA_VERSION);
}

#[test]
fn trust_boundary_parser_reads_only_marked_fence() {
    let text = format!(
        "```text\nsrc/ignored.rs\n```\n\n```text {TRUST_BOUNDARY_FENCE}\nsrc/verified/\ncargo-octet/src/prove.rs\n```\n"
    );
    let prefixes = parse_trust_boundary_prefixes(&text);

    assert_eq!(prefixes, vec!["cargo-octet/src/prove.rs".to_owned(), "src/verified/".to_owned()]);
    assert!(on_trust_boundary("src/verified/addition.rs", &prefixes));
    assert!(!on_trust_boundary("src/verifiedness/addition.rs", &prefixes));
    assert!(!on_trust_boundary("src/ignored.rs", &prefixes));
}

const ADD_BASELINE_SOURCE: &str = "pub fn add(a: u32, b: u32) -> u32 { a + b }\n";
const ADD_MUTANT_SOURCE: &str = "pub fn add(a: u32, b: u32) -> u32 { a.wrapping_add(b) }\n";
const NEGATIVE_VERIFIER: &str = "verus@0.2024";
const NEGATIVE_TOOLCHAIN: &str = "rustc 1.83.0";
const NEGATIVE_APPROVED_SPEC: &str = "spec.verus.addition-bound.v1";
const NEGATIVE_MUTATION_ID: &str = "mutation-wrap-add";
const BASELINE_VERIFIER_OUTPUT: &str = "baseline: obligation accepted\n";
const MUTANT_VERIFIER_OUTPUT: &str = "mutant: obligation rejected\n";

struct NegativeFixture {
    _temp: tempfile::TempDir,
    args: NegativeCheckArgs,
    evidence_path: PathBuf,
    evidence: serde_json::Value,
    baseline_deck: PathBuf,
    mutant_deck: PathBuf,
    baseline_source: PathBuf,
    mutant_source: PathBuf,
    mutant_raw_output: PathBuf,
    baseline_source_blake3: String,
    mutant_source_blake3: String,
}

fn negative_fixture(baseline_status: &str, mutant_status: &str) -> NegativeFixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    let baseline_source = root.join("baseline/src/codec.rs");
    let mutant_source = root.join("mutant/src/codec.rs");
    for source in [&baseline_source, &mutant_source] {
        fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
    }
    fs::write(&baseline_source, ADD_BASELINE_SOURCE).expect("write baseline source");
    fs::write(&mutant_source, ADD_MUTANT_SOURCE).expect("write mutant source");
    let baseline_source_blake3 = blake3::hash(ADD_BASELINE_SOURCE.as_bytes()).to_hex().to_string();
    let mutant_source_blake3 = blake3::hash(ADD_MUTANT_SOURCE.as_bytes()).to_hex().to_string();

    let baseline_raw_output = root.join("baseline/verifier-output.log");
    let mutant_raw_output = root.join("mutant/verifier-output.log");
    fs::write(&baseline_raw_output, BASELINE_VERIFIER_OUTPUT).expect("write baseline output");
    fs::write(&mutant_raw_output, MUTANT_VERIFIER_OUTPUT).expect("write mutant output");

    let baseline_deck = root.join("baseline/proof-cards.json");
    let mutant_deck = root.join("mutant/proof-cards.json");
    fs::write(
        &baseline_deck,
        serde_json::to_string_pretty(&negative_deck_json(baseline_status, &baseline_source)).expect("deck json"),
    )
    .expect("write baseline deck");
    fs::write(
        &mutant_deck,
        serde_json::to_string_pretty(&negative_deck_json(mutant_status, &mutant_source)).expect("deck json"),
    )
    .expect("write mutant deck");
    let baseline_deck_blake3 =
        blake3::hash(&fs::read(&baseline_deck).expect("baseline deck bytes")).to_hex().to_string();
    let mutant_deck_blake3 = blake3::hash(&fs::read(&mutant_deck).expect("mutant deck bytes")).to_hex().to_string();

    let evidence = json!({
        "schema_version": "octet-proof-negative-check-evidence/v1",
        "verifier": NEGATIVE_VERIFIER,
        "toolchain": NEGATIVE_TOOLCHAIN,
        "verification_mode": "verus",
        "command_argv": ["verus", "proofs/addition_bound.rs"],
        "verification_configuration_blake3": "c".repeat(64),
        "approved_specification_id": NEGATIVE_APPROVED_SPEC,
        "proof_or_property_id": PROPERTY_ID,
        "source_requirement_or_policy_id": POLICY_ID,
        "checked_rust_source_scope": CHECKED_SCOPE,
        "evidence_tier": "formal_proof",
        "mutation": {
            "mutation_id": NEGATIVE_MUTATION_ID,
            "patch_blake3": "d".repeat(64),
            "baseline_source_blake3": baseline_source_blake3,
            "mutant_source_blake3": mutant_source_blake3,
        },
        "baseline": negative_run_json(
            "accepted",
            None,
            &baseline_source,
            &baseline_deck,
            &baseline_deck_blake3,
            &baseline_raw_output,
        ),
        "mutant": negative_run_json(
            "rejected",
            Some(PROPERTY_ID),
            &mutant_source,
            &mutant_deck,
            &mutant_deck_blake3,
            &mutant_raw_output,
        ),
        "non_claims": [],
    });
    let evidence_path = root.join("verifier-evidence.json");
    fs::write(&evidence_path, serde_json::to_string_pretty(&evidence).expect("evidence json")).expect("write evidence");
    let args = NegativeCheckArgs {
        proof_cards: baseline_deck.clone(),
        mutant_cards: mutant_deck.clone(),
        proof_or_property_id: PROPERTY_ID.to_owned(),
        mutant_source: mutant_source.display().to_string(),
        verifier_evidence: None,
        output: None,
    };
    NegativeFixture {
        _temp: temp,
        args,
        evidence_path,
        evidence,
        baseline_deck,
        mutant_deck,
        baseline_source,
        mutant_source,
        mutant_raw_output,
        baseline_source_blake3,
        mutant_source_blake3,
    }
}

fn negative_run_json(
    outcome: &str,
    attribution: Option<&str>,
    source: &Path,
    deck: &Path,
    deck_blake3: &str,
    raw_output: &Path,
) -> serde_json::Value {
    json!({
        "outcome": outcome,
        "rejection_attribution": attribution,
        "approved_specification_id": NEGATIVE_APPROVED_SPEC,
        "proof_or_property_id": PROPERTY_ID,
        "source_requirement_or_policy_id": POLICY_ID,
        "checked_rust_source_scope": CHECKED_SCOPE,
        "source_path": source.display().to_string(),
        "source_blake3": blake3::hash(&fs::read(source).expect("source bytes")).to_hex().to_string(),
        "deck_path": deck.display().to_string(),
        "deck_blake3": deck_blake3,
        "raw_output": {
            "artifact": raw_output.display().to_string(),
            "blake3": blake3::hash(&fs::read(raw_output).expect("output bytes")).to_hex().to_string(),
        },
    })
}

fn negative_card_json(status: &str, source: &Path) -> serde_json::Value {
    json!({
        "proof_or_property_id": PROPERTY_ID,
        "source_requirement_or_policy_id": POLICY_ID,
        "policy_id": POLICY_ID,
        "checked_rust_source_scope": CHECKED_SCOPE,
        "source_paths": [source.display().to_string()],
        "evidence_tier": "formal_proof",
        "evidence_origin": "octet_rerun",
        "status": status,
        "verifier_mode": "verus",
        "verifier_identity_summary": NEGATIVE_VERIFIER,
        "toolchain_identity_summary": NEGATIVE_TOOLCHAIN,
        "artifact_blake3_identities": [{"artifact": RECEIPT, "blake3": "a".repeat(64)}],
        "non_claims": NON_CLAIMS,
        "diagnostics": [],
    })
}

fn negative_deck_json(status: &str, source: &Path) -> serde_json::Value {
    json!({
        "schema_version": "octet-proof-cards/v1",
        "cards": [negative_card_json(status, source)],
    })
}

fn write_negative_evidence(fixture: &NegativeFixture) {
    fs::write(&fixture.evidence_path, serde_json::to_string_pretty(&fixture.evidence).expect("evidence json"))
        .expect("write evidence");
}

fn record_fixture(fixture: &NegativeFixture) -> Result<i32, String> {
    write_negative_evidence(fixture);
    run_record_negative_check_with_evidence(&fixture.args, Some(&fixture.evidence_path), OutputFormat::Json)
}

fn recorded_negative_check(fixture: &NegativeFixture) -> proof_ref::ProofCardNegativeCheck {
    let deck = proof_ref::read_deck_file(&fixture.baseline_deck).expect("read deck");
    deck.cards
        .iter()
        .find(|card| card.proof_or_property_id == PROPERTY_ID)
        .expect("baseline card")
        .negative_check
        .clone()
        .expect("negative check")
}

fn deck_reference_json() -> serde_json::Value {
    json!({
        "schema_version": "octet-proof-evidence-ref/v1",
        "reference_id": "unchecked-narrowing-range-proof",
        "consumer_context": "allow tigerstyle::unchecked_narrowing at src/codec.rs:42",
        "proof_or_property_id": PROPERTY_ID,
        "source_requirement_or_policy_id": POLICY_ID,
        "checked_rust_source_scope": CHECKED_SCOPE,
        "expected_evidence_tier": "formal_proof",
        "required_status": "passed",
        "required_artifacts": [RECEIPT],
        "non_claims": NON_CLAIMS,
    })
}

fn validate_recorded_deck(fixture: &NegativeFixture) -> Vec<String> {
    let deck = proof_ref::read_deck_file(&fixture.baseline_deck).expect("read deck");
    let reference: proof_ref::ProofEvidenceRef = serde_json::from_value(deck_reference_json()).expect("reference json");
    proof_ref::validate_reference_against_source(&reference, &deck, "", "")
}

fn mutate_recorded_negative(fixture: &NegativeFixture, mutate: impl FnOnce(&mut serde_json::Value)) {
    let text = fs::read_to_string(&fixture.baseline_deck).expect("read deck");
    let mut value: serde_json::Value = serde_json::from_str(&text).expect("deck json");
    let card = value
        .get_mut("cards")
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|cards| cards.iter_mut().find(|card| card["proof_or_property_id"] == PROPERTY_ID))
        .expect("baseline card");
    mutate(&mut card["negative_check"]);
    fs::write(&fixture.baseline_deck, serde_json::to_string_pretty(&value).expect("deck json") + "\n")
        .expect("write deck");
}

#[test]
fn attributed_rejection_demonstrates_the_declared_mutation() {
    let fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);

    let code = record_fixture(&fixture).expect("record negative check");
    let negative = recorded_negative_check(&fixture);

    assert_eq!(code, 0);
    assert_eq!(negative.status, NEGATIVE_STATUS_DEMONSTRATED);
    assert_eq!(negative.schema_version, NEGATIVE_CHECK_SCHEMA_VERSION);
    assert_eq!(negative.mutant_card_status, CARD_STATUS_FAILED);
    assert!(negative.non_claims.iter().any(|non_claim| non_claim.contains("this mutation only")));
    let measured = negative.measured.as_ref().expect("measured identities");
    assert_eq!(measured.baseline_source_blake3, fixture.baseline_source_blake3);
    assert_eq!(measured.mutant_source_blake3, fixture.mutant_source_blake3);
    assert_ne!(measured.baseline_source_blake3, measured.mutant_source_blake3);
    assert_eq!(measured.mutation_id, NEGATIVE_MUTATION_ID);
    assert_eq!(measured.patch_blake3, "d".repeat(64));
    assert_eq!(negative.mutant_cards_blake3, measured.mutant_deck_blake3);
    assert!(
        validate_recorded_deck(&fixture).is_empty(),
        "valid record should validate: {:?}",
        validate_recorded_deck(&fixture)
    );
}

#[test]
fn failed_baseline_cannot_demonstrate_the_mutation() {
    let fixture = negative_fixture(CARD_STATUS_FAILED, CARD_STATUS_FAILED);

    let code = record_fixture(&fixture).expect("record negative check");
    let negative = recorded_negative_check(&fixture);

    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_INCONCLUSIVE);
    assert!(
        negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("passing baseline")),
        "unexpected diagnostics: {:?}",
        negative.diagnostics
    );
}

#[test]
fn missing_verifier_evidence_is_inconclusive() {
    let fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);

    let code = run_record_negative_check(&fixture.args, OutputFormat::Human).expect("record negative check");
    let negative = recorded_negative_check(&fixture);

    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_INCONCLUSIVE);
    assert!(
        negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("missing verifier evidence")),
        "unexpected diagnostics: {:?}",
        negative.diagnostics
    );
}

#[test]
#[allow(clippy::type_complexity)]
fn unverifiable_mutant_outcomes_are_inconclusive() {
    let mutations: Vec<(&str, Box<dyn FnOnce(&mut serde_json::Value)>)> = vec![
        ("compile error", Box::new(|evidence| evidence["mutant"]["outcome"] = json!("compile_error"))),
        ("crash", Box::new(|evidence| evidence["mutant"]["outcome"] = json!("crash"))),
        ("timeout", Box::new(|evidence| evidence["mutant"]["outcome"] = json!("timeout"))),
        ("no evidence", Box::new(|evidence| evidence["mutant"]["outcome"] = json!("no_evidence"))),
        (
            "mutant accepted",
            Box::new(|evidence| {
                evidence["mutant"]["outcome"] = json!("accepted");
                evidence["mutant"]["rejection_attribution"] = serde_json::Value::Null;
            }),
        ),
        (
            "unrelated obligation",
            Box::new(|evidence| evidence["mutant"]["rejection_attribution"] = json!("other.obligation")),
        ),
        (
            "missing attribution",
            Box::new(|evidence| evidence["mutant"]["rejection_attribution"] = serde_json::Value::Null),
        ),
        ("baseline rejected", Box::new(|evidence| evidence["baseline"]["outcome"] = json!("rejected"))),
        (
            "baseline compile error",
            Box::new(|evidence| evidence["baseline"]["outcome"] = json!("compile_error")),
        ),
        ("empty evidence document", Box::new(|evidence| *evidence = json!({}))),
        (
            "empty verifier output",
            Box::new(|evidence| {
                let artifact =
                    evidence["mutant"]["raw_output"]["artifact"].as_str().expect("raw output artifact").to_owned();
                fs::write(&artifact, "").expect("empty verifier output");
                evidence["mutant"]["raw_output"]["blake3"] = json!(blake3::hash(b"").to_hex().to_string());
            }),
        ),
    ];

    for (label, mutate) in mutations {
        let mut fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
        mutate(&mut fixture.evidence);
        let code = record_fixture(&fixture).expect("record negative check");
        let negative = recorded_negative_check(&fixture);

        assert_eq!(code, 1, "{label} should exit non-zero");
        assert_eq!(
            negative.status, NEGATIVE_STATUS_INCONCLUSIVE,
            "{label} should be inconclusive: {:?}",
            negative.diagnostics
        );
        assert!(!negative.diagnostics.is_empty(), "{label} should explain why it is inconclusive");
    }
}

#[test]
fn forged_failed_status_without_evidence_fails_validation() {
    let fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    let forged = [
        (
            "v1 annotation",
            json!({
                "schema_version": "octet-proof-negative-check/v1",
                "status": "property_demonstrated",
                "mutant_source": fixture.mutant_source.display().to_string(),
                "mutant_card_status": "failed",
                "mutant_card_evidence_tier": "formal_proof",
                "mutant_cards_blake3": "e".repeat(64),
                "non_claims": ["mutant rejection demonstrates discrimination for this mutation only"],
                "diagnostics": [],
            }),
            "cannot attribute verifier rejection",
        ),
        (
            "v2 annotation without evidence",
            json!({
                "schema_version": "octet-proof-negative-check/v2",
                "status": "property_demonstrated",
                "mutant_source": fixture.mutant_source.display().to_string(),
                "mutant_card_status": "failed",
                "mutant_card_evidence_tier": "formal_proof",
                "mutant_cards_blake3": "e".repeat(64),
                "non_claims": ["mutant rejection demonstrates discrimination for this mutation only"],
                "diagnostics": [],
            }),
            "missing verifier evidence",
        ),
    ];
    for (label, negative_check, expected) in forged {
        let mut card = negative_card_json(CARD_STATUS_PASSED, &fixture.baseline_source);
        card["negative_check"] = negative_check;
        fs::write(
            &fixture.baseline_deck,
            serde_json::to_string_pretty(&json!({
                "schema_version": "octet-proof-cards/v1",
                "cards": [card],
            }))
            .expect("deck json"),
        )
        .expect("write forged deck");

        let diagnostics = validate_recorded_deck(&fixture);

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
            "{label} demonstrated status should fail validation: {diagnostics:?}"
        );
    }
}

#[test]
fn unchanged_mutant_bytes_are_rejected() {
    let mut fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    fs::write(&fixture.mutant_source, ADD_BASELINE_SOURCE).expect("unchanged mutant source");
    fixture.evidence["mutant"]["source_blake3"] = json!(fixture.baseline_source_blake3.clone());
    fixture.evidence["mutation"]["mutant_source_blake3"] = json!(fixture.baseline_source_blake3.clone());

    let code = record_fixture(&fixture).expect("record negative check");
    let negative = recorded_negative_check(&fixture);

    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_INCONCLUSIVE);
    assert!(
        negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("unchanged")),
        "unexpected diagnostics: {:?}",
        negative.diagnostics
    );
}

#[test]
fn unsupported_mutant_schema_duplicate_ids_and_wrong_locator_cannot_pass() {
    let mut fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    fixture.args.mutant_source = "src/elsewhere_mutant.rs".to_owned();
    let code = record_fixture(&fixture).expect("record negative check");
    let negative = recorded_negative_check(&fixture);
    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_INCONCLUSIVE);
    assert!(
        negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("mutant source locator")),
        "unexpected diagnostics: {:?}",
        negative.diagnostics
    );

    let wrong_schema = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    let mut mutant_value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&wrong_schema.mutant_deck).expect("mutant deck")).expect("json");
    mutant_value["schema_version"] = json!("octet-proof-cards/v2");
    fs::write(&wrong_schema.mutant_deck, serde_json::to_string_pretty(&mutant_value).expect("json"))
        .expect("write mutant deck");
    let error = record_fixture(&wrong_schema).expect_err("unsupported mutant schema");
    assert!(error.contains("unsupported schema"), "unexpected error: {error}");

    let mut duplicate = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    let mutant_value = fs::read_to_string(&duplicate.mutant_deck).expect("mutant deck");
    let mut mutant_value: serde_json::Value = serde_json::from_str(&mutant_value).expect("json");
    let duplicate_card = mutant_value["cards"][0].clone();
    mutant_value["cards"] = json!([duplicate_card.clone(), duplicate_card]);
    fs::write(&duplicate.mutant_deck, serde_json::to_string_pretty(&mutant_value).expect("json"))
        .expect("write duplicate mutant deck");
    let duplicate_evidence_path = duplicate.evidence_path.clone();
    let duplicate_mutant = duplicate.mutant_deck.clone();
    duplicate.evidence["mutant"]["deck_blake3"] =
        json!(blake3::hash(&fs::read(&duplicate_mutant).expect("deck bytes")).to_hex().to_string());
    let code = record_fixture(&duplicate).expect("record negative check");
    let negative = recorded_negative_check(&duplicate);
    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_INCONCLUSIVE);
    assert!(
        negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("exactly one card")),
        "unexpected diagnostics: {:?}",
        negative.diagnostics
    );
    assert!(duplicate_evidence_path.is_file());

    let duplicate_baseline = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    let baseline_value = fs::read_to_string(&duplicate_baseline.baseline_deck).expect("baseline deck");
    let mut baseline_value: serde_json::Value = serde_json::from_str(&baseline_value).expect("json");
    let duplicate_card = baseline_value["cards"][0].clone();
    baseline_value["cards"] = json!([duplicate_card.clone(), duplicate_card]);
    fs::write(&duplicate_baseline.baseline_deck, serde_json::to_string_pretty(&baseline_value).expect("json"))
        .expect("write duplicate baseline deck");
    let error = record_fixture(&duplicate_baseline).expect_err("duplicate baseline card");
    assert!(error.contains("exactly one card"), "unexpected error: {error}");
}

#[test]
fn changed_approved_specification_cannot_pass() {
    let mut fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    fixture.evidence["mutant"]["approved_specification_id"] = json!("spec.other.v1");

    let code = record_fixture(&fixture).expect("record negative check");
    let negative = recorded_negative_check(&fixture);

    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_INCONCLUSIVE);
    assert!(
        negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("approved_specification_id")),
        "unexpected diagnostics: {:?}",
        negative.diagnostics
    );
}

#[test]
fn still_passing_mutant_remains_property_not_demonstrated() {
    let mut fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_PASSED);
    fixture.evidence["mutant"]["outcome"] = json!("accepted");
    fixture.evidence["mutant"]["rejection_attribution"] = serde_json::Value::Null;
    fs::write(&fixture.mutant_raw_output, "mutant: obligation accepted\n").expect("accepted output");
    fixture.evidence["mutant"]["raw_output"]["blake3"] =
        json!(blake3::hash(&fs::read(&fixture.mutant_raw_output).expect("output")).to_hex().to_string());

    let code = record_fixture(&fixture).expect("record negative check");
    let negative = recorded_negative_check(&fixture);

    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_NOT_DEMONSTRATED);
    assert_eq!(negative.mutant_card_status, CARD_STATUS_PASSED);
    assert!(negative.diagnostics.is_empty(), "unexpected diagnostics: {:?}", negative.diagnostics);
    assert!(validate_recorded_deck(&fixture).is_empty());
}

#[test]
fn negative_check_output_must_not_overwrite_inputs() {
    let mut fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    let mutant_deck = fixture.mutant_deck.clone();
    fixture.args.output = Some(mutant_deck);
    let error = record_fixture(&fixture).expect_err("aliasing mutant deck");
    assert!(error.contains("must not overwrite"), "unexpected error: {error}");

    let mut fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    let evidence_path = fixture.evidence_path.clone();
    fixture.args.output = Some(evidence_path);
    let error = record_fixture(&fixture).expect_err("aliasing verifier evidence");
    assert!(error.contains("must not overwrite"), "unexpected error: {error}");
}

#[test]
fn verification_detects_changed_source_and_verifier_output() {
    let fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    record_fixture(&fixture).expect("record negative check");
    assert!(validate_recorded_deck(&fixture).is_empty());

    fs::write(&fixture.mutant_source, "pub fn add(a: u32, b: u32) -> u32 { a ^ b }\n").expect("changed mutant source");
    let diagnostics = validate_recorded_deck(&fixture);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("mutant source") && diagnostic.contains("BLAKE3")),
        "changed source should fail validation: {diagnostics:?}"
    );
    fs::write(&fixture.mutant_source, ADD_MUTANT_SOURCE).expect("restore mutant source");
    assert!(validate_recorded_deck(&fixture).is_empty());

    fs::write(&fixture.mutant_raw_output, "tampered verifier output\n").expect("changed output");
    let diagnostics = validate_recorded_deck(&fixture);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("verifier output") && diagnostic.contains("BLAKE3")),
        "changed verifier output should fail validation: {diagnostics:?}"
    );
}

#[test]
fn verification_detects_changed_mutation_obligation_and_spec() {
    for (label, mutate) in [
        (
            "mutation identity",
            Box::new(|negative: &mut serde_json::Value| {
                negative["measured"]["mutation_id"] = json!("other-mutation");
            }) as Box<dyn FnOnce(&mut serde_json::Value)>,
        ),
        (
            "expected obligation",
            Box::new(|negative: &mut serde_json::Value| {
                negative["evidence"]["mutant"]["rejection_attribution"] = json!("other.obligation");
            }),
        ),
        (
            "approved specification",
            Box::new(|negative: &mut serde_json::Value| {
                negative["evidence"]["approved_specification_id"] = json!("spec.other.v1");
            }),
        ),
        (
            "retained source identity",
            Box::new(|negative: &mut serde_json::Value| {
                negative["measured"]["baseline_source_blake3"] = json!("0".repeat(64));
            }),
        ),
        (
            "missing evidence property ID",
            Box::new(|negative: &mut serde_json::Value| {
                negative["evidence"]["proof_or_property_id"] = json!("missing#rail-0");
            }),
        ),
        (
            "source locator",
            Box::new(|negative: &mut serde_json::Value| {
                negative["mutant_source"] = json!("src/other.rs");
            }),
        ),
        (
            "unsupported record schema",
            Box::new(|negative: &mut serde_json::Value| {
                negative["schema_version"] = json!("octet-proof-negative-check/v9");
            }),
        ),
    ] {
        let fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
        record_fixture(&fixture).expect("record negative check");
        assert!(validate_recorded_deck(&fixture).is_empty(), "{label}: initial record should validate");
        mutate_recorded_negative(&fixture, mutate);
        let diagnostics = validate_recorded_deck(&fixture);
        assert!(!diagnostics.is_empty(), "{label} change should fail validation");
    }
}

#[test]
fn missing_mutant_card_is_inconclusive_and_does_not_overwrite_mutant_deck() {
    let mut fixture = negative_fixture(CARD_STATUS_PASSED, CARD_STATUS_FAILED);
    fs::write(&fixture.mutant_deck, "{\n  \"schema_version\": \"octet-proof-cards/v1\",\n  \"cards\": []\n}\n")
        .expect("empty mutant deck");
    fixture.evidence["mutant"]["deck_blake3"] =
        json!(blake3::hash(&fs::read(&fixture.mutant_deck).expect("deck bytes")).to_hex().to_string());

    let code = record_fixture(&fixture).expect("record negative check");
    let negative = recorded_negative_check(&fixture);

    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_INCONCLUSIVE);
    assert_eq!(negative.mutant_card_status, "absent");
    assert!(
        negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("exactly one card")),
        "unexpected diagnostics: {:?}",
        negative.diagnostics
    );

    let mut aliasing = fixture;
    aliasing.args.output = Some(aliasing.mutant_deck.clone());
    let error = record_fixture(&aliasing).expect_err("aliasing output");
    assert!(error.contains("must not overwrite"));
}

#[test]
fn fixture_crate_allowance_is_line_42_and_maps_the_existing_rail() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("repository root");
    let fixture = repository.join("fixtures/proof-candidates");
    let report =
        build_report(&candidate_args(fixture.join("src"), None, Vec::new(), Some(fixture.join("trust-boundary.md"))))
            .expect("fixture report");

    assert_eq!(report.candidates.len(), 1);
    let candidate = &report.candidates[0];
    assert_eq!(candidate.lint, "unchecked_narrowing");
    assert_eq!(candidate.line, 42);
    assert_eq!(candidate.rail.as_ref().expect("mapped rail").proof_or_property_id, PROPERTY_ID);
    assert_eq!(candidate.proof_ref.status, STATUS_MISSING);
}
