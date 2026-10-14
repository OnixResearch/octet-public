use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::evidence::EvidenceTier;

pub const PROOF_GOAL_STATE_SCHEMA_VERSION: &str = "octet-proof-goal-state/v1";

const STATUS_PASSED: &str = "passed";
const FIELD_GOAL_ID: &str = "goal_id";
const FIELD_CLAIM_ID: &str = "claim_id";
const FIELD_CHECKED_SCOPE: &str = "checked_scope";
const FIELD_REQUIRED_ARTIFACTS: &str = "required_artifacts";
const FIELD_NON_CLAIMS: &str = "non_claims";
const DEFAULT_EXPLAIN_COMMAND_PREFIX: &str = "cargo octet prove --rail";
const ARTIFACT_VERIFY_COMMAND: &str = "cargo octet artifact verify --artifact-dir <artifact-dir>";
const NON_CLAIM_NO_LEAN_RUNTIME: &str = "not Lean execution";
const NON_CLAIM_NO_SOLVER_COMPLETENESS: &str = "not solver completeness";
const NON_CLAIM_NO_WHOLE_PROGRAM: &str = "not whole-program correctness";
const NON_CLAIM_NO_CERTIFICATION: &str = "not certification";
const NON_CLAIM_NO_VERIFIED_RUNTIME: &str = "not verified runtime";
const OVERCLAIM_LEAN_REQUIRED: &str = "requires Lean";
const OVERCLAIM_SOLVER_COMPLETENESS: &str = "solver completeness";
const OVERCLAIM_WHOLE_PROGRAM: &str = "whole-program correctness";
const OVERCLAIM_CERTIFICATION: &str = "certification";
const OVERCLAIM_VERIFIED_RUNTIME: &str = "verified runtime";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GoalArtifactRequirement {
    pub artifact: String,
    pub expected_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GoalEvidence {
    pub artifact: String,
    pub actual_blake3: String,
    pub evidence_tier: EvidenceTier,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProofGoalInput {
    pub goal_id: String,
    pub claim_id: String,
    pub checked_scope: String,
    pub assumptions: Vec<String>,
    pub required_artifacts: Vec<GoalArtifactRequirement>,
    pub evidence: Vec<GoalEvidence>,
    pub expected_evidence_tier: EvidenceTier,
    pub diagnostics: Vec<String>,
    pub non_claims: Vec<String>,
    pub boundary_wording: String,
    pub remediation_commands: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProofGoalState {
    pub schema_version: String,
    pub goal_id: String,
    pub claim_id: String,
    pub checked_scope: String,
    pub assumptions: Vec<String>,
    pub required_artifacts: Vec<GoalArtifactRequirement>,
    pub satisfied_evidence: Vec<GoalEvidence>,
    pub missing_obligations: Vec<String>,
    pub blockers: Vec<String>,
    pub remediation_commands: Vec<String>,
    pub non_claims: Vec<String>,
    pub diagnostics: Vec<String>,
    pub emits_formal_proof: bool,
}

#[must_use]
pub fn build_proof_goal_state(input: ProofGoalInput) -> ProofGoalState {
    let mut blockers = shape_blockers(&input);
    blockers.extend(boundary_blockers(&input.boundary_wording, &input.non_claims));
    blockers.extend(input.diagnostics.iter().map(|diagnostic| format!("diagnostic blocker: {diagnostic}")));

    let evidence_by_artifact = evidence_map(&input.evidence, &mut blockers);
    let mut satisfied_evidence = Vec::new();
    let mut missing_obligations = Vec::new();
    for requirement in &input.required_artifacts {
        match evidence_by_artifact.get(&requirement.artifact) {
            Some(evidence)
                if evidence.actual_blake3 == requirement.expected_blake3
                    && evidence.evidence_tier == input.expected_evidence_tier
                    && evidence.status == STATUS_PASSED =>
            {
                satisfied_evidence.push((*evidence).clone());
            }
            Some(evidence) if evidence.actual_blake3 != requirement.expected_blake3 => {
                missing_obligations.push(format!(
                    "stale artifact `{}`: expected blake3 {}, got {}",
                    requirement.artifact, requirement.expected_blake3, evidence.actual_blake3
                ));
            }
            Some(evidence) if evidence.evidence_tier != input.expected_evidence_tier => {
                missing_obligations.push(format!(
                    "unsupported evidence tier for `{}`: expected `{}`, got `{}`",
                    requirement.artifact,
                    input.expected_evidence_tier.as_str(),
                    evidence.evidence_tier.as_str()
                ));
            }
            Some(evidence) => {
                missing_obligations
                    .push(format!("artifact `{}` has non-passing status `{}`", requirement.artifact, evidence.status));
            }
            None => missing_obligations.push(format!("missing required artifact `{}`", requirement.artifact)),
        }
    }
    blockers.extend(missing_obligations.clone());

    let mut remediation_commands = input.remediation_commands;
    if !blockers.is_empty() {
        remediation_commands.push(format!("{DEFAULT_EXPLAIN_COMMAND_PREFIX} {} --explain", input.goal_id));
        remediation_commands.push(ARTIFACT_VERIFY_COMMAND.to_owned());
    }

    let expected_evidence_tier = input.expected_evidence_tier;
    let mut state = ProofGoalState {
        schema_version: PROOF_GOAL_STATE_SCHEMA_VERSION.to_owned(),
        goal_id: input.goal_id,
        claim_id: input.claim_id,
        checked_scope: input.checked_scope,
        assumptions: input.assumptions,
        required_artifacts: input.required_artifacts,
        satisfied_evidence,
        missing_obligations,
        blockers,
        remediation_commands,
        non_claims: input.non_claims,
        diagnostics: input.diagnostics,
        emits_formal_proof: false,
    };
    normalize_state(&mut state);
    state.emits_formal_proof = state.blockers.is_empty() && expected_evidence_tier == EvidenceTier::FormalProof;
    state
}

fn shape_blockers(input: &ProofGoalInput) -> Vec<String> {
    let mut blockers = Vec::new();
    push_blank(&mut blockers, FIELD_GOAL_ID, &input.goal_id);
    push_blank(&mut blockers, FIELD_CLAIM_ID, &input.claim_id);
    push_blank(&mut blockers, FIELD_CHECKED_SCOPE, &input.checked_scope);
    if input.required_artifacts.is_empty() {
        blockers.push(format!("missing required goal field `{FIELD_REQUIRED_ARTIFACTS}`"));
    }
    for artifact in &input.required_artifacts {
        push_blank(&mut blockers, "required_artifacts.artifact", &artifact.artifact);
        push_blank(&mut blockers, "required_artifacts.expected_blake3", &artifact.expected_blake3);
    }
    blockers.extend(required_non_claim_blockers(&input.non_claims));
    blockers
}

fn evidence_map<'a>(evidence: &'a [GoalEvidence], blockers: &mut Vec<String>) -> BTreeMap<String, &'a GoalEvidence> {
    let mut by_artifact = BTreeMap::new();
    for item in evidence {
        push_blank(blockers, "evidence.artifact", &item.artifact);
        push_blank(blockers, "evidence.actual_blake3", &item.actual_blake3);
        if by_artifact.insert(item.artifact.clone(), item).is_some() {
            blockers.push(format!("duplicate evidence artifact `{}`", item.artifact));
        }
    }
    by_artifact
}

fn boundary_blockers(wording: &str, non_claims: &[String]) -> Vec<String> {
    let mut blockers = Vec::new();
    let lower = wording.to_ascii_lowercase();
    for (phrase, label) in overclaim_patterns() {
        if positive_phrase_present(&lower, &phrase.to_ascii_lowercase()) {
            blockers.push(format!("goal-state wording overclaims `{label}`"));
        }
    }
    blockers.extend(required_non_claim_blockers(non_claims));
    blockers
}

fn overclaim_patterns() -> [(&'static str, &'static str); OVERCLAIM_PATTERN_COUNT] {
    [
        (OVERCLAIM_LEAN_REQUIRED, "Lean-required runtime"),
        (OVERCLAIM_SOLVER_COMPLETENESS, "solver completeness"),
        (OVERCLAIM_WHOLE_PROGRAM, "whole-program correctness"),
        (OVERCLAIM_CERTIFICATION, "certification"),
        (OVERCLAIM_VERIFIED_RUNTIME, "verified runtime"),
    ]
}

const OVERCLAIM_PATTERN_COUNT: usize = 5;

fn positive_phrase_present(lower_wording: &str, phrase: &str) -> bool {
    lower_wording.contains(phrase)
        && !lower_wording.contains(&format!("not {phrase}"))
        && !lower_wording.contains(&format!("no {phrase}"))
        && !lower_wording.contains(&format!("does not {phrase}"))
}

fn required_non_claim_blockers(non_claims: &[String]) -> Vec<String> {
    let normalized = non_claims.iter().map(|claim| claim.to_ascii_lowercase()).collect::<BTreeSet<_>>();
    required_non_claims()
        .iter()
        .filter(|required| !normalized.contains(&required.to_ascii_lowercase()))
        .map(|required| format!("missing proof goal {FIELD_NON_CLAIMS} entry `{required}`"))
        .collect()
}

fn required_non_claims() -> [&'static str; REQUIRED_NON_CLAIM_COUNT] {
    [
        NON_CLAIM_NO_LEAN_RUNTIME,
        NON_CLAIM_NO_SOLVER_COMPLETENESS,
        NON_CLAIM_NO_WHOLE_PROGRAM,
        NON_CLAIM_NO_CERTIFICATION,
        NON_CLAIM_NO_VERIFIED_RUNTIME,
    ]
}

const REQUIRED_NON_CLAIM_COUNT: usize = 5;

fn normalize_state(state: &mut ProofGoalState) {
    state.assumptions.sort();
    state.assumptions.dedup();
    state.required_artifacts.sort_by(|left, right| left.artifact.cmp(&right.artifact));
    state.required_artifacts.dedup();
    state.satisfied_evidence.sort_by(|left, right| left.artifact.cmp(&right.artifact));
    state.satisfied_evidence.dedup();
    state.missing_obligations.sort();
    state.missing_obligations.dedup();
    state.blockers.sort();
    state.blockers.dedup();
    state.remediation_commands.sort();
    state.remediation_commands.dedup();
    state.non_claims.sort();
    state.non_claims.dedup();
    state.diagnostics.sort();
    state.diagnostics.dedup();
}

fn push_blank(blockers: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        blockers.push(format!("missing required goal field `{field}`"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOAL_ID: &str = "octet.policy.bounds#rail-0";
    const CLAIM_ID: &str = "octet.policy.bounds";
    const CHECKED_SCOPE: &str = "policy:octet.policy.bounds#rail-0";
    const RECEIPT_ARTIFACT: &str = "proofs/octet.policy.bounds/rail-0.receipt.json";
    const CARD_ARTIFACT: &str = "proof-cards.json";
    const PROVENANCE_ARTIFACT: &str = "provenance.jsonl";
    const RECEIPT_HASH: &str = "receipt-blake3";
    const CARD_HASH: &str = "card-blake3";
    const PROVENANCE_HASH: &str = "provenance-blake3";
    const STALE_HASH: &str = "stale-blake3";
    const DOCS_TEXT: &str = include_str!("../../docs/proof-goal-state-feedback.md");

    #[test]
    fn satisfied_goal_state_reports_evidence_and_zero_blockers_deterministically() {
        let input = satisfied_input();

        let first = build_proof_goal_state(input.clone());
        let second = build_proof_goal_state(input);

        assert_eq!(first, second);
        assert_eq!(first.schema_version, PROOF_GOAL_STATE_SCHEMA_VERSION);
        assert_eq!(first.claim_id, CLAIM_ID);
        assert_eq!(first.checked_scope, CHECKED_SCOPE);
        assert_eq!(first.satisfied_evidence.len(), required_artifacts().len());
        assert!(first.blockers.is_empty(), "unexpected blockers: {:?}", first.blockers);
        assert!(first.missing_obligations.is_empty());
        assert!(first.emits_formal_proof);
        assert!(first.assumptions.contains(&"verifier identity is pinned".to_owned()));

        let first_json = serde_json::to_string(&first).expect("serialize first goal state");
        let second_json = serde_json::to_string(&second).expect("serialize second goal state");
        assert_eq!(first_json, second_json);
    }

    #[test]
    fn missing_stale_unsupported_and_unmatched_goal_inputs_block_formal_provenance() {
        let mut input = satisfied_input();
        input.evidence = vec![
            GoalEvidence {
                artifact: RECEIPT_ARTIFACT.to_owned(),
                actual_blake3: STALE_HASH.to_owned(),
                evidence_tier: EvidenceTier::FormalProof,
                status: STATUS_PASSED.to_owned(),
            },
            GoalEvidence {
                artifact: CARD_ARTIFACT.to_owned(),
                actual_blake3: CARD_HASH.to_owned(),
                evidence_tier: EvidenceTier::ManualReviewRequired,
                status: STATUS_PASSED.to_owned(),
            },
        ];
        input.diagnostics = vec!["unmatched proof ref `docs/ref.json`".to_owned()];

        let state = build_proof_goal_state(input);

        assert!(!state.emits_formal_proof);
        assert_contains(&state.blockers, "stale artifact");
        assert_contains(&state.blockers, "unsupported evidence tier");
        assert_contains(&state.blockers, "missing required artifact `provenance.jsonl`");
        assert_contains(&state.blockers, "unmatched proof ref");
        assert_contains(&state.remediation_commands, DEFAULT_EXPLAIN_COMMAND_PREFIX);
        assert_contains(&state.remediation_commands, ARTIFACT_VERIFY_COMMAND);
    }

    #[test]
    fn goal_state_rejects_forbidden_wording_and_missing_non_claims() {
        let mut input = satisfied_input();
        input.boundary_wording = "This proof feedback requires Lean and guarantees solver completeness, whole-program correctness, certification, and verified runtime.".to_owned();
        input.non_claims = vec![NON_CLAIM_NO_WHOLE_PROGRAM.to_owned()];

        let state = build_proof_goal_state(input);

        assert!(!state.emits_formal_proof);
        assert_contains(&state.blockers, "goal-state wording overclaims `Lean-required runtime`");
        assert_contains(&state.blockers, "goal-state wording overclaims `solver completeness`");
        assert_contains(&state.blockers, "goal-state wording overclaims `whole-program correctness`");
        assert_contains(&state.blockers, "goal-state wording overclaims `certification`");
        assert_contains(&state.blockers, "goal-state wording overclaims `verified runtime`");
        assert_contains(&state.blockers, "missing proof goal non_claims entry `not Lean execution`");
    }

    #[test]
    fn proof_goal_state_docs_name_feedback_and_boundaries() {
        for required in [
            PROOF_GOAL_STATE_SCHEMA_VERSION,
            "satisfied evidence",
            "missing obligations",
            "blockers",
            "remediation commands",
            "not Lean execution",
            "not solver completeness",
            "not whole-program correctness",
            "not verified runtime",
        ] {
            assert!(DOCS_TEXT.contains(required), "missing docs phrase: {required}");
        }
    }

    fn satisfied_input() -> ProofGoalInput {
        ProofGoalInput {
            goal_id: GOAL_ID.to_owned(),
            claim_id: CLAIM_ID.to_owned(),
            checked_scope: CHECKED_SCOPE.to_owned(),
            assumptions: vec![
                "proof receipt binds source and proof artifacts".to_owned(),
                "verifier identity is pinned".to_owned(),
            ],
            required_artifacts: required_artifacts(),
            evidence: satisfied_evidence(),
            expected_evidence_tier: EvidenceTier::FormalProof,
            diagnostics: Vec::new(),
            non_claims: required_non_claims().iter().map(|claim| (*claim).to_owned()).collect(),
            boundary_wording: "Lean-inspired obligation feedback only; not Lean execution and not solver completeness."
                .to_owned(),
            remediation_commands: Vec::new(),
        }
    }

    fn required_artifacts() -> Vec<GoalArtifactRequirement> {
        vec![
            GoalArtifactRequirement {
                artifact: RECEIPT_ARTIFACT.to_owned(),
                expected_blake3: RECEIPT_HASH.to_owned(),
            },
            GoalArtifactRequirement {
                artifact: CARD_ARTIFACT.to_owned(),
                expected_blake3: CARD_HASH.to_owned(),
            },
            GoalArtifactRequirement {
                artifact: PROVENANCE_ARTIFACT.to_owned(),
                expected_blake3: PROVENANCE_HASH.to_owned(),
            },
        ]
    }

    fn satisfied_evidence() -> Vec<GoalEvidence> {
        vec![
            GoalEvidence {
                artifact: RECEIPT_ARTIFACT.to_owned(),
                actual_blake3: RECEIPT_HASH.to_owned(),
                evidence_tier: EvidenceTier::FormalProof,
                status: STATUS_PASSED.to_owned(),
            },
            GoalEvidence {
                artifact: CARD_ARTIFACT.to_owned(),
                actual_blake3: CARD_HASH.to_owned(),
                evidence_tier: EvidenceTier::FormalProof,
                status: STATUS_PASSED.to_owned(),
            },
            GoalEvidence {
                artifact: PROVENANCE_ARTIFACT.to_owned(),
                actual_blake3: PROVENANCE_HASH.to_owned(),
                evidence_tier: EvidenceTier::FormalProof,
                status: STATUS_PASSED.to_owned(),
            },
        ]
    }

    fn assert_contains(values: &[String], expected: &str) {
        assert!(
            values.iter().any(|value| value.contains(expected)),
            "expected values to contain `{expected}`, got {values:?}"
        );
    }
}
