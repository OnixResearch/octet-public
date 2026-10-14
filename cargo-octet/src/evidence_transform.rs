use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::evidence::CHECKER_CARGO_OCTET;
use crate::evidence::EvidenceTier;
use crate::evidence::PROVENANCE_SCHEMA_VERSION;
use crate::evidence::ProvenanceRecord;
use crate::evidence::artifact_blake3_hex;

pub const EVIDENCE_TRANSFORM_INPUT_SCHEMA_VERSION: &str = "octet-evidence-transform-input/v1";
pub const EVIDENCE_TRANSFORM_REPORT_SCHEMA_VERSION: &str = "octet-evidence-transform-report/v1";
pub const EVIDENCE_TRANSFORM_CONTRACT_PATH: &str =
    "standards/evidence-transforms/contracts/bounded-evidence-transform.generated.ncl";

const CLAIM_BOUNDARY_TRANSFORM_ID: &str = "octet.transform.claim-boundary-normalize.v1";
const CLAIM_BOUNDARY_FAMILY: &str = "claim_boundary";
const CLAIM_BOUNDARY_INPUT_CONTRACT: &str = "octet-claim-boundary-text/v1";
const CLAIM_BOUNDARY_RULE_SET_ID: &str = "octet.rewrite.claim-boundary.v1";
const RULE_TRIM_WHITESPACE: &str = "trim-whitespace";
const RULE_COLLAPSE_ASCII_WHITESPACE: &str = "collapse-ascii-whitespace";
const RULE_SORT_NON_CLAIMS: &str = "sort-non-claims";
const SOURCE_ID_CLAIM_BOUNDARY_TRANSFORM: &str = "octet.transform.claim-boundary";
const FIELD_SCHEMA_VERSION: &str = "schema_version";
const FIELD_TRANSFORM_ID: &str = "transform_id";
const FIELD_TRANSFORM_FAMILY: &str = "transform_family";
const FIELD_INPUT_CONTRACT: &str = "input_contract";
const FIELD_OUTPUT_CONTRACT: &str = "output_contract";
const FIELD_RULE_SET_ID: &str = "rule_set_id";
const FIELD_REQUESTED_RULES: &str = "requested_rules";
const FIELD_BUDGET: &str = "budget";
const FIELD_INPUT: &str = "input";
const FIELD_PARENT_ARTIFACTS: &str = "parent_artifacts";
const FIELD_EXPECTED_PARENT_ARTIFACTS: &str = "expected_parent_artifacts";
const FIELD_CHECKED_SCOPE: &str = "checked_scope";
const FIELD_NON_CLAIMS: &str = "non_claims";
const FIELD_OUTPUT_ARTIFACT_LOCATOR: &str = "output_artifact_locator";
const NON_CLAIM_UNBOUNDED_TACTIC: &str = "not unbounded tactics";
const NON_CLAIM_COMPLETE_PROOF_SEARCH: &str = "not complete proof search";
const NON_CLAIM_SOLVER_COMPLETENESS: &str = "not solver completeness";
const NON_CLAIM_WHOLE_PROGRAM: &str = "not whole-program correctness";
const NON_CLAIM_CERTIFICATION: &str = "not certification";
const NON_CLAIM_BUSINESS_DOMAIN: &str = "not business-domain validation";
const NON_CLAIM_SEMANTIC_EQUIVALENCE: &str = "not semantic equivalence";
const NON_CLAIM_FULL_SAFETY: &str = "not full safety";
const NON_CLAIM_VERIFIED_RUNTIME: &str = "not verified runtime";
const OVERCLAIM_WHOLE_PROGRAM: &str = "whole-program correctness";
const OVERCLAIM_CERTIFICATION: &str = "certification";
const OVERCLAIM_BUSINESS_DOMAIN: &str = "business-domain validation";
const OVERCLAIM_SEMANTIC_EQUIVALENCE: &str = "semantic equivalence";
const OVERCLAIM_FULL_SAFETY: &str = "full safety";
const OVERCLAIM_VERIFIED_RUNTIME: &str = "verified runtime";
const OVERCLAIM_UNBOUNDED_TACTIC: &str = "unbounded tactic";
const OVERCLAIM_COMPLETE_PROOF_SEARCH: &str = "complete proof search";
const OVERCLAIM_SOLVER_COMPLETENESS: &str = "solver completeness";
const OUTPUT_IDENTITY_PAYLOAD_VERSION: &str = "octet-evidence-transform-output-identity/v1";
const PAYLOAD_FIELD_SEPARATOR: char = '\n';
const PAYLOAD_VALUE_SEPARATOR: char = '=';
const NORMALIZED_WORD_SEPARATOR: &str = " ";
const CONTRACT_INDENT: &str = "  ";
const HUMAN_ACTION_TRANSFORM_REVIEW: &str =
    "review bounded transform inputs, rule-set identity, parent artifacts, and scoped non-claims before release";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TransformBudget {
    pub max_input_bytes: usize,
    pub max_rules: usize,
    pub max_parent_artifacts: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TransformTextArtifact {
    pub locator: String,
    pub blake3: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TransformArtifactIdentity {
    pub locator: String,
    pub blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceTransformInput {
    pub schema_version: String,
    pub transform_id: String,
    pub transform_family: String,
    pub input_contract: String,
    pub output_contract: String,
    pub rule_set_id: String,
    pub requested_rules: Vec<String>,
    pub budget: TransformBudget,
    pub input: TransformTextArtifact,
    pub expected_parent_artifacts: Vec<TransformArtifactIdentity>,
    pub parent_artifacts: Vec<TransformArtifactIdentity>,
    pub output_artifact_locator: String,
    pub checked_scope: String,
    pub evidence_tier: EvidenceTier,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceTransformReport {
    pub schema_version: String,
    pub valid: bool,
    pub transform_id: String,
    pub transform_family: String,
    pub input_contract: String,
    pub output_contract: String,
    pub rule_set_id: String,
    pub applied_rules: Vec<String>,
    pub input_identities: Vec<TransformArtifactIdentity>,
    pub parent_artifacts: Vec<TransformArtifactIdentity>,
    pub output_artifact_locator: String,
    pub output_identity: String,
    pub checked_scope: String,
    pub evidence_tier: EvidenceTier,
    pub normalized_text: String,
    pub non_claims: Vec<String>,
    pub diagnostics: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ProvenanceRecord>,
}

#[must_use]
pub fn run_bounded_evidence_transform(input: &EvidenceTransformInput) -> EvidenceTransformReport {
    let mut diagnostics = transform_shape_diagnostics(input);
    diagnostics.extend(transform_budget_diagnostics(input));
    diagnostics.extend(transform_identity_diagnostics(input));
    diagnostics.extend(transform_parent_diagnostics(input));
    diagnostics.extend(transform_boundary_diagnostics(&input.input.text, &input.non_claims));
    diagnostics.sort();
    diagnostics.dedup();

    let can_run = diagnostics.is_empty();
    let normalized_text = if can_run {
        apply_rules(&input.input.text, &input.requested_rules)
    } else {
        String::new()
    };
    let non_claims = if can_run {
        normalize_non_claims(input.non_claims.clone(), &input.requested_rules)
    } else {
        input.non_claims.clone()
    };
    let input_identities = vec![TransformArtifactIdentity {
        locator: input.input.locator.clone(),
        blake3: input.input.blake3.clone(),
    }];
    let output_identity = if can_run {
        output_identity(input, &input_identities, &input.parent_artifacts, &normalized_text, &non_claims)
    } else {
        String::new()
    };
    let provenance = if can_run {
        Some(transform_provenance_record(input, &output_identity))
    } else {
        None
    };

    EvidenceTransformReport {
        schema_version: EVIDENCE_TRANSFORM_REPORT_SCHEMA_VERSION.to_owned(),
        valid: diagnostics.is_empty(),
        transform_id: input.transform_id.clone(),
        transform_family: input.transform_family.clone(),
        input_contract: input.input_contract.clone(),
        output_contract: input.output_contract.clone(),
        rule_set_id: input.rule_set_id.clone(),
        applied_rules: if can_run {
            input.requested_rules.clone()
        } else {
            Vec::new()
        },
        input_identities,
        parent_artifacts: input.parent_artifacts.clone(),
        output_artifact_locator: input.output_artifact_locator.clone(),
        output_identity,
        checked_scope: input.checked_scope.clone(),
        evidence_tier: input.evidence_tier,
        normalized_text,
        non_claims,
        diagnostics,
        provenance,
    }
}

#[must_use]
pub fn validate_evidence_transform_report(report: &EvidenceTransformReport) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_mismatch(
        &mut diagnostics,
        FIELD_SCHEMA_VERSION,
        EVIDENCE_TRANSFORM_REPORT_SCHEMA_VERSION,
        &report.schema_version,
    );
    push_blank(&mut diagnostics, FIELD_TRANSFORM_ID, &report.transform_id);
    push_blank(&mut diagnostics, FIELD_TRANSFORM_FAMILY, &report.transform_family);
    push_blank(&mut diagnostics, FIELD_INPUT_CONTRACT, &report.input_contract);
    push_blank(&mut diagnostics, FIELD_OUTPUT_CONTRACT, &report.output_contract);
    push_blank(&mut diagnostics, FIELD_RULE_SET_ID, &report.rule_set_id);
    push_blank(&mut diagnostics, FIELD_CHECKED_SCOPE, &report.checked_scope);
    push_blank(&mut diagnostics, FIELD_OUTPUT_ARTIFACT_LOCATOR, &report.output_artifact_locator);
    if report.valid && report.output_identity.trim().is_empty() {
        diagnostics.push("valid transform report missing output_identity".to_owned());
    }
    if report.valid && report.provenance.is_none() {
        diagnostics.push("valid transform report missing provenance".to_owned());
    }
    if report.valid && report.evidence_tier == EvidenceTier::FormalProof {
        diagnostics.push("bounded transform report cannot emit formal_proof evidence tier".to_owned());
    }
    diagnostics.extend(required_non_claim_diagnostics(&report.non_claims));
    diagnostics.extend(transform_boundary_diagnostics(&report.normalized_text, &report.non_claims));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

#[must_use]
pub fn render_evidence_transform_contract() -> String {
    let mut rendered = String::new();
    rendered.push_str("# @generated by cargo-octet/src/evidence_transform.rs; do not edit by hand.\n");
    rendered.push_str(
        "let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in\n",
    );
    rendered.push_str("let SchemaVersion = std.contract.from_predicate (fun value => value == \"");
    rendered.push_str(EVIDENCE_TRANSFORM_REPORT_SCHEMA_VERSION);
    rendered.push_str("\") in\n");
    rendered.push_str(
        "let EvidenceTier = std.contract.from_predicate (fun value => std.is_string value && std.array.elem value [\n",
    );
    for tier in supported_tier_strings() {
        rendered.push_str(CONTRACT_INDENT);
        rendered.push('"');
        rendered.push_str(tier);
        rendered.push_str("\",\n");
    }
    rendered.push_str("]) in\n");
    rendered.push_str("let StringArray = Array String in\n");
    rendered.push_str("let ArtifactIdentity = { locator | NonEmptyString, blake3 | NonEmptyString } in\n");
    rendered.push_str(
        "let ProvenanceRecord = { schema_version | NonEmptyString, source_id | NonEmptyString, checker_id | NonEmptyString, evidence_tier | EvidenceTier, artifact_locator | NonEmptyString, blake3 | NonEmptyString, checked_scope | NonEmptyString, parent_artifacts | StringArray | default = [], human_action | String | default = \"\", verifier_identity | String | default = \"\", proof_artifact_identity | String | default = \"\" } in\n",
    );
    rendered.push_str("{\n");
    rendered.push_str("  EvidenceTransformReportV1 = {\n");
    rendered.push_str("    schema_version | SchemaVersion,\n");
    rendered.push_str("    valid | Bool,\n");
    rendered.push_str("    transform_id | NonEmptyString,\n");
    rendered.push_str("    transform_family | NonEmptyString,\n");
    rendered.push_str("    input_contract | NonEmptyString,\n");
    rendered.push_str("    output_contract | NonEmptyString,\n");
    rendered.push_str("    rule_set_id | NonEmptyString,\n");
    rendered.push_str("    applied_rules | StringArray,\n");
    rendered.push_str("    input_identities | Array ArtifactIdentity,\n");
    rendered.push_str("    parent_artifacts | Array ArtifactIdentity,\n");
    rendered.push_str("    output_artifact_locator | NonEmptyString,\n");
    rendered.push_str("    output_identity | String,\n");
    rendered.push_str("    checked_scope | NonEmptyString,\n");
    rendered.push_str("    evidence_tier | EvidenceTier,\n");
    rendered.push_str("    normalized_text | String,\n");
    rendered.push_str("    non_claims | StringArray,\n");
    rendered.push_str("    diagnostics | StringArray,\n");
    rendered.push_str("    provenance | ProvenanceRecord,\n");
    rendered.push_str("  },\n");
    rendered.push_str("}\n");
    rendered
}

fn transform_shape_diagnostics(input: &EvidenceTransformInput) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_mismatch(
        &mut diagnostics,
        FIELD_SCHEMA_VERSION,
        EVIDENCE_TRANSFORM_INPUT_SCHEMA_VERSION,
        &input.schema_version,
    );
    push_blank(&mut diagnostics, FIELD_TRANSFORM_ID, &input.transform_id);
    push_blank(&mut diagnostics, FIELD_TRANSFORM_FAMILY, &input.transform_family);
    push_blank(&mut diagnostics, FIELD_INPUT_CONTRACT, &input.input_contract);
    push_blank(&mut diagnostics, FIELD_OUTPUT_CONTRACT, &input.output_contract);
    push_blank(&mut diagnostics, FIELD_RULE_SET_ID, &input.rule_set_id);
    push_blank(&mut diagnostics, FIELD_CHECKED_SCOPE, &input.checked_scope);
    push_blank(&mut diagnostics, FIELD_OUTPUT_ARTIFACT_LOCATOR, &input.output_artifact_locator);
    push_blank(&mut diagnostics, "input.locator", &input.input.locator);
    push_blank(&mut diagnostics, "input.blake3", &input.input.blake3);
    if input.requested_rules.is_empty() {
        diagnostics.push(missing_field(FIELD_REQUESTED_RULES));
    }
    if input.parent_artifacts.is_empty() {
        diagnostics.push(missing_field(FIELD_PARENT_ARTIFACTS));
    }
    if input.expected_parent_artifacts.is_empty() {
        diagnostics.push(missing_field(FIELD_EXPECTED_PARENT_ARTIFACTS));
    }
    if input.input.blake3 != artifact_blake3_hex(input.input.text.as_bytes()) {
        diagnostics.push(format!(
            "stale input artifact `{}`: expected blake3 {}, got {}",
            input.input.locator,
            artifact_blake3_hex(input.input.text.as_bytes()),
            input.input.blake3
        ));
    }
    diagnostics.extend(required_non_claim_diagnostics(&input.non_claims));
    diagnostics
}

fn transform_budget_diagnostics(input: &EvidenceTransformInput) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if input.budget.max_input_bytes == 0 {
        diagnostics.push(format!("transform {FIELD_BUDGET}.max_input_bytes must be positive"));
    }
    if input.budget.max_rules == 0 {
        diagnostics.push(format!("transform {FIELD_BUDGET}.max_rules must be positive"));
    }
    if input.budget.max_parent_artifacts == 0 {
        diagnostics.push(format!("transform {FIELD_BUDGET}.max_parent_artifacts must be positive"));
    }
    let input_bytes = input.input.text.len();
    if input_bytes > input.budget.max_input_bytes {
        diagnostics.push(format!(
            "transform budget exhausted for {FIELD_INPUT}: max_input_bytes={}, actual_bytes={input_bytes}",
            input.budget.max_input_bytes
        ));
    }
    let requested_rules = input.requested_rules.len();
    if requested_rules > input.budget.max_rules {
        diagnostics.push(format!(
            "transform budget exhausted for {FIELD_REQUESTED_RULES}: max_rules={}, actual_rules={requested_rules}",
            input.budget.max_rules
        ));
    }
    let parent_count = input.parent_artifacts.len();
    if parent_count > input.budget.max_parent_artifacts {
        diagnostics.push(format!(
            "transform budget exhausted for {FIELD_PARENT_ARTIFACTS}: max_parent_artifacts={}, actual_parent_artifacts={parent_count}",
            input.budget.max_parent_artifacts
        ));
    }
    diagnostics
}

fn transform_identity_diagnostics(input: &EvidenceTransformInput) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_mismatch(&mut diagnostics, FIELD_TRANSFORM_ID, CLAIM_BOUNDARY_TRANSFORM_ID, &input.transform_id);
    push_mismatch(&mut diagnostics, FIELD_TRANSFORM_FAMILY, CLAIM_BOUNDARY_FAMILY, &input.transform_family);
    push_mismatch(&mut diagnostics, FIELD_INPUT_CONTRACT, CLAIM_BOUNDARY_INPUT_CONTRACT, &input.input_contract);
    push_mismatch(
        &mut diagnostics,
        FIELD_OUTPUT_CONTRACT,
        EVIDENCE_TRANSFORM_REPORT_SCHEMA_VERSION,
        &input.output_contract,
    );
    push_mismatch(&mut diagnostics, FIELD_RULE_SET_ID, CLAIM_BOUNDARY_RULE_SET_ID, &input.rule_set_id);
    if input.evidence_tier == EvidenceTier::FormalProof {
        diagnostics.push("bounded transform cannot promote output to formal_proof evidence tier".to_owned());
    }
    diagnostics.extend(rule_diagnostics(&input.requested_rules));
    diagnostics
}

fn transform_parent_diagnostics(input: &EvidenceTransformInput) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let actual = artifact_map(&input.parent_artifacts, FIELD_PARENT_ARTIFACTS, &mut diagnostics);
    let expected = artifact_map(&input.expected_parent_artifacts, FIELD_EXPECTED_PARENT_ARTIFACTS, &mut diagnostics);
    for (locator, expected_blake3) in expected {
        match actual.get(&locator) {
            Some(actual_blake3) if actual_blake3 == &expected_blake3 => {}
            Some(actual_blake3) => diagnostics.push(format!(
                "stale parent artifact `{locator}`: expected blake3 {expected_blake3}, got {actual_blake3}"
            )),
            None => diagnostics.push(format!("missing expected parent artifact `{locator}`")),
        }
    }
    diagnostics
}

fn artifact_map(
    artifacts: &[TransformArtifactIdentity],
    field: &str,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for artifact in artifacts {
        push_blank(diagnostics, &format!("{field}.locator"), &artifact.locator);
        push_blank(diagnostics, &format!("{field}.blake3"), &artifact.blake3);
        if let Some(existing) = map.insert(artifact.locator.clone(), artifact.blake3.clone()) {
            diagnostics.push(format!(
                "duplicate {field} locator `{}` with blake3 values {existing} and {}",
                artifact.locator, artifact.blake3
            ));
        }
    }
    map
}

fn rule_diagnostics(requested_rules: &[String]) -> Vec<String> {
    let supported_rules = supported_rule_set();
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for rule in requested_rules {
        if rule.trim().is_empty() {
            diagnostics.push(format!("missing required transform field `{FIELD_REQUESTED_RULES}` entry"));
            continue;
        }
        if !seen.insert(rule.clone()) {
            diagnostics.push(format!("duplicate transform rule `{rule}`"));
        }
        if rule.contains("global") || rule.contains("hidden") || rule.contains("simp") {
            diagnostics.push(format!(
                "hidden or global transform rule `{rule}` is not declared by `{CLAIM_BOUNDARY_RULE_SET_ID}`"
            ));
            continue;
        }
        if !supported_rules.contains(rule.as_str()) {
            diagnostics
                .push(format!("unsupported transform rule `{rule}` for rule_set_id `{CLAIM_BOUNDARY_RULE_SET_ID}`"));
        }
    }
    diagnostics
}

fn supported_rule_set() -> BTreeSet<&'static str> {
    [
        RULE_TRIM_WHITESPACE,
        RULE_COLLAPSE_ASCII_WHITESPACE,
        RULE_SORT_NON_CLAIMS,
    ]
    .into_iter()
    .collect()
}

fn supported_tier_strings() -> [&'static str; SUPPORTED_TIER_COUNT] {
    [
        EvidenceTier::StaticLint.as_str(),
        EvidenceTier::PolicyCheck.as_str(),
        EvidenceTier::UiFixture.as_str(),
        EvidenceTier::AutofixVerified.as_str(),
        EvidenceTier::TranscriptAssertion.as_str(),
        EvidenceTier::FormalProof.as_str(),
        EvidenceTier::ManualReviewRequired.as_str(),
    ]
}

const SUPPORTED_TIER_COUNT: usize = 7;

fn required_non_claim_diagnostics(non_claims: &[String]) -> Vec<String> {
    let normalized = non_claims.iter().map(|claim| claim.to_ascii_lowercase()).collect::<BTreeSet<_>>();
    required_non_claims()
        .iter()
        .filter(|required| !normalized.contains(**required))
        .map(|required| format!("missing transform {FIELD_NON_CLAIMS} entry `{required}`"))
        .collect()
}

fn required_non_claims() -> [&'static str; REQUIRED_NON_CLAIM_COUNT] {
    [
        NON_CLAIM_UNBOUNDED_TACTIC,
        NON_CLAIM_COMPLETE_PROOF_SEARCH,
        NON_CLAIM_SOLVER_COMPLETENESS,
        NON_CLAIM_WHOLE_PROGRAM,
        NON_CLAIM_CERTIFICATION,
        NON_CLAIM_BUSINESS_DOMAIN,
        NON_CLAIM_SEMANTIC_EQUIVALENCE,
        NON_CLAIM_FULL_SAFETY,
        NON_CLAIM_VERIFIED_RUNTIME,
    ]
}

const REQUIRED_NON_CLAIM_COUNT: usize = 9;

fn transform_boundary_diagnostics(wording: &str, non_claims: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let lower_wording = wording.to_ascii_lowercase();
    for (phrase, label) in overclaim_patterns() {
        if positive_phrase_present(&lower_wording, phrase) {
            diagnostics.push(format!("bounded transform wording overclaims `{label}`"));
        }
    }
    diagnostics.extend(required_non_claim_diagnostics(non_claims));
    diagnostics
}

fn overclaim_patterns() -> [(&'static str, &'static str); OVERCLAIM_PATTERN_COUNT] {
    [
        (OVERCLAIM_UNBOUNDED_TACTIC, "unbounded tactics"),
        (OVERCLAIM_COMPLETE_PROOF_SEARCH, "complete proof search"),
        (OVERCLAIM_SOLVER_COMPLETENESS, "solver completeness"),
        (OVERCLAIM_WHOLE_PROGRAM, "whole-program correctness"),
        (OVERCLAIM_CERTIFICATION, "certification"),
        (OVERCLAIM_BUSINESS_DOMAIN, "business-domain validation"),
        (OVERCLAIM_SEMANTIC_EQUIVALENCE, "semantic equivalence"),
        (OVERCLAIM_FULL_SAFETY, "full safety"),
        (OVERCLAIM_VERIFIED_RUNTIME, "verified runtime"),
    ]
}

const OVERCLAIM_PATTERN_COUNT: usize = 9;

fn positive_phrase_present(lower_wording: &str, phrase: &str) -> bool {
    lower_wording.contains(phrase)
        && !lower_wording.contains(&format!("not {phrase}"))
        && !lower_wording.contains(&format!("no {phrase}"))
        && !lower_wording.contains(&format!("does not {phrase}"))
}

fn apply_rules(text: &str, requested_rules: &[String]) -> String {
    let mut current = text.to_owned();
    for rule in requested_rules {
        match rule.as_str() {
            RULE_TRIM_WHITESPACE => {
                current = current.lines().map(str::trim).collect::<Vec<_>>().join("\n");
            }
            RULE_COLLAPSE_ASCII_WHITESPACE => {
                current = current.split_whitespace().collect::<Vec<_>>().join(NORMALIZED_WORD_SEPARATOR);
            }
            RULE_SORT_NON_CLAIMS => {}
            _ => unreachable!("rule diagnostics prevent unsupported rule execution"),
        }
    }
    current
}

fn normalize_non_claims(mut non_claims: Vec<String>, requested_rules: &[String]) -> Vec<String> {
    if requested_rules.iter().any(|rule| rule == RULE_SORT_NON_CLAIMS) {
        non_claims.sort();
        non_claims.dedup();
    }
    non_claims
}

fn output_identity(
    input: &EvidenceTransformInput,
    input_identities: &[TransformArtifactIdentity],
    parent_artifacts: &[TransformArtifactIdentity],
    normalized_text: &str,
    non_claims: &[String],
) -> String {
    let mut payload = vec![
        payload_line("payload_version", OUTPUT_IDENTITY_PAYLOAD_VERSION),
        payload_line(FIELD_TRANSFORM_ID, &input.transform_id),
        payload_line(FIELD_TRANSFORM_FAMILY, &input.transform_family),
        payload_line(FIELD_INPUT_CONTRACT, &input.input_contract),
        payload_line(FIELD_OUTPUT_CONTRACT, &input.output_contract),
        payload_line(FIELD_RULE_SET_ID, &input.rule_set_id),
        payload_line(FIELD_CHECKED_SCOPE, &input.checked_scope),
        payload_line("evidence_tier", input.evidence_tier.as_str()),
        payload_line("normalized_text", normalized_text),
    ];
    for rule in &input.requested_rules {
        payload.push(payload_line("rule", rule));
    }
    for identity in sorted_identities(input_identities) {
        payload.push(payload_line("input_identity", &format!("{}:{}", identity.locator, identity.blake3)));
    }
    for identity in sorted_identities(parent_artifacts) {
        payload.push(payload_line("parent_artifact", &format!("{}:{}", identity.locator, identity.blake3)));
    }
    for non_claim in non_claims {
        payload.push(payload_line("non_claim", non_claim));
    }
    let rendered = payload.join(&PAYLOAD_FIELD_SEPARATOR.to_string());
    artifact_blake3_hex(rendered.as_bytes())
}

fn payload_line(field: &str, value: &str) -> String {
    format!("{field}{PAYLOAD_VALUE_SEPARATOR}{value}")
}

fn sorted_identities(identities: &[TransformArtifactIdentity]) -> Vec<TransformArtifactIdentity> {
    let mut sorted = identities.to_vec();
    sorted.sort_by(|left, right| left.locator.cmp(&right.locator).then_with(|| left.blake3.cmp(&right.blake3)));
    sorted
}

fn transform_provenance_record(input: &EvidenceTransformInput, output_identity: &str) -> ProvenanceRecord {
    ProvenanceRecord {
        schema_version: PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: SOURCE_ID_CLAIM_BOUNDARY_TRANSFORM.to_owned(),
        checker_id: CHECKER_CARGO_OCTET.to_owned(),
        evidence_tier: input.evidence_tier,
        artifact_locator: input.output_artifact_locator.clone(),
        blake3: output_identity.to_owned(),
        checked_scope: input.checked_scope.clone(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: (input.evidence_tier == EvidenceTier::ManualReviewRequired)
            .then(|| HUMAN_ACTION_TRANSFORM_REVIEW.to_owned()),
        parent_artifacts: input.parent_artifacts.iter().map(|parent| parent.locator.clone()).collect(),
    }
}

fn push_mismatch(diagnostics: &mut Vec<String>, field: &str, expected: &str, actual: &str) {
    if actual.trim().is_empty() {
        diagnostics.push(missing_field(field));
    } else if actual != expected {
        diagnostics.push(format!("unsupported transform field `{field}`: expected `{expected}`, got `{actual}`"));
    }
}

fn push_blank(diagnostics: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(missing_field(field));
    }
}

fn missing_field(field: &str) -> String {
    format!("missing required transform field `{field}`")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::*;
    use crate::evidence::validate_provenance_chain;

    const TEST_INPUT_LOCATOR: &str = "claims/proof-ref-boundary.md";
    const TEST_PARENT_LOCATOR: &str = "proof-cards.json";
    const TEST_OUTPUT_LOCATOR: &str = "generated/claim-boundary-transform.json";
    const TEST_PARENT_BYTES: &[u8] = b"{\"schema_version\":\"octet-proof-cards/v1\"}\n";
    const TEST_INPUT_TEXT: &str =
        "  Scoped proof-card linkage only; not whole-program correctness and not verified runtime.  ";
    const TEST_CHECKED_SCOPE: &str = "docs/trust-boundary-evidence.md#verus-proof-path-lane";
    const TEST_MAX_INPUT_BYTES: usize = 512;
    const TEST_MAX_RULES: usize = 3;
    const TEST_MAX_PARENT_ARTIFACTS: usize = 2;
    const TOO_SMALL_INPUT_BUDGET: usize = 8;
    const TOO_SMALL_RULE_BUDGET: usize = 1;
    const CONTRACT_READ_ERROR: &str = "read generated transform contract";
    const DOCS_TEXT: &str = include_str!("../../docs/bounded-evidence-transforms.md");

    #[test]
    fn bounded_transform_valid_input_is_deterministic_and_records_provenance() {
        let input = valid_input();

        let first = run_bounded_evidence_transform(&input);
        let second = run_bounded_evidence_transform(&input);

        assert_eq!(first, second);
        assert!(first.valid, "unexpected diagnostics: {:?}", first.diagnostics);
        assert_eq!(first.transform_id, CLAIM_BOUNDARY_TRANSFORM_ID);
        assert_eq!(first.rule_set_id, CLAIM_BOUNDARY_RULE_SET_ID);
        assert_eq!(first.applied_rules, requested_rules());
        assert_eq!(
            first.normalized_text,
            "Scoped proof-card linkage only; not whole-program correctness and not verified runtime."
        );
        assert!(!first.output_identity.is_empty());
        assert!(validate_evidence_transform_report(&first).is_empty());

        let first_json = serde_json::to_string(&first).expect("serialize first transform report");
        let second_json = serde_json::to_string(&second).expect("serialize second transform report");
        assert_eq!(first_json, second_json);

        let parent = parent_provenance_record();
        let derived = first.provenance.clone().expect("valid report should carry provenance");
        let expected_hashes = BTreeMap::from([
            (parent.artifact_locator.clone(), parent.blake3.clone()),
            (derived.artifact_locator.clone(), derived.blake3.clone()),
        ]);
        let diagnostics = validate_provenance_chain(&[parent, derived], &expected_hashes);
        assert!(diagnostics.is_empty(), "unexpected provenance diagnostics: {diagnostics:?}");
    }

    #[test]
    fn transform_rejects_budget_exhaustion_stale_parent_formal_tier_and_overclaim() {
        let mut input = valid_input();
        input.evidence_tier = EvidenceTier::FormalProof;
        input.input.text = "This transform proves whole-program correctness with complete proof search.".to_owned();
        input.input.blake3 = artifact_blake3_hex(input.input.text.as_bytes());
        input.budget.max_input_bytes = TOO_SMALL_INPUT_BUDGET;
        input.budget.max_rules = TOO_SMALL_RULE_BUDGET;
        input.parent_artifacts = vec![TransformArtifactIdentity {
            locator: TEST_PARENT_LOCATOR.to_owned(),
            blake3: "stale-parent-hash".to_owned(),
        }];

        let report = run_bounded_evidence_transform(&input);

        assert!(!report.valid);
        assert!(report.output_identity.is_empty());
        assert!(report.provenance.is_none());
        assert_contains(&report.diagnostics, "bounded transform cannot promote output to formal_proof evidence tier");
        assert_contains(&report.diagnostics, "transform budget exhausted for input");
        assert_contains(&report.diagnostics, "transform budget exhausted for requested_rules");
        assert_contains(&report.diagnostics, "stale parent artifact");
        assert_contains(&report.diagnostics, "bounded transform wording overclaims `whole-program correctness`");
        assert_contains(&report.diagnostics, "bounded transform wording overclaims `complete proof search`");
    }

    #[test]
    fn transform_rejects_unknown_hidden_rules_and_missing_non_claims() {
        let mut input = valid_input();
        input.requested_rules = vec!["global-simp".to_owned(), "surprise-rule".to_owned()];
        input.non_claims = vec![NON_CLAIM_WHOLE_PROGRAM.to_owned()];

        let report = run_bounded_evidence_transform(&input);

        assert!(!report.valid);
        assert_contains(&report.diagnostics, "hidden or global transform rule `global-simp`");
        assert_contains(&report.diagnostics, "unsupported transform rule `surprise-rule`");
        assert_contains(&report.diagnostics, "missing transform non_claims entry `not semantic equivalence`");
    }

    #[test]
    fn transform_report_validation_rejects_missing_identity_and_positive_boundary_text() {
        let mut report = run_bounded_evidence_transform(&valid_input());
        report.output_identity.clear();
        report.normalized_text = "The transform guarantees verified runtime and full safety.".to_owned();

        let diagnostics = validate_evidence_transform_report(&report);

        assert_contains(&diagnostics, "valid transform report missing output_identity");
        assert_contains(&diagnostics, "bounded transform wording overclaims `verified runtime`");
        assert_contains(&diagnostics, "bounded transform wording overclaims `full safety`");
    }

    #[test]
    fn evidence_transform_contract_is_fresh() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("repo root");
        let path = repo_root.join(EVIDENCE_TRANSFORM_CONTRACT_PATH);
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{CONTRACT_READ_ERROR} {}: {error}", path.display()));

        assert_eq!(expected, render_evidence_transform_contract());
    }

    #[test]
    fn bounded_transform_docs_name_boundaries_and_replay_surface() {
        for required in [
            CLAIM_BOUNDARY_TRANSFORM_ID,
            CLAIM_BOUNDARY_RULE_SET_ID,
            "budget exhaustion fails closed",
            "not unbounded tactics",
            "not solver completeness",
            "not semantic equivalence",
            "not verified runtime",
        ] {
            assert!(DOCS_TEXT.contains(required), "missing docs phrase: {required}");
        }
    }

    fn valid_input() -> EvidenceTransformInput {
        EvidenceTransformInput {
            schema_version: EVIDENCE_TRANSFORM_INPUT_SCHEMA_VERSION.to_owned(),
            transform_id: CLAIM_BOUNDARY_TRANSFORM_ID.to_owned(),
            transform_family: CLAIM_BOUNDARY_FAMILY.to_owned(),
            input_contract: CLAIM_BOUNDARY_INPUT_CONTRACT.to_owned(),
            output_contract: EVIDENCE_TRANSFORM_REPORT_SCHEMA_VERSION.to_owned(),
            rule_set_id: CLAIM_BOUNDARY_RULE_SET_ID.to_owned(),
            requested_rules: requested_rules(),
            budget: TransformBudget {
                max_input_bytes: TEST_MAX_INPUT_BYTES,
                max_rules: TEST_MAX_RULES,
                max_parent_artifacts: TEST_MAX_PARENT_ARTIFACTS,
            },
            input: TransformTextArtifact {
                locator: TEST_INPUT_LOCATOR.to_owned(),
                blake3: artifact_blake3_hex(TEST_INPUT_TEXT.as_bytes()),
                text: TEST_INPUT_TEXT.to_owned(),
            },
            expected_parent_artifacts: vec![parent_identity()],
            parent_artifacts: vec![parent_identity()],
            output_artifact_locator: TEST_OUTPUT_LOCATOR.to_owned(),
            checked_scope: TEST_CHECKED_SCOPE.to_owned(),
            evidence_tier: EvidenceTier::PolicyCheck,
            non_claims: required_non_claims().iter().map(|claim| (*claim).to_owned()).collect(),
        }
    }

    fn requested_rules() -> Vec<String> {
        vec![
            RULE_TRIM_WHITESPACE.to_owned(),
            RULE_COLLAPSE_ASCII_WHITESPACE.to_owned(),
            RULE_SORT_NON_CLAIMS.to_owned(),
        ]
    }

    fn parent_identity() -> TransformArtifactIdentity {
        TransformArtifactIdentity {
            locator: TEST_PARENT_LOCATOR.to_owned(),
            blake3: artifact_blake3_hex(TEST_PARENT_BYTES),
        }
    }

    fn parent_provenance_record() -> ProvenanceRecord {
        let parent = parent_identity();
        ProvenanceRecord {
            schema_version: PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: "octet.runner.transform-input".to_owned(),
            checker_id: CHECKER_CARGO_OCTET.to_owned(),
            evidence_tier: EvidenceTier::StaticLint,
            artifact_locator: parent.locator,
            blake3: parent.blake3,
            checked_scope: TEST_CHECKED_SCOPE.to_owned(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: None,
            parent_artifacts: Vec::new(),
        }
    }

    fn assert_contains(diagnostics: &[String], expected: &str) {
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
            "expected diagnostics to contain `{expected}`, got {diagnostics:?}"
        );
    }
}
