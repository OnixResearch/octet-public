use std::fmt::Write as _;

use crate::evidence;
use crate::evidence_rails;
use crate::output;

const GENERATED_CONTRACT_PATH: &str = "standards/evidence/contracts/generated.ncl";
const FINDING_CONTRACT_NAME: &str = "FindingV2";
const PROVENANCE_CONTRACT_NAME: &str = "ProvenanceRecordV1";
const EVIDENCE_RAIL_RECEIPT_CONTRACT_NAME: &str = "EvidenceRailReceiptV1";
const HEGEL_PROPERTY_DETAILS_CONTRACT_NAME: &str = "HegelPropertyReceiptDetailsV1";
const HEGEL_HARDENED_DETAILS_CONTRACT_NAME: &str = "HegelHardenedReceiptDetailsV1";
const MIRAI_PILOT_DECISION_CONTRACT_NAME: &str = "MiraiPilotDecisionV1";
const CARGO_FUZZ_DETAILS_CONTRACT_NAME: &str = "CargoFuzzReceiptDetailsV1";
const SARIF_DETAILS_CONTRACT_NAME: &str = "SarifImportReceiptDetailsV1";
const COVERAGE_DETAILS_CONTRACT_NAME: &str = "CoverageBaselineReceiptDetailsV1";
const MUTATION_DETAILS_CONTRACT_NAME: &str = "CargoMutantsReceiptDetailsV1";
const BENCHMARK_DETAILS_CONTRACT_NAME: &str = "BenchmarkEvidenceReceiptDetailsV1";
const CHAOSCONTROL_PROOF_STYLE_INPUT_CONTRACT_NAME: &str = "ChaosControlProofStyleInputContractV1";
const CHAOSCONTROL_PROOF_STYLE_INPUT_SCHEMA_VERSION: &str = "octet-chaoscontrol-proof-style-input-contract/v1";
const CHAOSCONTROL_CONSUMER_REPO: &str = "github:OnixResearch/chaoscontrol";
const CHAOSCONTROL_SELECTED_PROFILE: &str = "proof-style-focused";
const CHAOSCONTROL_POLICY_REGISTRY_DIGEST: &str = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const CHAOSCONTROL_INPUT_REVISION: &str = "chaoscontrol-proof-style-fixture-revision";
const CHAOSCONTROL_TRELLIS_PRIMITIVE_MANIFEST_DIGEST: &str =
    "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const CHAOSCONTROL_REQUIRED_NON_CLAIM: &str = "Octet validates profile and input identity only; it does not prove ChaosControl behavioral correctness, runtime determinism, or VMM correctness";
const CHAOSCONTROL_REQUIRED_FOCUSED_CRATES: &[&str] = &["chaoscontrol-core", "chaoscontrol-vmm"];
const CHAOSCONTROL_EXPECTED_COMMAND: &[&str] = &["cargo", "octet", "check", "--profile", "proof-style-focused"];
const FINDING_SCHEMA_VERSION: u32 = 2;
const OPTIONAL_STRING_DEFAULT: &str = "";
const EMPTY_ARRAY_DEFAULT: &str = "[]";
const EMPTY_RECORD_DEFAULT: &str = "{}";

static FINDING_FIELDS: &[ContractField] = &[
    ContractField::required("version", FieldKind::ExactNumber(FINDING_SCHEMA_VERSION)),
    ContractField::required("index", FieldKind::NonEmptyString),
    ContractField::required("stable_id", FieldKind::NonEmptyString),
    ContractField::required("semantic_id", FieldKind::NonEmptyString),
    ContractField::required("identity_status", FieldKind::NonEmptyString),
    ContractField::required("normalized_snippet_hash", FieldKind::NonEmptyString),
    ContractField::required("config_hash", FieldKind::NonEmptyString),
    ContractField::required("cache_status", FieldKind::NonEmptyString),
    ContractField::required("lint", FieldKind::NonEmptyString),
    ContractField::required("group", FieldKind::NonEmptyString),
    ContractField::required("severity", FieldKind::NonEmptyString),
    ContractField::required("file", FieldKind::NonEmptyString),
    ContractField::required("line", FieldKind::Number),
    ContractField::required("column", FieldKind::Number),
    ContractField::required("message", FieldKind::NonEmptyString),
    ContractField::optional_string("suggestion"),
    ContractField::optional_string("applicability"),
    ContractField::required("principle", FieldKind::NonEmptyString),
    ContractField::required("effect_family", FieldKind::NonEmptyString),
    ContractField::required("evidence_tier", FieldKind::EvidenceTier),
    ContractField::required("crate_name", FieldKind::NonEmptyString),
];

static PROVENANCE_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(evidence::PROVENANCE_SCHEMA_VERSION)),
    ContractField::required("source_id", FieldKind::NonEmptyString),
    ContractField::required("checker_id", FieldKind::NonEmptyString),
    ContractField::required("evidence_tier", FieldKind::EvidenceTier),
    ContractField::required("artifact_locator", FieldKind::NonEmptyString),
    ContractField::required("blake3", FieldKind::NonEmptyString),
    ContractField::required("checked_scope", FieldKind::NonEmptyString),
    ContractField::optional_string("verifier_identity"),
    ContractField::optional_string("proof_artifact_identity"),
    ContractField::optional_string("human_action"),
    ContractField {
        name: "parent_artifacts",
        kind: FieldKind::StringArray,
        default: Some(EMPTY_ARRAY_DEFAULT),
    },
];

static HEGEL_PROPERTY_DETAILS_FIELDS: &[ContractField] = &[
    ContractField::required("property_id", FieldKind::NonEmptyString),
    ContractField {
        name: "linked_ids",
        kind: FieldKind::StringArray,
        default: Some(EMPTY_ARRAY_DEFAULT),
    },
    ContractField::required("source_scope", FieldKind::NonEmptyString),
    ContractField::required("runner", FieldKind::Dyn),
    ContractField::required("settings", FieldKind::Dyn),
    ContractField::required("generator", FieldKind::Dyn),
    ContractField {
        name: "source_hashes",
        kind: FieldKind::StringArray,
        default: None,
    },
    ContractField {
        name: "counterexample",
        kind: FieldKind::Dyn,
        default: Some(EMPTY_RECORD_DEFAULT),
    },
    ContractField {
        name: "hardened",
        kind: FieldKind::Dyn,
        default: Some(EMPTY_RECORD_DEFAULT),
    },
];

static HEGEL_HARDENED_DETAILS_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(crate::hegel::RECEIPT_SCHEMA)),
    ContractField::required("profile_id", FieldKind::ExactString(crate::hegel::PROFILE_ID)),
    ContractField::required("profile_identity", FieldKind::NonEmptyString),
    ContractField::required("law_catalog_identity", FieldKind::NonEmptyString),
    ContractField::required("property_id", FieldKind::NonEmptyString),
    ContractField::required("linked_ids", FieldKind::StringArray),
    ContractField::required("source_scope", FieldKind::NonEmptyString),
    ContractField::required("producer", FieldKind::Dyn),
    ContractField::required("settings", FieldKind::Dyn),
    ContractField::required("generator", FieldKind::Dyn),
    ContractField::required("command_argv", FieldKind::StringArray),
    ContractField::required("cargo_lock_identity", FieldKind::NonEmptyString),
    ContractField::required("package_manifest_identity", FieldKind::NonEmptyString),
    ContractField::required("source_manifest_identity", FieldKind::NonEmptyString),
    ContractField::required("generator_manifest_identity", FieldKind::NonEmptyString),
    ContractField::required("summary_identity", FieldKind::NonEmptyString),
    ContractField::required("bounded_log_identity", FieldKind::NonEmptyString),
    ContractField::required("result", FieldKind::NonEmptyString),
    ContractField::required("cases_completed", FieldKind::Number),
    ContractField::required("issue_codes", FieldKind::StringArray),
    ContractField::required("projection_identity", FieldKind::NonEmptyString),
    ContractField::required("evidence_tier", FieldKind::EvidenceTier),
    ContractField::required("verification_role", FieldKind::NonEmptyString),
    ContractField::required("non_claims", FieldKind::StringArray),
];

static MIRAI_PILOT_DECISION_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(crate::mirai_pilot::DECISION_SCHEMA)),
    ContractField::required("profile_id", FieldKind::ExactString(crate::mirai_pilot::PROFILE_ID)),
    ContractField::required("profile_identity", FieldKind::NonEmptyString),
    ContractField::required("corpus_identity", FieldKind::NonEmptyString),
    ContractField::required("provider_revision", FieldKind::NonEmptyString),
    ContractField::required("provider_source_hash", FieldKind::NonEmptyString),
    ContractField::required("rust_toolchain", FieldKind::NonEmptyString),
    ContractField::required("target", FieldKind::NonEmptyString),
    ContractField::required("derivation_attr", FieldKind::NonEmptyString),
    ContractField::required("closure_components", FieldKind::StringArray),
    ContractField::required("command_argv", FieldKind::StringArray),
    ContractField::required("environment", FieldKind::StringArray),
    ContractField::required("decision", FieldKind::ExactString("blocked")),
    ContractField::required("blocker_code", FieldKind::ExactString(crate::mirai_pilot::EXPECTED_BLOCKER)),
    ContractField::required("failed_criteria", FieldKind::StringArray),
    ContractField::required("measurements", FieldKind::Dyn),
    ContractField::required("comparisons", FieldKind::Dyn),
    ContractField::required("execution_attempted", FieldKind::Dyn),
    ContractField::required("reproducible_decision", FieldKind::Dyn),
    ContractField::required("claims", FieldKind::StringArray),
    ContractField::required("evidence_tier", FieldKind::EvidenceTier),
    ContractField::required("verification_role", FieldKind::NonEmptyString),
    ContractField::required("non_claims", FieldKind::StringArray),
    ContractField::required("receipt_identity", FieldKind::NonEmptyString),
];

static CARGO_FUZZ_DETAILS_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(crate::cargo::fuzz::RECEIPT_SCHEMA_VERSION)),
    ContractField::required("profile_id", FieldKind::ExactString(crate::cargo::fuzz::PROFILE_ID)),
    ContractField::required("profile_identity", FieldKind::NonEmptyString),
    ContractField::required("target", FieldKind::ExactString(crate::cargo::fuzz::TARGET)),
    ContractField::required("engine", FieldKind::ExactString(crate::cargo::fuzz::ENGINE)),
    ContractField::required("toolchain", FieldKind::ExactString(crate::cargo::fuzz::TOOLCHAIN)),
    ContractField::required("compiler_identity", FieldKind::NonEmptyString),
    ContractField::required("sanitizer", FieldKind::ExactString(crate::cargo::fuzz::SANITIZER)),
    ContractField::required("command_argv", FieldKind::StringArray),
    ContractField::required("budget", FieldKind::Dyn),
    ContractField::required("initial_corpus", FieldKind::Dyn),
    ContractField::required("initial_corpus_root", FieldKind::NonEmptyString),
    ContractField::required("final_corpus", FieldKind::Dyn),
    ContractField::required("final_corpus_root", FieldKind::NonEmptyString),
    ContractField::required("outcome", FieldKind::NonEmptyString),
    ContractField::required("bounded_log_identity", FieldKind::NonEmptyString),
    ContractField::required("evidence_tier", FieldKind::EvidenceTier),
    ContractField::required("non_claims", FieldKind::StringArray),
];

static SARIF_DETAILS_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(crate::external::sarif::RECEIPT_SCHEMA)),
    ContractField::required("profile_id", FieldKind::NonEmptyString),
    ContractField::required("profile_identity", FieldKind::NonEmptyString),
    ContractField::required("raw_report_identity", FieldKind::NonEmptyString),
    ContractField::required("projection_identity", FieldKind::NonEmptyString),
    ContractField::required("source_driver", FieldKind::NonEmptyString),
    ContractField::required("supported_version", FieldKind::ExactString(crate::external::sarif::SUPPORTED_VERSION)),
    ContractField::required("findings", FieldKind::Dyn),
    ContractField::required("non_claims", FieldKind::StringArray),
];

static COVERAGE_DETAILS_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(crate::coverage::RECEIPT_SCHEMA)),
    ContractField::required("profile_id", FieldKind::ExactString(crate::coverage::PROFILE_ID)),
    ContractField::required("profile_identity", FieldKind::NonEmptyString),
    ContractField::required("command_argv", FieldKind::StringArray),
    ContractField::required("current", FieldKind::Dyn),
    ContractField::required("baseline", FieldKind::Dyn),
    ContractField::required("current_envelope_identity", FieldKind::NonEmptyString),
    ContractField::required("baseline_envelope_identity", FieldKind::NonEmptyString),
    ContractField::required("projection_identity", FieldKind::NonEmptyString),
    ContractField::required("comparison", FieldKind::Dyn),
    ContractField::required("evidence_tier", FieldKind::EvidenceTier),
    ContractField::required("non_claims", FieldKind::StringArray),
];

static MUTATION_DETAILS_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(crate::mutation::RECEIPT_SCHEMA)),
    ContractField::required("profile_id", FieldKind::ExactString(crate::mutation::PROFILE_ID)),
    ContractField::required("profile_identity", FieldKind::NonEmptyString),
    ContractField::required("provider_identity", FieldKind::NonEmptyString),
    ContractField::required("rust_toolchain", FieldKind::NonEmptyString),
    ContractField::required("command_argv", FieldKind::StringArray),
    ContractField::required("source_manifest_identity", FieldKind::NonEmptyString),
    ContractField::required("native_outcomes_identity", FieldKind::NonEmptyString),
    ContractField::required("native_inventory_identity", FieldKind::NonEmptyString),
    ContractField::required("projection_identity", FieldKind::NonEmptyString),
    ContractField::required("baseline", FieldKind::Dyn),
    ContractField::required("mutants", FieldKind::Dyn),
    ContractField::required("outcome_counts", FieldKind::Dyn),
    ContractField::required("gate_status", FieldKind::NonEmptyString),
    ContractField::required("evidence_tier", FieldKind::EvidenceTier),
    ContractField::required("non_claims", FieldKind::StringArray),
];

static BENCHMARK_DETAILS_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(crate::benchmark::RECEIPT_SCHEMA)),
    ContractField::required("profile_id", FieldKind::ExactString(crate::benchmark::PROFILE_ID)),
    ContractField::required("profile_identity", FieldKind::NonEmptyString),
    ContractField::required("provider_identity", FieldKind::NonEmptyString),
    ContractField::required("command_argv", FieldKind::StringArray),
    ContractField::required("baseline_identity", FieldKind::NonEmptyString),
    ContractField::required("candidate_identity", FieldKind::NonEmptyString),
    ContractField::required("allocation_identity", FieldKind::NonEmptyString),
    ContractField::required("copy_identity", FieldKind::NonEmptyString),
    ContractField::required("oracle_identity", FieldKind::NonEmptyString),
    ContractField::required("findings_identity", FieldKind::NonEmptyString),
    ContractField::required("projection_identity", FieldKind::NonEmptyString),
    ContractField::required("bounded_log_identity", FieldKind::NonEmptyString),
    ContractField::required("projection", FieldKind::Dyn),
    ContractField::required("public_claims", FieldKind::Dyn),
    ContractField::required("evidence_tier", FieldKind::EvidenceTier),
    ContractField::required("non_claims", FieldKind::StringArray),
];

static CHAOSCONTROL_PROOF_STYLE_INPUT_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(CHAOSCONTROL_PROOF_STYLE_INPUT_SCHEMA_VERSION)),
    ContractField::required("consumer_repo", FieldKind::NonEmptyString),
    ContractField::required("selected_profile", FieldKind::NonEmptyString),
    ContractField {
        name: "focused_crates",
        kind: FieldKind::StringArray,
        default: None,
    },
    ContractField::required("policy_registry_digest", FieldKind::NonEmptyString),
    ContractField::optional_string("trellis_primitive_manifest_digest"),
    ContractField::required("input_revision", FieldKind::NonEmptyString),
    ContractField {
        name: "expected_command",
        kind: FieldKind::StringArray,
        default: None,
    },
    ContractField {
        name: "non_claims",
        kind: FieldKind::StringArray,
        default: None,
    },
];

static EVIDENCE_RAIL_RECEIPT_FIELDS: &[ContractField] = &[
    ContractField::required("schema_version", FieldKind::ExactString(evidence_rails::RECEIPT_SCHEMA_VERSION)),
    ContractField::required("rail_id", FieldKind::NonEmptyString),
    ContractField::required("checker_id", FieldKind::NonEmptyString),
    ContractField::required("mode", FieldKind::NonEmptyString),
    ContractField {
        name: "command_argv",
        kind: FieldKind::StringArray,
        default: None,
    },
    ContractField::required("workspace_scope", FieldKind::NonEmptyString),
    ContractField::optional_string("config_path"),
    ContractField::optional_string("baseline_identity"),
    ContractField::optional_string("lockfile_identity"),
    ContractField {
        name: "checked_rule_families",
        kind: FieldKind::StringArray,
        default: None,
    },
    ContractField::required("status", FieldKind::NonEmptyString),
    ContractField::required("bounded_log_identity", FieldKind::NonEmptyString),
    ContractField::required("imported_input_identity", FieldKind::NonEmptyString),
    ContractField::optional_string("advisory_database_update"),
    ContractField::optional_string("profile"),
    ContractField {
        name: "non_claims",
        kind: FieldKind::StringArray,
        default: None,
    },
    ContractField {
        name: "diagnostics",
        kind: FieldKind::StringArray,
        default: Some(EMPTY_ARRAY_DEFAULT),
    },
    ContractField {
        name: "parent_artifacts",
        kind: FieldKind::StringArray,
        default: Some(EMPTY_ARRAY_DEFAULT),
    },
    ContractField {
        name: "property_classes",
        kind: FieldKind::StringArray,
        default: Some(EMPTY_ARRAY_DEFAULT),
    },
    ContractField {
        name: "hegel_property",
        kind: FieldKind::Dyn,
        default: Some(EMPTY_RECORD_DEFAULT),
    },
    ContractField {
        name: "cargo_fuzz",
        kind: FieldKind::Dyn,
        default: Some(EMPTY_RECORD_DEFAULT),
    },
    ContractField {
        name: "sarif_import",
        kind: FieldKind::Dyn,
        default: Some(EMPTY_RECORD_DEFAULT),
    },
    ContractField {
        name: "coverage_baseline",
        kind: FieldKind::Dyn,
        default: Some(EMPTY_RECORD_DEFAULT),
    },
    ContractField {
        name: "cargo_mutants",
        kind: FieldKind::Dyn,
        default: Some(EMPTY_RECORD_DEFAULT),
    },
    ContractField {
        name: "benchmark_evidence",
        kind: FieldKind::Dyn,
        default: Some(EMPTY_RECORD_DEFAULT),
    },
];

#[derive(Clone, Copy)]
enum FieldKind {
    ExactNumber(u32),
    ExactString(&'static str),
    NonEmptyString,
    String,
    Number,
    EvidenceTier,
    StringArray,
    Dyn,
}

#[derive(Clone, Copy)]
struct ContractField {
    name: &'static str,
    kind: FieldKind,
    default: Option<&'static str>,
}

struct ContractSchema {
    name: &'static str,
    fields: &'static [ContractField],
}

pub fn generated_contract_path() -> &'static str {
    GENERATED_CONTRACT_PATH
}

pub fn render_generated_contracts() -> String {
    let schemas = [
        finding_schema(),
        provenance_schema(),
        hegel_property_details_schema(),
        hegel_hardened_details_schema(),
        mirai_pilot_decision_schema(),
        cargo_fuzz_details_schema(),
        sarif_details_schema(),
        coverage_details_schema(),
        mutation_details_schema(),
        benchmark_details_schema(),
        evidence_rail_receipt_schema(),
        chaoscontrol_proof_style_input_schema(),
    ];
    let mut rendered = String::new();
    rendered.push_str("# @generated by cargo-octet/src/evidence_contracts.rs; do not edit by hand.\n");
    rendered.push_str("let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in\n");
    rendered.push_str(
        "let EvidenceTier = std.contract.from_predicate (fun value => std.is_string value && std.array.elem value [\n",
    );
    for tier in evidence_tier_discriminants() {
        writeln!(&mut rendered, "  \"{tier}\",").expect("render evidence tier discriminant");
    }
    rendered.push_str("]) in\n");
    rendered.push_str("{\n");
    for schema in schemas {
        render_schema(&mut rendered, &schema);
    }
    rendered.push_str("}\n");
    rendered
}

fn evidence_tier_discriminants() -> Vec<&'static str> {
    [
        evidence::EvidenceTier::StaticLint,
        evidence::EvidenceTier::PolicyCheck,
        evidence::EvidenceTier::UiFixture,
        evidence::EvidenceTier::AutofixVerified,
        evidence::EvidenceTier::TranscriptAssertion,
        evidence::EvidenceTier::FormalProof,
        evidence::EvidenceTier::ManualReviewRequired,
    ]
    .into_iter()
    .map(evidence::EvidenceTier::as_str)
    .collect()
}

fn finding_schema() -> ContractSchema {
    ContractSchema {
        name: FINDING_CONTRACT_NAME,
        fields: FINDING_FIELDS,
    }
}

fn provenance_schema() -> ContractSchema {
    ContractSchema {
        name: PROVENANCE_CONTRACT_NAME,
        fields: PROVENANCE_FIELDS,
    }
}

fn hegel_property_details_schema() -> ContractSchema {
    ContractSchema {
        name: HEGEL_PROPERTY_DETAILS_CONTRACT_NAME,
        fields: HEGEL_PROPERTY_DETAILS_FIELDS,
    }
}

fn hegel_hardened_details_schema() -> ContractSchema {
    ContractSchema {
        name: HEGEL_HARDENED_DETAILS_CONTRACT_NAME,
        fields: HEGEL_HARDENED_DETAILS_FIELDS,
    }
}

fn mirai_pilot_decision_schema() -> ContractSchema {
    ContractSchema {
        name: MIRAI_PILOT_DECISION_CONTRACT_NAME,
        fields: MIRAI_PILOT_DECISION_FIELDS,
    }
}

fn cargo_fuzz_details_schema() -> ContractSchema {
    ContractSchema {
        name: CARGO_FUZZ_DETAILS_CONTRACT_NAME,
        fields: CARGO_FUZZ_DETAILS_FIELDS,
    }
}

fn sarif_details_schema() -> ContractSchema {
    ContractSchema {
        name: SARIF_DETAILS_CONTRACT_NAME,
        fields: SARIF_DETAILS_FIELDS,
    }
}

fn coverage_details_schema() -> ContractSchema {
    ContractSchema {
        name: COVERAGE_DETAILS_CONTRACT_NAME,
        fields: COVERAGE_DETAILS_FIELDS,
    }
}

fn mutation_details_schema() -> ContractSchema {
    ContractSchema {
        name: MUTATION_DETAILS_CONTRACT_NAME,
        fields: MUTATION_DETAILS_FIELDS,
    }
}

fn benchmark_details_schema() -> ContractSchema {
    ContractSchema {
        name: BENCHMARK_DETAILS_CONTRACT_NAME,
        fields: BENCHMARK_DETAILS_FIELDS,
    }
}

fn evidence_rail_receipt_schema() -> ContractSchema {
    ContractSchema {
        name: EVIDENCE_RAIL_RECEIPT_CONTRACT_NAME,
        fields: EVIDENCE_RAIL_RECEIPT_FIELDS,
    }
}

fn chaoscontrol_proof_style_input_schema() -> ContractSchema {
    ContractSchema {
        name: CHAOSCONTROL_PROOF_STYLE_INPUT_CONTRACT_NAME,
        fields: CHAOSCONTROL_PROOF_STYLE_INPUT_FIELDS,
    }
}

impl ContractField {
    const fn required(name: &'static str, kind: FieldKind) -> Self {
        Self {
            name,
            kind,
            default: None,
        }
    }

    const fn optional_string(name: &'static str) -> Self {
        Self {
            name,
            kind: FieldKind::String,
            default: Some(OPTIONAL_STRING_DEFAULT),
        }
    }
}

fn render_schema(rendered: &mut String, schema: &ContractSchema) {
    writeln!(rendered, "  {} = {{", schema.name).expect("render schema header");
    for field in schema.fields {
        let default = field.default.map(render_default).unwrap_or_default();
        writeln!(rendered, "    {} | {}{},", field.name, render_field_kind(field.kind), default)
            .expect("render contract field");
    }
    rendered.push_str("  },\n");
}

fn render_default(value: &str) -> String {
    if value == EMPTY_ARRAY_DEFAULT {
        " | default = []".to_owned()
    } else if value == EMPTY_RECORD_DEFAULT {
        " | default = {}".to_owned()
    } else {
        format!(" | default = {value:?}")
    }
}

fn render_field_kind(kind: FieldKind) -> String {
    match kind {
        FieldKind::ExactNumber(value) => format!("std.contract.from_predicate (fun value => value == {value})"),
        FieldKind::ExactString(value) => {
            format!("std.contract.from_predicate (fun value => value == {value:?})")
        }
        FieldKind::NonEmptyString => "NonEmptyString".to_owned(),
        FieldKind::String => "String".to_owned(),
        FieldKind::Number => "Number".to_owned(),
        FieldKind::EvidenceTier => "EvidenceTier".to_owned(),
        FieldKind::StringArray => "Array String".to_owned(),
        FieldKind::Dyn => "Dyn".to_owned(),
    }
}

#[derive(Clone, Debug)]
struct ChaosControlProofStyleInputContract {
    schema_version: String,
    consumer_repo: String,
    selected_profile: String,
    focused_crates: Vec<String>,
    policy_registry_digest: String,
    trellis_primitive_manifest_digest: Option<String>,
    input_revision: String,
    expected_command: Vec<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChaosControlProofStyleInputReceipt {
    valid: bool,
    issues: Vec<&'static str>,
    consumer_repo: String,
    selected_profile: String,
    supported_claim: &'static str,
}

fn validate_chaoscontrol_proof_style_input(
    contract: &ChaosControlProofStyleInputContract,
) -> ChaosControlProofStyleInputReceipt {
    let mut issues = Vec::new();

    push_if(
        &mut issues,
        contract.schema_version != CHAOSCONTROL_PROOF_STYLE_INPUT_SCHEMA_VERSION,
        "schema_version_mismatch",
    );
    push_if(&mut issues, contract.consumer_repo != CHAOSCONTROL_CONSUMER_REPO, "consumer_repo_mismatch");
    push_if(&mut issues, contract.selected_profile != CHAOSCONTROL_SELECTED_PROFILE, "selected_profile_mismatch");
    push_if(
        &mut issues,
        !contains_all(&contract.focused_crates, CHAOSCONTROL_REQUIRED_FOCUSED_CRATES),
        "focused_crate_missing",
    );
    push_if(
        &mut issues,
        contract.policy_registry_digest != CHAOSCONTROL_POLICY_REGISTRY_DIGEST,
        "policy_registry_digest_stale",
    );
    push_if(
        &mut issues,
        contract.trellis_primitive_manifest_digest.as_deref() != Some(CHAOSCONTROL_TRELLIS_PRIMITIVE_MANIFEST_DIGEST),
        "trellis_manifest_digest_stale",
    );
    push_if(&mut issues, contract.input_revision != CHAOSCONTROL_INPUT_REVISION, "input_revision_stale");
    push_if(
        &mut issues,
        !command_matches(&contract.expected_command, CHAOSCONTROL_EXPECTED_COMMAND),
        "expected_command_mismatch",
    );
    validate_chaoscontrol_non_claims(&contract.non_claims, &mut issues);

    issues.sort_unstable();
    issues.dedup();
    ChaosControlProofStyleInputReceipt {
        valid: issues.is_empty(),
        issues,
        consumer_repo: contract.consumer_repo.clone(),
        selected_profile: contract.selected_profile.clone(),
        supported_claim: CHAOSCONTROL_REQUIRED_NON_CLAIM,
    }
}

fn push_if(issues: &mut Vec<&'static str>, condition: bool, issue: &'static str) {
    if condition {
        issues.push(issue);
    }
}

fn contains_all(values: &[String], required: &[&str]) -> bool {
    required.iter().all(|required_value| values.iter().any(|value| value == required_value))
}

fn command_matches(values: &[String], expected: &[&str]) -> bool {
    values.len() == expected.len()
        && values.iter().zip(expected.iter()).all(|(value, expected_value)| value == expected_value)
}

fn validate_chaoscontrol_non_claims(non_claims: &[String], issues: &mut Vec<&'static str>) {
    if non_claims.is_empty() {
        issues.push("non_claims_missing");
        return;
    }
    let joined = non_claims.join(" ");
    if !joined.contains(CHAOSCONTROL_REQUIRED_NON_CLAIM) {
        issues.push("non_claims_weakened");
    }
    if joined.contains("proves ChaosControl behavioral correctness")
        || joined.contains("proves runtime determinism")
        || joined.contains("proves VMM correctness")
    {
        issues.push("overbroad_claim");
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::*;

    const VALID_FIXTURES: &[&str] = &[
        "standards/evidence/fixtures/valid/finding_v2.ncl",
        "standards/evidence/fixtures/valid/provenance_record_v1.ncl",
        "standards/evidence/fixtures/valid/hegel_property_details_v1.ncl",
        "standards/evidence/fixtures/valid/hegel_hardened_details_v1.ncl",
        "standards/evidence/fixtures/valid/mirai_pilot_decision_v1.ncl",
        "standards/evidence/fixtures/valid/cargo_fuzz_details_v1.ncl",
        "standards/evidence/fixtures/valid/sarif_import_details_v1.ncl",
        "standards/evidence/fixtures/valid/coverage_baseline_details_v1.ncl",
        "standards/evidence/fixtures/valid/cargo_mutants_details_v1.ncl",
        "standards/evidence/fixtures/valid/evidence_rail_receipt_v1.ncl",
        "standards/evidence/fixtures/valid/chaoscontrol_proof_style_input_v1.ncl",
    ];
    const INVALID_FIXTURES: &[&str] = &[
        "standards/evidence/fixtures/invalid/finding_missing_lint.ncl",
        "standards/evidence/fixtures/invalid/finding_wrong_type.ncl",
        "standards/evidence/fixtures/invalid/provenance_bad_schema.ncl",
        "standards/evidence/fixtures/invalid/provenance_bad_tier.ncl",
        "standards/evidence/fixtures/invalid/hegel_property_missing_property_id.ncl",
        "standards/evidence/fixtures/invalid/hegel_hardened_wrong_profile.ncl",
        "standards/evidence/fixtures/invalid/mirai_pilot_wrong_decision.ncl",
        "standards/evidence/fixtures/invalid/cargo_fuzz_wrong_profile.ncl",
        "standards/evidence/fixtures/invalid/sarif_import_bad_version.ncl",
        "standards/evidence/fixtures/invalid/coverage_baseline_bad_profile.ncl",
        "standards/evidence/fixtures/invalid/cargo_mutants_bad_profile.ncl",
        "standards/evidence/fixtures/invalid/evidence_rail_receipt_missing_non_claims.ncl",
        "standards/evidence/fixtures/invalid/chaoscontrol_missing_profile.ncl",
        "standards/evidence/fixtures/invalid/chaoscontrol_missing_focused_crates.ncl",
    ];

    #[test]
    fn finding_schema_version_matches_emitted_finding_version() {
        assert_eq!(FINDING_SCHEMA_VERSION, output::finding_version());
    }

    #[test]
    fn generated_contracts_are_fresh() {
        let repo_root = repo_root();
        let path = repo_root.join(generated_contract_path());
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read generated contract {}: {error}", path.display()));
        let actual = render_generated_contracts();

        assert_eq!(expected, actual, "generated evidence contracts are stale");
    }

    #[test]
    #[ignore = "writes the checked generated Nickel contract"]
    fn regenerate_evidence_contracts() {
        let repo_root = repo_root();
        let path = repo_root.join(generated_contract_path());
        std::fs::write(&path, render_generated_contracts())
            .unwrap_or_else(|error| panic!("write generated contract {}: {error}", path.display()));
    }

    #[test]
    fn chaoscontrol_proof_style_current_fixture_passes_and_preserves_identity() {
        // r[verify octet.evidence_contracts.chaoscontrol_proof_style_inputs.fixtures.positive]
        // r[verify octet.evidence_contracts.chaoscontrol_proof_style_inputs.validation]
        let receipt = validate_chaoscontrol_proof_style_input(&chaoscontrol_contract_fixture());

        assert!(receipt.valid, "unexpected issues: {:?}", receipt.issues);
        assert!(receipt.issues.is_empty());
        assert_eq!(receipt.consumer_repo, CHAOSCONTROL_CONSUMER_REPO);
        assert_eq!(receipt.selected_profile, CHAOSCONTROL_SELECTED_PROFILE);
        assert_eq!(receipt.supported_claim, CHAOSCONTROL_REQUIRED_NON_CLAIM);
    }

    #[test]
    fn chaoscontrol_proof_style_stale_missing_and_overbroad_inputs_fail_closed() {
        // r[verify octet.evidence_contracts.chaoscontrol_proof_style_inputs.fixtures.negative]
        // r[verify octet.evidence_contracts.chaoscontrol_proof_style_inputs.docs]
        let mut stale = chaoscontrol_contract_fixture();
        stale.input_revision = "stale-chaoscontrol-revision".to_owned();
        stale.policy_registry_digest = "b3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_owned();
        let stale_receipt = validate_chaoscontrol_proof_style_input(&stale);
        assert!(!stale_receipt.valid);
        assert!(stale_receipt.issues.contains(&"input_revision_stale"));
        assert!(stale_receipt.issues.contains(&"policy_registry_digest_stale"));

        let mut missing_profile = chaoscontrol_contract_fixture();
        missing_profile.selected_profile = String::new();
        let missing_profile_receipt = validate_chaoscontrol_proof_style_input(&missing_profile);
        assert!(!missing_profile_receipt.valid);
        assert!(missing_profile_receipt.issues.contains(&"selected_profile_mismatch"));

        let mut missing_crate = chaoscontrol_contract_fixture();
        missing_crate.focused_crates = vec!["chaoscontrol-core".to_owned()];
        let missing_crate_receipt = validate_chaoscontrol_proof_style_input(&missing_crate);
        assert!(!missing_crate_receipt.valid);
        assert!(missing_crate_receipt.issues.contains(&"focused_crate_missing"));

        let mut overbroad = chaoscontrol_contract_fixture();
        overbroad.non_claims = vec![
            CHAOSCONTROL_REQUIRED_NON_CLAIM.to_owned(),
            "this proves ChaosControl behavioral correctness".to_owned(),
        ];
        let overbroad_receipt = validate_chaoscontrol_proof_style_input(&overbroad);
        assert!(!overbroad_receipt.valid);
        assert!(overbroad_receipt.issues.contains(&"overbroad_claim"));
    }

    #[test]
    fn chaoscontrol_proof_style_command_and_trellis_identity_are_checked() {
        // r[verify octet.evidence_contracts.chaoscontrol_proof_style_inputs.final_validation.fixtures]
        let mut contract = chaoscontrol_contract_fixture();
        contract.expected_command = vec!["cargo".to_owned(), "octet".to_owned(), "check".to_owned()];
        contract.trellis_primitive_manifest_digest = None;
        let receipt = validate_chaoscontrol_proof_style_input(&contract);

        assert!(!receipt.valid);
        assert!(receipt.issues.contains(&"expected_command_mismatch"));
        assert!(receipt.issues.contains(&"trellis_manifest_digest_stale"));
    }

    #[test]
    fn evidence_contract_fixtures_accept_valid_and_reject_invalid_records() {
        let repo_root = repo_root();
        for fixture in VALID_FIXTURES {
            let output = nickel_export(&repo_root, fixture);
            assert!(
                output.status.success(),
                "valid fixture {fixture} should pass: stderr={}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        for fixture in INVALID_FIXTURES {
            let output = nickel_export(&repo_root, fixture);
            assert!(
                !output.status.success(),
                "invalid fixture {fixture} should fail but stdout was {}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }

    fn chaoscontrol_contract_fixture() -> ChaosControlProofStyleInputContract {
        ChaosControlProofStyleInputContract {
            schema_version: CHAOSCONTROL_PROOF_STYLE_INPUT_SCHEMA_VERSION.to_owned(),
            consumer_repo: CHAOSCONTROL_CONSUMER_REPO.to_owned(),
            selected_profile: CHAOSCONTROL_SELECTED_PROFILE.to_owned(),
            focused_crates: CHAOSCONTROL_REQUIRED_FOCUSED_CRATES.iter().map(ToString::to_string).collect(),
            policy_registry_digest: CHAOSCONTROL_POLICY_REGISTRY_DIGEST.to_owned(),
            trellis_primitive_manifest_digest: Some(CHAOSCONTROL_TRELLIS_PRIMITIVE_MANIFEST_DIGEST.to_owned()),
            input_revision: CHAOSCONTROL_INPUT_REVISION.to_owned(),
            expected_command: CHAOSCONTROL_EXPECTED_COMMAND.iter().map(ToString::to_string).collect(),
            non_claims: vec![CHAOSCONTROL_REQUIRED_NON_CLAIM.to_owned()],
        }
    }

    fn nickel_export(repo_root: &Path, fixture: &str) -> std::process::Output {
        Command::new("nickel")
            .args(["export", "--format", "json", fixture])
            .current_dir(repo_root)
            .output()
            .unwrap_or_else(|error| panic!("run nickel for {fixture}: {error}"))
    }

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("repo root").to_path_buf()
    }
}
