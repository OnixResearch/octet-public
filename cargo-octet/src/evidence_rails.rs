use std::collections::BTreeMap;
use std::fs;
use std::io::Read as _;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use serde::Deserialize;
use serde::Serialize;

use crate::cap_fs::ArtifactRoot;
use crate::cap_fs::RootRelativePath;
use crate::cap_fs::SourceRoot;
use crate::cli::EvidenceRailArgs;
use crate::cli::OutputFormat;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;

pub const RECEIPT_SCHEMA_VERSION: &str = "octet-evidence-rail-receipt/v1";
pub const RECEIPT_ARTIFACT_NAME: &str = "evidence-rail-receipt.json";
pub const SUMMARY_ARTIFACT_NAME: &str = "summary.txt";
pub const PROVENANCE_ARTIFACT_NAME: &str = "provenance.jsonl";

const EXIT_SUCCESS: i32 = 0;
const EXIT_RAIL_FAILURE: i32 = 1;
const EXIT_INTEGRATION_FAILURE: i32 = 2;
const STATUS_PASSED: &str = "passed";
const STATUS_FAILED: &str = "failed";
const STATUS_MANUAL_REVIEW_REQUIRED: &str = "manual_review_required";
const MODE_IMPORT: &str = "import";
const MODE_RUN: &str = "run";
const LOCKED_REUSED: &str = "reused";
const LOCKED_UPDATED_PRIOR: &str = "updated_by_explicit_prior_step";
const LOCKED_UNAVAILABLE: &str = "unavailable";
const SUMMARY_REVIEW_ACTION: &str = "review optional evidence rail summary and receipt before using this artifact";
const EMPTY_LOG_IDENTITY: &str = "b3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adf687e5e01f59efb8f0b5a2";
const DYNAMIC_PROFILE_MIRI_UNSAFE_BOUNDARY: &str = "miri-unsafe-boundary";
const DYNAMIC_PROFILE_MIRI_SHORT: &str = "miri";
const DYNAMIC_COMMAND_MIRI_TEST: &str = "cargo miri test";
const DYNAMIC_PROFILE_LOOM_CONCURRENCY: &str = "loom-concurrency";
const DYNAMIC_PROFILE_LOOM_SHORT: &str = "loom";
const DYNAMIC_COMMAND_LOOM_TEST: &str = "cargo test --cfg loom";
const SUPPORTED_EVIDENCE_RAILS: &str = "dependency-audit, clippy-co-reporting, ast-grep, wasm-artifact, charon-semantic, dependency-trust, api-stability, dynamic-evidence, sarif-import, release-supply-chain, mutation-testing, coverage-baseline, benchmark-evidence, kani-model-check, hegel-property, z4-solver, or verifier-fixture";
const COVERAGE_PERCENT_MIN: f64 = 0.0;
const COVERAGE_PERCENT_MAX: f64 = 100.0;
const SCHEMA_VERSION_FIELD: &str = "schema_version";
const DEPENDENCY_AUDIT_SCHEMA_VERSION: &str = "cargo-deny-diagnostics/v1";
const COVERAGE_BASELINE_SCHEMA_VERSION: &str = "cargo-llvm-cov-summary/v1";
const KANI_MODEL_CHECK_SCHEMA_VERSION: &str = "cargo-kani-summary/v1";
const HEGEL_PROPERTY_SCHEMA_VERSION: &str = "hegel-property-summary/v1";
const Z4_SOLVER_SCHEMA_VERSION: &str = "z4-solver-summary/v1";
const VERIFIER_FIXTURE_SCHEMA_VERSION: &str = "verifier-fixture-intake/v1";
const HEGEL_GENERATED_TESTING_NON_CLAIM: &str = "Hegel checks bounded generated inputs only and does not prove correctness, semantic equivalence, verifier soundness, requirement satisfaction, release eligibility, or whole-program safety";
const Z4_SOLVER_NON_CLAIM: &str = "Z4 solver evidence records bounded solver output identity and diagnostics only; it does not prove source correctness, solver soundness, proof validity, requirement satisfaction, release eligibility, or whole-program safety";
const VERIFIER_FIXTURE_NON_CLAIM: &str = "Verifier fixtures are Octet-owned regression and UX inputs only; they do not prove source-verifier execution, verifier parity, proof soundness, solver soundness, source-code correctness, whole-program safety, or release eligibility";
const EMBEDDED_CLIPPY_PROFILE_LOCATOR: &str = "embedded:standards/clippy-restrictions/generated/profiles.json";
const HEGEL_FAST_CASE_BUDGET: u64 = 16;
const FORMAL_PROOF_TIER: &str = "formal_proof";
const BLAKE3_IDENTITY_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const SPECIALIZED_DIAGNOSTIC_CAPACITY: usize = 4;
const PROFILED_COVERAGE_FINDING_CAPACITY: usize = 12;
const KANI_STUB_KIND_VERIFIED: &str = "verified";
const KANI_STUB_KIND_PLAIN: &str = "plain";
const KANI_STUB_KIND_ENVIRONMENT: &str = "environment";
const KANI_COVERAGE_UNBOUNDED: &str = "unbounded";
const KANI_PROPERTY_CLASS_NORMALIZATION: &str = "normalization";
const KANI_PROPERTY_CLASS_ALGEBRAIC_LAW: &str = "algebraic-law";
const KANI_PROPERTY_CLASS_ENCODE_DECODE: &str = "encode-decode";
const KANI_PROPERTY_CLASS_VALIDITY_RANGE: &str = "validity-range";
const KANI_PROPERTY_CLASS_LOOP_TERMINATION: &str = "loop-termination";
const KANI_PROPERTY_CLASS_TRAIT_CONSISTENCY: &str = "trait-consistency";
const KANI_PROPERTY_CLASS_BOUNDARY_PANIC: &str = "boundary-panic";
const Z4_STATUS_SAT: &str = "sat";
const Z4_STATUS_UNSAT: &str = "unsat";
const Z4_STATUS_UNKNOWN: &str = "unknown";
const Z4_STATUS_TIMEOUT: &str = "timeout";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EvidenceRailKind {
    DependencyAudit,
    ClippyCoReporting,
    AstGrepStructural,
    WasmArtifact,
    CharonSemantic,
    DependencyTrust,
    ApiStability,
    DynamicEvidence,
    SarifImport,
    ReleaseSupplyChain,
    MutationTesting,
    CoverageBaseline,
    BenchmarkEvidence,
    KaniModelCheck,
    HegelProperty,
    Z4Solver,
    VerifierFixture,
}

impl EvidenceRailKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "dependency-audit" | "cargo-deny" => Ok(Self::DependencyAudit),
            "clippy-co-reporting" | "clippy" | "external-lint" => Ok(Self::ClippyCoReporting),
            "ast-grep" | "ast-grep-structural" | "structural-rail" => Ok(Self::AstGrepStructural),
            "wasm-artifact" | "wasm-component" | "wasm" => Ok(Self::WasmArtifact),
            "charon-semantic" | "charon" | "llbc-semantic" => Ok(Self::CharonSemantic),
            "dependency-trust" | "cargo-vet" => Ok(Self::DependencyTrust),
            "api-stability" | "semver" | "cargo-semver-checks" => Ok(Self::ApiStability),
            "dynamic-evidence" => Ok(Self::DynamicEvidence),
            "sarif-import" | "external-sarif" => Ok(Self::SarifImport),
            "release-supply-chain" => Ok(Self::ReleaseSupplyChain),
            "mutation-testing" | "cargo-mutants" | "mutants" => Ok(Self::MutationTesting),
            "coverage-baseline" | "coverage" | "cargo-llvm-cov" => Ok(Self::CoverageBaseline),
            "benchmark-evidence" | "benchmark" => Ok(Self::BenchmarkEvidence),
            "kani-model-check" | "kani" | "cargo-kani" => Ok(Self::KaniModelCheck),
            "hegel-property" | "hegel" | "hegel-rs" => Ok(Self::HegelProperty),
            "z4-solver" | "z4" => Ok(Self::Z4Solver),
            "verifier-fixture" | "verifier-fixture-intake" => Ok(Self::VerifierFixture),
            other => Err(format!("unsupported evidence rail `{other}`; expected {SUPPORTED_EVIDENCE_RAILS}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DependencyAudit => "dependency-audit",
            Self::ClippyCoReporting => "clippy-co-reporting",
            Self::AstGrepStructural => "ast-grep",
            Self::WasmArtifact => "wasm-artifact",
            Self::CharonSemantic => "charon-semantic",
            Self::DependencyTrust => "dependency-trust",
            Self::ApiStability => "api-stability",
            Self::DynamicEvidence => "dynamic-evidence",
            Self::SarifImport => "sarif-import",
            Self::ReleaseSupplyChain => "release-supply-chain",
            Self::MutationTesting => "mutation-testing",
            Self::CoverageBaseline => "coverage-baseline",
            Self::BenchmarkEvidence => "benchmark-evidence",
            Self::KaniModelCheck => "kani-model-check",
            Self::HegelProperty => "hegel-property",
            Self::Z4Solver => "z4-solver",
            Self::VerifierFixture => "verifier-fixture",
        }
    }

    pub fn checker_id(self) -> &'static str {
        match self {
            Self::DependencyAudit => crate::evidence::CHECKER_CARGO_DENY,
            Self::ClippyCoReporting => crate::evidence::CHECKER_CLIPPY,
            Self::AstGrepStructural => crate::evidence::CHECKER_AST_GREP,
            Self::WasmArtifact | Self::CharonSemantic => crate::evidence::CHECKER_CARGO_OCTET,
            Self::DependencyTrust => crate::evidence::CHECKER_CARGO_VET,
            Self::ApiStability => crate::evidence::CHECKER_CARGO_SEMVER_CHECKS,
            Self::DynamicEvidence => crate::evidence::CHECKER_OCTET_DYNAMIC_EVIDENCE,
            Self::SarifImport => crate::evidence::CHECKER_SARIF_IMPORT,
            Self::ReleaseSupplyChain => crate::evidence::CHECKER_OCTET_RELEASE_SUPPLY_CHAIN,
            Self::MutationTesting => crate::evidence::CHECKER_CARGO_MUTANTS,
            Self::CoverageBaseline => crate::evidence::CHECKER_CARGO_LLVM_COV,
            Self::BenchmarkEvidence => crate::evidence::CHECKER_OCTET_BENCHMARK,
            Self::KaniModelCheck => crate::evidence::CHECKER_CARGO_KANI,
            Self::HegelProperty => crate::evidence::CHECKER_HEGEL_RS,
            Self::Z4Solver => crate::evidence::CHECKER_Z4_SOLVER,
            Self::VerifierFixture => crate::evidence::CHECKER_CARGO_OCTET,
        }
    }

    pub fn source_id(self) -> &'static str {
        match self {
            Self::DependencyAudit => "octet.policy.dependency-audit",
            Self::ClippyCoReporting => "octet.external_lint.clippy-co-reporting",
            Self::AstGrepStructural => "octet.structural_rail.ast-grep",
            Self::WasmArtifact => "octet.artifact.wasm-tools",
            Self::CharonSemantic => "octet.charon_semantic_analysis.recorded-snapshot",
            Self::DependencyTrust => "octet.dependency_trust.cargo-vet",
            Self::ApiStability => "octet.api_stability.semver-rail",
            Self::DynamicEvidence => "octet.dynamic_evidence.profile-runner",
            Self::SarifImport => "octet.external_lint.sarif-import",
            Self::ReleaseSupplyChain => "octet.release_supply_chain.local-receipt",
            Self::MutationTesting => "octet.mutation_testing.cargo-mutants",
            Self::CoverageBaseline => "octet.coverage_baseline.cargo-llvm-cov",
            Self::BenchmarkEvidence => "octet.benchmark_evidence.profile-runner",
            Self::KaniModelCheck => "octet.kani_model_check.cargo-kani",
            Self::HegelProperty => "octet.dynamic_evidence.hegel-property",
            Self::Z4Solver => "octet.experimental_solver_rails.z4-solver",
            Self::VerifierFixture => "octet.runner.verifier-fixture-intake",
        }
    }

    pub fn evidence_tier(self) -> EvidenceTier {
        match self {
            Self::ClippyCoReporting | Self::AstGrepStructural | Self::SarifImport => EvidenceTier::StaticLint,
            Self::WasmArtifact => EvidenceTier::PolicyCheck,
            Self::CharonSemantic => EvidenceTier::TranscriptAssertion,
            Self::DynamicEvidence
            | Self::MutationTesting
            | Self::CoverageBaseline
            | Self::BenchmarkEvidence
            | Self::KaniModelCheck
            | Self::HegelProperty
            | Self::Z4Solver
            | Self::VerifierFixture => EvidenceTier::TranscriptAssertion,
            _ => EvidenceTier::PolicyCheck,
        }
    }

    fn checked_rule_families(self) -> Vec<String> {
        match self {
            Self::DependencyAudit => string_vec(&["advisories", "licenses", "bans", "sources", "duplicates"]),
            Self::ClippyCoReporting => string_vec(&["clippy-diagnostics", "source-severity", "octet-gate-outcome"]),
            Self::AstGrepStructural => string_vec(&[
                "rule-bundle-identity",
                "positive-and-negative-rule-coverage",
                "source-spans-and-metavariables",
                "promotion-lanes",
                "dry-run-codemods",
                "recorded-only-valence-boundary",
            ]),
            Self::WasmArtifact => string_vec(&[
                "exact-byte-validation",
                "module-component-classification",
                "imports-exports-worlds",
                "memory-table-growth-start",
                "producer-and-custom-metadata",
                "resource-limits",
                "exact-and-interface-identities",
                "recorded-only-valence-boundary",
            ]),
            Self::CharonSemantic => string_vec(&[
                "provider-and-extraction-identity",
                "exact-item-and-source-span",
                "resolved-local-calls-and-dependencies",
                "trait-resolution-and-control-flow",
                "closed-effect-candidates-and-unknowns",
                "source-token-comparison-without-authority-merge",
                "role-separated-blake3-artifacts",
                "recorded-only-stack-boundary",
            ]),
            Self::DependencyTrust => string_vec(&["audits", "imports", "exemptions", "unaudited-gaps"]),
            Self::ApiStability => string_vec(&["public-api", "baseline", "feature-set", "target-triple"]),
            Self::SarifImport => string_vec(&[
                "producer-profile-identity",
                "raw-report-identity",
                "normalized-projection-identity",
                "source-rule-severity-location-preservation",
                "local-bounded-intake",
            ]),
            Self::DynamicEvidence => string_vec(&[
                "profile-selection",
                "allowed-command",
                "acceptance-signal",
                "cargo-fuzz-target-and-budget",
                "cargo-fuzz-corpus-and-crash-identity",
            ]),
            Self::ReleaseSupplyChain => string_vec(&[
                "auditable-binary-metadata",
                "sbom",
                "vulnerability-scan",
                "attestation-compatibility",
            ]),
            Self::MutationTesting => string_vec(&[
                "mutant-generation",
                "caught-mutants",
                "survived-mutants",
                "timeout-mutants",
                "test-suite-kill-signal",
            ]),
            Self::CoverageBaseline => string_vec(&[
                "coverage-summary",
                "accepted-baseline",
                "regression-detection",
                "threshold-failures",
            ]),
            Self::BenchmarkEvidence => string_vec(&[
                "typed-workload-profile",
                "bounded-raw-samples",
                "environment-comparability",
                "allocation-intervals",
                "llvm-copy-sites",
                "independent-signal-oracle",
                "receipt-backed-public-claims",
            ]),
            Self::KaniModelCheck => string_vec(&[
                "harness-identity",
                "toolchain-identity",
                "solver-identity",
                "unwind-status",
                "contract-status",
                "stub-classification",
                "bounded-model-check-non-claims",
            ]),
            Self::HegelProperty => string_vec(&[
                "property-identity",
                "linked-requirement-ids",
                "source-scope",
                "runner-settings",
                "generator-identity",
                "counterexample-identity",
                "generated-testing-non-claims",
            ]),
            Self::Z4Solver => string_vec(&[
                "solver-identity",
                "query-identity",
                "logic-profile",
                "result-status",
                "proof-model-core-references",
                "unknown-timeout-diagnostics",
                "role-and-non-claims",
            ]),
            Self::VerifierFixture => string_vec(&[
                "fixture-identity",
                "syntax-family",
                "expected-verdict",
                "diagnostic-expectations",
                "assumption-and-unsupported-feature-labels",
                "role-and-non-claims",
            ]),
        }
    }

    fn non_claims(self) -> Vec<String> {
        match self {
            Self::DependencyAudit => string_vec(&[
                "license policy results are not legal advice",
                "advisory database state does not prove dependencies are vulnerability-free",
                "dependency audit evidence is optional and not part of starter cargo octet check",
            ]),
            Self::ClippyCoReporting => string_vec(&[
                "Clippy findings are complementary external hygiene evidence, not Octet source lint replacements",
                "source severity is preserved separately from Octet gate outcome",
            ]),
            Self::AstGrepStructural => string_vec(&[
                crate::ast_grep_rail::REQUIRED_NON_CLAIM,
                "ast-grep findings do not prove Rust type correctness, panic freedom, memory safety, proof discharge, release eligibility, or whole-program safety",
                "ast-grep codemods remain proposed diffs until focused validation accepts the declared migration scope",
            ]),
            Self::WasmArtifact => string_vec(&[
                crate::wasm_artifact::REQUIRED_NON_CLAIM,
                "Matching normalized interfaces do not prove semantic equivalence or identify one implementation",
                "Static validation does not prove sandbox effectiveness, capability authority, provenance truth, release eligibility, or whole-system safety",
                "Producer metadata is recorded input and is not provenance truth",
            ]),
            Self::CharonSemantic => string_vec(&[
                "Charon semantic analysis is optional and not part of starter cargo octet check",
                "Charon semantic evidence does not prove Charon or rustc correctness",
                "Charon semantic evidence does not prove effect absence, complete Rust support, complete runtime reachability, semantic equivalence, or whole-program correctness",
                "Charon semantic evidence does not produce formal-proof, property, complete-closure, or release-ready authority",
                "mechanized-llbc and charon-rudra remain recorded-only references and transfer no proof, implementation, or analysis authority",
                "Valence and Kamacite remain evidence and interchange boundaries rather than compiler IR or semantic-analysis authorities",
            ]),
            Self::DependencyTrust => string_vec(&[
                "cargo-vet audit coverage does not prove dependencies are benign or vulnerability-free",
                "dependency trust evidence remains separate from dependency audit and advisory evidence",
            ]),
            Self::ApiStability => string_vec(&[
                "API stability evidence covers selected public API checks only",
                "compatible semver output is not a whole-program behavior compatibility proof",
            ]),
            Self::SarifImport => string_vec(&[
                crate::external::sarif::REQUIRED_NON_CLAIM,
                "SARIF import is local-only and does not execute or attest the external producer",
                "source severity remains separate from Octet gate outcome",
            ]),
            Self::DynamicEvidence => string_vec(&[
                "dynamic evidence profiles are opt-in and not required by default cargo octet check",
                "Miri, Loom, profiling, or model-checking evidence does not prove whole-program correctness",
                crate::cargo::fuzz::REQUIRED_NON_CLAIM,
            ]),
            Self::ReleaseSupplyChain => string_vec(&[
                "local release receipts are SLSA/in-toto compatible shapes, not hosted-service attestations",
                "SBOM and vulnerability scan evidence does not prove the binary is vulnerability-free",
                "auditable dependency metadata does not prove source correctness",
            ]),
            Self::MutationTesting => string_vec(&[
                "mutation testing evidence samples the test suite sensitivity and does not prove the tests are complete",
                "survived or timed-out mutants require review before promotion rather than formal-proof treatment",
                "cargo-mutants evidence is optional and not part of starter cargo octet check",
            ]),
            Self::CoverageBaseline => string_vec(&[
                "coverage evidence measures executed code regions and does not prove behavioral correctness",
                "coverage baseline evidence is optional and not part of starter cargo octet check",
                "passing coverage thresholds does not replace review of uncovered critical paths",
                crate::coverage::REQUIRED_NON_CLAIM,
            ]),
            Self::BenchmarkEvidence => string_vec(&[
                "benchmark evidence is optional and not part of starter cargo octet check",
                "selected workloads and signal corpora do not represent all Rust projects, defects, or environments",
                "static copy sites are not runtime copy-volume observations",
                crate::benchmark::REQUIRED_NON_CLAIM,
            ]),
            Self::KaniModelCheck => string_vec(&[
                "Kani model-check evidence is optional and not part of starter cargo octet check",
                "Kani receipts are scoped to named harnesses, source hashes, solver identity, and declared unwind or contract status",
                "Kani evidence does not prove whole-program correctness, verifier soundness, concurrency correctness, unmodeled aliasing, or unmodeled FFI behavior",
                "plain Kani stubs are boundary assumptions unless linked to a passing verified contract receipt",
            ]),
            Self::HegelProperty => string_vec(&[
                "Hegel property evidence is optional and not part of starter cargo octet check",
                "Hegel receipts are scoped to named properties, source hashes, runner settings, generator identity, and case budgets",
                HEGEL_GENERATED_TESTING_NON_CLAIM,
            ]),
            Self::Z4Solver => string_vec(&[
                "Z4 solver evidence is optional and not part of starter cargo octet check",
                "Z4 receipts are scoped to solver identity, query hash, logic profile, result status, artifact references, diagnostics, roles, and explicit non-claims",
                Z4_SOLVER_NON_CLAIM,
            ]),
            Self::VerifierFixture => string_vec(&[
                "Verifier fixture intake is optional and not part of starter cargo octet check",
                "Verifier fixtures are normalized regression inputs and do not require external verifier runtimes",
                VERIFIER_FIXTURE_NON_CLAIM,
            ]),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRailFinding {
    pub family: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct HegelRunnerSummary {
    pub crate_version: String,
    pub backend: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct HegelSettingsSummary {
    pub case_budget: u64,
    pub phases: Vec<String>,
    pub seed: Option<u64>,
    pub derandomized: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct HegelGeneratorSummary {
    pub family: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct HegelCounterexampleSummary {
    pub summary_hash: String,
    pub issue_codes: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct HegelPropertyReceiptDetails {
    pub property_id: String,
    pub linked_ids: Vec<String>,
    pub source_scope: String,
    pub runner: HegelRunnerSummary,
    pub settings: HegelSettingsSummary,
    pub generator: HegelGeneratorSummary,
    pub source_hashes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counterexample: Option<HegelCounterexampleSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardened: Option<crate::hegel::ReceiptDetails>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Z4SolverIdentity {
    pub name: String,
    pub version: String,
    pub provenance: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Z4QueryIdentity {
    pub identity: String,
    pub logic: String,
    pub profile: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Z4SolverReceiptDetails {
    pub solver: Z4SolverIdentity,
    pub query: Z4QueryIdentity,
    pub result_status: String,
    pub verification_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsat_core_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unknown_reason: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifierFixtureEntry {
    pub fixture_id: String,
    pub source_artifact_ref: String,
    pub syntax_family: String,
    pub expected_verdict: String,
    pub expected_diagnostics: Vec<String>,
    pub assumptions: Vec<String>,
    pub unsupported_features: Vec<String>,
    pub verifier_family: String,
    pub fixture_role: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifierFixtureReceiptDetails {
    pub fixtures: Vec<VerifierFixtureEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRailReceipt {
    pub schema_version: String,
    pub rail_id: String,
    pub checker_id: String,
    pub mode: String,
    pub command_argv: Vec<String>,
    pub workspace_scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile_identity: Option<String>,
    pub checked_rule_families: Vec<String>,
    pub status: String,
    pub finding_counts: BTreeMap<String, u64>,
    pub bounded_log_identity: String,
    pub imported_input_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_database_update: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_exit_code: Option<i32>,
    pub non_claims: Vec<String>,
    pub diagnostics: Vec<String>,
    pub findings: Vec<EvidenceRailFinding>,
    pub parent_artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub property_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clippy_restrictions: Option<crate::clippy_restrictions::ClippyRestrictionReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ast_grep: Option<crate::ast_grep_rail::AstGrepReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm_artifact: Option<crate::wasm_artifact::WasmArtifactReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charon_semantic: Option<crate::charon_semantic::CharonSemanticReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hegel_property: Option<HegelPropertyReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z4_solver: Option<Z4SolverReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier_fixtures: Option<VerifierFixtureReceiptDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_fuzz: Option<crate::cargo::fuzz::ReceiptDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sarif_import: Option<crate::external::sarif::ReceiptDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_baseline: Option<crate::coverage::ReceiptDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_mutants: Option<crate::mutation::ReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmark_evidence: Option<crate::benchmark::ReceiptDetails>,
}

pub struct EvaluationInput<'a> {
    pub rail: EvidenceRailKind,
    pub mode: &'a str,
    pub import_text: &'a str,
    pub workspace_scope: &'a str,
    pub config_path: Option<&'a str>,
    pub baseline_identity: Option<String>,
    pub lockfile_identity: Option<String>,
    pub locked: bool,
    pub advisory_updated_by_prior_step: bool,
    pub profile: Option<&'a str>,
    pub requested_command: Option<&'a str>,
    pub parent_artifacts: Vec<String>,
    pub tool_exit_code: Option<i32>,
}

struct CoverageFindings {
    entries: Vec<EvidenceRailFinding>,
}

impl CoverageFindings {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(PROFILED_COVERAGE_FINDING_CAPACITY),
        }
    }

    fn push(&mut self, finding: EvidenceRailFinding) {
        self.entries.push(finding);
    }

    fn finish(self) -> Vec<EvidenceRailFinding> {
        self.entries
    }
}

struct InputIdentities {
    config: Option<String>,
    baseline: Option<String>,
    lockfile: Option<String>,
}

struct RailParseOutput {
    findings: Vec<EvidenceRailFinding>,
    finding_counts: BTreeMap<String, u64>,
    diagnostics: Vec<String>,
    status: String,
    profile: Option<String>,
    property_classes: Vec<String>,
    hegel_property: Option<HegelPropertyReceiptDetails>,
    z4_solver: Option<Z4SolverReceiptDetails>,
    verifier_fixtures: Option<VerifierFixtureReceiptDetails>,
}

pub fn run(args: &EvidenceRailArgs) -> Result<i32, String> {
    if args.output_format == OutputFormat::Sarif {
        return Err("evidence collect does not support SARIF output; use human or json".to_owned());
    }

    let workspace_root = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
    let workspace_scope = workspace_root.display().to_string();
    let mut config_path = args.config.as_ref().map(|path| path.display().to_string());
    let prepared_clippy_run = prepare_clippy_run(args)?;
    if let Some(prepared) = &prepared_clippy_run {
        config_path = Some(prepared.config_path.clone());
    }
    let (mode, import_text, tool_exit_code) = input_text(args, prepared_clippy_run.as_ref())?;
    let identities = input_identities(args, prepared_clippy_run.as_ref(), workspace_root, &import_text)?;
    let normalized_command = normalized_run_command(args, prepared_clippy_run.as_ref())?;
    let mut receipt = evaluate(EvaluationInput {
        rail: args.rail,
        mode,
        import_text: &import_text,
        workspace_scope: &workspace_scope,
        config_path: config_path.as_deref(),
        baseline_identity: identities.baseline,
        lockfile_identity: identities.lockfile,
        locked: args.locked,
        advisory_updated_by_prior_step: args.update_advisory_database,
        profile: args.profile.as_deref(),
        requested_command: normalized_command.as_deref(),
        parent_artifacts: args
            .parent_artifacts
            .iter()
            .filter(|parent| {
                args.rail != EvidenceRailKind::WasmArtifact
                    || !crate::wasm_artifact::is_external_handoff_identity(parent)
            })
            .cloned()
            .collect(),
        tool_exit_code,
    });
    receipt.config_identity = identities.config;
    receipt.clippy_restrictions = prepared_clippy_run.as_ref().map(|prepared| prepared.details.clone());

    let mut additional_artifacts = BTreeMap::new();
    if args.rail == EvidenceRailKind::WasmArtifact {
        let input = args.input.as_deref().ok_or_else(|| "wasm-artifact rail requires `--input PATH`".to_owned())?;
        let bytes = fs::read(input).map_err(|error| format!("read Wasm artifact {}: {error}", input.display()))?;
        additional_artifacts.insert(crate::wasm_artifact::INPUT_ARTIFACT_NAME.to_owned(), bytes);
    }
    if let Some(details) = receipt.charon_semantic.as_ref() {
        additional_artifacts.extend(crate::charon_semantic::artifact_payloads(details, &import_text)?);
    }
    if let Some(details) = receipt.sarif_import.as_ref() {
        additional_artifacts.insert(crate::external::sarif::RAW_ARTIFACT.to_owned(), import_text.as_bytes().to_vec());
        additional_artifacts.insert(
            crate::external::sarif::PROJECTION_ARTIFACT.to_owned(),
            crate::external::sarif::projection_bytes(details)?,
        );
    }
    if let Some(details) = receipt.coverage_baseline.as_ref() {
        additional_artifacts.extend(crate::coverage::request::artifact_payloads(details, &import_text)?);
    }
    if let Some(details) = receipt.cargo_mutants.as_ref() {
        additional_artifacts.extend(crate::mutation::request::artifact_payloads(details, &import_text)?);
    }
    if let Some(details) = receipt.benchmark_evidence.as_ref() {
        additional_artifacts.extend(crate::benchmark::request::artifact_payloads(details, &import_text)?);
    }
    if let Some(details) = receipt.hegel_property.as_ref().and_then(|details| details.hardened.as_ref()) {
        additional_artifacts.extend(crate::hegel::request::artifact_payloads(details, &import_text)?);
    }
    let receipt_json = render_receipt_json(&receipt)?;
    let summary = render_summary_for_command(&receipt, &args.command_line);
    let provenance_jsonl = render_provenance_jsonl(&receipt, &receipt_json, &summary)?;
    write_artifacts(&args.artifact_dir, &receipt_json, &summary, &provenance_jsonl, &additional_artifacts)?;
    render_cli_output(&receipt, args.output_format)?;

    Ok(exit_code_for_receipt(&receipt))
}

fn input_identities(
    args: &EvidenceRailArgs,
    prepared_clippy_run: Option<&PreparedClippyRun>,
    workspace_root: &Path,
    import_text: &str,
) -> Result<InputIdentities, String> {
    let is_profiled_coverage =
        args.rail == EvidenceRailKind::CoverageBaseline && args.profile.as_deref() == Some(crate::coverage::PROFILE_ID);
    let is_profiled_mutation =
        args.rail == EvidenceRailKind::MutationTesting && args.profile.as_deref() == Some(crate::mutation::PROFILE_ID);
    let is_profiled_benchmark = args.rail == EvidenceRailKind::BenchmarkEvidence
        && args.profile.as_deref() == Some(crate::benchmark::PROFILE_ID);
    let is_profiled_hegel =
        args.rail == EvidenceRailKind::HegelProperty && args.profile.as_deref() == Some(crate::hegel::PROFILE_ID);
    let config = if is_profiled_coverage {
        Some(crate::coverage::PROFILE_IDENTITY.to_owned())
    } else if is_profiled_mutation {
        Some(crate::mutation::PROFILE_IDENTITY.to_owned())
    } else if is_profiled_benchmark {
        Some(crate::benchmark::PROFILE_IDENTITY.to_owned())
    } else if is_profiled_hegel {
        Some(crate::hegel::PROFILE_IDENTITY.to_owned())
    } else if let Some(prepared) = prepared_clippy_run {
        Some(prepared.config_identity.clone())
    } else {
        args.config.as_ref().map(|path| path_identity(path)).transpose()?
    };
    let baseline = if is_profiled_coverage {
        crate::coverage::request::baseline_identity(import_text)
    } else if is_profiled_benchmark {
        crate::benchmark::request::baseline_identity(import_text)
    } else {
        args.baseline.as_ref().map(|path| path_identity(path)).transpose()?
    };
    Ok(InputIdentities {
        config,
        baseline,
        lockfile: path_identity_if_present(&workspace_root.join("Cargo.lock"))?,
    })
}

fn normalized_run_command(
    args: &EvidenceRailArgs,
    prepared_clippy_run: Option<&PreparedClippyRun>,
) -> Result<Option<String>, String> {
    if args.rail == EvidenceRailKind::HegelProperty && args.profile.as_deref() == Some(crate::hegel::PROFILE_ID) {
        return crate::hegel::selected_profile(crate::hegel::PROFILE_ID)
            .map(|profile| Some(profile.command_argv.join(" ")));
    }
    if !args.run {
        return Ok(args.command.clone());
    }
    match args.rail {
        EvidenceRailKind::CharonSemantic => Ok(Some(crate::charon_semantic::profile_command_argv()?.join(" "))),
        EvidenceRailKind::BenchmarkEvidence => Ok(None),
        EvidenceRailKind::HegelProperty if args.profile.as_deref() == Some(crate::hegel::PROFILE_ID) => {
            crate::hegel::selected_profile(crate::hegel::PROFILE_ID).map(|profile| Some(profile.command_argv.join(" ")))
        }
        EvidenceRailKind::AstGrepStructural | EvidenceRailKind::DynamicEvidence => {
            Ok(Some(run_mode_command_argv(args)?.join(" ")))
        }
        EvidenceRailKind::ClippyCoReporting => Ok(Some(
            prepared_clippy_run
                .map_or_else(default_clippy_command_argv, |prepared| prepared.command_argv.clone())
                .join(" "),
        )),
        _ => Ok(args.command.clone()),
    }
}

pub fn evaluate(input: EvaluationInput<'_>) -> EvidenceRailReceipt {
    let mut parsed = parse_rail_input(&input);
    let charon_semantic = if input.rail == EvidenceRailKind::CharonSemantic {
        crate::charon_semantic::evaluate_envelope(input.import_text).details
    } else {
        None
    };
    let ast_grep = if input.rail == EvidenceRailKind::AstGrepStructural {
        crate::ast_grep_rail::evaluate_envelope(input.import_text).details
    } else {
        None
    };
    let wasm_artifact = if input.rail == EvidenceRailKind::WasmArtifact {
        crate::wasm_artifact::evaluate_envelope(input.import_text).details
    } else {
        None
    };
    let cargo_fuzz = fuzz_details(&input);
    let sarif_import = sarif_details(&input);
    let coverage_baseline = coverage_details(&input);
    let cargo_mutants = mutation_details(&input);
    let benchmark_evidence = benchmark_details(&input);
    parsed.diagnostics.sort();
    parsed.diagnostics.dedup();
    let status = if parsed.diagnostics.is_empty() {
        parsed.status
    } else {
        STATUS_FAILED.to_owned()
    };
    let bounded_log_identity = receipt_log_identity(&input);

    EvidenceRailReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
        rail_id: input.rail.as_str().to_owned(),
        checker_id: input.rail.checker_id().to_owned(),
        mode: input.mode.to_owned(),
        command_argv: receipt_command_argv(
            &input,
            cargo_fuzz.as_ref(),
            coverage_baseline.as_ref(),
            cargo_mutants.as_ref(),
            benchmark_evidence.as_ref(),
        ),
        workspace_scope: input.workspace_scope.to_owned(),
        config_path: input.config_path.map(ToOwned::to_owned),
        config_identity: None,
        baseline_identity: input.baseline_identity,
        lockfile_identity: input.lockfile_identity,
        checked_rule_families: input.rail.checked_rule_families(),
        status,
        finding_counts: parsed.finding_counts,
        bounded_log_identity,
        imported_input_identity: prefixed_identity(input.import_text.as_bytes()),
        advisory_database_update: advisory_database_update(
            input.rail,
            input.locked,
            input.advisory_updated_by_prior_step,
        ),
        profile: parsed.profile.or_else(|| input.profile.map(ToOwned::to_owned)),
        tool_exit_code: input.tool_exit_code,
        non_claims: input.rail.non_claims(),
        diagnostics: parsed.diagnostics,
        findings: parsed.findings,
        parent_artifacts: input.parent_artifacts,
        property_classes: parsed.property_classes,
        clippy_restrictions: None,
        ast_grep,
        wasm_artifact,
        charon_semantic,
        hegel_property: parsed.hegel_property,
        z4_solver: parsed.z4_solver,
        verifier_fixtures: parsed.verifier_fixtures,
        cargo_fuzz,
        sarif_import,
        coverage_baseline,
        cargo_mutants,
        benchmark_evidence,
    }
}

fn receipt_log_identity(input: &EvaluationInput<'_>) -> String {
    crate::coverage::request::bounded_log_identity(input.import_text)
        .or_else(|| crate::mutation::request::bounded_log_identity(input.import_text))
        .or_else(|| crate::benchmark::request::bounded_log_identity(input.import_text))
        .or_else(|| crate::hegel::request::bounded_log_identity(input.import_text))
        .unwrap_or_else(|| {
            if input.import_text.is_empty() {
                EMPTY_LOG_IDENTITY.to_owned()
            } else {
                prefixed_identity(input.import_text.as_bytes())
            }
        })
}

fn fuzz_details(input: &EvaluationInput<'_>) -> Option<crate::cargo::fuzz::ReceiptDetails> {
    if input.rail == EvidenceRailKind::DynamicEvidence && input.profile == Some(crate::cargo::fuzz::PROFILE_ID) {
        crate::cargo::fuzz::evaluate_envelope(input.import_text).details
    } else {
        None
    }
}

fn sarif_details(input: &EvaluationInput<'_>) -> Option<crate::external::sarif::ReceiptDetails> {
    if input.rail == EvidenceRailKind::SarifImport {
        input
            .profile
            .and_then(|profile| crate::external::sarif::evaluate(input.import_text, profile).details)
    } else {
        None
    }
}

fn coverage_details(input: &EvaluationInput<'_>) -> Option<crate::coverage::ReceiptDetails> {
    if input.rail == EvidenceRailKind::CoverageBaseline && input.profile == Some(crate::coverage::PROFILE_ID) {
        crate::coverage::request::evaluate(input.import_text, crate::coverage::PROFILE_ID).details
    } else {
        None
    }
}

fn mutation_details(input: &EvaluationInput<'_>) -> Option<crate::mutation::ReceiptDetails> {
    if input.rail == EvidenceRailKind::MutationTesting && input.profile == Some(crate::mutation::PROFILE_ID) {
        crate::mutation::request::evaluate(input.import_text, crate::mutation::PROFILE_ID).details
    } else {
        None
    }
}

fn benchmark_details(input: &EvaluationInput<'_>) -> Option<crate::benchmark::ReceiptDetails> {
    if input.rail == EvidenceRailKind::BenchmarkEvidence && input.profile == Some(crate::benchmark::PROFILE_ID) {
        crate::benchmark::request::evaluate(input.import_text, crate::benchmark::PROFILE_ID).details
    } else {
        None
    }
}

fn receipt_command_argv(
    input: &EvaluationInput<'_>,
    fuzz: Option<&crate::cargo::fuzz::ReceiptDetails>,
    coverage: Option<&crate::coverage::ReceiptDetails>,
    mutation: Option<&crate::mutation::ReceiptDetails>,
    benchmark: Option<&crate::benchmark::ReceiptDetails>,
) -> Vec<String> {
    if let Some(details) = fuzz {
        return details.command_argv.clone();
    }
    if let Some(details) = coverage {
        return details.command_argv.clone();
    }
    if let Some(details) = mutation {
        return details.command_argv.clone();
    }
    if let Some(details) = benchmark {
        return details.command_argv.clone();
    }
    if matches!(input.rail, EvidenceRailKind::CharonSemantic | EvidenceRailKind::SarifImport)
        && input.mode == MODE_IMPORT
    {
        return Vec::new();
    }
    command_argv(input.rail, input.config_path, input.baseline_identity.as_deref(), input.requested_command)
}

pub fn render_receipt_json(receipt: &EvidenceRailReceipt) -> Result<String, String> {
    let mut rendered =
        serde_json::to_string_pretty(receipt).map_err(|error| format!("serialize evidence receipt: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn render_summary(receipt: &EvidenceRailReceipt) -> String {
    let mut summary = String::new();
    summary.push_str("--- octet evidence rail summary ---\n");
    summary.push_str(&format!("Rail: {}\n", receipt.rail_id));
    summary.push_str(&format!("Checker: {}\n", receipt.checker_id));
    summary.push_str(&format!("Status: {}\n", receipt.status));
    summary.push_str(&format!("Mode: {}\n", receipt.mode));
    summary.push_str(&format!("Workspace scope: {}\n", receipt.workspace_scope));
    summary.push_str(&format!("Input identity: {}\n", receipt.imported_input_identity));
    summary.push_str("Finding counts:\n");
    for (family, count) in &receipt.finding_counts {
        summary.push_str(&format!("  {family}: {count}\n"));
    }
    summary.push_str("Non-claims:\n");
    for non_claim in &receipt.non_claims {
        summary.push_str(&format!("  - {non_claim}\n"));
    }
    if !receipt.diagnostics.is_empty() {
        summary.push_str("Diagnostics:\n");
        for diagnostic in &receipt.diagnostics {
            summary.push_str(&format!("  - {diagnostic}\n"));
        }
    }
    summary.push_str("---\n");
    summary
}

fn render_summary_for_command(receipt: &EvidenceRailReceipt, command_line: &str) -> String {
    let mut summary = render_summary(receipt);
    if command_line.trim().is_empty() {
        return summary;
    }
    if let Some(prefix) = summary.strip_suffix("---\n") {
        summary = format!("{prefix}Octet command: {command_line}\n---\n");
    }
    summary
}

pub fn render_provenance_jsonl(
    receipt: &EvidenceRailReceipt,
    receipt_json: &str,
    summary: &str,
) -> Result<String, String> {
    let mut receipt_parents = receipt.parent_artifacts.clone();
    let mut records = Vec::new();
    let (specialized_records, specialized_parents) = specialized_provenance(receipt);
    records.extend(specialized_records);
    receipt_parents.extend(specialized_parents);
    records.push(ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: source_id_for_receipt(receipt),
        checker_id: receipt.checker_id.clone(),
        evidence_tier: receipt_tier(receipt),
        artifact_locator: RECEIPT_ARTIFACT_NAME.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(receipt_json.as_bytes()),
        checked_scope: receipt.workspace_scope.clone(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: None,
        parent_artifacts: receipt_parents,
    });
    records.push(ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: "octet.runner.artifact-bundle".to_owned(),
        checker_id: crate::evidence::CHECKER_CARGO_OCTET.to_owned(),
        evidence_tier: EvidenceTier::ManualReviewRequired,
        artifact_locator: SUMMARY_ARTIFACT_NAME.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(summary.as_bytes()),
        checked_scope: receipt.workspace_scope.clone(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: Some(SUMMARY_REVIEW_ACTION.to_owned()),
        parent_artifacts: vec![RECEIPT_ARTIFACT_NAME.to_owned()],
    });
    let mut rendered = String::new();
    for record in records {
        rendered.push_str(&serde_json::to_string(&record).map_err(|error| format!("serialize provenance: {error}"))?);
        rendered.push('\n');
    }
    Ok(rendered)
}

fn specialized_provenance(receipt: &EvidenceRailReceipt) -> (Vec<ProvenanceRecord>, Vec<String>) {
    let mut records = Vec::new();
    let mut parents = Vec::new();
    if let Some(details) = receipt.wasm_artifact.as_ref() {
        records.push(ProvenanceRecord {
            schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: "octet.artifact.wasm-input".to_owned(),
            checker_id: crate::evidence::CHECKER_CARGO_OCTET.to_owned(),
            evidence_tier: EvidenceTier::PolicyCheck,
            artifact_locator: crate::wasm_artifact::INPUT_ARTIFACT_NAME.to_owned(),
            blake3: details
                .exact_artifact_identity
                .strip_prefix("b3:")
                .unwrap_or(&details.exact_artifact_identity)
                .to_owned(),
            checked_scope: receipt.workspace_scope.clone(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: None,
            parent_artifacts: Vec::new(),
        });
        parents.push(crate::wasm_artifact::INPUT_ARTIFACT_NAME.to_owned());
    }
    if let Some(details) = receipt.charon_semantic.as_ref() {
        for artifact in &details.artifacts {
            let parent_artifacts = match artifact.role.as_str() {
                "snapshot" => Vec::new(),
                "projection" => vec![crate::charon_semantic::RAW_SNAPSHOT_ARTIFACT.to_owned()],
                "comparison" | "analysis" => vec![crate::charon_semantic::PROJECTION_ARTIFACT.to_owned()],
                _ => Vec::new(),
            };
            records.push(ProvenanceRecord {
                schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
                source_id: format!("octet.charon_semantic.{}", artifact.role),
                checker_id: crate::evidence::CHECKER_CARGO_OCTET.to_owned(),
                evidence_tier: EvidenceTier::TranscriptAssertion,
                artifact_locator: artifact.name.clone(),
                blake3: artifact.identity.strip_prefix("b3:").unwrap_or(&artifact.identity).to_owned(),
                checked_scope: receipt.workspace_scope.clone(),
                verifier_identity: None,
                proof_artifact_identity: None,
                human_action: None,
                parent_artifacts,
            });
            parents.push(artifact.name.clone());
        }
    }
    if let Some(details) = receipt.sarif_import.as_ref() {
        records.extend(sarif_provenance(details, &receipt.workspace_scope));
        parents.extend([
            crate::external::sarif::RAW_ARTIFACT.to_owned(),
            crate::external::sarif::PROJECTION_ARTIFACT.to_owned(),
        ]);
    }
    if let Some(details) = receipt.coverage_baseline.as_ref() {
        records.extend(coverage_provenance(details, &receipt.workspace_scope));
        parents.extend([
            crate::coverage::SOURCE_MANIFEST_ARTIFACT.to_owned(),
            crate::coverage::CURRENT_ENVELOPE_ARTIFACT.to_owned(),
            crate::coverage::BASELINE_ENVELOPE_ARTIFACT.to_owned(),
            crate::coverage::PROJECTION_ARTIFACT.to_owned(),
        ]);
    }
    if let Some(details) = receipt.cargo_mutants.as_ref() {
        let mutation_records = mutation_provenance(details, &receipt.workspace_scope);
        parents.extend(mutation_records.iter().map(|record| record.artifact_locator.clone()));
        records.extend(mutation_records);
    }
    if let Some(details) = receipt.benchmark_evidence.as_ref() {
        let benchmark_records = benchmark_provenance(details, &receipt.workspace_scope);
        parents.extend(benchmark_records.iter().map(|record| record.artifact_locator.clone()));
        records.extend(benchmark_records);
    }
    if let Some(details) = receipt.hegel_property.as_ref().and_then(|details| details.hardened.as_ref()) {
        let hegel_records = hegel_provenance(details, &receipt.workspace_scope);
        parents.extend(hegel_records.iter().map(|record| record.artifact_locator.clone()));
        records.extend(hegel_records);
    }
    (records, parents)
}

fn sarif_provenance(details: &crate::external::sarif::ReceiptDetails, checked_scope: &str) -> Vec<ProvenanceRecord> {
    vec![
        ProvenanceRecord {
            schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: "octet.external_lint.sarif-raw".to_owned(),
            checker_id: crate::evidence::CHECKER_SARIF_IMPORT.to_owned(),
            evidence_tier: EvidenceTier::StaticLint,
            artifact_locator: crate::external::sarif::RAW_ARTIFACT.to_owned(),
            blake3: details.raw_report_identity.trim_start_matches("b3:").to_owned(),
            checked_scope: checked_scope.to_owned(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: None,
            parent_artifacts: Vec::new(),
        },
        ProvenanceRecord {
            schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: "octet.external_lint.sarif-projection".to_owned(),
            checker_id: crate::evidence::CHECKER_SARIF_IMPORT.to_owned(),
            evidence_tier: EvidenceTier::StaticLint,
            artifact_locator: crate::external::sarif::PROJECTION_ARTIFACT.to_owned(),
            blake3: details.projection_identity.trim_start_matches("b3:").to_owned(),
            checked_scope: checked_scope.to_owned(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: None,
            parent_artifacts: vec![crate::external::sarif::RAW_ARTIFACT.to_owned()],
        },
    ]
}

fn coverage_provenance(details: &crate::coverage::ReceiptDetails, checked_scope: &str) -> Vec<ProvenanceRecord> {
    let source_identity =
        details.current.source_identity.strip_prefix("source:").unwrap_or(&details.current.source_identity);
    let record = |source: &str, locator: &str, identity: &str, parents: Vec<String>| ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: source.to_owned(),
        checker_id: crate::evidence::CHECKER_CARGO_LLVM_COV.to_owned(),
        evidence_tier: EvidenceTier::TranscriptAssertion,
        artifact_locator: locator.to_owned(),
        blake3: identity.trim_start_matches("b3:").to_owned(),
        checked_scope: checked_scope.to_owned(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: None,
        parent_artifacts: parents,
    };
    vec![
        record(
            "octet.coverage_baseline.source-manifest",
            crate::coverage::SOURCE_MANIFEST_ARTIFACT,
            source_identity,
            Vec::new(),
        ),
        record(
            "octet.coverage_baseline.current-raw",
            crate::coverage::CURRENT_RAW_ARTIFACT,
            &details.current.raw_report_identity,
            vec![crate::coverage::SOURCE_MANIFEST_ARTIFACT.to_owned()],
        ),
        record(
            "octet.coverage_baseline.current-envelope",
            crate::coverage::CURRENT_ENVELOPE_ARTIFACT,
            &details.current_envelope_identity,
            vec![
                crate::coverage::SOURCE_MANIFEST_ARTIFACT.to_owned(),
                crate::coverage::CURRENT_RAW_ARTIFACT.to_owned(),
            ],
        ),
        record(
            "octet.coverage_baseline.accepted-envelope",
            crate::coverage::BASELINE_ENVELOPE_ARTIFACT,
            &details.baseline_envelope_identity,
            vec![crate::coverage::SOURCE_MANIFEST_ARTIFACT.to_owned()],
        ),
        record(
            "octet.coverage_baseline.comparison",
            crate::coverage::PROJECTION_ARTIFACT,
            &details.projection_identity,
            vec![
                crate::coverage::CURRENT_ENVELOPE_ARTIFACT.to_owned(),
                crate::coverage::BASELINE_ENVELOPE_ARTIFACT.to_owned(),
            ],
        ),
    ]
}

fn benchmark_provenance(details: &crate::benchmark::ReceiptDetails, checked_scope: &str) -> Vec<ProvenanceRecord> {
    let record = |source: &str, locator: &str, identity: &str, parents: Vec<String>| ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: source.to_owned(),
        checker_id: crate::evidence::CHECKER_OCTET_BENCHMARK.to_owned(),
        evidence_tier: EvidenceTier::TranscriptAssertion,
        artifact_locator: locator.to_owned(),
        blake3: identity.trim_start_matches("b3:").to_owned(),
        checked_scope: checked_scope.to_owned(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: None,
        parent_artifacts: parents,
    };
    let raw = [
        ("baseline", crate::benchmark::BASELINE_ARTIFACT, details.baseline_identity.as_str()),
        ("candidate", crate::benchmark::CANDIDATE_ARTIFACT, details.candidate_identity.as_str()),
        ("allocations", crate::benchmark::ALLOCATION_ARTIFACT, details.allocation_identity.as_str()),
        ("copy-sites", crate::benchmark::COPY_ARTIFACT, details.copy_identity.as_str()),
        ("signal-oracle", crate::benchmark::ORACLE_ARTIFACT, details.oracle_identity.as_str()),
        ("signal-findings", crate::benchmark::FINDINGS_ARTIFACT, details.findings_identity.as_str()),
    ];
    let mut records: Vec<ProvenanceRecord> = raw
        .iter()
        .map(|(role, locator, identity)| {
            record(&format!("octet.benchmark_evidence.{role}"), locator, identity, Vec::new())
        })
        .collect();
    records.push(record(
        "octet.benchmark_evidence.projection",
        crate::benchmark::PROJECTION_ARTIFACT,
        &details.projection_identity,
        raw.iter().map(|(_, locator, _)| (*locator).to_owned()).collect(),
    ));
    records
}

fn hegel_provenance(details: &crate::hegel::ReceiptDetails, checked_scope: &str) -> Vec<ProvenanceRecord> {
    let record = |role: &str, locator: &str, identity: &str, parents: Vec<String>| ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: format!("octet.dynamic_evidence.hegel-property.{role}"),
        checker_id: crate::evidence::CHECKER_HEGEL_RS.to_owned(),
        evidence_tier: EvidenceTier::TranscriptAssertion,
        artifact_locator: locator.to_owned(),
        blake3: identity.trim_start_matches("b3:").to_owned(),
        checked_scope: checked_scope.to_owned(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: None,
        parent_artifacts: parents,
    };
    let raw = [
        ("summary", crate::hegel::SUMMARY_ARTIFACT, details.summary_identity.as_str()),
        ("bounded-log", crate::hegel::LOG_ARTIFACT, details.bounded_log_identity.as_str()),
        ("source-manifest", crate::hegel::SOURCE_MANIFEST_ARTIFACT, details.source_manifest_identity.as_str()),
        (
            "generator-manifest",
            crate::hegel::GENERATOR_MANIFEST_ARTIFACT,
            details.generator_manifest_identity.as_str(),
        ),
    ];
    let mut records: Vec<ProvenanceRecord> =
        raw.iter().map(|(role, locator, identity)| record(role, locator, identity, Vec::new())).collect();
    if let Some(identity) = details.counterexample_identity.as_ref() {
        records.push(record("counterexample", crate::hegel::COUNTEREXAMPLE_ARTIFACT, identity, vec![
            crate::hegel::SUMMARY_ARTIFACT.to_owned(),
        ]));
    }
    let mut parents = raw.iter().map(|(_, locator, _)| (*locator).to_owned()).collect::<Vec<_>>();
    if details.counterexample_identity.is_some() {
        parents.push(crate::hegel::COUNTEREXAMPLE_ARTIFACT.to_owned());
    }
    records.push(record("projection", crate::hegel::PROJECTION_ARTIFACT, &details.projection_identity, parents));
    records
}

fn mutation_provenance(details: &crate::mutation::ReceiptDetails, checked_scope: &str) -> Vec<ProvenanceRecord> {
    let record = |source: &str, locator: &str, identity: &str, parents: Vec<String>| ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: source.to_owned(),
        checker_id: crate::evidence::CHECKER_CARGO_MUTANTS.to_owned(),
        evidence_tier: EvidenceTier::TranscriptAssertion,
        artifact_locator: locator.to_owned(),
        blake3: identity.trim_start_matches("b3:").to_owned(),
        checked_scope: checked_scope.to_owned(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: None,
        parent_artifacts: parents,
    };
    let mut records = vec![
        record(
            "octet.mutation_testing.source-manifest",
            crate::mutation::SOURCE_MANIFEST_ARTIFACT,
            &details.source_manifest_identity,
            Vec::new(),
        ),
        record(
            "octet.mutation_testing.native-outcomes",
            crate::mutation::OUTCOMES_ARTIFACT,
            &details.native_outcomes_identity,
            vec![crate::mutation::SOURCE_MANIFEST_ARTIFACT.to_owned()],
        ),
        record(
            "octet.mutation_testing.native-inventory",
            crate::mutation::INVENTORY_ARTIFACT,
            &details.native_inventory_identity,
            vec![crate::mutation::SOURCE_MANIFEST_ARTIFACT.to_owned()],
        ),
        record(
            "octet.mutation_testing.projection",
            crate::mutation::PROJECTION_ARTIFACT,
            &details.projection_identity,
            vec![
                crate::mutation::OUTCOMES_ARTIFACT.to_owned(),
                crate::mutation::INVENTORY_ARTIFACT.to_owned(),
            ],
        ),
        record(
            "octet.mutation_testing.baseline-log",
            &details.baseline.log.locator,
            &details.baseline.log.identity,
            vec![crate::mutation::OUTCOMES_ARTIFACT.to_owned()],
        ),
    ];
    for mutant in &details.mutants {
        records.push(record("octet.mutation_testing.mutant-log", &mutant.log.locator, &mutant.log.identity, vec![
            crate::mutation::OUTCOMES_ARTIFACT.to_owned(),
        ]));
        if let Some(diff) = mutant.diff.as_ref() {
            records.push(record("octet.mutation_testing.mutant-diff", &diff.locator, &diff.identity, vec![
                crate::mutation::INVENTORY_ARTIFACT.to_owned(),
            ]));
        }
    }
    records
}

pub fn validate_receipt(receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported evidence rail schema_version `{}`; expected `{RECEIPT_SCHEMA_VERSION}`",
            receipt.schema_version
        ));
    }
    let rail = match EvidenceRailKind::parse(&receipt.rail_id) {
        Ok(rail) => rail,
        Err(error) => {
            diagnostics.push(error);
            return diagnostics;
        }
    };
    if receipt.checker_id != rail.checker_id() {
        diagnostics.push(format!(
            "evidence rail `{}` must use checker_id `{}`, got `{}`",
            receipt.rail_id,
            rail.checker_id(),
            receipt.checker_id
        ));
    }
    if receipt.workspace_scope.trim().is_empty() {
        diagnostics.push("evidence rail receipt missing workspace_scope".to_owned());
    }
    if receipt.imported_input_identity.trim().is_empty() || receipt.bounded_log_identity.trim().is_empty() {
        diagnostics.push("evidence rail receipt missing BLAKE3 input identity".to_owned());
    }
    if receipt.non_claims.is_empty() {
        diagnostics.push("evidence rail receipt must carry at least one explicit non-claim".to_owned());
    }
    if receipt.config_identity.as_deref().is_some_and(|identity| !is_b3_identity(identity)) {
        diagnostics.push("evidence rail config_identity must be a BLAKE3 identity".to_owned());
    }
    validate_clippy_restriction_receipt(rail, receipt, &mut diagnostics);
    validate_ast_grep_receipt(rail, receipt, &mut diagnostics);
    validate_wasm_artifact_receipt(rail, receipt, &mut diagnostics);
    validate_charon_semantic_receipt(rail, receipt, &mut diagnostics);
    diagnostics.extend(hegel_diagnostics(rail, receipt));
    diagnostics.extend(z4_diagnostics(rail, receipt));
    diagnostics.extend(verifier_diagnostics(rail, receipt));
    diagnostics.extend(fuzz_diagnostics(rail, receipt));
    diagnostics.extend(sarif_diagnostics(rail, receipt));
    diagnostics.extend(coverage_diagnostics(rail, receipt));
    diagnostics.extend(mutation_diagnostics(rail, receipt));
    diagnostics.extend(benchmark_diagnostics(rail, receipt));
    diagnostics
}

fn hegel_diagnostics(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SPECIALIZED_DIAGNOSTIC_CAPACITY);
    match (rail, receipt.hegel_property.as_ref()) {
        (EvidenceRailKind::HegelProperty, Some(details)) => {
            if details.property_id.trim().is_empty() {
                diagnostics.push("hegel-property receipt missing property_id".to_owned());
            }
            if !receipt.non_claims.iter().any(|claim| claim == HEGEL_GENERATED_TESTING_NON_CLAIM) {
                diagnostics.push("hegel-property receipt missing generated-testing non-claim".to_owned());
            }
            if let Some(hardened) = details.hardened.as_ref() {
                diagnostics.extend(crate::hegel::validate(hardened));
                if receipt.profile.as_deref() != Some(crate::hegel::PROFILE_ID)
                    || receipt.config_identity.as_deref() != Some(crate::hegel::PROFILE_IDENTITY)
                    || receipt.command_argv != hardened.command_argv
                {
                    diagnostics.push("profiled Hegel receipt profile, config, or command identity is stale".to_owned());
                }
            } else if receipt.profile.as_deref() == Some(crate::hegel::PROFILE_ID) {
                diagnostics.push("profiled Hegel receipt is missing hardened measurements".to_owned());
            }
        }
        (EvidenceRailKind::HegelProperty, None) => {
            diagnostics.push("hegel-property receipt missing hegel_property details".to_owned());
        }
        (_, Some(_)) => {
            diagnostics.push("hegel_property details are only valid on hegel-property rail receipts".to_owned());
        }
        (_, None) => {}
    }
    diagnostics
}

fn z4_diagnostics(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SPECIALIZED_DIAGNOSTIC_CAPACITY);
    match (rail, receipt.z4_solver.as_ref()) {
        (EvidenceRailKind::Z4Solver, Some(details)) => {
            if details.solver.name.trim().is_empty() {
                diagnostics.push("z4-solver receipt missing solver name".to_owned());
            }
            if !is_b3_identity(&details.query.identity) {
                diagnostics.push("z4-solver receipt query identity must be a BLAKE3 identity".to_owned());
            }
            if !receipt.non_claims.iter().any(|claim| claim == Z4_SOLVER_NON_CLAIM) {
                diagnostics.push("z4-solver receipt missing solver non-claim".to_owned());
            }
        }
        (EvidenceRailKind::Z4Solver, None) => {
            diagnostics.push("z4-solver receipt missing z4_solver details".to_owned())
        }
        (_, Some(_)) => diagnostics.push("z4_solver details are only valid on z4-solver rail receipts".to_owned()),
        (_, None) => {}
    }
    diagnostics
}

fn verifier_diagnostics(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SPECIALIZED_DIAGNOSTIC_CAPACITY);
    match (rail, receipt.verifier_fixtures.as_ref()) {
        (EvidenceRailKind::VerifierFixture, Some(details)) => {
            if details.fixtures.is_empty() {
                diagnostics.push("verifier-fixture receipt must contain fixtures".to_owned());
            }
            if !receipt.non_claims.iter().any(|claim| claim == VERIFIER_FIXTURE_NON_CLAIM) {
                diagnostics.push("verifier-fixture receipt missing fixture non-claim".to_owned());
            }
        }
        (EvidenceRailKind::VerifierFixture, None) => {
            diagnostics.push("verifier-fixture receipt missing verifier_fixtures details".to_owned());
        }
        (_, Some(_)) => {
            diagnostics.push("verifier_fixtures details are only valid on verifier-fixture rail receipts".to_owned());
        }
        (_, None) => {}
    }
    diagnostics
}

fn fuzz_diagnostics(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SPECIALIZED_DIAGNOSTIC_CAPACITY);
    let is_selected =
        rail == EvidenceRailKind::DynamicEvidence && receipt.profile.as_deref() == Some(crate::cargo::fuzz::PROFILE_ID);
    match (is_selected, receipt.cargo_fuzz.as_ref()) {
        (true, Some(details)) => {
            diagnostics.extend(crate::cargo::fuzz::validate(details));
            if receipt.command_argv != details.command_argv {
                diagnostics.push("cargo-fuzz receipt command does not match typed details".to_owned());
            }
            if !receipt.non_claims.iter().any(|claim| claim == crate::cargo::fuzz::REQUIRED_NON_CLAIM) {
                diagnostics.push("cargo-fuzz evidence receipt is missing its bounded non-claim".to_owned());
            }
        }
        (true, None) => diagnostics.push("cargo-fuzz dynamic receipt is missing cargo_fuzz details".to_owned()),
        (false, Some(_)) => diagnostics.push("cargo_fuzz details require the cargo-fuzz dynamic profile".to_owned()),
        (false, None) => {}
    }
    diagnostics
}

fn sarif_diagnostics(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SPECIALIZED_DIAGNOSTIC_CAPACITY);
    match (rail, receipt.sarif_import.as_ref()) {
        (EvidenceRailKind::SarifImport, Some(details)) => {
            if receipt.profile.as_deref() != Some(details.profile_id.as_str()) {
                diagnostics.push("SARIF receipt profile does not match typed details".to_owned());
            }
            if details.profile_identity != crate::external::sarif::PROFILE_IDENTITY {
                diagnostics.push("SARIF receipt profile identity is stale".to_owned());
            }
            if receipt.config_identity.as_deref() != Some(crate::external::sarif::PROFILE_IDENTITY) {
                diagnostics.push("SARIF receipt config identity is stale".to_owned());
            }
            if details.raw_report_identity != receipt.imported_input_identity {
                diagnostics.push("SARIF raw report identity does not match imported bytes".to_owned());
            }
            match crate::external::sarif::projection_bytes(details) {
                Ok(bytes) if crate::external::sarif::prefixed_identity(&bytes) == details.projection_identity => {}
                Ok(_) => diagnostics.push("SARIF normalized projection identity is stale".to_owned()),
                Err(error) => diagnostics.push(error),
            }
            if !receipt.non_claims.iter().any(|claim| claim == crate::external::sarif::REQUIRED_NON_CLAIM) {
                diagnostics.push("SARIF receipt is missing the required non-claim".to_owned());
            }
        }
        (EvidenceRailKind::SarifImport, None) => {
            diagnostics.push("SARIF import receipt is missing typed details".to_owned())
        }
        (_, Some(_)) => diagnostics.push("sarif_import details require the SARIF import rail".to_owned()),
        (_, None) => {}
    }
    diagnostics
}

fn coverage_diagnostics(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SPECIALIZED_DIAGNOSTIC_CAPACITY);
    let is_selected =
        rail == EvidenceRailKind::CoverageBaseline && receipt.profile.as_deref() == Some(crate::coverage::PROFILE_ID);
    match (is_selected, receipt.coverage_baseline.as_ref()) {
        (true, Some(details)) => {
            diagnostics.extend(crate::coverage::validate(details));
            if receipt.config_identity.as_deref() != Some(crate::coverage::PROFILE_IDENTITY) {
                diagnostics.push("coverage receipt config identity is stale".to_owned());
            }
            if receipt.baseline_identity.as_deref() != Some(details.baseline_envelope_identity.as_str()) {
                diagnostics.push("coverage receipt baseline envelope identity is stale".to_owned());
            }
            if receipt.command_argv != details.command_argv {
                diagnostics.push("coverage receipt command does not match typed details".to_owned());
            }
        }
        (true, None) => diagnostics.push("profiled coverage receipt is missing typed details".to_owned()),
        (false, Some(_)) => {
            diagnostics.push("coverage_baseline details require the checked coverage profile".to_owned())
        }
        (false, None) => {}
    }
    diagnostics
}

fn mutation_diagnostics(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SPECIALIZED_DIAGNOSTIC_CAPACITY);
    let is_selected =
        rail == EvidenceRailKind::MutationTesting && receipt.profile.as_deref() == Some(crate::mutation::PROFILE_ID);
    match (is_selected, receipt.cargo_mutants.as_ref()) {
        (true, Some(details)) => {
            diagnostics.extend(crate::mutation::validate(details));
            if receipt.config_identity.as_deref() != Some(crate::mutation::PROFILE_IDENTITY) {
                diagnostics.push("mutation receipt config identity is stale".to_owned());
            }
            if receipt.command_argv != details.command_argv {
                diagnostics.push("mutation receipt command does not match typed details".to_owned());
            }
        }
        (true, None) => diagnostics.push("profiled mutation receipt is missing typed details".to_owned()),
        (false, Some(_)) => diagnostics.push("cargo_mutants details require the checked mutation profile".to_owned()),
        (false, None) => {}
    }
    diagnostics
}

fn benchmark_diagnostics(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SPECIALIZED_DIAGNOSTIC_CAPACITY);
    let is_selected =
        rail == EvidenceRailKind::BenchmarkEvidence && receipt.profile.as_deref() == Some(crate::benchmark::PROFILE_ID);
    match (is_selected, receipt.benchmark_evidence.as_ref()) {
        (true, Some(details)) => {
            diagnostics.extend(crate::benchmark::validate(details));
            if receipt.config_identity.as_deref() != Some(crate::benchmark::PROFILE_IDENTITY) {
                diagnostics.push("benchmark receipt config identity is stale".to_owned());
            }
            if receipt.baseline_identity.as_deref() != Some(details.baseline_identity.as_str()) {
                diagnostics.push("benchmark receipt baseline identity is stale".to_owned());
            }
            if receipt.command_argv != details.command_argv {
                diagnostics.push("benchmark receipt command does not match typed details".to_owned());
            }
        }
        (true, None) => diagnostics.push("profiled benchmark receipt is missing typed details".to_owned()),
        (false, Some(_)) => {
            diagnostics.push("benchmark_evidence details require the checked benchmark profile".to_owned());
        }
        (false, None) => {}
    }
    diagnostics
}

fn validate_clippy_restriction_receipt(
    rail: EvidenceRailKind,
    receipt: &EvidenceRailReceipt,
    diagnostics: &mut Vec<String>,
) {
    match (rail, receipt.clippy_restrictions.as_ref()) {
        (EvidenceRailKind::ClippyCoReporting, Some(details)) => {
            diagnostics.extend(crate::clippy_restrictions::receipt_details_diagnostics(details));
            if receipt.mode != MODE_RUN {
                diagnostics.push("Clippy restriction profile details are only valid for run-mode receipts".to_owned());
            }
            if receipt.profile.as_deref() != Some(details.profile.profile_id.as_str()) {
                diagnostics
                    .push("Clippy restriction receipt profile does not match selected profile details".to_owned());
            }
            if receipt.config_path.as_deref().is_none_or(str::is_empty) {
                diagnostics.push("profiled Clippy restriction receipt missing config_path".to_owned());
            }
            if receipt
                .config_identity
                .as_deref()
                .is_none_or(|identity| !crate::clippy_restrictions::is_blake3_identity(identity))
            {
                diagnostics.push("profiled Clippy restriction receipt missing BLAKE3 config_identity".to_owned());
            }
            let expected_command = crate::clippy_restrictions::command_argv(&details.profile);
            if receipt.command_argv != expected_command {
                diagnostics.push("profiled Clippy restriction receipt command argv does not match profile".to_owned());
            }
        }
        (EvidenceRailKind::ClippyCoReporting, None)
            if receipt.mode == MODE_RUN && receipt.profile.as_deref().is_some() =>
        {
            diagnostics.push("profiled Clippy restriction run receipt missing profile details".to_owned());
        }
        (EvidenceRailKind::ClippyCoReporting, None) => {}
        (_, Some(_)) => {
            diagnostics.push("clippy_restrictions details are only valid on Clippy co-reporting receipts".to_owned());
        }
        (_, None) => {}
    }
}

fn validate_ast_grep_receipt(rail: EvidenceRailKind, receipt: &EvidenceRailReceipt, diagnostics: &mut Vec<String>) {
    match (rail, receipt.ast_grep.as_ref()) {
        (EvidenceRailKind::AstGrepStructural, Some(details)) => {
            if details.verification_role != crate::ast_grep_rail::RECORDED_ONLY_ROLE {
                diagnostics.push("ast-grep receipt must remain recorded_only evidence".to_owned());
            }
            if details.findings.iter().any(|finding| finding.rule_id.trim().is_empty()) {
                diagnostics.push("ast-grep receipt contains a finding without a rule id".to_owned());
            }
            if !receipt.non_claims.iter().any(|claim| claim == crate::ast_grep_rail::REQUIRED_NON_CLAIM) {
                diagnostics.push("ast-grep receipt missing syntax-only non-claim".to_owned());
            }
        }
        (EvidenceRailKind::AstGrepStructural, None) => {
            diagnostics.push("ast-grep receipt missing ast_grep details".to_owned());
        }
        (_, Some(_)) => {
            diagnostics.push("ast_grep details are only valid on ast-grep rail receipts".to_owned());
        }
        (_, None) => {}
    }
}

fn validate_charon_semantic_receipt(
    rail: EvidenceRailKind,
    receipt: &EvidenceRailReceipt,
    diagnostics: &mut Vec<String>,
) {
    match (rail, receipt.charon_semantic.as_ref()) {
        (EvidenceRailKind::CharonSemantic, Some(details)) => {
            diagnostics.extend(crate::charon_semantic::validate_details(details, &receipt.non_claims));
            if receipt.config_identity.as_deref() != Some(crate::charon_semantic::EXPECTED_PROFILE_IDENTITY) {
                diagnostics
                    .push("Charon semantic receipt config identity does not match the shipped profile".to_owned());
            }
            if !receipt.property_classes.is_empty() {
                diagnostics.push("Charon semantic receipt must not carry property classes".to_owned());
            }
            if receipt.profile.as_deref() != Some(crate::charon_semantic::shipped_profile_id()) {
                diagnostics.push("Charon semantic receipt profile selection drifted".to_owned());
            }
            let expected_command = crate::charon_semantic::profile_command_argv().unwrap_or_default();
            if receipt.mode == MODE_RUN && receipt.command_argv != expected_command {
                diagnostics
                    .push("Charon semantic run receipt command argv does not match the closed profile".to_owned());
            }
            if receipt.mode == MODE_IMPORT && !receipt.command_argv.is_empty() {
                diagnostics.push("Charon semantic import receipt must not claim an executed command".to_owned());
            }
        }
        (EvidenceRailKind::CharonSemantic, None) => {
            diagnostics.push("charon-semantic receipt missing charon_semantic details".to_owned());
        }
        (_, Some(_)) => {
            diagnostics.push("charon_semantic details are only valid on charon-semantic rail receipts".to_owned());
        }
        (_, None) => {}
    }
}

fn validate_wasm_artifact_receipt(
    rail: EvidenceRailKind,
    receipt: &EvidenceRailReceipt,
    diagnostics: &mut Vec<String>,
) {
    match (rail, receipt.wasm_artifact.as_ref()) {
        (EvidenceRailKind::WasmArtifact, Some(details)) => {
            crate::wasm_artifact::validate_receipt_details(details, &receipt.non_claims, diagnostics);
        }
        (EvidenceRailKind::WasmArtifact, None) => {
            diagnostics.push("wasm-artifact receipt missing wasm_artifact details".to_owned());
        }
        (_, Some(_)) => {
            diagnostics.push("wasm_artifact details are only valid on wasm-artifact rail receipts".to_owned());
        }
        (_, None) => {}
    }
}

pub fn expected_bundle_files() -> Vec<String> {
    vec![
        PROVENANCE_ARTIFACT_NAME.to_owned(),
        RECEIPT_ARTIFACT_NAME.to_owned(),
        SUMMARY_ARTIFACT_NAME.to_owned(),
    ]
}

pub fn expected_bundle_files_for_receipt(receipt: Option<&EvidenceRailReceipt>) -> Vec<String> {
    let mut files = expected_bundle_files();
    if receipt.is_some_and(|receipt| receipt.wasm_artifact.is_some()) {
        files.push(crate::wasm_artifact::INPUT_ARTIFACT_NAME.to_owned());
    }
    if let Some(details) = receipt.and_then(|receipt| receipt.charon_semantic.as_ref()) {
        files.extend(crate::charon_semantic::expected_artifact_names(Some(details)));
    }
    if receipt.is_some_and(|receipt| receipt.sarif_import.is_some()) {
        files.push(crate::external::sarif::RAW_ARTIFACT.to_owned());
        files.push(crate::external::sarif::PROJECTION_ARTIFACT.to_owned());
    }
    if receipt.is_some_and(|receipt| receipt.coverage_baseline.is_some()) {
        files.push(crate::coverage::SOURCE_MANIFEST_ARTIFACT.to_owned());
        files.push(crate::coverage::CURRENT_RAW_ARTIFACT.to_owned());
        files.push(crate::coverage::CURRENT_ENVELOPE_ARTIFACT.to_owned());
        files.push(crate::coverage::BASELINE_ENVELOPE_ARTIFACT.to_owned());
        files.push(crate::coverage::PROJECTION_ARTIFACT.to_owned());
    }
    if let Some(details) = receipt.and_then(|receipt| receipt.cargo_mutants.as_ref()) {
        files.extend([
            crate::mutation::SOURCE_MANIFEST_ARTIFACT.to_owned(),
            crate::mutation::OUTCOMES_ARTIFACT.to_owned(),
            crate::mutation::INVENTORY_ARTIFACT.to_owned(),
            crate::mutation::PROJECTION_ARTIFACT.to_owned(),
            details.baseline.log.locator.clone(),
        ]);
        for mutant in &details.mutants {
            files.push(mutant.log.locator.clone());
            if let Some(diff) = mutant.diff.as_ref() {
                files.push(diff.locator.clone());
            }
        }
    }
    if receipt.is_some_and(|receipt| receipt.benchmark_evidence.is_some()) {
        files.extend([
            crate::benchmark::BASELINE_ARTIFACT.to_owned(),
            crate::benchmark::CANDIDATE_ARTIFACT.to_owned(),
            crate::benchmark::ALLOCATION_ARTIFACT.to_owned(),
            crate::benchmark::COPY_ARTIFACT.to_owned(),
            crate::benchmark::ORACLE_ARTIFACT.to_owned(),
            crate::benchmark::FINDINGS_ARTIFACT.to_owned(),
            crate::benchmark::PROJECTION_ARTIFACT.to_owned(),
        ]);
    }
    if let Some(details) = receipt
        .and_then(|receipt| receipt.hegel_property.as_ref())
        .and_then(|details| details.hardened.as_ref())
    {
        files.extend([
            crate::hegel::SUMMARY_ARTIFACT.to_owned(),
            crate::hegel::LOG_ARTIFACT.to_owned(),
            crate::hegel::SOURCE_MANIFEST_ARTIFACT.to_owned(),
            crate::hegel::GENERATOR_MANIFEST_ARTIFACT.to_owned(),
            crate::hegel::PROJECTION_ARTIFACT.to_owned(),
        ]);
        if details.counterexample_identity.is_some() {
            files.push(crate::hegel::COUNTEREXAMPLE_ARTIFACT.to_owned());
        }
    }
    files.sort();
    files
}

pub fn receipt_tier_from_rail_id(rail_id: &str) -> EvidenceTier {
    EvidenceRailKind::parse(rail_id).map_or(EvidenceTier::PolicyCheck, EvidenceRailKind::evidence_tier)
}

fn parse_rail_input(input: &EvaluationInput<'_>) -> RailParseOutput {
    match input.rail {
        EvidenceRailKind::DependencyAudit => parse_dependency_audit(input),
        EvidenceRailKind::ClippyCoReporting => parse_clippy(input),
        EvidenceRailKind::AstGrepStructural => parse_ast_grep(input),
        EvidenceRailKind::WasmArtifact => parse_wasm_artifact(input),
        EvidenceRailKind::CharonSemantic => parse_charon_semantic(input),
        EvidenceRailKind::DependencyTrust => parse_dependency_trust(input),
        EvidenceRailKind::ApiStability => parse_api_stability(input),
        EvidenceRailKind::DynamicEvidence => parse_dynamic_evidence(input),
        EvidenceRailKind::SarifImport => parse_sarif_import(input),
        EvidenceRailKind::ReleaseSupplyChain => parse_release_supply_chain(input),
        EvidenceRailKind::MutationTesting => parse_mutation_testing(input),
        EvidenceRailKind::CoverageBaseline => parse_coverage_baseline(input),
        EvidenceRailKind::BenchmarkEvidence => parse_benchmark_evidence(input),
        EvidenceRailKind::KaniModelCheck => parse_kani_model_check(input),
        EvidenceRailKind::HegelProperty => parse_hegel_property(input),
        EvidenceRailKind::Z4Solver => parse_z4_solver(input),
        EvidenceRailKind::VerifierFixture => parse_verifier_fixture(input),
    }
}

fn parse_dependency_audit(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    if input.config_path.is_none_or(str::is_empty) {
        diagnostics
            .push("dependency-audit rail requires an explicit deny.toml config path or template path".to_owned());
    }
    if input.locked && input.advisory_updated_by_prior_step && input.mode == MODE_RUN {
        diagnostics
            .push("locked dependency-audit evidence must not update advisory databases during the run".to_owned());
    }
    let value = parse_json_value(input.import_text, &mut diagnostics);
    let mut findings = Vec::new();
    if let Some(value) = value {
        validate_dependency_audit_report_shape(&value, &mut diagnostics);
        for item in array_field(&value, "diagnostics") {
            let family = string_field(item, "category").unwrap_or("unknown").to_owned();
            let severity = string_field(item, "severity").unwrap_or("warning").to_owned();
            let message = string_field(item, "message").unwrap_or("cargo-deny finding").to_owned();
            findings.push(EvidenceRailFinding {
                family,
                severity,
                message,
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_clippy(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    for (line_offset, line) in input.import_text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let line_number = line_offset + 1;
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(format!("malformed Clippy JSON line {line_number}: {error}"));
                continue;
            }
        };
        if value.get("reason").and_then(serde_json::Value::as_str) != Some("compiler-message") {
            continue;
        }
        let message = &value["message"];
        let Some(code) = message.get("code").and_then(|code| code.get("code")).and_then(serde_json::Value::as_str)
        else {
            diagnostics.push(format!("Clippy JSON line {line_number} missing message.code.code"));
            continue;
        };
        if !code.starts_with("clippy::") {
            continue;
        }
        let Some(text) = message.get("message").and_then(serde_json::Value::as_str) else {
            diagnostics.push(format!("Clippy JSON line {line_number} missing message.message"));
            continue;
        };
        let primary = primary_span(message);
        findings.push(EvidenceRailFinding {
            family: code.trim_start_matches("clippy::").to_owned(),
            severity: clippy_gate_severity(message.get("level").and_then(serde_json::Value::as_str)),
            message: text.to_owned(),
            source_severity: message.get("level").and_then(serde_json::Value::as_str).map(ToOwned::to_owned),
            file: primary
                .and_then(|span| span.get("file_name"))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned),
            line: primary.and_then(|span| span.get("line_start")).and_then(serde_json::Value::as_u64),
            column: primary.and_then(|span| span.get("column_start")).and_then(serde_json::Value::as_u64),
        });
    }
    if let Some(exit_code) = input.tool_exit_code {
        let denied_finding_present = findings.iter().any(|finding| finding.severity == "error");
        if exit_code != EXIT_SUCCESS && !denied_finding_present {
            diagnostics.push(format!("Clippy tool exited with nonzero status {exit_code}"));
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_ast_grep(input: &EvaluationInput<'_>) -> RailParseOutput {
    let evaluation = crate::ast_grep_rail::evaluate_envelope(input.import_text);
    let profile = evaluation.details.as_ref().map(|details| details.profile_id.clone());
    let findings = evaluation
        .details
        .as_ref()
        .map(|details| {
            details
                .findings
                .iter()
                .map(|finding| EvidenceRailFinding {
                    family: finding.rule_id.clone(),
                    severity: finding.severity.clone(),
                    message: finding.message.clone(),
                    source_severity: Some(finding.severity.clone()),
                    file: Some(finding.file.clone()),
                    line: Some(finding.start.line),
                    column: Some(finding.start.column),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let finding_counts = counts_by_family(&findings);
    let status = match evaluation.status {
        crate::ast_grep_rail::AstGrepEvaluationStatus::Passed => STATUS_PASSED,
        crate::ast_grep_rail::AstGrepEvaluationStatus::ManualReview => STATUS_MANUAL_REVIEW_REQUIRED,
        crate::ast_grep_rail::AstGrepEvaluationStatus::Failed => STATUS_FAILED,
    }
    .to_owned();
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics: evaluation.diagnostics,
        status,
        profile,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_wasm_artifact(input: &EvaluationInput<'_>) -> RailParseOutput {
    let evaluation = crate::wasm_artifact::evaluate_envelope(input.import_text);
    let profile = evaluation.details.as_ref().map(|details| details.profile_id.clone());
    let findings = evaluation
        .details
        .as_ref()
        .map(|details| {
            details
                .findings
                .iter()
                .map(|finding| EvidenceRailFinding {
                    family: finding.issue_code.clone(),
                    severity: finding.severity.clone(),
                    message: finding.message.clone(),
                    source_severity: Some(finding.severity.clone()),
                    file: Some(crate::wasm_artifact::INPUT_ARTIFACT_NAME.to_owned()),
                    line: None,
                    column: None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let finding_counts = counts_by_family(&findings);
    let status = match evaluation.status {
        crate::wasm_artifact::WasmEvaluationStatus::Passed => STATUS_PASSED,
        crate::wasm_artifact::WasmEvaluationStatus::ManualReview => STATUS_MANUAL_REVIEW_REQUIRED,
        crate::wasm_artifact::WasmEvaluationStatus::Failed => STATUS_FAILED,
    }
    .to_owned();
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics: evaluation.diagnostics,
        status,
        profile,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_charon_semantic(input: &EvaluationInput<'_>) -> RailParseOutput {
    let evaluation = crate::charon_semantic::evaluate_envelope(input.import_text);
    let findings = evaluation
        .findings
        .into_iter()
        .map(|finding| EvidenceRailFinding {
            family: finding.family,
            severity: finding.severity.clone(),
            message: finding.message,
            source_severity: Some(finding.severity),
            file: finding.file,
            line: finding.line,
            column: None,
        })
        .collect::<Vec<_>>();
    let finding_counts = counts_by_family(&findings);
    let status = if !evaluation.diagnostics.is_empty() {
        STATUS_FAILED
    } else if findings.is_empty() {
        STATUS_PASSED
    } else {
        STATUS_MANUAL_REVIEW_REQUIRED
    };
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics: evaluation.diagnostics,
        status: status.to_owned(),
        profile: evaluation.profile_id,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_dependency_trust(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        for import in array_field(&value, "imports") {
            if string_field(import, "name").is_none_or(str::is_empty)
                || string_field(import, "source").is_none_or(str::is_empty)
            {
                diagnostics.push("dependency-trust import entries must expose name and source identity".to_owned());
            }
        }
        for exemption in array_field(&value, "exemptions") {
            let crate_name = string_field(exemption, "crate").unwrap_or("unknown crate");
            let reason = string_field(exemption, "reason").unwrap_or("missing reason");
            findings.push(EvidenceRailFinding {
                family: "exemption".to_owned(),
                severity: "manual_review".to_owned(),
                message: format!("cargo-vet exemption visible for {crate_name}: {reason}"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
        for gap in array_field(&value, "unaudited") {
            let crate_name = string_field(gap, "crate").unwrap_or("unknown crate");
            findings.push(EvidenceRailFinding {
                family: "unaudited-gap".to_owned(),
                severity: "error".to_owned(),
                message: format!("cargo-vet unaudited dependency gap for {crate_name}"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = if findings.iter().any(|finding| finding.family == "unaudited-gap") {
        STATUS_FAILED.to_owned()
    } else if findings.iter().any(|finding| finding.family == "exemption") {
        STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
    } else {
        STATUS_PASSED.to_owned()
    };
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_api_stability(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    if input.baseline_identity.is_none() {
        diagnostics.push("api-stability rail requires an accepted baseline path or identity".to_owned());
    }
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        for breakage in array_field(&value, "breakages") {
            let item = string_field(breakage, "item").unwrap_or("public item");
            let kind = string_field(breakage, "kind").unwrap_or("api breakage");
            findings.push(EvidenceRailFinding {
                family: kind.to_owned(),
                severity: "error".to_owned(),
                message: format!("public API compatibility breakage: {item}"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_dynamic_evidence(input: &EvaluationInput<'_>) -> RailParseOutput {
    if input.profile == Some(crate::cargo::fuzz::PROFILE_ID) {
        return parse_fuzz(input);
    }
    let mut diagnostics = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    let mut findings = Vec::new();
    let mut profile = input.profile.map(ToOwned::to_owned);
    if let Some(value) = value {
        let selected_profile =
            input.profile.or_else(|| string_field(&value, "profile")).unwrap_or("default").to_owned();
        profile = Some(selected_profile.clone());
        let known_profiles = string_array_field(&value, "known_profiles");
        if !known_profiles.is_empty() && !known_profiles.iter().any(|candidate| candidate == &selected_profile) {
            diagnostics.push(format!("unknown dynamic evidence profile `{selected_profile}`"));
        }
        let command = input.requested_command.or_else(|| string_field(&value, "command")).unwrap_or_default();
        let mut allowed_commands = string_array_field(&value, "allowed_commands");
        if allowed_commands.is_empty()
            && let Some(allowed_command) = allowed_dynamic_command(&selected_profile)
        {
            allowed_commands.push(allowed_command.to_owned());
        }
        if !command.is_empty()
            && !allowed_commands.is_empty()
            && !allowed_commands.iter().any(|allowed| allowed == command)
        {
            diagnostics
                .push(format!("dynamic evidence command `{command}` is not allowed by profile `{selected_profile}`"));
        }
        if value.get("tool_available").and_then(serde_json::Value::as_bool) == Some(false) {
            diagnostics.push(format!("dynamic evidence tool for profile `{selected_profile}` is unavailable"));
        }
        let expected_signal = string_field(&value, "expected_signal").unwrap_or("passed");
        let observed_signal = string_field(&value, "acceptance_signal").unwrap_or("missing");
        if expected_signal != observed_signal {
            diagnostics.push(format!(
                "dynamic evidence acceptance signal mismatch for `{selected_profile}`; expected `{expected_signal}`, got `{observed_signal}`"
            ));
        }
        if value.get("finding_count").and_then(serde_json::Value::as_u64).unwrap_or_default() > 0 {
            findings.push(EvidenceRailFinding {
                family: "dynamic-finding".to_owned(),
                severity: "error".to_owned(),
                message: format!("dynamic evidence profile `{selected_profile}` reported findings"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_fuzz(input: &EvaluationInput<'_>) -> RailParseOutput {
    let evaluation = crate::cargo::fuzz::evaluate_envelope(input.import_text);
    let mut findings = Vec::with_capacity(1);
    if evaluation.is_crash_observed {
        findings.push(EvidenceRailFinding {
            family: "cargo-fuzz-crash".to_owned(),
            severity: "error".to_owned(),
            message: "cargo-fuzz discovered a reproducible crash".to_owned(),
            source_severity: None,
            file: None,
            line: None,
            column: None,
        });
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics: evaluation.diagnostics,
        status,
        profile: Some(crate::cargo::fuzz::PROFILE_ID.to_owned()),
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_sarif_import(input: &EvaluationInput<'_>) -> RailParseOutput {
    let Some(profile) = input.profile else {
        return RailParseOutput {
            findings: Vec::new(),
            finding_counts: BTreeMap::new(),
            diagnostics: vec!["SARIF import requires `--profile NAME`".to_owned()],
            status: STATUS_FAILED.to_owned(),
            profile: None,
            property_classes: Vec::new(),
            hegel_property: None,
            z4_solver: None,
            verifier_fixtures: None,
        };
    };
    let evaluation = crate::external::sarif::evaluate(input.import_text, profile);
    let findings = evaluation
        .details
        .as_ref()
        .map(|details| {
            details
                .findings
                .iter()
                .map(|finding| EvidenceRailFinding {
                    family: finding.source_rule_id.clone(),
                    severity: finding.gate_severity.clone(),
                    message: finding.message.clone(),
                    source_severity: Some(finding.source_level.clone()),
                    file: Some(finding.file.clone()),
                    line: Some(finding.line),
                    column: Some(finding.column),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics: evaluation.diagnostics,
        status,
        profile: Some(profile.to_owned()),
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_release_supply_chain(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        let metadata_present = value
            .get("auditable_binary")
            .and_then(|binary| binary.get("dependency_metadata"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !metadata_present {
            diagnostics.push("release supply-chain rail requires auditable Rust dependency metadata".to_owned());
        }
        for vulnerability in array_field(&value, "vulnerabilities") {
            let id = string_field(vulnerability, "id").unwrap_or("unknown-vulnerability");
            let severity = string_field(vulnerability, "severity").unwrap_or("warning");
            findings.push(EvidenceRailFinding {
                family: "vulnerability".to_owned(),
                severity: severity.to_owned(),
                message: format!("vulnerability scan finding {id}"),
                source_severity: Some(severity.to_owned()),
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = if findings.is_empty() {
        STATUS_PASSED.to_owned()
    } else {
        STATUS_FAILED.to_owned()
    };
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_mutation_testing(input: &EvaluationInput<'_>) -> RailParseOutput {
    if input.profile == Some(crate::mutation::PROFILE_ID) {
        return parse_profiled_mutation(input);
    }
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        push_mutant_findings(
            &value,
            "survived_mutants",
            "survived-mutant",
            "manual_review",
            "survived mutant",
            &mut findings,
        );
        push_mutant_findings(&value, "survivors", "survived-mutant", "manual_review", "survived mutant", &mut findings);
        push_mutant_findings(
            &value,
            "timeout_mutants",
            "timeout-mutant",
            "manual_review",
            "timed-out mutant",
            &mut findings,
        );
        push_mutant_findings(&value, "timeouts", "timeout-mutant", "manual_review", "timed-out mutant", &mut findings);
        push_outcome_mutant_findings(&value, &mut findings);
        push_aggregate_mutation_finding(
            &value,
            "survived",
            "survived-mutant",
            "manual_review",
            "survived mutant",
            &mut findings,
        );
        push_aggregate_mutation_finding(
            &value,
            "timeout",
            "timeout-mutant",
            "manual_review",
            "timed-out mutant",
            &mut findings,
        );
        push_aggregate_mutation_finding(
            &value,
            "timed_out",
            "timeout-mutant",
            "manual_review",
            "timed-out mutant",
            &mut findings,
        );
    }
    let finding_counts = counts_by_family(&findings);
    let status = if findings.is_empty() {
        STATUS_PASSED.to_owned()
    } else {
        STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
    };
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_benchmark_evidence(input: &EvaluationInput<'_>) -> RailParseOutput {
    let evaluation = crate::benchmark::request::evaluate(input.import_text, crate::benchmark::PROFILE_ID);
    let findings = evaluation.details.as_ref().map(benchmark_findings).unwrap_or_default();
    let status = evaluation.details.as_ref().map_or_else(
        || STATUS_FAILED.to_owned(),
        |details| match details.projection.performance.result {
            crate::benchmark::ComparisonResult::Regressed => STATUS_FAILED.to_owned(),
            crate::benchmark::ComparisonResult::Inconclusive | crate::benchmark::ComparisonResult::Incomparable => {
                STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
            }
            crate::benchmark::ComparisonResult::Improved | crate::benchmark::ComparisonResult::Stable => {
                if details.projection.signals.false_positive > 0
                    || details.projection.signals.false_negative > 0
                    || details.projection.signals.unexpected_suppression > 0
                {
                    STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
                } else {
                    STATUS_PASSED.to_owned()
                }
            }
        },
    );
    RailParseOutput {
        finding_counts: counts_by_family(&findings),
        findings,
        diagnostics: evaluation.diagnostics,
        status,
        profile: Some(crate::benchmark::PROFILE_ID.to_owned()),
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn benchmark_findings(details: &crate::benchmark::ReceiptDetails) -> Vec<EvidenceRailFinding> {
    let mut findings = Vec::new();
    match details.projection.performance.result {
        crate::benchmark::ComparisonResult::Regressed => {
            findings.push(kani_finding("benchmark-regression", "error", "benchmark candidate regressed"));
        }
        crate::benchmark::ComparisonResult::Inconclusive | crate::benchmark::ComparisonResult::Incomparable => {
            findings.push(kani_finding(
                "benchmark-inconclusive",
                "manual_review",
                "benchmark comparison is inconclusive or incomparable",
            ));
        }
        crate::benchmark::ComparisonResult::Improved | crate::benchmark::ComparisonResult::Stable => {}
    }
    let signals = &details.projection.signals;
    for (family, count) in [
        ("benchmark-false-positive", signals.false_positive),
        ("benchmark-false-negative", signals.false_negative),
        ("benchmark-unexpected-suppression", signals.unexpected_suppression),
    ] {
        if count > 0 {
            findings.push(kani_finding(family, "manual_review", &format!("benchmark signal count is {count}")));
        }
    }
    findings
}

fn parse_profiled_mutation(input: &EvaluationInput<'_>) -> RailParseOutput {
    let evaluation = crate::mutation::request::evaluate(input.import_text, crate::mutation::PROFILE_ID);
    let findings = evaluation.details.as_ref().map(profiled_mutation_findings).unwrap_or_default();
    let status = evaluation
        .details
        .as_ref()
        .map_or_else(|| STATUS_FAILED.to_owned(), |details| details.gate_status.clone());
    RailParseOutput {
        finding_counts: counts_by_family(&findings),
        findings,
        diagnostics: evaluation.diagnostics,
        status,
        profile: Some(crate::mutation::PROFILE_ID.to_owned()),
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn profiled_mutation_findings(details: &crate::mutation::ReceiptDetails) -> Vec<EvidenceRailFinding> {
    let mut findings = Vec::with_capacity(details.mutants.len());
    if details.baseline.outcome != "passed" {
        findings.push(kani_finding("mutation-baseline", "error", "unmutated baseline failed"));
    }
    for mutant in &details.mutants {
        if mutant.outcome == "caught" {
            continue;
        }
        let severity = if matches!(mutant.outcome.as_str(), "timeout" | "unviable") {
            "manual_review"
        } else {
            "error"
        };
        findings.push(EvidenceRailFinding {
            family: format!("mutation-{}", mutant.outcome),
            severity: severity.to_owned(),
            message: format!("cargo-mutants outcome `{}` for `{}`", mutant.outcome, mutant.name),
            source_severity: Some(mutant.outcome.clone()),
            file: Some(mutant.file.clone()),
            line: Some(mutant.line),
            column: Some(mutant.column),
        });
    }
    findings
}

fn parse_coverage_baseline(input: &EvaluationInput<'_>) -> RailParseOutput {
    if input.profile == Some(crate::coverage::PROFILE_ID) {
        return parse_profiled_coverage(input);
    }
    let mut diagnostics = Vec::new();
    if input.baseline_identity.is_none() {
        diagnostics.push("coverage-baseline rail requires an accepted baseline path or identity".to_owned());
    }
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        validate_coverage_report_shape(&value, &mut diagnostics);
        validate_coverage_summary(&value, &mut diagnostics);
        push_coverage_findings(&value, "regressions", "coverage-regression", "coverage regression", &mut findings);
        push_coverage_findings(
            &value,
            "threshold_failures",
            "coverage-threshold",
            "coverage threshold failure",
            &mut findings,
        );
        push_aggregate_coverage_finding(
            &value,
            "regression_count",
            "coverage-regression",
            "coverage regression",
            &mut findings,
        );
        push_aggregate_coverage_finding(
            &value,
            "threshold_failure_count",
            "coverage-threshold",
            "coverage threshold failure",
            &mut findings,
        );
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_profiled_coverage(input: &EvaluationInput<'_>) -> RailParseOutput {
    let evaluation = crate::coverage::request::evaluate(input.import_text, crate::coverage::PROFILE_ID);
    let mut findings = CoverageFindings::new();
    let status = if let Some(details) = evaluation.details.as_ref() {
        append_coverage_policy_findings(&details.comparison, &mut findings);
        if details.comparison.passed {
            STATUS_PASSED.to_owned()
        } else {
            STATUS_FAILED.to_owned()
        }
    } else {
        STATUS_FAILED.to_owned()
    };
    let findings = findings.finish();
    RailParseOutput {
        finding_counts: counts_by_family(&findings),
        findings,
        diagnostics: evaluation.diagnostics,
        status,
        profile: Some(crate::coverage::PROFILE_ID.to_owned()),
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn append_coverage_policy_findings(comparison: &crate::coverage::Comparison, findings: &mut CoverageFindings) {
    for (name, metric) in [
        ("lines", comparison.lines),
        ("branches", comparison.branches),
        ("functions", comparison.functions),
        ("regions", comparison.regions),
    ] {
        append_coverage_metric_findings(name, &metric, findings);
    }
}

fn append_coverage_metric_findings(
    name: &str,
    metric: &crate::coverage::MetricComparison,
    findings: &mut CoverageFindings,
) {
    for (passed, family, message) in [
        (metric.threshold_passed, "coverage-threshold", "threshold failed"),
        (metric.regression_passed, "coverage-regression", "regression detected"),
        (metric.denominator_passed, "coverage-denominator", "denominator changed"),
    ] {
        if !passed {
            findings.push(EvidenceRailFinding {
                family: family.to_owned(),
                severity: "error".to_owned(),
                message: format!("coverage {name} {message}"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
}

fn parse_kani_model_check(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let mut property_classes = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        validate_optional_schema_version(&value, KANI_MODEL_CHECK_SCHEMA_VERSION, "kani-model-check", &mut diagnostics);
        validate_required_object_strings(
            &value,
            "harness",
            "Kani harness identity",
            &["package", "target", "path", "name"],
            &mut diagnostics,
        );
        validate_required_object_strings(
            &value,
            "toolchain",
            "Kani toolchain identity",
            &["kani", "rustc", "cbmc", "solver"],
            &mut diagnostics,
        );
        validate_required_string_field(&value, "source_identity", "Kani source identity", &mut diagnostics);
        if string_field(&value, "source_identity_status") == Some("stale") {
            diagnostics.push("Kani source identity is marked stale".to_owned());
        }

        push_kani_verdict_finding(&value, &mut findings, &mut diagnostics);
        validate_kani_assumptions(&value, &mut diagnostics);
        push_kani_property_findings(&value, &mut findings, &mut property_classes, &mut diagnostics);
        let unwind_passed = push_kani_unwind_findings(&value, &mut findings, &mut diagnostics);
        let contract_passed = push_kani_contract_findings(&value, &mut findings, &mut diagnostics);
        let boundary_stub_seen = push_kani_stub_findings(&value, &mut findings, &mut diagnostics);
        validate_kani_claims(&value, unwind_passed, contract_passed, boundary_stub_seen, &mut diagnostics);
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_kani_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: input.profile.map(ToOwned::to_owned),
        property_classes,
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_hegel_property(input: &EvaluationInput<'_>) -> RailParseOutput {
    if input.profile == Some(crate::hegel::PROFILE_ID) {
        return parse_profiled_hegel_property(input);
    }
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    let hegel_property =
        value.as_ref().map(|value| parse_hegel_property_details(value, &mut findings, &mut diagnostics));
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: input.profile.map(ToOwned::to_owned),
        property_classes: Vec::new(),
        hegel_property,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn parse_profiled_hegel_property(input: &EvaluationInput<'_>) -> RailParseOutput {
    let evaluation = crate::hegel::request::evaluate(input.import_text, crate::hegel::PROFILE_ID);
    let mut findings = Vec::new();
    if evaluation.details.as_ref().is_some_and(|details| details.result == "counterexample") {
        findings.push(EvidenceRailFinding {
            family: "hegel-counterexample".to_owned(),
            severity: "error".to_owned(),
            message: "profiled Hegel law produced a retained counterexample".to_owned(),
            source_severity: None,
            file: None,
            line: None,
            column: None,
        });
    }
    let hegel_property = evaluation.details.as_ref().map(profiled_hegel_details);
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics: evaluation.diagnostics,
        status,
        profile: Some(crate::hegel::PROFILE_ID.to_owned()),
        property_classes: Vec::new(),
        hegel_property,
        z4_solver: None,
        verifier_fixtures: None,
    }
}

fn profiled_hegel_details(details: &crate::hegel::ReceiptDetails) -> HegelPropertyReceiptDetails {
    HegelPropertyReceiptDetails {
        property_id: details.property_id.clone(),
        linked_ids: details.linked_ids.clone(),
        source_scope: details.source_scope.clone(),
        runner: HegelRunnerSummary {
            crate_version: details.producer.hegel_version.clone(),
            backend: details.producer.backend.clone(),
        },
        settings: HegelSettingsSummary {
            case_budget: details.settings.case_budget,
            phases: sorted_strings(details.settings.phases.clone()),
            seed: None,
            derandomized: details.settings.derandomized,
        },
        generator: HegelGeneratorSummary {
            family: details.generator.family.clone(),
            content_hash: details.generator.content_hash.clone(),
        },
        source_hashes: vec![details.source_manifest_identity.clone()],
        counterexample: details.counterexample_identity.as_ref().map(|identity| HegelCounterexampleSummary {
            summary_hash: identity.clone(),
            issue_codes: details.issue_codes.clone(),
        }),
        hardened: Some(details.clone()),
    }
}

fn parse_verifier_fixture(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    let verifier_fixtures =
        value.as_ref().map(|value| parse_verifier_fixture_details(value, &mut findings, &mut diagnostics));
    let finding_counts = counts_by_family(&findings);
    let status = status_from_verifier_fixture_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: input.profile.map(ToOwned::to_owned),
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver: None,
        verifier_fixtures,
    }
}

fn parse_verifier_fixture_details(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) -> VerifierFixtureReceiptDetails {
    validate_optional_schema_version(value, VERIFIER_FIXTURE_SCHEMA_VERSION, "verifier-fixture", diagnostics);
    let non_claims = string_array_field(value, "non_claims");
    if !non_claims.iter().any(|claim| claim == VERIFIER_FIXTURE_NON_CLAIM) {
        diagnostics.push("verifier fixture receipt missing fixture non-claim".to_owned());
    }
    let fixtures = parse_verifier_fixture_entries(value, findings, diagnostics);
    VerifierFixtureReceiptDetails { fixtures }
}

fn parse_verifier_fixture_entries(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) -> Vec<VerifierFixtureEntry> {
    let Some(fixtures_value) = value.get("fixtures") else {
        diagnostics.push("verifier fixture receipt requires fixtures array".to_owned());
        return Vec::new();
    };
    let Some(items) = fixtures_value.as_array() else {
        diagnostics.push("verifier fixture receipt field `fixtures` must be an array".to_owned());
        return Vec::new();
    };
    let mut fixtures = Vec::new();
    for (index, item) in items.iter().enumerate() {
        validate_verifier_fixture_entry(item, index, findings, diagnostics);
        fixtures.push(verifier_fixture_entry(item));
    }
    fixtures
}

fn validate_verifier_fixture_entry(
    item: &serde_json::Value,
    index: usize,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) {
    let Some(record) = item.as_object() else {
        diagnostics.push(format!("verifier fixture item fixtures[{index}] must be an object"));
        return;
    };
    for required in [
        "fixture_id",
        "source_artifact_ref",
        "syntax_family",
        "expected_verdict",
        "verifier_family",
        "fixture_role",
    ] {
        match record.get(required).and_then(serde_json::Value::as_str) {
            Some(value) if !value.trim().is_empty() => {}
            _ => diagnostics
                .push(format!("verifier fixture item fixtures[{index}] missing non-empty string `{required}`")),
        }
    }
    let fixture_id = string_field(item, "fixture_id").unwrap_or("unknown verifier fixture");
    match string_field(item, "expected_verdict") {
        Some("accepted") => {}
        Some("expected_failure") => findings.push(verifier_fixture_finding(
            "verifier-expected-failure",
            "manual_review",
            &format!("verifier fixture is an expected negative case: {fixture_id}"),
        )),
        Some(other) => diagnostics.push(format!(
            "verifier fixture `{fixture_id}` expected_verdict `{other}` is unsupported; expected accepted or expected_failure"
        )),
        None => {}
    }
    validate_verifier_fixture_arrays(item, index, diagnostics);
    validate_verifier_fixture_claims(item, fixture_id, diagnostics);
}

fn validate_verifier_fixture_arrays(item: &serde_json::Value, index: usize, diagnostics: &mut Vec<String>) {
    for field in ["expected_diagnostics", "assumptions", "unsupported_features"] {
        let Some(value) = item.get(field) else {
            diagnostics.push(format!("verifier fixture item fixtures[{index}] missing array `{field}`"));
            continue;
        };
        if !value.is_array() {
            diagnostics.push(format!("verifier fixture item fixtures[{index}] field `{field}` must be an array"));
        }
    }
}

fn validate_verifier_fixture_claims(item: &serde_json::Value, fixture_id: &str, diagnostics: &mut Vec<String>) {
    let role = string_field(item, "fixture_role").unwrap_or_default();
    if contains_verifier_fixture_overclaim(role) {
        diagnostics
            .push(format!("verifier fixture `{fixture_id}` contains unsupported proof or correctness overclaim"));
    }
    if let Some(claims) = item.get("claims") {
        for field in [
            "verifier_parity",
            "proof_soundness",
            "source_correctness",
            "release_eligible",
        ] {
            if claims.get(field).and_then(serde_json::Value::as_bool) == Some(true) {
                diagnostics.push(format!("verifier fixture `{fixture_id}` cannot set claims.{field}=true"));
            }
        }
    }
}

fn verifier_fixture_entry(item: &serde_json::Value) -> VerifierFixtureEntry {
    VerifierFixtureEntry {
        fixture_id: string_field(item, "fixture_id").unwrap_or_default().to_owned(),
        source_artifact_ref: string_field(item, "source_artifact_ref").unwrap_or_default().to_owned(),
        syntax_family: string_field(item, "syntax_family").unwrap_or_default().to_owned(),
        expected_verdict: string_field(item, "expected_verdict").unwrap_or_default().to_owned(),
        expected_diagnostics: string_array_field(item, "expected_diagnostics"),
        assumptions: string_array_field(item, "assumptions"),
        unsupported_features: string_array_field(item, "unsupported_features"),
        verifier_family: string_field(item, "verifier_family").unwrap_or_default().to_owned(),
        fixture_role: string_field(item, "fixture_role").unwrap_or_default().to_owned(),
    }
}

fn contains_verifier_fixture_overclaim(text: &str) -> bool {
    for marker in [
        "verifier parity",
        "proof soundness",
        "solver soundness",
        "source correctness",
        "whole-program safety",
        "release eligibility",
    ] {
        if text.contains(marker) {
            return true;
        }
    }
    false
}

fn verifier_fixture_finding(family: &str, severity: &str, message: &str) -> EvidenceRailFinding {
    EvidenceRailFinding {
        family: family.to_owned(),
        severity: severity.to_owned(),
        message: message.to_owned(),
        source_severity: None,
        file: None,
        line: None,
        column: None,
    }
}

fn status_from_verifier_fixture_findings(findings: &[EvidenceRailFinding]) -> String {
    if findings.iter().any(|finding| finding.severity == "error") {
        STATUS_FAILED.to_owned()
    } else if findings.is_empty() {
        STATUS_PASSED.to_owned()
    } else {
        STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
    }
}

fn parse_z4_solver(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    let z4_solver = value.as_ref().map(|value| parse_z4_solver_details(value, &mut findings, &mut diagnostics));
    let finding_counts = counts_by_family(&findings);
    let status = status_from_z4_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: input.profile.map(ToOwned::to_owned),
        property_classes: Vec::new(),
        hegel_property: None,
        z4_solver,
        verifier_fixtures: None,
    }
}

fn parse_z4_solver_details(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) -> Z4SolverReceiptDetails {
    validate_optional_schema_version(value, Z4_SOLVER_SCHEMA_VERSION, "z4-solver", diagnostics);
    validate_required_object_strings(
        value,
        "solver",
        "Z4 solver identity",
        &["name", "version", "provenance"],
        diagnostics,
    );
    validate_required_object_strings(
        value,
        "query",
        "Z4 query identity",
        &["identity", "logic", "profile"],
        diagnostics,
    );
    validate_required_string_field(value, "verification_role", "Z4 verification role", diagnostics);
    validate_z4_result_status(value, findings, diagnostics);
    validate_z4_boundary(value, diagnostics);

    let solver = value.get("solver").unwrap_or(&serde_json::Value::Null);
    let query = value.get("query").unwrap_or(&serde_json::Value::Null);
    Z4SolverReceiptDetails {
        solver: Z4SolverIdentity {
            name: string_field(solver, "name").unwrap_or_default().to_owned(),
            version: string_field(solver, "version").unwrap_or_default().to_owned(),
            provenance: string_field(solver, "provenance").unwrap_or_default().to_owned(),
        },
        query: Z4QueryIdentity {
            identity: string_field(query, "identity").unwrap_or_default().to_owned(),
            logic: string_field(query, "logic").unwrap_or_default().to_owned(),
            profile: string_field(query, "profile").unwrap_or_default().to_owned(),
        },
        result_status: z4_status(value).unwrap_or_default().to_owned(),
        verification_role: string_field(value, "verification_role").unwrap_or_default().to_owned(),
        proof_ref: string_field(value, "proof_ref").map(ToOwned::to_owned),
        model_ref: string_field(value, "model_ref").map(ToOwned::to_owned),
        unsat_core_ref: string_field(value, "unsat_core_ref").map(ToOwned::to_owned),
        unknown_reason: string_field(value, "unknown_reason").map(ToOwned::to_owned),
    }
}

fn validate_z4_result_status(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) {
    match z4_status(value) {
        Some(Z4_STATUS_SAT) => {
            if string_field(value, "model_ref").is_none_or(str::is_empty) {
                diagnostics.push("Z4 SAT receipt requires model_ref".to_owned());
            }
            findings.push(z4_finding("z4-sat", "manual_review", "Z4 reported SAT; review the model diagnostic"));
        }
        Some(Z4_STATUS_UNSAT) => {
            if string_field(value, "proof_ref").is_none_or(str::is_empty)
                && string_field(value, "unsat_core_ref").is_none_or(str::is_empty)
            {
                diagnostics.push("Z4 UNSAT receipt requires proof_ref or unsat_core_ref".to_owned());
            }
        }
        Some(Z4_STATUS_UNKNOWN) => {
            if string_field(value, "unknown_reason").is_none_or(str::is_empty) {
                diagnostics.push("Z4 unknown receipt requires unknown_reason".to_owned());
            }
            findings.push(z4_finding(
                "z4-unknown",
                "manual_review",
                "Z4 reported unknown; preserve as non-proof diagnostic evidence",
            ));
        }
        Some(Z4_STATUS_TIMEOUT) => {
            if string_field(value, "unknown_reason").is_none_or(str::is_empty) {
                diagnostics.push("Z4 timeout receipt requires unknown_reason".to_owned());
            }
            findings.push(z4_finding(
                "z4-timeout",
                "manual_review",
                "Z4 timed out; preserve as non-proof diagnostic evidence",
            ));
        }
        Some(other) => diagnostics
            .push(format!("Z4 result status `{other}` is unsupported; expected sat, unsat, unknown, or timeout")),
        None => diagnostics.push("Z4 receipt requires result_status".to_owned()),
    }
    validate_array_records(value, "diagnostics", "Z4 diagnostic", &["severity", "message"], diagnostics);
    for diagnostic in array_field(value, "diagnostics") {
        let severity = string_field(diagnostic, "severity").unwrap_or("manual_review");
        let message = string_field(diagnostic, "message").unwrap_or("Z4 diagnostic");
        findings.push(z4_finding("z4-diagnostic", z4_diagnostic_severity(severity), message));
    }
}

fn validate_z4_boundary(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    if let Some(query) = value.get("query")
        && let Some(identity) = string_field(query, "identity")
        && !is_b3_identity(identity)
    {
        diagnostics.push(format!("Z4 query identity `{identity}` must be a BLAKE3 identity"));
    }
    let non_claims = string_array_field(value, "non_claims");
    if !non_claims.iter().any(|claim| claim == Z4_SOLVER_NON_CLAIM) {
        diagnostics.push("Z4 receipt missing solver non-claim".to_owned());
    }
    if string_field(value, "evidence_tier") == Some(FORMAL_PROOF_TIER) {
        diagnostics.push("Z4 solver receipts cannot be promoted to formal_proof".to_owned());
    }
    let role = string_field(value, "verification_role").unwrap_or_default();
    if contains_z4_overclaim(role) {
        diagnostics.push("Z4 receipt contains unsupported proof or correctness overclaim".to_owned());
    }
    if let Some(claims) = value.get("claims") {
        for field in [
            "proves_correctness",
            "proves_proof_validity",
            "proves_solver_soundness",
            "release_eligible",
        ] {
            if claims.get(field).and_then(serde_json::Value::as_bool) == Some(true) {
                diagnostics.push(format!("Z4 receipt cannot set claims.{field}=true"));
            }
        }
    }
}

fn z4_status(value: &serde_json::Value) -> Option<&str> {
    string_field(value, "result_status").or_else(|| string_field(value, "status"))
}

fn z4_diagnostic_severity(severity: &str) -> &str {
    match severity {
        "error" => "error",
        "warning" | "warn" | "info" | "note" => "manual_review",
        _ => "manual_review",
    }
}

fn contains_z4_overclaim(text: &str) -> bool {
    for marker in [
        "formal proof",
        "proves correctness",
        "proves proof validity",
        "proves solver soundness",
        "proves requirement satisfaction",
        "release eligibility",
        "whole-program correctness",
    ] {
        if text.contains(marker) {
            return true;
        }
    }
    false
}

fn z4_finding(family: &str, severity: &str, message: &str) -> EvidenceRailFinding {
    EvidenceRailFinding {
        family: family.to_owned(),
        severity: severity.to_owned(),
        message: message.to_owned(),
        source_severity: None,
        file: None,
        line: None,
        column: None,
    }
}

fn status_from_z4_findings(findings: &[EvidenceRailFinding]) -> String {
    if findings.iter().any(|finding| finding.severity == "error") {
        STATUS_FAILED.to_owned()
    } else if findings.is_empty() {
        STATUS_PASSED.to_owned()
    } else {
        STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
    }
}

fn parse_hegel_property_details(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) -> HegelPropertyReceiptDetails {
    validate_optional_schema_version(value, HEGEL_PROPERTY_SCHEMA_VERSION, "hegel-property", diagnostics);
    validate_required_string_field(value, "property_id", "Hegel property identity", diagnostics);
    validate_required_string_field(value, "source_scope", "Hegel source scope", diagnostics);
    validate_required_object_strings(
        value,
        "runner",
        "Hegel runner identity",
        &["crate_version", "backend"],
        diagnostics,
    );
    validate_required_object_strings(
        value,
        "generator",
        "Hegel generator identity",
        &["family", "content_hash"],
        diagnostics,
    );
    validate_hegel_generator_hash(value, diagnostics);
    validate_hegel_settings(value, diagnostics);
    validate_b3_array(value, "source_hashes", "Hegel source hash", diagnostics);
    validate_hegel_boundary(value, diagnostics);

    let status = string_field(value, "status");
    let counterexample = value
        .get("counterexample")
        .and_then(|counterexample| parse_hegel_counterexample(counterexample, diagnostics));
    match status {
        Some(STATUS_PASSED) if counterexample.is_some() => {
            diagnostics.push("Hegel passed receipt must not include a counterexample".to_owned());
        }
        Some(STATUS_PASSED) => {}
        Some(STATUS_FAILED) => {
            findings.push(hegel_finding(value));
            if counterexample.is_none() {
                diagnostics.push("Hegel failed receipt requires counterexample identity".to_owned());
            }
        }
        Some(other) => diagnostics.push(format!("Hegel status `{other}` is unsupported; expected passed or failed")),
        None => diagnostics.push("Hegel receipt requires status `passed` or `failed`".to_owned()),
    }

    HegelPropertyReceiptDetails {
        property_id: string_field(value, "property_id").unwrap_or_default().to_owned(),
        linked_ids: sorted_strings(string_array_field(value, "linked_ids")),
        source_scope: string_field(value, "source_scope").unwrap_or_default().to_owned(),
        runner: HegelRunnerSummary {
            crate_version: value
                .get("runner")
                .and_then(|runner| string_field(runner, "crate_version"))
                .unwrap_or_default()
                .to_owned(),
            backend: value
                .get("runner")
                .and_then(|runner| string_field(runner, "backend"))
                .unwrap_or_default()
                .to_owned(),
        },
        settings: HegelSettingsSummary {
            case_budget: value
                .get("settings")
                .and_then(|settings| u64_field(settings, "case_budget"))
                .unwrap_or_default(),
            phases: value
                .get("settings")
                .map(|settings| sorted_strings(string_array_field(settings, "phases")))
                .unwrap_or_default(),
            seed: value.get("settings").and_then(|settings| u64_field(settings, "seed")),
            derandomized: value
                .get("settings")
                .and_then(|settings| settings.get("derandomized"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        },
        generator: HegelGeneratorSummary {
            family: value
                .get("generator")
                .and_then(|generator| string_field(generator, "family"))
                .unwrap_or_default()
                .to_owned(),
            content_hash: value
                .get("generator")
                .and_then(|generator| string_field(generator, "content_hash"))
                .unwrap_or_default()
                .to_owned(),
        },
        source_hashes: sorted_strings(string_array_field(value, "source_hashes")),
        counterexample,
        hardened: None,
    }
}

fn parse_hegel_counterexample(
    counterexample: &serde_json::Value,
    diagnostics: &mut Vec<String>,
) -> Option<HegelCounterexampleSummary> {
    let Some(summary_hash) = string_field(counterexample, "summary_hash") else {
        diagnostics.push("Hegel counterexample requires summary_hash".to_owned());
        return None;
    };
    if !is_b3_identity(summary_hash) {
        diagnostics.push("Hegel counterexample summary_hash must be a BLAKE3 identity".to_owned());
    }
    Some(HegelCounterexampleSummary {
        summary_hash: summary_hash.to_owned(),
        issue_codes: sorted_strings(string_array_field(counterexample, "issue_codes")),
    })
}

fn validate_hegel_generator_hash(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    let content_hash = value.get("generator").and_then(|generator| string_field(generator, "content_hash"));
    if let Some(content_hash) = content_hash
        && !is_b3_identity(content_hash)
    {
        diagnostics.push("Hegel generator content_hash must be a BLAKE3 identity".to_owned());
    }
}

fn validate_hegel_settings(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    let Some(settings) = value.get("settings") else {
        diagnostics.push("Hegel receipt requires settings object".to_owned());
        return;
    };
    let Some(settings) = settings.as_object() else {
        diagnostics.push("Hegel settings field must be an object".to_owned());
        return;
    };
    match settings.get("case_budget").and_then(serde_json::Value::as_u64) {
        Some(budget) if budget >= HEGEL_FAST_CASE_BUDGET => {}
        Some(_) => diagnostics
            .push(format!("Hegel settings case_budget must be at least named fast budget {HEGEL_FAST_CASE_BUDGET}")),
        None => diagnostics.push("Hegel settings requires case_budget".to_owned()),
    }
    let phases = settings.get("phases").and_then(serde_json::Value::as_array);
    if phases.is_none_or(|phases| phases.is_empty()) {
        diagnostics.push("Hegel settings requires at least one phase".to_owned());
    }
    if !settings.contains_key("derandomized") {
        diagnostics.push("Hegel settings requires derandomized flag".to_owned());
    }
}

fn validate_b3_array(value: &serde_json::Value, field: &str, label: &str, diagnostics: &mut Vec<String>) {
    let hashes = string_array_field(value, field);
    if hashes.is_empty() {
        diagnostics.push(format!("{label} field `{field}` must contain at least one BLAKE3 identity"));
        return;
    }
    for hash in hashes {
        if !is_b3_identity(&hash) {
            diagnostics.push(format!("{label} `{hash}` must be a BLAKE3 identity"));
        }
    }
}

fn validate_hegel_boundary(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    let non_claims = string_array_field(value, "non_claims");
    if !non_claims.iter().any(|claim| claim == HEGEL_GENERATED_TESTING_NON_CLAIM) {
        diagnostics.push("Hegel receipt missing generated-testing non-claim".to_owned());
    }
    let claim_text = non_claims.join(" ");
    for marker in [
        "formal proof",
        "exhaustive correctness",
        "proves correctness",
        "proves semantic equivalence",
        "proves verifier soundness",
        "proves requirement satisfaction",
        "proves release eligibility",
        "whole-program correctness",
    ] {
        if claim_text.contains(marker) {
            diagnostics.push("Hegel receipt contains unsupported proof or correctness overclaim".to_owned());
            break;
        }
    }
    if string_field(value, "evidence_tier") == Some(FORMAL_PROOF_TIER) {
        diagnostics.push("Hegel property receipts cannot be promoted to formal_proof".to_owned());
    }
}

fn hegel_finding(value: &serde_json::Value) -> EvidenceRailFinding {
    let property_id = string_field(value, "property_id").unwrap_or("unknown property");
    EvidenceRailFinding {
        family: "hegel-counterexample".to_owned(),
        severity: "error".to_owned(),
        message: format!("Hegel found a counterexample for {property_id}"),
        source_severity: None,
        file: None,
        line: None,
        column: None,
    }
}

fn is_b3_identity(value: &str) -> bool {
    let Some(hex) = value.strip_prefix(BLAKE3_IDENTITY_PREFIX) else {
        return false;
    };
    hex.len() == BLAKE3_HEX_LENGTH && hex.chars().all(|character| character.is_ascii_hexdigit())
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn validate_required_object_strings(
    value: &serde_json::Value,
    field: &str,
    label: &str,
    required_strings: &[&str],
    diagnostics: &mut Vec<String>,
) {
    let Some(object) = value.get(field) else {
        diagnostics.push(format!("{label} requires `{field}` object"));
        return;
    };
    let Some(object) = object.as_object() else {
        diagnostics.push(format!("{label} field `{field}` must be an object"));
        return;
    };
    for required in required_strings {
        match object.get(*required).and_then(serde_json::Value::as_str) {
            Some(value) if !value.trim().is_empty() => {}
            _ => diagnostics.push(format!("{label} field `{field}.{required}` must be a non-empty string")),
        }
    }
}

fn validate_required_string_field(value: &serde_json::Value, field: &str, label: &str, diagnostics: &mut Vec<String>) {
    match value.get(field).and_then(serde_json::Value::as_str) {
        Some(value) if !value.trim().is_empty() => {}
        _ => diagnostics.push(format!("{label} field `{field}` must be a non-empty string")),
    }
}

fn push_kani_verdict_finding(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) {
    match string_field(value, "verdict") {
        Some(STATUS_PASSED) => {}
        Some(STATUS_FAILED) => {
            findings.push(kani_finding("kani-check-failure", "error", "Kani reported a failing verification verdict"))
        }
        Some(other) => diagnostics.push(format!("Kani verdict `{other}` is unsupported; expected passed or failed")),
        None => diagnostics.push("Kani report requires verdict `passed` or `failed`".to_owned()),
    }
}

fn validate_kani_assumptions(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    if string_field(value, "assumption_status") == Some("vacuous") {
        diagnostics.push("Kani assumptions are marked vacuous".to_owned());
    }
    let property_claim = value
        .get("claims")
        .and_then(|claims| claims.get("property_evidence"))
        .and_then(serde_json::Value::as_bool)
        == Some(true);
    if property_claim && value.get("assumptions_reviewed").and_then(serde_json::Value::as_bool) == Some(false) {
        diagnostics.push("Kani property evidence requires reviewed assumptions".to_owned());
    }
}

fn push_kani_property_findings(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    property_classes: &mut Vec<String>,
    diagnostics: &mut Vec<String>,
) {
    for property in array_field(value, "properties") {
        let name = string_field(property, "name").unwrap_or("unnamed property");
        if let Some(property_class) = string_field(property, "property_class") {
            push_kani_property_class(property_class, property_classes, diagnostics);
        }
        match string_field(property, "status") {
            Some(STATUS_PASSED) => {}
            Some(STATUS_FAILED) => {
                findings.push(kani_finding("kani-property-failure", "error", &format!("Kani property failed: {name}")))
            }
            Some(other) => diagnostics.push(format!("Kani property `{name}` has unsupported status `{other}`")),
            None => diagnostics.push(format!("Kani property `{name}` missing status")),
        }
    }
}

fn push_kani_property_class(property_class: &str, property_classes: &mut Vec<String>, diagnostics: &mut Vec<String>) {
    if !known_kani_property_classes().contains(&property_class) {
        diagnostics.push(format!("unsupported Kani property class `{property_class}`"));
        return;
    }
    if !property_classes.iter().any(|known| known == property_class) {
        property_classes.push(property_class.to_owned());
    }
}

fn known_kani_property_classes() -> &'static [&'static str] {
    &[
        KANI_PROPERTY_CLASS_NORMALIZATION,
        KANI_PROPERTY_CLASS_ALGEBRAIC_LAW,
        KANI_PROPERTY_CLASS_ENCODE_DECODE,
        KANI_PROPERTY_CLASS_VALIDITY_RANGE,
        KANI_PROPERTY_CLASS_LOOP_TERMINATION,
        KANI_PROPERTY_CLASS_TRAIT_CONSISTENCY,
        KANI_PROPERTY_CLASS_BOUNDARY_PANIC,
    ]
}

fn push_kani_unwind_findings(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) -> bool {
    let Some(unwind) = value.get("unwind") else {
        return false;
    };
    let Some(unwind) = unwind.as_object() else {
        diagnostics.push("Kani unwind field must be an object".to_owned());
        return false;
    };
    match unwind.get("status").and_then(serde_json::Value::as_str) {
        Some(STATUS_PASSED) => true,
        Some(STATUS_FAILED) => {
            findings.push(kani_finding(
                "unwind-assertion-failure",
                "error",
                "Kani reported failed unwinding assertions",
            ));
            false
        }
        Some("not_applicable" | "bounded_only") => false,
        Some(other) => {
            diagnostics.push(format!("Kani unwind status `{other}` is unsupported"));
            false
        }
        None => {
            diagnostics.push("Kani unwind object missing status".to_owned());
            false
        }
    }
}

fn push_kani_contract_findings(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) -> bool {
    let mut contract_passed = false;
    for contract in array_field(value, "contracts") {
        let name = string_field(contract, "name").unwrap_or("unnamed contract");
        match string_field(contract, "status") {
            Some(STATUS_PASSED) => contract_passed = true,
            Some(STATUS_FAILED) => {
                findings.push(kani_finding("contract-failure", "error", &format!("Kani contract failed: {name}")))
            }
            Some(other) => diagnostics.push(format!("Kani contract `{name}` has unsupported status `{other}`")),
            None => diagnostics.push(format!("Kani contract `{name}` missing status")),
        }
    }
    contract_passed
}

fn push_kani_stub_findings(
    value: &serde_json::Value,
    findings: &mut Vec<EvidenceRailFinding>,
    diagnostics: &mut Vec<String>,
) -> bool {
    let mut boundary_stub_seen = false;
    for stub in array_field(value, "stubs") {
        let target = string_field(stub, "target").unwrap_or("unknown target");
        match string_field(stub, "kind") {
            Some(KANI_STUB_KIND_VERIFIED) => {
                validate_required_string_field(stub, "contract_receipt", "Kani verified stub", diagnostics);
            }
            Some(KANI_STUB_KIND_PLAIN | KANI_STUB_KIND_ENVIRONMENT) => {
                boundary_stub_seen = true;
                findings.push(kani_finding(
                    "stub-boundary",
                    "manual_review",
                    &format!("Kani stub boundary requires review: {target}"),
                ));
            }
            Some(other) => diagnostics.push(format!("Kani stub `{target}` has unsupported kind `{other}`")),
            None => diagnostics.push(format!("Kani stub `{target}` missing kind")),
        }
    }
    boundary_stub_seen
}

fn validate_kani_claims(
    value: &serde_json::Value,
    unwind_passed: bool,
    contract_passed: bool,
    boundary_stub_seen: bool,
    diagnostics: &mut Vec<String>,
) {
    let Some(claims) = value.get("claims") else {
        return;
    };
    let unbounded_claim = string_field(claims, "coverage") == Some(KANI_COVERAGE_UNBOUNDED)
        || claims.get("unbounded").and_then(serde_json::Value::as_bool) == Some(true);
    if unbounded_claim && !unwind_passed && !contract_passed {
        diagnostics.push(
            "Kani unbounded claim requires passing unwinding assertions or a passing contract receipt".to_owned(),
        );
    }
    let property_claim = claims.get("property_evidence").and_then(serde_json::Value::as_bool) == Some(true);
    if property_claim && boundary_stub_seen {
        diagnostics.push("plain or environmental Kani stubs cannot be promoted to property evidence".to_owned());
    }
}

fn kani_finding(family: &str, severity: &str, message: &str) -> EvidenceRailFinding {
    EvidenceRailFinding {
        family: family.to_owned(),
        severity: severity.to_owned(),
        message: message.to_owned(),
        source_severity: None,
        file: None,
        line: None,
        column: None,
    }
}

fn status_from_kani_findings(findings: &[EvidenceRailFinding]) -> String {
    if findings.iter().any(|finding| finding.severity == "error") {
        STATUS_FAILED.to_owned()
    } else if findings.iter().any(|finding| finding.severity == "manual_review") {
        STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
    } else {
        STATUS_PASSED.to_owned()
    }
}

fn input_text(
    args: &EvidenceRailArgs,
    prepared_clippy_run: Option<&PreparedClippyRun>,
) -> Result<(&'static str, String, Option<i32>), String> {
    if args.rail == EvidenceRailKind::WasmArtifact {
        return analyze_wasm_input(args).map(|envelope| (MODE_IMPORT, envelope, None));
    }
    if args.rail == EvidenceRailKind::CharonSemantic {
        return analyze_charon_semantic_input(args);
    }
    if args.rail == EvidenceRailKind::SarifImport {
        return analyze_sarif_input(args).map(|report| (MODE_IMPORT, report, None));
    }
    if args.rail == EvidenceRailKind::MutationTesting && args.profile.as_deref() == Some(crate::mutation::PROFILE_ID) {
        return analyze_profiled_mutation_input(args);
    }
    if args.rail == EvidenceRailKind::CoverageBaseline && args.profile.as_deref() == Some(crate::coverage::PROFILE_ID) {
        return analyze_profiled_coverage_input(args);
    }
    if args.rail == EvidenceRailKind::BenchmarkEvidence && args.profile.as_deref() == Some(crate::benchmark::PROFILE_ID)
    {
        return analyze_profiled_benchmark_input(args);
    }
    if args.rail == EvidenceRailKind::HegelProperty && args.profile.as_deref() == Some(crate::hegel::PROFILE_ID) {
        return analyze_profiled_hegel_input(args);
    }
    if args.rail == EvidenceRailKind::DynamicEvidence
        && args.profile.as_deref() == Some(crate::cargo::fuzz::PROFILE_ID)
        && args.run
    {
        dynamic_run_command_argv(args.profile.as_deref(), args.command.as_deref())?;
        let workspace_root = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
        return crate::cargo::fuzz::shell::run(workspace_root)
            .map(|(receipt, exit_code)| (MODE_RUN, receipt, Some(exit_code)));
    }
    match (&args.input, args.run) {
        (Some(input), false) => {
            let text = if args.rail == EvidenceRailKind::DynamicEvidence
                && args.profile.as_deref() == Some(crate::cargo::fuzz::PROFILE_ID)
            {
                read_cargo_fuzz_receipt(input)?
            } else {
                fs::read_to_string(input)
                    .map_err(|error| format!("read evidence input {}: {error}", input.display()))?
            };
            Ok((MODE_IMPORT, text, None))
        }
        (None, true) => {
            run_tool(args, prepared_clippy_run).map(|output| (MODE_RUN, output.stdout_stderr, Some(output.exit_code)))
        }
        (Some(_), true) => Err("evidence collect accepts either `--input PATH` or `--run`, not both".to_owned()),
        (None, false) => Err("evidence collect requires `--input PATH` or `--run`".to_owned()),
    }
}

fn analyze_profiled_hegel_input(args: &EvidenceRailArgs) -> Result<(&'static str, String, Option<i32>), String> {
    const REQUEST_LIMIT_MULTIPLIER: u64 = 8;

    if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
        return Err("profiled Hegel evidence rejects arbitrary `--command` authority".to_owned());
    }
    if args.run && args.input.is_some() {
        return Err("profiled Hegel evidence accepts either `--input PATH` or `--run`, not both".to_owned());
    }
    if !args.external_reports.is_empty() || !args.parent_artifacts.is_empty() {
        return Err("profiled Hegel evidence rejects undeclared external reports and parent artifacts".to_owned());
    }
    let profile_id = args
        .profile
        .as_deref()
        .ok_or_else(|| "profiled Hegel evidence requires `--profile NAME`".to_owned())?;
    let profile = crate::hegel::selected_profile(profile_id)?;
    let config = args
        .config
        .as_deref()
        .ok_or_else(|| "profiled Hegel evidence requires `--config PATH`".to_owned())?;
    let workspace = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
    let config_bytes =
        crate::hegel::safe_read::read_checked_input(workspace, config, profile.limits.max_summary_bytes)?;
    if crate::hegel::prefixed_identity(&config_bytes) != crate::hegel::PROFILE_IDENTITY {
        return Err("Hegel profile bytes do not match the checked Nickel registry".to_owned());
    }
    if args.run {
        let observation = crate::hegel::shell::run(workspace, &profile)?;
        return Ok((MODE_RUN, crate::hegel::request::render(&observation.input)?, Some(observation.exit_code)));
    }
    let input = args
        .input
        .as_deref()
        .ok_or_else(|| "profiled Hegel evidence requires `--input PATH` or `--run`".to_owned())?;
    let request_limit = u64::try_from(profile.limits.max_log_bytes)
        .map_err(|_| "Hegel request log limit does not fit u64".to_owned())?
        .checked_mul(REQUEST_LIMIT_MULTIPLIER)
        .ok_or_else(|| "Hegel request byte limit overflowed".to_owned())?;
    let bytes = crate::hegel::safe_read::read_checked_input(workspace, input, request_limit)?;
    let text = String::from_utf8(bytes).map_err(|error| format!("profiled Hegel input is not UTF-8: {error}"))?;
    Ok((MODE_IMPORT, text, None))
}

fn analyze_sarif_input(args: &EvidenceRailArgs) -> Result<String, String> {
    if args.run {
        return Err("SARIF import accepts a local `--input PATH` and rejects `--run`".to_owned());
    }
    if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
        return Err("SARIF import rejects arbitrary `--command` authority".to_owned());
    }
    let input = args.input.as_deref().ok_or_else(|| "SARIF import requires `--input PATH`".to_owned())?;
    let config = args.config.as_deref().ok_or_else(|| "SARIF import requires `--config PATH`".to_owned())?;
    let profile = args.profile.as_deref().ok_or_else(|| "SARIF import requires `--profile NAME`".to_owned())?;
    crate::external::sarif::selected_profile(profile)?;
    let workspace = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
    let root = SourceRoot::open_existing(workspace)?;
    let config_bytes = root.read_bounded(&RootRelativePath::new(config)?, crate::external::sarif::MAX_REPORT_BYTES)?;
    if crate::external::sarif::prefixed_identity(&config_bytes) != crate::external::sarif::PROFILE_IDENTITY {
        return Err("SARIF profile bytes do not match the checked Nickel registry".to_owned());
    }
    let report = root.read_bounded(&RootRelativePath::new(input)?, crate::external::sarif::MAX_REPORT_BYTES)?;
    String::from_utf8(report).map_err(|error| format!("SARIF report is not UTF-8: {error}"))
}

fn analyze_profiled_mutation_input(args: &EvidenceRailArgs) -> Result<(&'static str, String, Option<i32>), String> {
    if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
        return Err("profiled mutation testing rejects arbitrary `--command` authority".to_owned());
    }
    if args.run && args.input.is_some() {
        return Err("profiled mutation testing accepts either `--input PATH` or `--run`, not both".to_owned());
    }
    let profile_id = args
        .profile
        .as_deref()
        .ok_or_else(|| "profiled mutation testing requires `--profile NAME`".to_owned())?;
    let profile = crate::mutation::selected_profile(profile_id)?;
    let config = args
        .config
        .as_deref()
        .ok_or_else(|| "profiled mutation testing requires `--config PATH`".to_owned())?;
    if args.external_reports.len() != 1 {
        return Err("profiled mutation testing requires one source manifest through `--external-report`".to_owned());
    }
    let workspace = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
    let root = SourceRoot::open_existing(workspace)?;
    let config_bytes = root.read_bounded(&RootRelativePath::new(config)?, profile.max_report_bytes)?;
    if crate::mutation::prefixed_identity(&config_bytes) != crate::mutation::PROFILE_IDENTITY {
        return Err("mutation profile bytes do not match the checked Nickel registry".to_owned());
    }
    let source_bytes =
        root.read_bounded(&RootRelativePath::new(&args.external_reports[0])?, profile.max_report_bytes)?;
    let source_manifest_text =
        String::from_utf8(source_bytes).map_err(|error| format!("mutation source manifest is not UTF-8: {error}"))?;
    let (mode, parts, exit_code) = if args.run {
        let observation = crate::mutation::shell::run(workspace, &profile, source_manifest_text)?;
        (MODE_RUN, observation.parts, Some(observation.exit_code))
    } else {
        let input = args
            .input
            .as_deref()
            .ok_or_else(|| "profiled mutation testing requires `--input PATH` or `--run`".to_owned())?;
        if RootRelativePath::new(input)?.display() != profile.native_outcomes_path {
            return Err(format!("profiled mutation input must be `{}`", profile.native_outcomes_path));
        }
        let mut parts = crate::mutation::shell::collect_native(workspace, &profile)?;
        parts.source_manifest_text = source_manifest_text;
        (MODE_IMPORT, parts, None)
    };
    Ok((mode, crate::mutation::request::render(parts)?, exit_code))
}

fn analyze_profiled_benchmark_input(args: &EvidenceRailArgs) -> Result<(&'static str, String, Option<i32>), String> {
    if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
        return Err("profiled benchmark evidence rejects arbitrary `--command` authority".to_owned());
    }
    if args.run && args.input.is_some() {
        return Err("profiled benchmark evidence accepts either `--input PATH` or `--run`, not both".to_owned());
    }
    let profile_id = args
        .profile
        .as_deref()
        .ok_or_else(|| "profiled benchmark evidence requires `--profile NAME`".to_owned())?;
    let profile = crate::benchmark::selected_profile(profile_id)?;
    let config = args
        .config
        .as_deref()
        .ok_or_else(|| "profiled benchmark evidence requires `--config PATH`".to_owned())?;
    let workspace = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
    let root = SourceRoot::open_existing(workspace)?;
    let config_bytes = root.read_bounded(&RootRelativePath::new(config)?, profile.limits.max_artifact_bytes)?;
    if crate::benchmark::prefixed_identity(&config_bytes) != crate::benchmark::PROFILE_IDENTITY {
        return Err("benchmark profile bytes do not match the checked Nickel registry".to_owned());
    }
    if args.run {
        if !args.external_reports.is_empty() {
            return Err("profiled benchmark run rejects untyped external reports".to_owned());
        }
        let baseline = args
            .baseline
            .as_deref()
            .ok_or_else(|| "profiled benchmark run requires `--baseline PATH`".to_owned())?;
        let bytes = root.read_bounded(&RootRelativePath::new(baseline)?, profile.limits.max_artifact_bytes)?;
        let baseline: crate::benchmark::RunSet =
            serde_json::from_slice(&bytes).map_err(|error| format!("parse benchmark baseline run set: {error}"))?;
        let observation = crate::benchmark::shell::run(workspace, &profile, baseline)?;
        return Ok((MODE_RUN, crate::benchmark::request::render(&observation.input)?, Some(observation.exit_code)));
    }
    if args.baseline.is_some() || !args.external_reports.is_empty() {
        return Err(
            "profiled benchmark import accepts one complete typed input and rejects separate parents".to_owned()
        );
    }
    let input = args
        .input
        .as_deref()
        .ok_or_else(|| "profiled benchmark import requires `--input PATH`".to_owned())?;
    let bytes = root.read_bounded(&RootRelativePath::new(input)?, profile.limits.max_artifact_bytes)?;
    let text = String::from_utf8(bytes).map_err(|error| format!("benchmark input is not UTF-8: {error}"))?;
    crate::benchmark::request::parse(&text)?;
    Ok((MODE_IMPORT, text, None))
}

fn analyze_profiled_coverage_input(args: &EvidenceRailArgs) -> Result<(&'static str, String, Option<i32>), String> {
    if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
        return Err("profiled coverage rejects arbitrary `--command` authority".to_owned());
    }
    if args.run && args.input.is_some() {
        return Err("profiled coverage accepts either `--input PATH` or `--run`, not both".to_owned());
    }
    let profile_id = args.profile.as_deref().ok_or_else(|| "profiled coverage requires `--profile NAME`".to_owned())?;
    let profile = crate::coverage::selected_profile(profile_id)?;
    let config = args.config.as_deref().ok_or_else(|| "profiled coverage requires `--config PATH`".to_owned())?;
    let baseline = args.baseline.as_deref().ok_or_else(|| "profiled coverage requires `--baseline PATH`".to_owned())?;
    let workspace = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
    let root = SourceRoot::open_existing(workspace)?;
    let (source_identity, source_manifest_text) =
        selected_coverage_source_identity(&args.external_reports, &root, profile.max_report_bytes)?;
    let config_bytes = root.read_bounded(&RootRelativePath::new(config)?, profile.max_report_bytes)?;
    if crate::coverage::prefixed_identity(&config_bytes) != crate::coverage::PROFILE_IDENTITY {
        return Err("coverage profile bytes do not match the checked Nickel registry".to_owned());
    }
    let baseline_bytes = root.read_bounded(&RootRelativePath::new(baseline)?, profile.max_report_bytes)?;
    let baseline_text =
        String::from_utf8(baseline_bytes).map_err(|error| format!("coverage baseline input is not UTF-8: {error}"))?;
    let baseline_text = crate::coverage::prepare_baseline_text(&baseline_text, profile_id, &source_identity)?;
    let (mode, current_text, log_identity, exit_code) = if args.run {
        let observation = crate::coverage::shell::run(workspace, &profile)?;
        (MODE_RUN, observation.current_text, observation.bounded_log_identity, Some(observation.exit_code))
    } else {
        let input = args
            .input
            .as_deref()
            .ok_or_else(|| "profiled coverage requires `--input PATH` or `--run`".to_owned())?;
        let current_bytes = root.read_bounded(&RootRelativePath::new(input)?, profile.max_report_bytes)?;
        let current_text = String::from_utf8(current_bytes)
            .map_err(|error| format!("coverage current report is not UTF-8: {error}"))?;
        (MODE_IMPORT, current_text, crate::coverage::prefixed_identity(b""), None)
    };
    let request = crate::coverage::request::render(
        current_text,
        baseline_text,
        source_identity,
        source_manifest_text,
        log_identity,
    )?;
    Ok((mode, request, exit_code))
}

fn selected_coverage_source_identity(
    reports: &[std::path::PathBuf],
    root: &SourceRoot,
    max_bytes: u64,
) -> Result<(String, String), String> {
    if reports.len() != 1 {
        return Err("profiled coverage requires one `--external-report` source manifest".to_owned());
    }
    let bytes = root.read_bounded(&RootRelativePath::new(&reports[0])?, max_bytes)?;
    if bytes.is_empty() {
        return Err("profiled coverage source manifest is empty".to_owned());
    }
    let identity = format!("source:{}", crate::coverage::prefixed_identity(&bytes));
    let text =
        String::from_utf8(bytes).map_err(|error| format!("profiled coverage source manifest is not UTF-8: {error}"))?;
    Ok((identity, text))
}

fn read_cargo_fuzz_receipt(path: &Path) -> Result<String, String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("inspect cargo-fuzz receipt {}: {error}", path.display()))?;
    if metadata.len() > crate::cargo::fuzz::MAX_RECEIPT_BYTES {
        return Err(format!("cargo-fuzz receipt exceeds {} byte limit", crate::cargo::fuzz::MAX_RECEIPT_BYTES));
    }
    let file = fs::File::open(path).map_err(|error| format!("open cargo-fuzz receipt {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(crate::cargo::fuzz::MAX_RECEIPT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read cargo-fuzz receipt {}: {error}", path.display()))?;
    let observed_bytes =
        u64::try_from(bytes.len()).map_err(|_| "cargo-fuzz receipt byte count does not fit u64".to_owned())?;
    if observed_bytes > crate::cargo::fuzz::MAX_RECEIPT_BYTES {
        return Err("cargo-fuzz receipt grew beyond its byte limit during read".to_owned());
    }
    String::from_utf8(bytes).map_err(|error| format!("cargo-fuzz receipt {} is not UTF-8: {error}", path.display()))
}

fn analyze_charon_semantic_input(args: &EvidenceRailArgs) -> Result<(&'static str, String, Option<i32>), String> {
    let config_path = args
        .config
        .as_deref()
        .ok_or_else(|| "charon-semantic rail requires `--config PATH` to the generated profile".to_owned())?;
    let selected_profile =
        args.profile.as_deref().ok_or_else(|| "charon-semantic rail requires `--profile NAME`".to_owned())?;
    if selected_profile != crate::charon_semantic::shipped_profile_id() {
        return Err(format!("unsupported Charon semantic profile `{selected_profile}`"));
    }
    if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
        return Err("charon-semantic rail rejects arbitrary --command authority".to_owned());
    }
    let profile_text = fs::read_to_string(config_path)
        .map_err(|error| format!("read Charon semantic profile {}: {error}", config_path.display()))?;
    if crate::charon_semantic::prefixed_identity(profile_text.as_bytes())
        != crate::charon_semantic::EXPECTED_PROFILE_IDENTITY
    {
        return Err("Charon semantic profile bytes do not match the shipped profile identity".to_owned());
    }
    let comparison_parent_texts = args
        .external_reports
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .map_err(|error| format!("read Charon semantic comparison parent {}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (mode, snapshot_text, exit_code) = match (&args.input, args.run) {
        (Some(path), false) => {
            let text = fs::read_to_string(path)
                .map_err(|error| format!("read Charon snapshot {}: {error}", path.display()))?;
            (MODE_IMPORT, text, None)
        }
        (None, true) => {
            let workspace_root = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
            let output = crate::charon_semantic::execute_from_profile(workspace_root, &profile_text);
            if !output.diagnostics.is_empty() {
                return Err(format!("pinned Charon semantic execution failed:\n- {}", output.diagnostics.join("\n- ")));
            }
            (MODE_RUN, output.snapshot_text, Some(output.exit_code))
        }
        (Some(_), true) => {
            return Err("charon-semantic rail accepts either `--input PATH` or `--run`, not both".to_owned());
        }
        (None, false) => return Err("charon-semantic rail requires `--input PATH` or `--run`".to_owned()),
    };
    let envelope = crate::charon_semantic::build_envelope(snapshot_text, comparison_parent_texts)?;
    Ok((mode, envelope, exit_code))
}

fn analyze_wasm_input(args: &EvidenceRailArgs) -> Result<String, String> {
    if args.run {
        return Err("wasm-artifact rail analyzes `--input PATH` directly and rejects `--run`".to_owned());
    }
    let input_path = args.input.as_deref().ok_or_else(|| "wasm-artifact rail requires `--input PATH`".to_owned())?;
    let config_path = args
        .config
        .as_deref()
        .ok_or_else(|| "wasm-artifact rail requires `--config PATH` to a generated profile registry".to_owned())?;
    let profile_id =
        args.profile.as_deref().ok_or_else(|| "wasm-artifact rail requires `--profile NAME`".to_owned())?;
    let artifact_bytes =
        fs::read(input_path).map_err(|error| format!("read Wasm artifact {}: {error}", input_path.display()))?;
    let registry_bytes = fs::read(config_path)
        .map_err(|error| format!("read Wasm artifact registry {}: {error}", config_path.display()))?;
    let registry_text = std::str::from_utf8(&registry_bytes)
        .map_err(|error| format!("Wasm artifact registry {} is not UTF-8: {error}", config_path.display()))?;
    let registry = crate::wasm_artifact::parse_registry(registry_text)?;
    let external_report_texts = args
        .external_reports
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .map_err(|error| format!("read external runtime report {}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let registry_identity = crate::wasm_artifact::registry_identity(&registry_bytes);
    let envelope = if external_report_texts.is_empty() {
        crate::wasm_artifact::analyze_bytes(
            &artifact_bytes,
            &registry,
            profile_id,
            registry_identity,
            &args.parent_artifacts,
        )
    } else {
        crate::wasm_artifact::analyze_bytes_with_external(
            &artifact_bytes,
            &registry,
            profile_id,
            registry_identity,
            &args.parent_artifacts,
            &external_report_texts,
        )
    }
    .map_err(|diagnostics| format!("invalid Wasm artifact profile:\n- {}", diagnostics.join("\n- ")))?;
    crate::wasm_artifact::render_envelope(&envelope)
}

struct ToolOutput {
    stdout_stderr: String,
    exit_code: i32,
}

fn run_tool(args: &EvidenceRailArgs, prepared_clippy_run: Option<&PreparedClippyRun>) -> Result<ToolOutput, String> {
    if args.rail == EvidenceRailKind::AstGrepStructural {
        return run_ast_grep_tool(args);
    }
    let command_parts = prepared_clippy_run
        .map(|prepared| prepared.command_argv.clone())
        .map_or_else(|| run_mode_command_argv(args), Ok)?;
    let (program, rest) =
        command_parts.split_first().ok_or_else(|| "evidence rail command argv is empty".to_owned())?;
    let mut command = Command::new(program);
    command.args(rest).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(workspace_root) = args.workspace_root.as_deref() {
        command.current_dir(workspace_root);
    }
    let output = command
        .output()
        .map_err(|error| format!("run evidence rail command `{}`: {error}", command_parts.join(" ")))?;
    let mut stdout_stderr = String::new();
    stdout_stderr.push_str(&String::from_utf8_lossy(&output.stdout));
    if args.rail != EvidenceRailKind::ClippyCoReporting {
        stdout_stderr.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    Ok(ToolOutput {
        stdout_stderr,
        exit_code: output.status.code().unwrap_or(EXIT_INTEGRATION_FAILURE),
    })
}

fn run_ast_grep_tool(args: &EvidenceRailArgs) -> Result<ToolOutput, String> {
    let profile = load_ast_grep_profile(args)?;
    let workspace_root = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
    let canonical_root = fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize ast-grep workspace root {}: {error}", workspace_root.display()))?;
    let mut bundle_files = BTreeMap::new();
    for relative in &profile.bundle_inputs {
        let path = workspace_root.join(relative);
        let canonical_path = fs::canonicalize(&path)
            .map_err(|error| format!("canonicalize ast-grep bundle input {}: {error}", path.display()))?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(format!("ast-grep bundle input `{relative}` escapes the workspace root"));
        }
        let bytes = fs::read(&canonical_path)
            .map_err(|error| format!("read ast-grep bundle input {}: {error}", canonical_path.display()))?;
        bundle_files.insert(relative.clone(), bytes);
    }
    let observed_identity = crate::ast_grep_rail::compute_bundle_identity(&profile, &bundle_files)
        .map_err(|diagnostics| format!("invalid ast-grep rule bundle:\n- {}", diagnostics.join("\n- ")))?;
    if observed_identity != profile.bundle_identity {
        return Err(format!(
            "stale ast-grep rule bundle identity; expected `{}`, observed `{observed_identity}`",
            profile.bundle_identity
        ));
    }

    let version_output = Command::new("ast-grep")
        .arg("--version")
        .current_dir(workspace_root)
        .output()
        .map_err(|error| format!("run ast-grep --version: {error}"))?;
    if !version_output.status.success() {
        return Err(format!("ast-grep --version failed: {}", String::from_utf8_lossy(&version_output.stderr).trim()));
    }
    let version_text = String::from_utf8_lossy(&version_output.stdout);
    let observed_version = version_text
        .trim()
        .strip_prefix("ast-grep ")
        .ok_or_else(|| format!("unexpected ast-grep version output `{}`", version_text.trim()))?
        .to_owned();

    let command_parts = crate::ast_grep_rail::scan_command_argv(&profile);
    let (program, rest) = command_parts.split_first().ok_or_else(|| "ast-grep command argv is empty".to_owned())?;
    let output = Command::new(program)
        .args(rest)
        .current_dir(workspace_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("run ast-grep structural rail: {error}"))?;
    let exit_code = output.status.code().unwrap_or(EXIT_INTEGRATION_FAILURE);
    let source_findings = if output.stdout.is_empty() && !output.status.success() {
        serde_json::Value::Array(Vec::new())
    } else {
        serde_json::from_slice(&output.stdout).map_err(|error| format!("parse ast-grep JSON output: {error}"))?
    };
    let envelope = crate::ast_grep_rail::render_scan_envelope(
        profile,
        observed_version,
        source_findings,
        String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code,
    )?;
    Ok(ToolOutput {
        stdout_stderr: envelope,
        exit_code,
    })
}

fn load_ast_grep_profile(args: &EvidenceRailArgs) -> Result<crate::ast_grep_rail::AstGrepProfile, String> {
    let path = args
        .config
        .as_deref()
        .ok_or_else(|| "ast-grep run requires `--config PATH` to a generated structural profile".to_owned())?;
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read ast-grep structural profile {}: {error}", path.display()))?;
    let profile = crate::ast_grep_rail::parse_profile(&text)?;
    let diagnostics = crate::ast_grep_rail::validate_profile(&profile);
    if diagnostics.is_empty() {
        Ok(profile)
    } else {
        Err(format!("invalid ast-grep structural profile:\n- {}", diagnostics.join("\n- ")))
    }
}

#[derive(Clone, Debug)]
struct PreparedClippyRun {
    config_path: String,
    config_identity: String,
    command_argv: Vec<String>,
    details: crate::clippy_restrictions::ClippyRestrictionReceiptDetails,
}

fn prepare_clippy_run(args: &EvidenceRailArgs) -> Result<Option<PreparedClippyRun>, String> {
    if !args.run || args.rail != EvidenceRailKind::ClippyCoReporting {
        return Ok(None);
    }
    if args.profile.is_some() && args.command.as_deref().is_some_and(|command| !command.is_empty()) {
        return Err("profiled Clippy run rejects arbitrary --command authority; use the checked profile".to_owned());
    }
    match (args.config.as_deref(), args.profile.as_deref()) {
        (None, None) => Ok(None),
        (Some(_), None) => Err("profiled Clippy run requires `--profile NAME` with `--config PATH`".to_owned()),
        (None, Some(profile_id)) => {
            let catalog = crate::clippy_restrictions::shipped_catalog()?;
            prepare_selected_clippy_profile(
                &catalog,
                profile_id,
                EMBEDDED_CLIPPY_PROFILE_LOCATOR.to_owned(),
                crate::clippy_restrictions::shipped_catalog_identity(),
            )
            .map(Some)
        }
        (Some(config_path), Some(profile_id)) => {
            let text = fs::read_to_string(config_path).map_err(|error| {
                format!("read Clippy restriction profile catalog {}: {error}", config_path.display())
            })?;
            let catalog = crate::clippy_restrictions::checked_catalog(&text)?;
            prepare_selected_clippy_profile(
                &catalog,
                profile_id,
                config_path.display().to_string(),
                prefixed_identity(text.as_bytes()),
            )
            .map(Some)
        }
    }
}

fn prepare_selected_clippy_profile(
    catalog: &crate::clippy_restrictions::ClippyRestrictionCatalog,
    profile_id: &str,
    config_path: String,
    config_identity: String,
) -> Result<PreparedClippyRun, String> {
    let profile = crate::clippy_restrictions::select_profile(catalog, profile_id)?;
    let details = crate::clippy_restrictions::receipt_details(&profile)?;
    Ok(PreparedClippyRun {
        config_path,
        config_identity,
        command_argv: crate::clippy_restrictions::command_argv(&profile),
        details,
    })
}

fn default_clippy_command_argv() -> Vec<String> {
    string_vec(&["cargo", "clippy", "--message-format=json"])
}

fn run_mode_command_argv(args: &EvidenceRailArgs) -> Result<Vec<String>, String> {
    let config_path = args.config.as_ref().map(|path| path.display().to_string());
    let baseline_identity = args.baseline.as_ref().map(|path| path.display().to_string());
    if args.rail == EvidenceRailKind::ClippyCoReporting {
        return prepare_clippy_run(args)
            .map(|prepared| prepared.map_or_else(default_clippy_command_argv, |prepared| prepared.command_argv));
    }
    if args.rail == EvidenceRailKind::AstGrepStructural {
        if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
            return Err("ast-grep run rejects arbitrary --command authority; use the checked profile".to_owned());
        }
        return load_ast_grep_profile(args).map(|profile| crate::ast_grep_rail::scan_command_argv(&profile));
    }
    if args.rail == EvidenceRailKind::DynamicEvidence {
        return dynamic_run_command_argv(args.profile.as_deref(), args.command.as_deref());
    }
    if args.rail == EvidenceRailKind::MutationTesting && args.profile.as_deref() == Some(crate::mutation::PROFILE_ID) {
        if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
            return Err("profiled mutation testing rejects arbitrary --command authority".to_owned());
        }
        return crate::mutation::selected_profile(crate::mutation::PROFILE_ID).map(|profile| profile.command_argv);
    }
    if args.rail == EvidenceRailKind::CoverageBaseline && args.profile.as_deref() == Some(crate::coverage::PROFILE_ID) {
        if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
            return Err("profiled coverage rejects arbitrary --command authority".to_owned());
        }
        return crate::coverage::selected_profile(crate::coverage::PROFILE_ID).map(|profile| profile.command_argv);
    }
    if args.rail == EvidenceRailKind::BenchmarkEvidence && args.profile.as_deref() == Some(crate::benchmark::PROFILE_ID)
    {
        if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
            return Err("profiled benchmark evidence rejects arbitrary --command authority".to_owned());
        }
        let profile = crate::benchmark::selected_profile(crate::benchmark::PROFILE_ID)?;
        return profile
            .workloads
            .first()
            .map(|workload| workload.command_argv.clone())
            .ok_or_else(|| "benchmark profile has no workloads".to_owned());
    }
    if args.rail == EvidenceRailKind::HegelProperty && args.profile.as_deref() == Some(crate::hegel::PROFILE_ID) {
        if args.command.as_deref().is_some_and(|command| !command.is_empty()) {
            return Err("profiled Hegel evidence rejects arbitrary --command authority".to_owned());
        }
        return crate::hegel::selected_profile(crate::hegel::PROFILE_ID).map(|profile| profile.command_argv);
    }
    if args.rail == EvidenceRailKind::HegelProperty {
        return Err(
            "hegel-property run requires the checked profiled producer and rejects arbitrary --command authority"
                .to_owned(),
        );
    }
    if args.rail == EvidenceRailKind::Z4Solver && args.command.as_deref().is_none_or(str::is_empty) {
        return Err("z4-solver run requires an explicit --command and never bootstraps Z4 automatically".to_owned());
    }
    if args.rail == EvidenceRailKind::VerifierFixture && args.command.as_deref().is_none_or(str::is_empty) {
        return Err(
            "verifier-fixture run requires an explicit --command and never bootstraps external verifiers automatically"
                .to_owned(),
        );
    }
    Ok(command_argv(
        args.rail,
        config_path.as_deref(),
        baseline_identity.as_deref(),
        args.command.as_deref(),
    ))
}

fn command_argv(
    rail: EvidenceRailKind,
    config_path: Option<&str>,
    baseline_identity: Option<&str>,
    requested_command: Option<&str>,
) -> Vec<String> {
    match rail {
        EvidenceRailKind::DependencyAudit => {
            let mut argv = string_vec(&["cargo", "deny", "check"]);
            if let Some(config_path) = config_path {
                argv.push("--config".to_owned());
                argv.push(config_path.to_owned());
            }
            argv
        }
        EvidenceRailKind::ClippyCoReporting => {
            requested_command.map_or_else(default_clippy_command_argv, split_command)
        }
        EvidenceRailKind::AstGrepStructural => {
            requested_command.map_or_else(|| string_vec(&["ast-grep", "scan", "--json=compact"]), split_command)
        }
        EvidenceRailKind::WasmArtifact => {
            string_vec(&["cargo", "octet", "evidence", "collect", "--rail", "wasm-artifact"])
        }
        EvidenceRailKind::CharonSemantic => crate::charon_semantic::profile_command_argv().unwrap_or_default(),
        EvidenceRailKind::DependencyTrust => string_vec(&["cargo", "vet", "--output-format=json"]),
        EvidenceRailKind::ApiStability => {
            let mut argv = string_vec(&["cargo", "semver-checks", "check-release"]);
            if let Some(baseline) = baseline_identity {
                argv.push("--baseline-root".to_owned());
                argv.push(baseline.to_owned());
            }
            argv
        }
        EvidenceRailKind::DynamicEvidence => requested_command.map(split_command).unwrap_or_default(),
        EvidenceRailKind::SarifImport => Vec::new(),
        EvidenceRailKind::ReleaseSupplyChain => string_vec(&["cargo", "auditable", "build"]),
        EvidenceRailKind::MutationTesting => string_vec(&["cargo", "mutants", "--json"]),
        EvidenceRailKind::CoverageBaseline => string_vec(&["cargo", "llvm-cov", "--json", "--summary-only"]),
        EvidenceRailKind::BenchmarkEvidence => string_vec(&["cargo", "metadata", "--no-deps", "--format-version", "1"]),
        EvidenceRailKind::KaniModelCheck => string_vec(&["cargo", "kani"]),
        EvidenceRailKind::HegelProperty | EvidenceRailKind::Z4Solver | EvidenceRailKind::VerifierFixture => {
            requested_command.map(split_command).unwrap_or_default()
        }
    }
}

fn dynamic_run_command_argv(profile: Option<&str>, requested_command: Option<&str>) -> Result<Vec<String>, String> {
    let profile = profile.ok_or_else(|| "dynamic evidence run requires `--profile NAME`".to_owned())?;
    if profile == crate::cargo::fuzz::PROFILE_ID {
        if requested_command.is_some_and(|command| !command.is_empty()) {
            return Err("cargo-fuzz dynamic evidence rejects arbitrary --command authority".to_owned());
        }
        return Ok(crate::cargo::fuzz::command_argv());
    }
    let requested_command =
        requested_command.ok_or_else(|| "dynamic evidence run requires `--command COMMAND`".to_owned())?;
    let allowed_command = allowed_dynamic_command(profile).ok_or_else(|| {
        format!(
            "unknown dynamic evidence profile `{profile}`; expected `{DYNAMIC_PROFILE_MIRI_UNSAFE_BOUNDARY}`, `{DYNAMIC_PROFILE_LOOM_CONCURRENCY}`, or `{}`",
            crate::cargo::fuzz::PROFILE_ID
        )
    })?;
    if requested_command != allowed_command {
        return Err(format!(
            "dynamic evidence command `{requested_command}` is not allowed by profile `{profile}`; expected `{allowed_command}`"
        ));
    }
    Ok(split_command(allowed_command))
}

fn allowed_dynamic_command(profile: &str) -> Option<&'static str> {
    match profile {
        DYNAMIC_PROFILE_MIRI_UNSAFE_BOUNDARY | DYNAMIC_PROFILE_MIRI_SHORT => Some(DYNAMIC_COMMAND_MIRI_TEST),
        DYNAMIC_PROFILE_LOOM_CONCURRENCY | DYNAMIC_PROFILE_LOOM_SHORT => Some(DYNAMIC_COMMAND_LOOM_TEST),
        _ => None,
    }
}

fn split_command(command: &str) -> Vec<String> {
    command.split_whitespace().map(ToOwned::to_owned).collect()
}

fn render_cli_output(receipt: &EvidenceRailReceipt, output_format: OutputFormat) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet evidence rail rail={} checker={} status={} artifact={}",
                receipt.rail_id, receipt.checker_id, receipt.status, RECEIPT_ARTIFACT_NAME
            );
            for diagnostic in &receipt.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
        }
        OutputFormat::Json => println!("{}", render_receipt_json(receipt)?.trim_end()),
        OutputFormat::Sarif => unreachable!("SARIF rejected before rendering"),
    }
    Ok(())
}

fn write_artifacts(
    artifact_dir: &Path,
    receipt_json: &str,
    summary: &str,
    provenance_jsonl: &str,
    additional_artifacts: &BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    let root = ArtifactRoot::open_or_create(artifact_dir)?;
    remove_stale_artifacts(&root, additional_artifacts.keys())?;
    root.write_atomic(&RootRelativePath::new(RECEIPT_ARTIFACT_NAME)?, receipt_json.as_bytes())?;
    root.write(&RootRelativePath::new(SUMMARY_ARTIFACT_NAME)?, summary.as_bytes())?;
    root.write(&RootRelativePath::new(PROVENANCE_ARTIFACT_NAME)?, provenance_jsonl.as_bytes())?;
    for (name, bytes) in additional_artifacts {
        root.write(&RootRelativePath::new(name)?, bytes)?;
    }
    Ok(())
}

fn remove_stale_artifacts<'a>(
    root: &ArtifactRoot,
    additional_names: impl Iterator<Item = &'a String>,
) -> Result<(), String> {
    let mut expected = expected_bundle_files();
    expected.extend(additional_names.cloned());
    for file_name in root.read_entry_names()? {
        if expected.iter().any(|expected| expected == &file_name) {
            continue;
        }
        let relative = RootRelativePath::new(file_name.as_str())?;
        root.remove_file_or_dir_all(&relative)?;
    }
    Ok(())
}

fn exit_code_for_receipt(receipt: &EvidenceRailReceipt) -> i32 {
    if !receipt.diagnostics.is_empty() {
        EXIT_INTEGRATION_FAILURE
    } else if receipt.status == STATUS_PASSED {
        EXIT_SUCCESS
    } else {
        EXIT_RAIL_FAILURE
    }
}

fn source_id_for_receipt(receipt: &EvidenceRailReceipt) -> String {
    EvidenceRailKind::parse(&receipt.rail_id)
        .map(EvidenceRailKind::source_id)
        .unwrap_or("octet.policy.unknown")
        .to_owned()
}

fn receipt_tier(receipt: &EvidenceRailReceipt) -> EvidenceTier {
    receipt_tier_from_rail_id(&receipt.rail_id)
}

fn advisory_database_update(
    rail: EvidenceRailKind,
    locked: bool,
    advisory_updated_by_prior_step: bool,
) -> Option<String> {
    if rail != EvidenceRailKind::DependencyAudit {
        return None;
    }
    if advisory_updated_by_prior_step {
        Some(LOCKED_UPDATED_PRIOR.to_owned())
    } else if locked {
        Some(LOCKED_REUSED.to_owned())
    } else {
        Some(LOCKED_UNAVAILABLE.to_owned())
    }
}

fn validate_dependency_audit_report_shape(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    validate_optional_schema_version(value, DEPENDENCY_AUDIT_SCHEMA_VERSION, "dependency-audit", diagnostics);
    validate_array_records(
        value,
        "diagnostics",
        "dependency-audit diagnostic",
        &["category", "severity", "message"],
        diagnostics,
    );
}

fn validate_coverage_report_shape(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    validate_optional_schema_version(value, COVERAGE_BASELINE_SCHEMA_VERSION, "coverage-baseline", diagnostics);
    let Some(summary) = value.get("summary") else {
        diagnostics.push("coverage-baseline external schema requires `summary` object".to_owned());
        return;
    };
    let Some(summary_object) = summary.as_object() else {
        diagnostics.push("coverage-baseline external schema field `summary` must be an object".to_owned());
        return;
    };
    for field in ["line_percent", "branch_percent", "function_percent"] {
        if let Some(percent) = summary_object.get(field)
            && !percent.is_number()
        {
            diagnostics.push(format!("coverage-baseline external schema field `summary.{field}` must be a number"));
        }
    }
    validate_array_records(value, "regressions", "coverage regression", &["metric"], diagnostics);
    validate_array_records(value, "threshold_failures", "coverage threshold failure", &["metric"], diagnostics);
}

fn validate_optional_schema_version(
    value: &serde_json::Value,
    expected: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(schema_version) = value.get(SCHEMA_VERSION_FIELD) else {
        return;
    };
    match schema_version.as_str() {
        Some(actual) if actual == expected => {}
        Some(actual) => diagnostics
            .push(format!("{label} external schema version `{actual}` is unsupported; expected `{expected}`")),
        None => diagnostics.push(format!("{label} external schema field `{SCHEMA_VERSION_FIELD}` must be a string")),
    }
}

fn validate_array_records(
    value: &serde_json::Value,
    field: &str,
    label: &str,
    required_strings: &[&str],
    diagnostics: &mut Vec<String>,
) {
    let Some(items) = value.get(field) else {
        return;
    };
    let Some(items) = items.as_array() else {
        diagnostics.push(format!("{label} external schema field `{field}` must be an array"));
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let Some(record) = item.as_object() else {
            diagnostics.push(format!("{label} external schema item `{field}[{index}]` must be an object"));
            continue;
        };
        for required in required_strings {
            match record.get(*required).and_then(serde_json::Value::as_str) {
                Some(value) if !value.trim().is_empty() => {}
                _ => diagnostics.push(format!(
                    "{label} external schema item `{field}[{index}]` missing non-empty string `{required}`"
                )),
            }
        }
    }
}

fn parse_json_value(text: &str, diagnostics: &mut Vec<String>) -> Option<serde_json::Value> {
    if text.trim().is_empty() {
        return Some(serde_json::json!({}));
    }
    match serde_json::from_str(text) {
        Ok(value) => Some(value),
        Err(error) => {
            diagnostics.push(format!("malformed evidence JSON: {error}"));
            None
        }
    }
}

fn array_field<'a>(value: &'a serde_json::Value, field: &str) -> Vec<&'a serde_json::Value> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map_or_else(Vec::new, |items| items.iter().collect())
}

fn string_array_field(value: &serde_json::Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map(|items| items.iter().filter_map(serde_json::Value::as_str).map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}

fn string_field<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(serde_json::Value::as_str)
}

fn u64_field(value: &serde_json::Value, field: &str) -> Option<u64> {
    value.get(field).and_then(serde_json::Value::as_u64)
}

fn f64_field(value: &serde_json::Value, field: &str) -> Option<f64> {
    value.get(field).and_then(serde_json::Value::as_f64)
}

fn summary_u64_field(value: &serde_json::Value, field: &str) -> Option<u64> {
    value
        .get("summary")
        .and_then(|summary| u64_field(summary, field))
        .or_else(|| u64_field(value, field))
}

fn push_mutant_findings(
    value: &serde_json::Value,
    field: &str,
    family: &str,
    severity: &str,
    label: &str,
    findings: &mut Vec<EvidenceRailFinding>,
) {
    for mutant in array_field(value, field) {
        findings.push(mutant_finding(mutant, family, severity, label));
    }
}

fn push_outcome_mutant_findings(value: &serde_json::Value, findings: &mut Vec<EvidenceRailFinding>) {
    for outcome in array_field(value, "outcomes") {
        let status = string_field(outcome, "status").or_else(|| string_field(outcome, "outcome"));
        match status {
            Some("survived" | "missed") => {
                findings.push(mutant_finding(outcome, "survived-mutant", "manual_review", "survived mutant"))
            }
            Some("timeout" | "timed-out" | "timed_out") => {
                findings.push(mutant_finding(outcome, "timeout-mutant", "manual_review", "timed-out mutant"))
            }
            _ => {}
        }
    }
}

fn push_aggregate_mutation_finding(
    value: &serde_json::Value,
    count_field: &str,
    family: &str,
    severity: &str,
    label: &str,
    findings: &mut Vec<EvidenceRailFinding>,
) {
    if findings.iter().any(|finding| finding.family == family) {
        return;
    }
    let Some(count) = summary_u64_field(value, count_field) else {
        return;
    };
    if count == 0 {
        return;
    }
    findings.push(EvidenceRailFinding {
        family: family.to_owned(),
        severity: severity.to_owned(),
        message: format!("{label} count: {count}"),
        source_severity: None,
        file: None,
        line: None,
        column: None,
    });
}

fn mutant_finding(mutant: &serde_json::Value, family: &str, severity: &str, label: &str) -> EvidenceRailFinding {
    let name = string_field(mutant, "name")
        .or_else(|| string_field(mutant, "id"))
        .or_else(|| string_field(mutant, "description"))
        .unwrap_or("unnamed mutant");
    EvidenceRailFinding {
        family: family.to_owned(),
        severity: severity.to_owned(),
        message: format!("{label}: {name}"),
        source_severity: None,
        file: string_field(mutant, "file").map(ToOwned::to_owned),
        line: u64_field(mutant, "line"),
        column: u64_field(mutant, "column"),
    }
}

fn validate_coverage_summary(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    let summary = value.get("summary").unwrap_or(value);
    let percent_fields = [
        ("line_percent", "line coverage"),
        ("branch_percent", "branch coverage"),
        ("function_percent", "function coverage"),
    ];
    let mut saw_percentage = false;
    for (field, label) in percent_fields {
        let Some(percent) = f64_field(summary, field) else {
            continue;
        };
        saw_percentage = true;
        if !(COVERAGE_PERCENT_MIN..=COVERAGE_PERCENT_MAX).contains(&percent) {
            diagnostics.push(format!(
                "coverage-baseline rail {label} percentage `{percent}` must be between {COVERAGE_PERCENT_MIN} and {COVERAGE_PERCENT_MAX}"
            ));
        }
    }
    if !saw_percentage {
        diagnostics.push("coverage-baseline rail requires line, branch, or function percentage summary".to_owned());
    }
}

fn push_coverage_findings(
    value: &serde_json::Value,
    field: &str,
    family: &str,
    label: &str,
    findings: &mut Vec<EvidenceRailFinding>,
) {
    for item in array_field(value, field) {
        let metric = string_field(item, "metric").unwrap_or("coverage metric");
        let message = string_field(item, "message").map_or_else(|| format!("{label}: {metric}"), ToOwned::to_owned);
        findings.push(EvidenceRailFinding {
            family: family.to_owned(),
            severity: "error".to_owned(),
            message,
            source_severity: None,
            file: None,
            line: None,
            column: None,
        });
    }
}

fn push_aggregate_coverage_finding(
    value: &serde_json::Value,
    count_field: &str,
    family: &str,
    label: &str,
    findings: &mut Vec<EvidenceRailFinding>,
) {
    if findings.iter().any(|finding| finding.family == family) {
        return;
    }
    let Some(count) = u64_field(value, count_field) else {
        return;
    };
    if count == 0 {
        return;
    }
    findings.push(EvidenceRailFinding {
        family: family.to_owned(),
        severity: "error".to_owned(),
        message: format!("{label} count: {count}"),
        source_severity: None,
        file: None,
        line: None,
        column: None,
    });
}

fn primary_span(message: &serde_json::Value) -> Option<&serde_json::Value> {
    message.get("spans").and_then(serde_json::Value::as_array).and_then(|spans| {
        spans.iter().find(|span| span.get("is_primary").and_then(serde_json::Value::as_bool) == Some(true))
    })
}

fn clippy_gate_severity(source_level: Option<&str>) -> String {
    match source_level {
        Some("error") => "error".to_owned(),
        Some(level) => format!("source-{level}"),
        None => "source-warning".to_owned(),
    }
}

fn counts_by_family(findings: &[EvidenceRailFinding]) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for finding in findings {
        *counts.entry(finding.family.clone()).or_insert(0) += 1;
    }
    counts
}

fn status_from_error_findings(findings: &[EvidenceRailFinding]) -> String {
    if findings.iter().any(|finding| finding.severity == "error") {
        STATUS_FAILED.to_owned()
    } else {
        STATUS_PASSED.to_owned()
    }
}

fn path_identity(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {} for BLAKE3 identity: {error}", path.display()))?;
    Ok(prefixed_identity(&bytes))
}

fn path_identity_if_present(path: &Path) -> Result<Option<String>, String> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(prefixed_identity(&bytes))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("read {} for BLAKE3 identity: {error}", path.display())),
    }
}

fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", crate::evidence::artifact_blake3_hex(bytes))
}

fn string_vec(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use hegel::TestCase;
    use hegel::generators as gs;

    use super::*;

    const WORKSPACE_SCOPE: &str = "fixture-workspace";
    const DENY_CONFIG: &str = "deny.toml";
    const BASELINE_IDENTITY: &str = "b3:baseline";
    const COVERAGE_SOURCE_MANIFEST: &str = "coverage source manifest fixture\n";
    const COVERAGE_CURRENT_COVERED: u64 = 92;
    const COVERAGE_CURRENT_BASIS_POINTS: u64 = 9_200;
    const COVERAGE_ARTIFACT_COUNT: usize = 5;
    const COVERAGE_METRIC_COUNT: u64 = 4;
    const HEGEL_PROPERTY_CASES: u64 = HEGEL_FAST_CASE_BUDGET;
    const PROPERTY_MIN_SALT: u64 = 0;
    const PROPERTY_MAX_SALT: u64 = 1_000;
    const CLIPPY_JSON: &str = r#"{"reason":"compiler-message","message":{"code":{"code":"clippy::unwrap_used"},"level":"warning","message":"used unwrap","spans":[{"is_primary":true,"file_name":"src/lib.rs","line_start":10,"column_start":5}]}}
"#;

    fn clippy_profile_catalog_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../standards/clippy-restrictions/generated/profiles.json")
    }

    fn cargo_fuzz_import_args(
        input: std::path::PathBuf,
        artifact_dir: std::path::PathBuf,
    ) -> crate::cli::EvidenceRailArgs {
        crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::DynamicEvidence,
            input: Some(input),
            artifact_dir,
            workspace_root: None,
            config: Some(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dynamic-evidence-profiles.json"),
            ),
            baseline: None,
            profile: Some(crate::cargo::fuzz::PROFILE_ID.to_owned()),
            command: None,
            parent_artifacts: Vec::new(),
            external_reports: Vec::new(),
            output_format: OutputFormat::Human,
            locked: false,
            update_advisory_database: false,
            run: false,
            command_line: "cargo octet evidence collect --rail dynamic-evidence --profile cargo-fuzz-libfuzzer --input fixture.json".to_owned(),
        }
    }

    fn sarif_import_args(workspace_root: std::path::PathBuf) -> crate::cli::EvidenceRailArgs {
        crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::SarifImport,
            input: Some(std::path::PathBuf::from("report.json")),
            artifact_dir: std::path::PathBuf::from("artifacts/sarif-import-test"),
            workspace_root: Some(workspace_root),
            config: Some(std::path::PathBuf::from("profiles.json")),
            baseline: None,
            profile: Some("codeql-local".to_owned()),
            command: None,
            parent_artifacts: Vec::new(),
            external_reports: Vec::new(),
            output_format: OutputFormat::Human,
            locked: false,
            update_advisory_database: false,
            run: false,
            command_line: "cargo octet evidence collect --rail sarif-import --profile codeql-local --input report.json --config profiles.json".to_owned(),
        }
    }

    fn clippy_run_args(
        config: Option<std::path::PathBuf>,
        profile: Option<&str>,
        command: Option<&str>,
    ) -> crate::cli::EvidenceRailArgs {
        crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::ClippyCoReporting,
            input: None,
            artifact_dir: std::path::PathBuf::from("artifacts/clippy-profile-test"),
            workspace_root: None,
            config,
            baseline: None,
            profile: profile.map(ToOwned::to_owned),
            command: command.map(ToOwned::to_owned),
            parent_artifacts: Vec::new(),
            external_reports: Vec::new(),
            output_format: OutputFormat::Human,
            locked: false,
            update_advisory_database: false,
            run: true,
            command_line: "cargo octet evidence collect --rail clippy-co-reporting --run".to_owned(),
        }
    }

    #[hegel::test(test_cases = HEGEL_PROPERTY_CASES)]
    fn hegel_receipt_normalizes_replay_sets_and_rejects_bad_hashes(tc: TestCase) {
        let salt = draw_hegel_salt(&tc);
        let first = hegel_receipt(&hegel_passed_json(salt, false));
        let second = hegel_receipt(&hegel_passed_json(salt, true));
        let first_details = first.hegel_property.as_ref().expect("first Hegel receipt details");
        let second_details = second.hegel_property.as_ref().expect("second Hegel receipt details");

        assert_eq!(first.status, STATUS_PASSED);
        assert_eq!(second.status, STATUS_PASSED);
        assert_eq!(first_details.linked_ids, second_details.linked_ids);
        assert_eq!(first_details.source_hashes, second_details.source_hashes);
        assert!(first_details.settings.derandomized);

        let invalid = hegel_receipt(&hegel_bad_hash_json(salt));
        assert_eq!(invalid.status, STATUS_FAILED);
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("BLAKE3")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("generated-testing non-claim")));
    }

    #[test]
    fn dependency_audit_clean_fixture_passes_with_locked_boundary() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyAudit,
            mode: MODE_IMPORT,
            import_text: r#"{"diagnostics":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some(DENY_CONFIG),
            baseline_identity: None,
            lockfile_identity: Some("b3:lock".to_owned()),
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });

        assert_eq!(receipt.status, STATUS_PASSED);
        assert_eq!(receipt.checker_id, crate::evidence::CHECKER_CARGO_DENY);
        assert_eq!(receipt.advisory_database_update.as_deref(), Some(LOCKED_REUSED));
        assert!(receipt.command_argv.iter().any(|arg| arg == DENY_CONFIG));
    }

    #[test]
    fn dependency_audit_external_schema_rejects_malformed_report_shape() {
        let receipt = dependency_audit_receipt(
            r#"{"schema_version":"cargo-deny-diagnostics/v0","diagnostics":[{"category":"licenses","severity":7}]}"#,
            Some(DENY_CONFIG),
        );

        assert_eq!(receipt.status, STATUS_FAILED);
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported")));
        assert!(
            receipt
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing non-empty string `severity`"))
        );
        assert!(
            receipt
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing non-empty string `message`"))
        );
    }

    #[test]
    fn dependency_audit_rejects_missing_config_and_preserves_denied_category() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyAudit,
            mode: MODE_IMPORT,
            import_text: r#"{"diagnostics":[{"category":"licenses","severity":"error","message":"license denied"}]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });

        assert_eq!(receipt.status, STATUS_FAILED);
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("deny.toml")));
        assert_eq!(receipt.findings[0].family, "licenses");
        assert_eq!(receipt.finding_counts.get("licenses"), Some(&1));
    }

    #[test]
    fn clippy_import_preserves_source_severity_without_static_lint_claim() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ClippyCoReporting,
            mode: MODE_IMPORT,
            import_text: CLIPPY_JSON,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        let receipt_json = render_receipt_json(&receipt).expect("receipt renders");
        let summary = render_summary(&receipt);
        let provenance = render_provenance_jsonl(&receipt, &receipt_json, &summary).expect("provenance renders");

        assert_eq!(receipt.status, STATUS_PASSED);
        assert_eq!(receipt.findings[0].source_severity.as_deref(), Some("warning"));
        assert_eq!(receipt.findings[0].severity, "source-warning");
        assert!(receipt.non_claims.iter().any(|claim| claim.contains("not Octet source lint replacements")));
        assert!(provenance.contains("\"evidence_tier\":\"static_lint\""));
    }

    #[test]
    fn clippy_import_rejects_malformed_json_and_missing_required_fields() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ClippyCoReporting,
            mode: MODE_IMPORT,
            import_text: "not-json\n{\"reason\":\"compiler-message\",\"message\":{}}\n",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });

        assert_eq!(receipt.status, STATUS_FAILED);
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed Clippy JSON")));
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("message.code.code")));
    }

    #[test]
    fn profiled_clippy_run_builds_checked_command_and_rejects_unchecked_authority() {
        let config = clippy_profile_catalog_path();
        let args = clippy_run_args(Some(config.clone()), Some("production-safety"), None);
        let prepared = prepare_clippy_run(&args).expect("profile preparation").expect("selected profile");
        assert_eq!(prepared.details.profile.profile_id, "production-safety");
        assert!(prepared.command_argv.iter().any(|arg| arg == "clippy::mem_forget"));
        assert!(!prepared.command_argv.iter().any(|arg| arg == "clippy::restriction"));

        let missing_profile = prepare_clippy_run(&clippy_run_args(Some(config.clone()), None, None))
            .expect_err("config without profile should fail");
        let embedded = prepare_clippy_run(&clippy_run_args(None, Some("production-safety"), None))
            .expect("shipped profile should prepare")
            .expect("shipped profile selected");
        let arbitrary = prepare_clippy_run(&clippy_run_args(
            Some(config),
            Some("production-safety"),
            Some("cargo clippy --all-features"),
        ))
        .expect_err("profiled arbitrary command should fail");
        assert!(missing_profile.contains("--profile"));
        assert_eq!(embedded.config_path, EMBEDDED_CLIPPY_PROFILE_LOCATOR);
        assert!(crate::clippy_restrictions::is_blake3_identity(&embedded.config_identity));
        assert_eq!(embedded.command_argv, prepared.command_argv);
        assert!(arbitrary.contains("arbitrary --command authority"));
    }

    #[test]
    fn profiled_clippy_receipt_binds_policy_and_rejects_tampering() {
        let config = clippy_profile_catalog_path();
        let args = clippy_run_args(Some(config.clone()), Some("production-safety"), None);
        let prepared = prepare_clippy_run(&args).expect("profile preparation").expect("selected profile");
        let normalized_command = prepared.command_argv.join(" ");
        let mut receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ClippyCoReporting,
            mode: MODE_RUN,
            import_text: "",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("standards/clippy-restrictions/generated/profiles.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("production-safety"),
            requested_command: Some(&normalized_command),
            parent_artifacts: Vec::new(),
            tool_exit_code: Some(EXIT_SUCCESS),
        });
        receipt.config_identity = Some(path_identity(&config).expect("config identity"));
        receipt.clippy_restrictions = Some(prepared.details);
        assert_eq!(receipt.status, STATUS_PASSED);
        assert!(validate_receipt(&receipt).is_empty());

        let mut command_tamper = receipt.clone();
        command_tamper.command_argv.push("--all-features".to_owned());
        let mut identity_tamper = receipt.clone();
        identity_tamper.config_identity = None;
        let mut profile_tamper = receipt;
        profile_tamper.clippy_restrictions.as_mut().expect("profile details").profile_identity =
            "b3:tampered".to_owned();
        assert!(validate_receipt(&command_tamper).iter().any(|diagnostic| diagnostic.contains("command argv")));
        assert!(validate_receipt(&identity_tamper).iter().any(|diagnostic| diagnostic.contains("config_identity")));
        assert!(validate_receipt(&profile_tamper).iter().any(|diagnostic| diagnostic.contains("selected-profile")));
    }

    #[test]
    fn clippy_nonzero_tool_exit_fails_without_parsing_stderr_noise() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ClippyCoReporting,
            mode: MODE_RUN,
            import_text: "",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: Some(EXIT_RAIL_FAILURE),
        });
        assert_eq!(receipt.status, STATUS_FAILED);
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("nonzero status")));
        assert!(!receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed Clippy JSON")));
    }

    #[test]
    fn ast_grep_receipt_preserves_candidate_findings_and_rejects_role_promotion() {
        let profile =
            crate::ast_grep_rail::parse_profile(include_str!("../../standards/ast-grep/generated/profile.json"))
                .expect("ast-grep profile");
        let envelope = crate::ast_grep_rail::render_scan_envelope(
            profile,
            "0.41.1".to_owned(),
            serde_json::json!([{
                "ruleId": "unsafe-block-review",
                "severity": "warning",
                "message": "review unsafe block",
                "file": "src/lib.rs",
                "range": {"start": {"line": 1, "column": 2}, "end": {"line": 1, "column": 8}},
                "metaVariables": {"multi": {"BODY": [{"text": "call()"}]}}
            }]),
            String::new(),
            0,
        )
        .expect("ast-grep envelope");
        let mut receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::AstGrepStructural,
            mode: MODE_IMPORT,
            import_text: &envelope,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("standards/ast-grep/generated/profile.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        let receipt_json = render_receipt_json(&receipt).expect("receipt renders");
        let summary = render_summary(&receipt);
        let provenance = render_provenance_jsonl(&receipt, &receipt_json, &summary).expect("provenance renders");

        assert_eq!(receipt.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(receipt.checker_id, crate::evidence::CHECKER_AST_GREP);
        assert_eq!(receipt.findings[0].line, Some(2));
        assert!(receipt.ast_grep.as_ref().expect("ast-grep details").raw_findings.is_array());
        assert!(validate_receipt(&receipt).is_empty());
        assert!(provenance.contains("\"evidence_tier\":\"static_lint\""));

        receipt.ast_grep.as_mut().expect("ast-grep details").verification_role = "formal_proof".to_owned();
        assert!(validate_receipt(&receipt).iter().any(|diagnostic| diagnostic.contains("recorded_only")));
    }

    #[test]
    fn wasm_artifact_receipt_binds_exact_bytes_interface_and_input_provenance() {
        let registry_bytes = include_bytes!("../../standards/wasm-artifact/generated/profiles.json");
        let registry =
            crate::wasm_artifact::parse_registry(std::str::from_utf8(registry_bytes).expect("profile registry UTF-8"))
                .expect("profile registry");
        let artifact = wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/positive/module.wat"))
            .expect("module fixture");
        let envelope = crate::wasm_artifact::analyze_bytes(
            &artifact,
            &registry,
            "portable-component-baseline",
            crate::wasm_artifact::registry_identity(registry_bytes),
            &[],
        )
        .expect("Wasm analysis");
        let envelope_text = crate::wasm_artifact::render_envelope(&envelope).expect("render Wasm envelope");
        let mut receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::WasmArtifact,
            mode: MODE_IMPORT,
            import_text: &envelope_text,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("standards/wasm-artifact/generated/profiles.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("portable-component-baseline"),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        let receipt_json = render_receipt_json(&receipt).expect("receipt renders");
        let summary = render_summary(&receipt);
        let provenance = render_provenance_jsonl(&receipt, &receipt_json, &summary).expect("provenance renders");
        let details = receipt.wasm_artifact.as_ref().expect("Wasm details");

        assert_eq!(receipt.status, STATUS_PASSED);
        assert_eq!(details.input_artifact, crate::wasm_artifact::INPUT_ARTIFACT_NAME);
        assert_ne!(details.exact_artifact_identity, details.interface_identity);
        assert!(validate_receipt(&receipt).is_empty());
        assert!(provenance.contains("octet.artifact.wasm-input"));
        assert!(provenance.contains(crate::wasm_artifact::INPUT_ARTIFACT_NAME));

        receipt.wasm_artifact.as_mut().expect("Wasm details").verification_role = "formal_proof".to_owned();
        assert!(validate_receipt(&receipt).iter().any(|diagnostic| diagnostic.contains("recorded_only or boundary")));
    }

    #[test]
    fn charon_semantic_receipt_binds_recorded_artifacts_and_independent_comparison() {
        const EXPECTED_COMPARISON_AGREEMENTS: u64 = 3;
        let snapshot =
            include_str!("../../standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/llbc.llbc");
        let parent = include_str!("../../standards/charon-semantic/generated/comparison-parent.json");
        let envelope = crate::charon_semantic::build_envelope(snapshot.to_owned(), vec![parent.to_owned()])
            .expect("Charon semantic envelope");
        let mut receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::CharonSemantic,
            mode: MODE_IMPORT,
            import_text: &envelope,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("standards/charon-semantic/generated/profile.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some(crate::charon_semantic::shipped_profile_id()),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        receipt.config_identity = Some(crate::charon_semantic::EXPECTED_PROFILE_IDENTITY.to_owned());
        let receipt_json = render_receipt_json(&receipt).expect("receipt renders");
        let summary = render_summary(&receipt);
        let provenance = render_provenance_jsonl(&receipt, &receipt_json, &summary).expect("provenance renders");
        let details = receipt.charon_semantic.as_ref().expect("Charon semantic details");
        let comparison = details.comparison.as_ref().expect("comparison details");

        assert_eq!(receipt.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert!(receipt.command_argv.is_empty());
        assert_eq!(comparison.summary.agree, EXPECTED_COMPARISON_AGREEMENTS);
        assert_eq!(comparison.summary.disagree, 0);
        assert!(!comparison.authority_merged);
        assert!(validate_receipt(&receipt).is_empty());
        assert!(provenance.contains("transcript_assertion"));
        assert!(provenance.contains(crate::charon_semantic::RAW_SNAPSHOT_ARTIFACT));

        let details = receipt.charon_semantic.as_mut().expect("Charon semantic details");
        details.artifacts[0].authority = "formal-proof".to_owned();
        assert!(validate_receipt(&receipt).iter().any(|diagnostic| diagnostic.contains("proof-authority")));
    }

    #[test]
    fn dependency_trust_records_coverage_exemptions_gaps_and_import_identity() {
        let covered = dependency_trust_receipt(
            r#"{"imports":[{"name":"google","source":"https://example.invalid/audits.toml"}],"exemptions":[],"unaudited":[]}"#,
        );
        assert_eq!(covered.status, STATUS_PASSED);

        let exempted = dependency_trust_receipt(
            r#"{"imports":[{"name":"local","source":"audits.toml"}],"exemptions":[{"crate":"serde","reason":"bootstrap"}],"unaudited":[]}"#,
        );
        assert_eq!(exempted.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(exempted.finding_counts.get("exemption"), Some(&1));

        let gap = dependency_trust_receipt(
            r#"{"imports":[{"name":"missing-source"}],"exemptions":[],"unaudited":[{"crate":"bytes"}]}"#,
        );
        assert_eq!(gap.status, STATUS_FAILED);
        assert!(gap.diagnostics.iter().any(|diagnostic| diagnostic.contains("name and source identity")));
        assert_eq!(gap.finding_counts.get("unaudited-gap"), Some(&1));
    }

    #[test]
    fn dependency_audit_and_trust_remain_separate_entries() {
        let audit = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyAudit,
            mode: MODE_IMPORT,
            import_text: r#"{"diagnostics":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some(DENY_CONFIG),
            baseline_identity: None,
            lockfile_identity: None,
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        let trust = dependency_trust_receipt(r#"{"imports":[],"exemptions":[],"unaudited":[]}"#);

        assert_ne!(audit.rail_id, trust.rail_id);
        assert_ne!(audit.checker_id, trust.checker_id);
        assert!(trust.non_claims.iter().any(|claim| claim.contains("separate from dependency audit")));
    }

    #[test]
    fn api_stability_requires_baseline_and_reports_removed_public_item() {
        let compatible = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ApiStability,
            mode: MODE_IMPORT,
            import_text: r#"{"breakages":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: Some(BASELINE_IDENTITY.to_owned()),
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        assert_eq!(compatible.status, STATUS_PASSED);

        let removed = evaluate(EvaluationInput {
            baseline_identity: None,
            import_text: r#"{"breakages":[{"kind":"removed-public-item","item":"pub fn old"}]}"#,
            ..api_stability_input()
        });
        assert_eq!(removed.status, STATUS_FAILED);
        assert!(removed.diagnostics.iter().any(|diagnostic| diagnostic.contains("accepted baseline")));
        assert_eq!(removed.finding_counts.get("removed-public-item"), Some(&1));
    }

    #[test]
    fn dynamic_evidence_accepts_configured_profile_and_rejects_profile_errors() {
        let valid = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DynamicEvidence,
            mode: MODE_IMPORT,
            import_text: r#"{"known_profiles":["miri"],"profile":"miri","allowed_commands":["cargo miri test"],"command":"cargo miri test","tool_available":true,"expected_signal":"passed","acceptance_signal":"passed","finding_count":0}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("miri"),
            requested_command: Some("cargo miri test"),
            parent_artifacts: Vec::new(),
            tool_exit_code: Some(EXIT_SUCCESS),
        });
        assert_eq!(valid.status, STATUS_PASSED);
        assert!(valid.non_claims.iter().any(|claim| claim.contains("not required by default")));

        let invalid = evaluate(EvaluationInput {
            import_text: r#"{"known_profiles":["loom"],"profile":"miri","allowed_commands":["cargo loom test"],"command":"cargo miri test","tool_available":false,"expected_signal":"passed","acceptance_signal":"failed","finding_count":1}"#,
            ..dynamic_input()
        });
        assert_eq!(invalid.status, STATUS_FAILED);
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("unknown dynamic evidence profile")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("not allowed")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("unavailable")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("acceptance signal mismatch")));
    }

    #[test]
    fn cargo_fuzz_profile_preserves_typed_receipt_and_command_isolation() {
        let corpus = vec![crate::cargo::fuzz::ArtifactIdentity {
            path: "corpus/seed".to_owned(),
            length: 4,
            blake3: "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        }];
        let root = crate::cargo::fuzz::corpus_root(&corpus);
        let details = crate::cargo::fuzz::ReceiptDetails {
            schema_version: crate::cargo::fuzz::RECEIPT_SCHEMA_VERSION.to_owned(),
            profile_id: crate::cargo::fuzz::PROFILE_ID.to_owned(),
            profile_identity: crate::cargo::fuzz::EXPECTED_PROFILE_IDENTITY.to_owned(),
            target: crate::cargo::fuzz::TARGET.to_owned(),
            engine: crate::cargo::fuzz::ENGINE.to_owned(),
            toolchain: crate::cargo::fuzz::TOOLCHAIN.to_owned(),
            compiler_identity: "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
            sanitizer: crate::cargo::fuzz::SANITIZER.to_owned(),
            command_argv: crate::cargo::fuzz::command_argv(),
            budget: crate::cargo::fuzz::Budget {
                max_total_time_seconds: crate::cargo::fuzz::MAX_TOTAL_TIME_SECONDS,
                max_runs: crate::cargo::fuzz::MAX_RUNS,
                seed: crate::cargo::fuzz::FIXED_SEED,
            },
            initial_corpus: corpus.clone(),
            initial_corpus_root: root.clone(),
            final_corpus: corpus,
            final_corpus_root: root,
            dictionary: None,
            outcome: "clean".to_owned(),
            crash_input: None,
            minimized_reproducer: None,
            replay_argv: Vec::new(),
            bounded_log_identity: "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_owned(),
            evidence_tier: crate::cargo::fuzz::EVIDENCE_TIER.to_owned(),
            claimed_properties: Vec::new(),
            non_claims: vec![crate::cargo::fuzz::REQUIRED_NON_CLAIM.to_owned()],
        };
        let text = serde_json::to_string(&details).expect("serialize cargo-fuzz fixture");
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DynamicEvidence,
            mode: MODE_IMPORT,
            import_text: &text,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("dynamic-evidence-profiles.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some(crate::cargo::fuzz::PROFILE_ID),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        assert_eq!(receipt.status, STATUS_PASSED, "{:?}", receipt.diagnostics);
        assert_eq!(receipt.command_argv, crate::cargo::fuzz::command_argv());
        assert_eq!(receipt.cargo_fuzz, Some(details));
        assert!(validate_receipt(&receipt).is_empty());
        assert!(dynamic_run_command_argv(Some(crate::cargo::fuzz::PROFILE_ID), None).is_ok());
        assert!(dynamic_run_command_argv(Some(crate::cargo::fuzz::PROFILE_ID), Some("cargo fuzz run other")).is_err());
        assert_eq!(
            dynamic_run_command_argv(Some(DYNAMIC_PROFILE_MIRI_SHORT), Some(DYNAMIC_COMMAND_MIRI_TEST))
                .expect("Miri command"),
            split_command(DYNAMIC_COMMAND_MIRI_TEST)
        );
        assert_eq!(
            dynamic_run_command_argv(Some(DYNAMIC_PROFILE_LOOM_SHORT), Some(DYNAMIC_COMMAND_LOOM_TEST))
                .expect("Loom command"),
            split_command(DYNAMIC_COMMAND_LOOM_TEST)
        );
    }

    #[test]
    fn cargo_fuzz_receipt_reads_are_bounded() {
        let directory = tempfile::tempdir().expect("create receipt fixture directory");
        let valid_path = directory.path().join("valid.json");
        let valid = include_str!("../tests/fixtures/evidence-rails/cargo-fuzz-clean.json");
        fs::write(&valid_path, valid).expect("write valid cargo-fuzz receipt");
        assert_eq!(read_cargo_fuzz_receipt(&valid_path).expect("read valid receipt"), valid);

        let oversized_path = directory.path().join("oversized.json");
        let oversized_length =
            usize::try_from(crate::cargo::fuzz::MAX_RECEIPT_BYTES + 1).expect("cargo-fuzz receipt limit fits usize");
        fs::write(&oversized_path, vec![b'x'; oversized_length]).expect("write oversized receipt");
        let error = read_cargo_fuzz_receipt(&oversized_path).expect_err("oversized receipt must fail");
        assert!(error.contains("exceeds"));
    }

    #[test]
    fn cargo_fuzz_import_publishes_without_dynamic_tool_execution() {
        let directory = tempfile::tempdir().expect("create cargo-fuzz import directory");
        let input = directory.path().join("receipt.json");
        let artifacts = directory.path().join("artifacts");
        fs::write(&input, include_str!("../tests/fixtures/evidence-rails/cargo-fuzz-clean.json"))
            .expect("write cargo-fuzz import receipt");
        let args = cargo_fuzz_import_args(input, artifacts.clone());

        assert_eq!(run(&args).expect("import cargo-fuzz receipt"), EXIT_SUCCESS);
        let receipt = fs::read_to_string(artifacts.join(RECEIPT_ARTIFACT_NAME)).expect("read published receipt");
        let parsed: EvidenceRailReceipt = serde_json::from_str(&receipt).expect("parse published receipt");
        assert_eq!(parsed.status, STATUS_PASSED);
        assert!(parsed.cargo_fuzz.is_some());
        assert!(artifacts.join(SUMMARY_ARTIFACT_NAME).is_file());
        assert!(artifacts.join(PROVENANCE_ARTIFACT_NAME).is_file());
    }

    #[test]
    fn cargo_fuzz_crash_and_profile_tampering_fail_closed() {
        let clean_text = serde_json::to_string(&crate::cargo::fuzz::ReceiptDetails {
            schema_version: crate::cargo::fuzz::RECEIPT_SCHEMA_VERSION.to_owned(),
            profile_id: crate::cargo::fuzz::PROFILE_ID.to_owned(),
            profile_identity: "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_owned(),
            target: crate::cargo::fuzz::TARGET.to_owned(),
            engine: crate::cargo::fuzz::ENGINE.to_owned(),
            toolchain: crate::cargo::fuzz::TOOLCHAIN.to_owned(),
            compiler_identity: "b3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_owned(),
            sanitizer: crate::cargo::fuzz::SANITIZER.to_owned(),
            command_argv: crate::cargo::fuzz::command_argv(),
            budget: crate::cargo::fuzz::Budget {
                max_total_time_seconds: 1,
                max_runs: 1,
                seed: 1,
            },
            initial_corpus: Vec::new(),
            initial_corpus_root: crate::cargo::fuzz::corpus_root(&[]),
            final_corpus: Vec::new(),
            final_corpus_root: crate::cargo::fuzz::corpus_root(&[]),
            dictionary: None,
            outcome: "crash".to_owned(),
            crash_input: None,
            minimized_reproducer: None,
            replay_argv: Vec::new(),
            bounded_log_identity: "b3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned(),
            evidence_tier: "formal_proof".to_owned(),
            claimed_properties: vec!["exhaustive coverage".to_owned()],
            non_claims: Vec::new(),
        })
        .expect("serialize invalid cargo-fuzz fixture");
        let receipt = evaluate(EvaluationInput {
            import_text: &clean_text,
            profile: Some(crate::cargo::fuzz::PROFILE_ID),
            ..dynamic_input()
        });
        assert_eq!(receipt.status, STATUS_FAILED);
        assert_eq!(receipt.finding_counts.get("cargo-fuzz-crash"), Some(&1));
        assert!(receipt.diagnostics.iter().any(|value| value.contains("profile identity")));
        assert!(receipt.diagnostics.iter().any(|value| value.contains("formal") || value.contains("transcript")));
    }

    #[test]
    fn sarif_import_preserves_source_identity_and_local_artifact_links() {
        let report = include_str!("../tests/fixtures/evidence-rails/sarif-codeql.json");
        let mut receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::SarifImport,
            mode: MODE_IMPORT,
            import_text: report,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("standards/sarif/profiles.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("codeql-local"),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        receipt.config_identity = Some(crate::external::sarif::PROFILE_IDENTITY.to_owned());
        assert_eq!(receipt.status, STATUS_PASSED, "{:?}", receipt.diagnostics);
        assert_eq!(receipt.checker_id, crate::evidence::CHECKER_SARIF_IMPORT);
        assert_eq!(receipt.findings[0].source_severity.as_deref(), Some("warning"));
        assert_eq!(receipt.findings[0].severity, "warning");
        assert!(receipt.sarif_import.is_some());
        assert!(validate_receipt(&receipt).is_empty());

        let receipt_json = render_receipt_json(&receipt).expect("render SARIF receipt");
        let summary = render_summary(&receipt);
        let provenance = render_provenance_jsonl(&receipt, &receipt_json, &summary).expect("render SARIF provenance");
        assert!(provenance.contains(crate::external::sarif::RAW_ARTIFACT));
        assert!(provenance.contains(crate::external::sarif::PROJECTION_ARTIFACT));
        assert!(provenance.contains("octet.external_lint.sarif-projection"));
        assert!(!provenance.contains("formal_proof"));
    }

    #[test]
    fn sarif_import_rejects_error_findings_unsafe_paths_and_role_promotion() {
        let error_report = include_str!("../tests/fixtures/evidence-rails/sarif-semgrep.json");
        let error_receipt = evaluate(EvaluationInput {
            import_text: error_report,
            profile: Some("semgrep-local"),
            ..sarif_input()
        });
        assert_eq!(error_receipt.status, STATUS_FAILED);
        assert_eq!(error_receipt.findings[0].source_severity.as_deref(), Some("error"));

        let unsafe_report = include_str!("../tests/fixtures/evidence-rails/sarif-unsafe.json");
        let unsafe_receipt = evaluate(EvaluationInput {
            import_text: unsafe_report,
            ..sarif_input()
        });
        assert_eq!(unsafe_receipt.status, STATUS_FAILED);
        assert!(unsafe_receipt.diagnostics.iter().any(|value| value.contains("unsafe")));

        let mut promoted = error_receipt;
        promoted.non_claims = vec!["formal_proof".to_owned()];
        assert!(!validate_receipt(&promoted).is_empty());
    }

    #[test]
    fn sarif_import_shell_is_local_bounded_and_authority_free() {
        let directory = tempfile::tempdir().expect("create SARIF import fixture directory");
        let report = include_str!("../tests/fixtures/evidence-rails/sarif-codeql.json");
        std::fs::write(directory.path().join("report.json"), report).expect("write SARIF report fixture");
        std::fs::write(directory.path().join("profiles.json"), include_bytes!("../../standards/sarif/profiles.json"))
            .expect("write SARIF profile fixture");

        let args = sarif_import_args(directory.path().to_path_buf());
        assert_eq!(analyze_sarif_input(&args).expect("read local SARIF report"), report);

        let run_error = analyze_sarif_input(&crate::cli::EvidenceRailArgs { run: true, ..args })
            .expect_err("SARIF run authority is rejected");
        assert!(run_error.contains("rejects `--run`"));

        let oversized_bytes =
            usize::try_from(crate::external::sarif::MAX_REPORT_BYTES).expect("SARIF byte limit fits usize") + 1;
        std::fs::write(directory.path().join("report.json"), vec![b' '; oversized_bytes])
            .expect("write oversized SARIF report fixture");
        let size_error = analyze_sarif_input(&sarif_import_args(directory.path().to_path_buf()))
            .expect_err("oversized SARIF report is rejected");
        assert!(size_error.contains("exceeds"));
    }

    #[test]
    fn release_supply_chain_records_auditable_metadata_sbom_vulnerabilities_and_parents() {
        let parent = "artifacts/octet/status.json".to_owned();
        let clean = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ReleaseSupplyChain,
            mode: MODE_IMPORT,
            import_text: r#"{"auditable_binary":{"dependency_metadata":true},"sbom":{"packages":2},"vulnerabilities":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: vec![parent.clone()],
            tool_exit_code: None,
        });
        assert_eq!(clean.status, STATUS_PASSED);
        assert_eq!(clean.parent_artifacts, vec![parent]);

        let vulnerable = evaluate(EvaluationInput {
            import_text: r#"{"auditable_binary":{"dependency_metadata":false},"vulnerabilities":[{"id":"RUSTSEC-0000-0000","severity":"critical"}]}"#,
            ..release_input()
        });
        assert_eq!(vulnerable.status, STATUS_FAILED);
        assert!(
            vulnerable
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("auditable Rust dependency metadata"))
        );
        assert_eq!(vulnerable.finding_counts.get("vulnerability"), Some(&1));
        assert!(vulnerable.non_claims.iter().any(|claim| claim.contains("not hosted-service attestations")));
    }

    #[test]
    fn mutation_testing_records_clean_and_survived_mutants() {
        const SURVIVED_MUTANT_LINE: u64 = 42;

        let clean = mutation_receipt(r#"{"summary":{"total":2,"caught":2,"survived":0,"timeout":0},"outcomes":[]}"#);
        assert_eq!(clean.status, STATUS_PASSED);
        assert_eq!(clean.checker_id, crate::evidence::CHECKER_CARGO_MUTANTS);
        assert!(clean.non_claims.iter().any(|claim| claim.contains("does not prove the tests are complete")));

        let survived = mutation_receipt(
            r#"{"summary":{"total":2,"caught":1,"survived":1,"timeout":0},"survived_mutants":[{"name":"replace + with -","file":"src/lib.rs","line":42}]}"#,
        );
        assert_eq!(survived.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(survived.finding_counts.get("survived-mutant"), Some(&1));
        assert_eq!(survived.findings[0].line, Some(SURVIVED_MUTANT_LINE));
    }

    #[test]
    fn profiled_mutation_receipts_preserve_native_outcomes_and_artifacts() {
        // r[verify octet.mutation_testing.verification]
        let mut caught = profiled_mutation_receipt("CaughtMutant");
        assert_eq!(caught.status, STATUS_PASSED, "{:?}", caught.diagnostics);
        assert!(caught.findings.is_empty());
        caught.config_identity = Some(crate::mutation::PROFILE_IDENTITY.to_owned());
        assert!(validate_receipt(&caught).is_empty());
        let details = caught.cargo_mutants.as_ref().expect("typed mutation details");
        let request = profiled_mutation_request("CaughtMutant");
        let artifacts =
            crate::mutation::request::artifact_payloads(details, &request).expect("mutation artifact payloads");
        assert!(artifacts.contains_key(crate::mutation::PROJECTION_ARTIFACT));
        let receipt_json = render_receipt_json(&caught).expect("render mutation receipt");
        let provenance = render_provenance_jsonl(&caught, &receipt_json, &render_summary(&caught))
            .expect("render mutation provenance");
        assert!(provenance.contains("octet.mutation_testing.native-outcomes"));
        assert!(!provenance.contains("formal_proof"));

        let missed = profiled_mutation_receipt("MissedMutant");
        assert_eq!(missed.status, STATUS_FAILED);
        assert_eq!(missed.finding_counts.get("mutation-missed"), Some(&1));
    }

    #[test]
    fn coverage_baseline_external_schema_rejects_bad_summary_shape() {
        let bad_version = coverage_receipt(
            r#"{"schema_version":"cargo-llvm-cov-summary/v0","summary":{"line_percent":"many"},"regressions":[{}],"threshold_failures":[]}"#,
            Some(BASELINE_IDENTITY.to_owned()),
        );

        assert_eq!(bad_version.status, STATUS_FAILED);
        assert!(bad_version.diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported")));
        assert!(bad_version.diagnostics.iter().any(|diagnostic| diagnostic.contains("summary.line_percent")));
        assert!(
            bad_version
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing non-empty string `metric`"))
        );
    }

    #[test]
    fn coverage_baseline_requires_baseline_and_reports_regression() {
        let clean = coverage_receipt(
            r#"{"summary":{"line_percent":92.5,"branch_percent":81.0,"function_percent":100.0},"regressions":[],"threshold_failures":[]}"#,
            Some(BASELINE_IDENTITY.to_owned()),
        );
        assert_eq!(clean.status, STATUS_PASSED);
        assert_eq!(clean.checker_id, crate::evidence::CHECKER_CARGO_LLVM_COV);
        assert_eq!(clean.baseline_identity.as_deref(), Some(BASELINE_IDENTITY));
        assert!(clean.non_claims.iter().any(|claim| claim.contains("does not prove behavioral correctness")));

        let regressed = coverage_receipt(
            r#"{"summary":{"line_percent":88.0},"regressions":[{"metric":"line_percent","message":"line coverage fell below accepted baseline"}]}"#,
            None,
        );
        assert_eq!(regressed.status, STATUS_FAILED);
        assert!(regressed.diagnostics.iter().any(|diagnostic| diagnostic.contains("accepted baseline")));
        assert_eq!(regressed.finding_counts.get("coverage-regression"), Some(&1));
    }

    #[test]
    fn profiled_coverage_binds_native_counts_artifacts_and_policy() {
        // r[verify octet.coverage_baseline.receipt]
        let current = include_str!("../tests/fixtures/evidence-rails/coverage-native-current.json");
        let mut receipt = profiled_coverage_receipt(current);
        assert_eq!(receipt.status, STATUS_PASSED, "{:?}", receipt.diagnostics);
        assert!(receipt.findings.is_empty());
        let details = receipt.coverage_baseline.as_ref().expect("profiled coverage details");
        assert_eq!(details.current.counts.lines.covered, COVERAGE_CURRENT_COVERED);
        assert_eq!(details.current.counts.lines.percentage_basis_points, COVERAGE_CURRENT_BASIS_POINTS);
        assert_eq!(receipt.command_argv, details.command_argv);
        assert!(validate_receipt(&receipt).is_empty());

        let receipt_json = render_receipt_json(&receipt).expect("render coverage receipt");
        let summary = render_summary(&receipt);
        let provenance =
            render_provenance_jsonl(&receipt, &receipt_json, &summary).expect("render coverage provenance");
        assert!(provenance.contains(crate::coverage::CURRENT_RAW_ARTIFACT));
        assert!(provenance.contains(crate::coverage::BASELINE_ENVELOPE_ARTIFACT));
        assert!(provenance.contains(crate::coverage::PROJECTION_ARTIFACT));
        assert!(!provenance.contains("formal_proof"));

        let request = profiled_coverage_request(current);
        let artifacts =
            crate::coverage::request::artifact_payloads(details, &request).expect("coverage artifact payloads");
        assert_eq!(artifacts.len(), COVERAGE_ARTIFACT_COUNT);

        receipt.coverage_baseline.as_mut().expect("coverage details").comparison.lines.current.covered = 1;
        assert!(validate_receipt(&receipt).iter().any(|value| value.contains("comparison is stale")));
    }

    #[test]
    fn profiled_coverage_rejects_regression_percentages_and_command_authority() {
        // r[verify octet.coverage_baseline.verification]
        let regression = include_str!("../tests/fixtures/evidence-rails/coverage-native-regression.json");
        let receipt = profiled_coverage_receipt(regression);
        assert_eq!(receipt.status, STATUS_FAILED);
        assert_eq!(receipt.finding_counts.get("coverage-regression"), Some(&COVERAGE_METRIC_COUNT));

        let percentages = include_str!("../tests/fixtures/evidence-rails/coverage-native-percentage-only.json");
        let receipt = profiled_coverage_receipt(percentages);
        assert_eq!(receipt.status, STATUS_FAILED);
        assert!(receipt.coverage_baseline.is_none());
        assert!(receipt.diagnostics.iter().any(|value| value.contains("covered count")));

        let mut stale_source: serde_json::Value =
            serde_json::from_str(&profiled_coverage_request(regression)).expect("parse coverage request");
        stale_source["source_manifest_text"] = serde_json::Value::String("stale".to_owned());
        let stale_source = serde_json::to_string(&stale_source).expect("serialize stale source request");
        assert!(
            crate::coverage::request::evaluate(&stale_source, crate::coverage::PROFILE_ID)
                .diagnostics
                .iter()
                .any(|value| value.contains("source manifest identity is stale"))
        );

        let args = crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::CoverageBaseline,
            input: None,
            artifact_dir: "artifacts/coverage".into(),
            workspace_root: None,
            config: None,
            baseline: None,
            profile: Some(crate::coverage::PROFILE_ID.to_owned()),
            command: Some("arbitrary scanner".to_owned()),
            parent_artifacts: Vec::new(),
            external_reports: vec!["source-manifest.json".into()],
            output_format: OutputFormat::Human,
            locked: true,
            update_advisory_database: false,
            run: true,
            command_line: "cargo octet evidence collect --rail coverage-baseline --run".to_owned(),
        };
        assert!(run_mode_command_argv(&args).expect_err("arbitrary command rejected").contains("authority"));
    }

    #[test]
    fn profiled_coverage_import_uses_capability_rooted_checked_inputs() {
        // r[verify octet.coverage_baseline.boundary]
        let directory = tempfile::tempdir().expect("create profiled coverage import fixture");
        let baseline = include_str!("../tests/fixtures/evidence-rails/coverage-native-baseline.json");
        let source_manifest = b"coverage source manifest fixture\n";
        let source_identity = format!("source:{}", crate::coverage::prefixed_identity(source_manifest));
        let baseline_envelope =
            crate::coverage::baseline_envelope_text(baseline, crate::coverage::PROFILE_ID, &source_identity)
                .expect("coverage baseline envelope");
        std::fs::write(
            directory.path().join("current.json"),
            include_bytes!("../tests/fixtures/evidence-rails/coverage-native-current.json"),
        )
        .expect("write current coverage fixture");
        std::fs::write(directory.path().join("baseline.json"), baseline_envelope)
            .expect("write coverage baseline fixture");
        std::fs::write(directory.path().join("source-manifest.json"), source_manifest)
            .expect("write coverage source manifest fixture");
        std::fs::write(
            directory.path().join("profiles.json"),
            include_bytes!("../../standards/coverage/profiles.json"),
        )
        .expect("write coverage profile fixture");
        let artifact_dir = directory.path().join("artifacts");
        let args = crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::CoverageBaseline,
            input: Some("current.json".into()),
            artifact_dir: artifact_dir.clone(),
            workspace_root: Some(directory.path().to_path_buf()),
            config: Some("profiles.json".into()),
            baseline: Some("baseline.json".into()),
            profile: Some(crate::coverage::PROFILE_ID.to_owned()),
            command: None,
            parent_artifacts: Vec::new(),
            external_reports: vec!["source-manifest.json".into()],
            output_format: OutputFormat::Human,
            locked: true,
            update_advisory_database: false,
            run: false,
            command_line: "cargo octet evidence collect --rail coverage-baseline --input current.json".to_owned(),
        };
        let (mode, request, exit_code) = analyze_profiled_coverage_input(&args).expect("read profiled coverage inputs");
        assert_eq!(mode, MODE_IMPORT);
        assert_eq!(exit_code, None);
        assert!(crate::coverage::request::evaluate(&request, crate::coverage::PROFILE_ID).diagnostics.is_empty());
        assert_eq!(run(&args).expect("publish profiled coverage import"), EXIT_SUCCESS);
        for file in expected_bundle_files_for_receipt(None).into_iter().chain([
            crate::coverage::SOURCE_MANIFEST_ARTIFACT.to_owned(),
            crate::coverage::CURRENT_RAW_ARTIFACT.to_owned(),
            crate::coverage::CURRENT_ENVELOPE_ARTIFACT.to_owned(),
            crate::coverage::BASELINE_ENVELOPE_ARTIFACT.to_owned(),
            crate::coverage::PROJECTION_ARTIFACT.to_owned(),
        ]) {
            assert!(artifact_dir.join(file).is_file());
        }
        let valence_path = directory.path().join("valence.json");
        assert_eq!(
            crate::artifact::run(&crate::cli::ArtifactArgs {
                artifact_dir: artifact_dir.clone(),
                output_format: OutputFormat::Human,
                emit_valence_provenance: Some(valence_path.clone()),
            })
            .expect("verify profiled coverage artifact bundle"),
            EXIT_SUCCESS
        );
        let valence = std::fs::read_to_string(valence_path).expect("read coverage Valence export");
        assert!(valence.contains("coverage_baseline"));
        assert!(!valence.contains("formal_proof"));

        std::fs::write(artifact_dir.join(crate::coverage::CURRENT_RAW_ARTIFACT), b"tampered")
            .expect("tamper current coverage artifact");
        assert_eq!(
            crate::artifact::run(&crate::cli::ArtifactArgs {
                artifact_dir: artifact_dir.clone(),
                output_format: OutputFormat::Human,
                emit_valence_provenance: None,
            })
            .expect("reject tampered coverage artifact bundle"),
            EXIT_RAIL_FAILURE
        );

        std::fs::write(directory.path().join("profiles.json"), b"{}").expect("write stale coverage profile");
        assert!(
            analyze_profiled_coverage_input(&args)
                .expect_err("stale profile rejected")
                .contains("checked Nickel")
        );
    }

    #[test]
    fn kani_model_check_records_contracts_stubs_and_failure_modes() {
        let valid = kani_receipt(include_str!("../tests/fixtures/evidence-rails/kani-model-check-contract.json"));
        let valid_json = render_receipt_json(&valid).expect("receipt renders");
        let valid_summary = render_summary(&valid);
        let valid_provenance =
            render_provenance_jsonl(&valid, &valid_json, &valid_summary).expect("provenance renders");

        assert_eq!(valid.status, STATUS_PASSED);
        assert_eq!(valid.checker_id, crate::evidence::CHECKER_CARGO_KANI);
        assert!(valid.checked_rule_families.iter().any(|family| family == "contract-status"));
        assert_eq!(valid.property_classes, vec![KANI_PROPERTY_CLASS_ENCODE_DECODE.to_owned()]);
        assert!(valid.non_claims.iter().any(|claim| claim.contains("whole-program correctness")));
        assert!(valid_provenance.contains("\"source_id\":\"octet.kani_model_check.cargo-kani\""));
        assert!(valid_provenance.contains("\"evidence_tier\":\"transcript_assertion\""));

        let invalid = kani_receipt(include_str!("../tests/fixtures/evidence-rails/kani-model-check-invalid.json"));
        assert_eq!(invalid.status, STATUS_FAILED);
        assert_eq!(invalid.finding_counts.get("kani-check-failure"), Some(&1));
        assert_eq!(invalid.finding_counts.get("unwind-assertion-failure"), Some(&1));
        assert_eq!(invalid.finding_counts.get("stub-boundary"), Some(&1));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("source identity")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("stale")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported Kani property class")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("unbounded claim")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("cannot be promoted")));
    }

    #[test]
    fn hegel_property_receipts_preserve_bounded_generated_testing_boundary() {
        let valid = hegel_receipt(include_str!("../tests/fixtures/evidence-rails/hegel-property-passed.json"));
        let valid_json = render_receipt_json(&valid).expect("receipt renders");
        let valid_summary = render_summary(&valid);
        let valid_provenance =
            render_provenance_jsonl(&valid, &valid_json, &valid_summary).expect("provenance renders");
        let details = valid.hegel_property.as_ref().expect("hegel details are present");

        assert_eq!(valid.status, STATUS_PASSED);
        assert!(validate_receipt(&valid).is_empty());
        assert_eq!(valid.checker_id, crate::evidence::CHECKER_HEGEL_RS);
        assert!(valid.checked_rule_families.iter().any(|family| family == "generator-identity"));
        assert_eq!(details.property_id, "valence.evidence-ir.canonical-identity-order");
        assert!(details.settings.case_budget >= HEGEL_FAST_CASE_BUDGET);
        assert!(details.counterexample.is_none());
        assert!(valid.non_claims.iter().any(|claim| claim == HEGEL_GENERATED_TESTING_NON_CLAIM));
        assert!(valid_provenance.contains("\"source_id\":\"octet.dynamic_evidence.hegel-property\""));
        assert!(valid_provenance.contains("\"evidence_tier\":\"transcript_assertion\""));

        let invalid = hegel_receipt(include_str!("../tests/fixtures/evidence-rails/hegel-property-invalid.json"));
        assert_eq!(invalid.status, STATUS_FAILED);
        assert_eq!(invalid.finding_counts.get("hegel-counterexample"), Some(&1));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("case_budget")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("derandomized")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("BLAKE3")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("generated-testing non-claim")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("overclaim")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("formal_proof")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("counterexample")));

        let mut drifted = valid.clone();
        drifted.rail_id = EvidenceRailKind::DependencyAudit.as_str().to_owned();
        assert!(validate_receipt(&drifted).iter().any(|diagnostic| diagnostic.contains("only valid")));
    }

    #[test]
    fn verifier_fixture_intake_preserves_regression_only_boundaries() {
        let positive =
            verifier_fixture_receipt(include_str!("../tests/fixtures/evidence-rails/verifier-fixture-positive.json"));
        let positive_json = render_receipt_json(&positive).expect("receipt renders");
        let positive_summary = render_summary(&positive);
        let positive_provenance =
            render_provenance_jsonl(&positive, &positive_json, &positive_summary).expect("provenance renders");
        let details = positive.verifier_fixtures.as_ref().expect("fixture details are present");

        assert_eq!(positive.status, STATUS_PASSED);
        assert!(validate_receipt(&positive).is_empty());
        assert_eq!(positive.checker_id, crate::evidence::CHECKER_CARGO_OCTET);
        assert!(positive.checked_rule_families.iter().any(|family| family == "fixture-identity"));
        assert_eq!(details.fixtures.len(), 5);
        assert!(details.fixtures.iter().any(|fixture| fixture.fixture_id == "bounds-harness-positive"));
        assert!(details.fixtures.iter().any(|fixture| fixture.syntax_family == "proof-assertion"));
        assert!(positive.non_claims.iter().any(|claim| claim == VERIFIER_FIXTURE_NON_CLAIM));
        assert!(positive_provenance.contains("\"source_id\":\"octet.runner.verifier-fixture-intake\""));
        assert!(positive_provenance.contains("\"evidence_tier\":\"transcript_assertion\""));

        let negative =
            verifier_fixture_receipt(include_str!("../tests/fixtures/evidence-rails/verifier-fixture-negative.json"));
        assert_eq!(negative.status, STATUS_FAILED);
        assert_eq!(negative.finding_counts.get("verifier-expected-failure"), Some(&3));
        assert!(negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported")));
        assert!(negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("non-claim")));
        assert!(negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("expected_verdict")));
        assert!(negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("overclaim")));
        assert!(negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("claims.verifier_parity")));

        let mut drifted = positive.clone();
        drifted.rail_id = EvidenceRailKind::DependencyAudit.as_str().to_owned();
        assert!(validate_receipt(&drifted).iter().any(|diagnostic| diagnostic.contains("only valid")));
    }

    #[test]
    fn z4_solver_receipts_preserve_non_proof_solver_boundaries() {
        let unsat = z4_receipt(include_str!("../tests/fixtures/evidence-rails/z4-solver-unsat-proof.json"));
        let unsat_json = render_receipt_json(&unsat).expect("receipt renders");
        let unsat_summary = render_summary(&unsat);
        let unsat_provenance =
            render_provenance_jsonl(&unsat, &unsat_json, &unsat_summary).expect("provenance renders");
        let details = unsat.z4_solver.as_ref().expect("z4 details are present");

        assert_eq!(unsat.status, STATUS_PASSED);
        assert!(validate_receipt(&unsat).is_empty());
        assert_eq!(unsat.checker_id, crate::evidence::CHECKER_Z4_SOLVER);
        assert!(unsat.checked_rule_families.iter().any(|family| family == "query-identity"));
        assert_eq!(details.result_status, Z4_STATUS_UNSAT);
        assert!(details.proof_ref.is_some());
        assert!(details.query.identity.starts_with(BLAKE3_IDENTITY_PREFIX));
        assert!(unsat.non_claims.iter().any(|claim| claim == Z4_SOLVER_NON_CLAIM));
        assert!(unsat_provenance.contains("\"source_id\":\"octet.experimental_solver_rails.z4-solver\""));
        assert!(unsat_provenance.contains("\"evidence_tier\":\"transcript_assertion\""));

        let sat = z4_receipt(include_str!("../tests/fixtures/evidence-rails/z4-solver-sat-diagnostic.json"));
        assert_eq!(sat.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(sat.finding_counts.get("z4-sat"), Some(&1));
        assert_eq!(sat.finding_counts.get("z4-diagnostic"), Some(&1));

        let unknown = z4_receipt(include_str!("../tests/fixtures/evidence-rails/z4-solver-unknown.json"));
        assert_eq!(unknown.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(unknown.finding_counts.get("z4-unknown"), Some(&1));
        assert_eq!(
            unknown.z4_solver.as_ref().and_then(|details| details.unknown_reason.as_deref()),
            Some("resource-limit")
        );

        let invalid = z4_receipt(include_str!("../tests/fixtures/evidence-rails/z4-solver-invalid-overclaim.json"));
        assert_eq!(invalid.status, STATUS_FAILED);
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("BLAKE3")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("non-claim")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("overclaim")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("formal_proof")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("claims.proves_correctness")));

        let mut drifted = unsat.clone();
        drifted.rail_id = EvidenceRailKind::DependencyAudit.as_str().to_owned();
        assert!(validate_receipt(&drifted).iter().any(|diagnostic| diagnostic.contains("only valid")));
    }

    #[test]
    fn kani_panic_pilot_fixtures_and_templates_capture_assumption_review() {
        let passed = kani_receipt(include_str!("../tests/fixtures/evidence-rails/kani-panic-pilot-passed.json"));
        assert_eq!(passed.status, STATUS_PASSED);
        assert_eq!(passed.property_classes, vec![KANI_PROPERTY_CLASS_BOUNDARY_PANIC.to_owned()]);

        let rejected =
            kani_receipt(include_str!("../tests/fixtures/evidence-rails/kani-panic-pilot-invalid-assumption.json"));
        assert_eq!(rejected.status, STATUS_FAILED);
        assert!(rejected.diagnostics.iter().any(|diagnostic| diagnostic.contains("vacuous")));
        assert!(rejected.diagnostics.iter().any(|diagnostic| diagnostic.contains("reviewed assumptions")));

        let guide = include_str!("../../docs/kani-panic-freedom-pilots.md");
        let template = include_str!("../../templates/kani-panic-harness.rs");
        assert!(guide.contains("Positive and negative evidence"));
        assert!(guide.contains("cargo octet evidence collect --rail kani-model-check"));
        assert!(template.contains("kani::assume"));
        assert!(template.contains("check_invalid_index_is_rejected"));
    }

    #[test]
    fn checked_in_fixture_matrix_covers_positive_and_negative_rails() {
        let clean_deny = include_str!("../tests/fixtures/evidence-rails/dependency-audit-clean.json");
        let denied = include_str!("../tests/fixtures/evidence-rails/dependency-audit-denied.json");
        let clippy = include_str!("../tests/fixtures/evidence-rails/clippy-warning.jsonl");
        let trust = include_str!("../tests/fixtures/evidence-rails/dependency-trust-covered.json");
        let semver_removed = include_str!("../tests/fixtures/evidence-rails/api-stability-removed.json");
        let dynamic_mismatch = include_str!("../tests/fixtures/evidence-rails/dynamic-profile-mismatch.json");
        let cargo_fuzz_clean = include_str!("../tests/fixtures/evidence-rails/cargo-fuzz-clean.json");
        let cargo_fuzz_crash = include_str!("../tests/fixtures/evidence-rails/cargo-fuzz-crash.json");
        let cargo_fuzz_invalid = include_str!("../tests/fixtures/evidence-rails/cargo-fuzz-invalid.json");
        let release_clean = include_str!("../tests/fixtures/evidence-rails/release-supply-chain-clean.json");
        let mutation_clean = include_str!("../tests/fixtures/evidence-rails/mutation-testing-clean.json");
        let coverage_regression = include_str!("../tests/fixtures/evidence-rails/coverage-baseline-regression.json");
        let coverage_native = include_str!("../tests/fixtures/evidence-rails/coverage-native-current.json");
        let coverage_percentage_only =
            include_str!("../tests/fixtures/evidence-rails/coverage-native-percentage-only.json");
        let kani_contract = include_str!("../tests/fixtures/evidence-rails/kani-model-check-contract.json");
        let kani_invalid = include_str!("../tests/fixtures/evidence-rails/kani-model-check-invalid.json");
        let kani_panic_passed = include_str!("../tests/fixtures/evidence-rails/kani-panic-pilot-passed.json");
        let kani_panic_invalid =
            include_str!("../tests/fixtures/evidence-rails/kani-panic-pilot-invalid-assumption.json");
        let hegel_passed = include_str!("../tests/fixtures/evidence-rails/hegel-property-passed.json");
        let hegel_invalid = include_str!("../tests/fixtures/evidence-rails/hegel-property-invalid.json");
        let verifier_positive = include_str!("../tests/fixtures/evidence-rails/verifier-fixture-positive.json");
        let verifier_negative = include_str!("../tests/fixtures/evidence-rails/verifier-fixture-negative.json");
        let z4_unsat = include_str!("../tests/fixtures/evidence-rails/z4-solver-unsat-proof.json");
        let z4_sat = include_str!("../tests/fixtures/evidence-rails/z4-solver-sat-diagnostic.json");
        let z4_unknown = include_str!("../tests/fixtures/evidence-rails/z4-solver-unknown.json");
        let z4_invalid = include_str!("../tests/fixtures/evidence-rails/z4-solver-invalid-overclaim.json");

        assert_eq!(dependency_audit_receipt(clean_deny, Some(DENY_CONFIG)).status, STATUS_PASSED);
        assert_eq!(dependency_audit_receipt(denied, Some(DENY_CONFIG)).finding_counts.get("licenses"), Some(&1));
        assert_eq!(clippy_receipt(clippy).findings[0].family, "unwrap_used");
        assert_eq!(dependency_trust_receipt(trust).status, STATUS_PASSED);
        assert_eq!(api_removed_receipt(semver_removed).finding_counts.get("removed-public-item"), Some(&1));
        assert!(!dynamic_mismatch_receipt(dynamic_mismatch).diagnostics.is_empty());
        assert!(crate::cargo::fuzz::evaluate_envelope(cargo_fuzz_clean).diagnostics.is_empty());
        assert!(crate::cargo::fuzz::evaluate_envelope(cargo_fuzz_crash).is_crash_observed);
        assert!(!crate::cargo::fuzz::evaluate_envelope(cargo_fuzz_invalid).diagnostics.is_empty());
        assert_eq!(release_receipt(release_clean).status, STATUS_PASSED);
        assert_eq!(mutation_receipt(mutation_clean).status, STATUS_PASSED);
        assert_eq!(coverage_receipt(coverage_regression, Some(BASELINE_IDENTITY.to_owned())).status, STATUS_FAILED);
        assert_eq!(profiled_coverage_receipt(coverage_native).status, STATUS_PASSED);
        assert_eq!(profiled_coverage_receipt(coverage_percentage_only).status, STATUS_FAILED);
        assert_eq!(kani_receipt(kani_contract).status, STATUS_PASSED);
        assert_eq!(kani_receipt(kani_invalid).status, STATUS_FAILED);
        assert_eq!(kani_receipt(kani_panic_passed).status, STATUS_PASSED);
        assert_eq!(kani_receipt(kani_panic_invalid).status, STATUS_FAILED);
        assert_eq!(hegel_receipt(hegel_passed).status, STATUS_PASSED);
        assert_eq!(hegel_receipt(hegel_invalid).status, STATUS_FAILED);
        assert_eq!(verifier_fixture_receipt(verifier_positive).status, STATUS_PASSED);
        assert_eq!(verifier_fixture_receipt(verifier_negative).status, STATUS_FAILED);
        assert_eq!(z4_receipt(z4_unsat).status, STATUS_PASSED);
        assert_eq!(z4_receipt(z4_sat).status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(z4_receipt(z4_unknown).status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(z4_receipt(z4_invalid).status, STATUS_FAILED);
    }

    #[test]
    fn run_mode_command_argv_covers_selected_external_tools() {
        assert_eq!(command_argv(EvidenceRailKind::DependencyAudit, Some(DENY_CONFIG), None, None), vec![
            "cargo",
            "deny",
            "check",
            "--config",
            DENY_CONFIG
        ]);
        assert_eq!(command_argv(EvidenceRailKind::ClippyCoReporting, None, None, None), vec![
            "cargo",
            "clippy",
            "--message-format=json"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::DependencyTrust, None, None, None), vec![
            "cargo",
            "vet",
            "--output-format=json"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::ApiStability, None, Some("baseline"), None), vec![
            "cargo",
            "semver-checks",
            "check-release",
            "--baseline-root",
            "baseline"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::DynamicEvidence, None, None, Some("cargo miri test")), vec![
            "cargo", "miri", "test"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::ReleaseSupplyChain, None, None, None), vec![
            "cargo",
            "auditable",
            "build"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::MutationTesting, None, None, None), vec![
            "cargo", "mutants", "--json"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::CoverageBaseline, None, None, None), vec![
            "cargo",
            "llvm-cov",
            "--json",
            "--summary-only"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::KaniModelCheck, None, None, None), vec!["cargo", "kani"]);
        assert_eq!(
            command_argv(
                EvidenceRailKind::HegelProperty,
                None,
                None,
                Some("cargo test -p valence-core hegel_property")
            ),
            vec!["cargo", "test", "-p", "valence-core", "hegel_property"]
        );
        assert_eq!(command_argv(EvidenceRailKind::Z4Solver, None, None, Some("z4 solve --json query.smt2")), vec![
            "z4",
            "solve",
            "--json",
            "query.smt2"
        ]);
        assert_eq!(
            command_argv(
                EvidenceRailKind::VerifierFixture,
                None,
                None,
                Some("cargo test verifier_fixture_normalization")
            ),
            vec!["cargo", "test", "verifier_fixture_normalization"]
        );
    }

    #[test]
    fn dynamic_run_mode_rejects_unconfigured_commands_before_spawn() {
        let args = crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::DynamicEvidence,
            input: None,
            artifact_dir: std::path::PathBuf::from("artifacts/octet-evidence"),
            workspace_root: None,
            config: None,
            baseline: None,
            profile: Some(DYNAMIC_PROFILE_MIRI_UNSAFE_BOUNDARY.to_owned()),
            command: Some("cargo test".to_owned()),
            parent_artifacts: Vec::new(),
            external_reports: Vec::new(),
            output_format: OutputFormat::Human,
            locked: false,
            update_advisory_database: false,
            run: true,
            command_line: "cargo octet evidence collect --rail dynamic-evidence --run".to_owned(),
        };

        let error = run_mode_command_argv(&args).expect_err("disallowed dynamic command is rejected before spawn");
        assert!(error.contains("not allowed by profile"));

        let allowed = crate::cli::EvidenceRailArgs {
            command: Some(DYNAMIC_COMMAND_MIRI_TEST.to_owned()),
            ..args
        };
        assert_eq!(run_mode_command_argv(&allowed).expect("allowed dynamic command"), vec!["cargo", "miri", "test"]);
    }

    #[test]
    fn verifier_fixture_run_mode_requires_explicit_command_before_spawn() {
        let args = crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::VerifierFixture,
            input: None,
            artifact_dir: std::path::PathBuf::from("artifacts/octet-evidence"),
            workspace_root: None,
            config: None,
            baseline: None,
            profile: None,
            command: None,
            parent_artifacts: Vec::new(),
            external_reports: Vec::new(),
            output_format: OutputFormat::Human,
            locked: false,
            update_advisory_database: false,
            run: true,
            command_line: "cargo octet evidence collect --rail verifier-fixture --run".to_owned(),
        };

        let error = run_mode_command_argv(&args).expect_err("implicit verifier command is rejected before spawn");
        assert!(error.contains("explicit --command"));

        let allowed = crate::cli::EvidenceRailArgs {
            command: Some("cargo test verifier_fixture_normalization".to_owned()),
            ..args
        };
        assert_eq!(run_mode_command_argv(&allowed).expect("allowed verifier fixture command"), vec![
            "cargo",
            "test",
            "verifier_fixture_normalization"
        ]);
    }

    #[test]
    fn z4_run_mode_requires_explicit_command_before_spawn() {
        let args = crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::Z4Solver,
            input: None,
            artifact_dir: std::path::PathBuf::from("artifacts/octet-evidence"),
            workspace_root: None,
            config: None,
            baseline: None,
            profile: None,
            command: None,
            parent_artifacts: Vec::new(),
            external_reports: Vec::new(),
            output_format: OutputFormat::Human,
            locked: false,
            update_advisory_database: false,
            run: true,
            command_line: "cargo octet evidence collect --rail z4-solver --run".to_owned(),
        };

        let error = run_mode_command_argv(&args).expect_err("implicit Z4 command is rejected before spawn");
        assert!(error.contains("explicit --command"));

        let allowed = crate::cli::EvidenceRailArgs {
            command: Some("z4 solve --json query.smt2".to_owned()),
            ..args
        };
        assert_eq!(run_mode_command_argv(&allowed).expect("allowed z4 command"), vec![
            "z4",
            "solve",
            "--json",
            "query.smt2"
        ]);
    }

    #[test]
    fn hegel_run_mode_requires_checked_profile_and_rejects_command_authority() {
        let legacy = crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::HegelProperty,
            input: None,
            artifact_dir: std::path::PathBuf::from("artifacts/octet-evidence"),
            workspace_root: None,
            config: None,
            baseline: None,
            profile: None,
            command: None,
            parent_artifacts: Vec::new(),
            external_reports: Vec::new(),
            output_format: OutputFormat::Human,
            locked: false,
            update_advisory_database: false,
            run: true,
            command_line: "cargo octet evidence collect --rail hegel-property --run".to_owned(),
        };
        let error = run_mode_command_argv(&legacy).expect_err("unprofiled Hegel run is rejected before spawn");
        assert!(error.contains("checked profiled producer"));

        let profiled = crate::cli::EvidenceRailArgs {
            profile: Some(crate::hegel::PROFILE_ID.to_owned()),
            config: Some(std::path::PathBuf::from("standards/hegel/profiles.json")),
            ..legacy
        };
        assert_eq!(
            run_mode_command_argv(&profiled).expect("fixed Hegel command"),
            crate::hegel::selected_profile(crate::hegel::PROFILE_ID).expect("profile").command_argv
        );
        let arbitrary = crate::cli::EvidenceRailArgs {
            command: Some("cargo test arbitrary".to_owned()),
            ..profiled
        };
        assert!(run_mode_command_argv(&arbitrary).expect_err("command rejected").contains("authority"));
    }

    #[test]
    fn receipt_validation_rejects_schema_checker_and_non_claim_drift() {
        let mut receipt = dependency_trust_receipt(r#"{"imports":[],"exemptions":[],"unaudited":[]}"#);
        receipt.schema_version = "old".to_owned();
        receipt.checker_id = "wrong".to_owned();
        receipt.non_claims.clear();

        let diagnostics = validate_receipt(&receipt);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("schema_version")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("checker_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("non-claim")));
    }

    fn dependency_audit_receipt(import_text: &str, config_path: Option<&str>) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyAudit,
            mode: MODE_IMPORT,
            import_text,
            workspace_scope: WORKSPACE_SCOPE,
            config_path,
            baseline_identity: None,
            lockfile_identity: None,
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        })
    }

    fn clippy_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            rail: EvidenceRailKind::ClippyCoReporting,
            mode: MODE_IMPORT,
            import_text,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        })
    }

    fn dependency_trust_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyTrust,
            mode: MODE_IMPORT,
            import_text,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        })
    }

    fn api_removed_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..api_stability_input()
        })
    }

    fn dynamic_mismatch_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..dynamic_input()
        })
    }

    fn release_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..release_input()
        })
    }

    fn mutation_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..mutation_input()
        })
    }

    fn profiled_mutation_receipt(outcome: &str) -> EvidenceRailReceipt {
        let request = profiled_mutation_request(outcome);
        evaluate(EvaluationInput {
            rail: EvidenceRailKind::MutationTesting,
            mode: MODE_IMPORT,
            import_text: &request,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("standards/mutation/profiles.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: Some(crate::mutation::PROFILE_ID),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        })
    }

    fn profiled_mutation_request(outcome: &str) -> String {
        let mut artifacts = std::collections::BTreeMap::new();
        artifacts.insert("log/baseline.log".to_owned(), "baseline log".to_owned());
        artifacts.insert("log/mutant.log".to_owned(), "mutant log".to_owned());
        artifacts.insert("diff/mutant.diff".to_owned(), "mutant diff".to_owned());
        crate::mutation::request::render(crate::mutation::request::InputParts {
            outcomes_text: serde_json::json!({"outcomes": [
                {"scenario": "Baseline", "summary": "Success", "log_path": "log/baseline.log", "diff_path": null},
                {"scenario": {"Mutant": {"name": "src/lib.rs:1:1: replace value"}}, "summary": outcome,
                 "log_path": "log/mutant.log", "diff_path": "diff/mutant.diff"}
            ]}).to_string(),
            inventory_text: serde_json::json!([{
                "name": "src/lib.rs:1:1: replace value", "package": "octet-lint-metadata",
                "file": "src/lib.rs", "replacement": "false", "genre": "FnValue",
                "span": {"start": {"line": 1, "column": 1}}
            }]).to_string(),
            source_manifest_text: serde_json::json!({
                "schema_version": "octet-source-manifest/v1",
                "files": [{"path": "src/lib.rs", "blake3": "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]
            }).to_string(),
            artifacts,
            bounded_log_identity: prefixed_identity(b""),
        }).expect("profiled mutation request")
    }

    fn coverage_receipt(import_text: &str, baseline_identity: Option<String>) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            baseline_identity,
            ..coverage_input()
        })
    }

    fn profiled_coverage_receipt(current: &str) -> EvidenceRailReceipt {
        let request = profiled_coverage_request(current);
        let baseline_identity = crate::coverage::request::baseline_identity(&request);
        let mut receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::CoverageBaseline,
            mode: MODE_IMPORT,
            import_text: &request,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("standards/coverage/profiles.json"),
            baseline_identity,
            lockfile_identity: Some("b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_owned()),
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: Some(crate::coverage::PROFILE_ID),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        receipt.config_identity = Some(crate::coverage::PROFILE_IDENTITY.to_owned());
        receipt
    }

    fn profiled_coverage_request(current: &str) -> String {
        let baseline = include_str!("../tests/fixtures/evidence-rails/coverage-native-baseline.json");
        let source_identity =
            format!("source:{}", crate::coverage::prefixed_identity(COVERAGE_SOURCE_MANIFEST.as_bytes()));
        let envelope = crate::coverage::baseline_envelope_text(baseline, crate::coverage::PROFILE_ID, &source_identity)
            .expect("coverage baseline envelope");
        crate::coverage::request::render(
            current.to_owned(),
            envelope,
            source_identity,
            COVERAGE_SOURCE_MANIFEST.to_owned(),
            prefixed_identity(b""),
        )
        .expect("profiled coverage request")
    }

    fn kani_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..kani_input()
        })
    }

    fn hegel_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..hegel_input()
        })
    }

    fn z4_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..z4_input()
        })
    }

    fn verifier_fixture_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..verifier_fixture_input()
        })
    }

    fn draw_hegel_salt(tc: &TestCase) -> u64 {
        tc.draw(gs::integers::<u64>().min_value(PROPERTY_MIN_SALT).max_value(PROPERTY_MAX_SALT))
    }

    fn hegel_passed_json(salt: u64, reversed: bool) -> String {
        let mut linked_ids = vec![format!("r[octet.hegel.{salt}.b]"), format!("r[octet.hegel.{salt}.a]")];
        let mut source_hashes = vec![hegel_identity("source-b", salt), hegel_identity("source-a", salt)];
        if reversed {
            linked_ids.reverse();
            source_hashes.reverse();
        }
        serde_json::json!({
            "schema_version": HEGEL_PROPERTY_SCHEMA_VERSION,
            "property_id": format!("octet.evidence-rails.hegel.{salt}"),
            "linked_ids": linked_ids,
            "source_scope": "cargo-octet/src/evidence_rails.rs::evaluate",
            "runner": {
                "crate_version": "0.14.12",
                "backend": "native"
            },
            "settings": {
                "case_budget": HEGEL_FAST_CASE_BUDGET,
                "phases": ["generate", "shrink"],
                "seed": salt,
                "derandomized": true
            },
            "generator": {
                "family": "octet-evidence-rail-arbitrary",
                "content_hash": hegel_identity("generator", salt)
            },
            "source_hashes": source_hashes,
            "status": STATUS_PASSED,
            "evidence_tier": "transcript_assertion",
            "non_claims": [HEGEL_GENERATED_TESTING_NON_CLAIM]
        })
        .to_string()
    }

    fn hegel_bad_hash_json(salt: u64) -> String {
        serde_json::json!({
            "schema_version": HEGEL_PROPERTY_SCHEMA_VERSION,
            "property_id": format!("octet.evidence-rails.hegel.bad-hash.{salt}"),
            "linked_ids": [format!("r[octet.hegel.{salt}.bad]")],
            "source_scope": "cargo-octet/src/evidence_rails.rs::evaluate",
            "runner": {
                "crate_version": "0.14.12",
                "backend": "native"
            },
            "settings": {
                "case_budget": HEGEL_FAST_CASE_BUDGET,
                "phases": ["generate"],
                "seed": salt,
                "derandomized": true
            },
            "generator": {
                "family": "octet-evidence-rail-arbitrary",
                "content_hash": hegel_identity("generator", salt)
            },
            "source_hashes": ["not-a-b3-identity"],
            "status": STATUS_PASSED,
            "evidence_tier": "transcript_assertion",
            "non_claims": []
        })
        .to_string()
    }

    fn hegel_identity(prefix: &str, salt: u64) -> String {
        prefixed_identity(format!("{prefix}:{salt}").as_bytes())
    }

    fn api_stability_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::ApiStability,
            mode: MODE_IMPORT,
            import_text: r#"{"breakages":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: Some(BASELINE_IDENTITY.to_owned()),
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn sarif_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::SarifImport,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some("standards/sarif/profiles.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("codeql-local"),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn dynamic_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::DynamicEvidence,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("miri"),
            requested_command: Some("cargo miri test"),
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn release_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::ReleaseSupplyChain,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn mutation_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::MutationTesting,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn coverage_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::CoverageBaseline,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: Some(BASELINE_IDENTITY.to_owned()),
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn kani_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::KaniModelCheck,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn hegel_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::HegelProperty,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("fast"),
            requested_command: Some("cargo test hegel_property"),
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn z4_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::Z4Solver,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("bounded-diagnostic"),
            requested_command: Some("z4 solve --json query.smt2"),
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn verifier_fixture_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::VerifierFixture,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("regression-fixtures"),
            requested_command: Some("cargo test verifier_fixture_normalization"),
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }
}
