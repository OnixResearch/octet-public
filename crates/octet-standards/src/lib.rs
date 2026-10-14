#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
//! Functional core for the Tiger Style policy checker.

pub mod architecture;
pub mod artifacts;
pub mod capability_manifest;
pub mod customs;
pub mod dependency_direction;
pub mod diagnostic;
pub mod fixture;
pub mod model;
pub mod nickel_export;
pub mod proof;
pub mod proof_artifact;
pub mod receipt;
pub mod report;
pub mod resource;
pub mod runner;
pub mod source;
pub mod suppression;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::architecture::CARGO_METADATA_SOURCE_PATH;
    use crate::architecture::check_architecture;
    use crate::capability_manifest::check_capability_manifest;
    use crate::fixture::FixtureManifest;
    use crate::fixture::PolicyFixtureCoverage;
    use crate::fixture::validate_hard_policy_fixture_coverage;
    use crate::model::Classification;
    use crate::model::EnforcementLevel;
    use crate::model::EnforcementMode;
    use crate::model::EvidenceCommand;
    use crate::model::FamilySelection;
    use crate::model::FixtureCoverage;
    use crate::model::FixtureStatus;
    use crate::model::Mechanism;
    use crate::model::MechanismKind;
    use crate::model::PolicyEntry;
    use crate::model::PolicyRegistry;
    use crate::model::ProofRail;
    use crate::model::ReceiptProvenance;
    use crate::model::ReceiptVerifier;
    use crate::model::Severity;
    use crate::model::SuppressionRequirements;
    use crate::proof::PROOF_STATUS_PASSED;
    use crate::proof::VerusLogArtifact;
    use crate::proof::VerusProofReceiptInput;
    use crate::proof::build_verus_proof_receipt;
    use crate::proof::check_verified_proof_rails;
    use crate::report::render_json;
    use crate::report::render_markdown;
    use crate::resource::check_resources;
    use crate::runner::CheckInput;
    use crate::runner::run_checks;
    use crate::source::SourceFile;

    const DEVTOOLS_POLICY_ID: &str = "tigerstyle.devtools.lint-suppressions";
    const EMBEDDED_POLICY_ID: &str = "tigerstyle.embedded.bounded-queues";
    const FCIS_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
    const EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID: &str = "tigerstyle.architecture.external-domain-leakage";
    const ROOT_THINNESS_POLICY_ID: &str = "tigerstyle.architecture.root-crate-thinness";
    const PUBLIC_API_BUDGET_POLICY_ID: &str = "tigerstyle.architecture.public-api-budget";
    const EFFECT_CAPABILITY_POLICY_ID: &str = "tigerstyle.architecture.effect-capability-boundaries";
    const HEXAGONAL_PORT_POLICY_ID: &str = "tigerstyle.architecture.hexagonal-ports";
    const TYPE_STATE_POLICY_ID: &str = "tigerstyle.idioms.nominal-type-boundaries";
    const VERIFIED_POLICY_ID: &str = "tigerstyle.verification.deterministic-effect-handlers";
    const SOURCE_PATH: &str = "src/lib.rs";
    const DEVTOOLS_SOURCE: &str = "docs/standards/source/devtools.adoc#enforcing-not-advising";
    const EMBEDDED_SOURCE: &str = "docs/standards/source/embedded.adoc#bounded-everything";
    const OWNER: &str = "owner=platform";
    const RATIONALE: &str = "rationale=hardware boundary";
    const REVIEW: &str = "review=next hardware revision";
    const SAFER_DEFAULT: &str = "safer_default=bounded queue";
    const MISSING_SUPPRESSION_METADATA_DIAGNOSTICS: usize = 4;
    const MISSING_FIXTURE_COVERAGE_DIAGNOSTICS: usize = 8;
    const THICK_ROOT_LINE_COUNT: usize = 50;
    const BOUNDED_CHANNEL_CAPACITY: u16 = 16;
    const PUBLIC_API_BUDGET_FIXTURE_PATH: &str =
        "standards/check-fixtures/architecture/advisory/public-api-budget/src/lib.rs";
    const PUBLIC_API_BUDGET_FIXTURE_SOURCE: &str =
        include_str!("../../../standards/check-fixtures/architecture/advisory/public-api-budget/src/lib.rs");
    const DEPENDENCY_DIRECTION_CLEAN_METADATA: &str =
        include_str!("../../../standards/check-fixtures/architecture/dependency-direction/clean.json");
    const DEPENDENCY_DIRECTION_CORE_TO_ADAPTER_METADATA: &str =
        include_str!("../../../standards/check-fixtures/architecture/dependency-direction/core-to-adapter.json");
    const DEPENDENCY_DIRECTION_CORE_TO_EXTERNAL_METADATA: &str =
        include_str!("../../../standards/check-fixtures/architecture/dependency-direction/core-to-external.json");
    const DEPENDENCY_DIRECTION_SHELL_OUTWARD_METADATA: &str =
        include_str!("../../../standards/check-fixtures/architecture/dependency-direction/shell-outward.json");
    const DEPENDENCY_DIRECTION_ADAPTER_MISSING_CORE_METADATA: &str =
        include_str!("../../../standards/check-fixtures/architecture/dependency-direction/adapter-missing-core.json");
    const DEPENDENCY_DIRECTION_TRANSITIVE_METADATA: &str =
        include_str!("../../../standards/check-fixtures/architecture/dependency-direction/transitive-leakage.json");

    #[test]
    fn valid_suppression_is_reported_as_active() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, valid_suppression_line(DEVTOOLS_POLICY_ID))],
            selection: FamilySelection::all(),
        });

        assert!(report.diagnostics.is_empty());
        assert_eq!(report.active_suppressions.len(), 1);
        assert_eq!(report.active_suppressions[0].policy_id, DEVTOOLS_POLICY_ID);
        assert_eq!(report.active_suppressions[0].line, 1);
    }

    #[test]
    fn missing_suppression_metadata_returns_blocking_diagnostics() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(
                SOURCE_PATH,
                "// tigerstyle-allow: policy=tigerstyle.devtools.lint-suppressions; owner=platform",
            )],
            selection: FamilySelection::all(),
        });

        assert!(report.has_blocking_diagnostics());
        assert_eq!(report.active_suppressions.len(), 0);
        assert_eq!(report.diagnostics.len(), MISSING_SUPPRESSION_METADATA_DIAGNOSTICS);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("required `source` metadata"))
        );
    }

    #[test]
    fn blanket_suppression_scope_is_rejected() {
        let line = format!("{}; scope=crate", valid_suppression_line(DEVTOOLS_POLICY_ID));
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, line)],
            selection: FamilySelection::all(),
        });

        assert!(report.has_blocking_diagnostics());
        assert_eq!(report.active_suppressions.len(), 0);
        assert_eq!(report.diagnostics[0].message, "blanket suppression scope `crate` is not allowed");
    }

    #[test]
    fn unknown_policy_is_rejected() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(
                SOURCE_PATH,
                "// tigerstyle-allow: policy=tigerstyle.unknown.rule; source=doc; owner=a; rationale=b; review=c; safer_default=d",
            )],
            selection: FamilySelection::all(),
        });

        assert!(report.has_blocking_diagnostics());
        assert_eq!(report.diagnostics[0].policy_id, DEVTOOLS_POLICY_ID);
        assert_eq!(report.diagnostics[0].message, "suppression references unknown policy `tigerstyle.unknown.rule`");
    }

    #[test]
    fn family_selection_skips_unselected_policy() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, valid_suppression_line(EMBEDDED_POLICY_ID))],
            selection: FamilySelection::from_families(["devtools"]),
        });

        assert!(report.diagnostics.is_empty());
        assert!(report.active_suppressions.is_empty());
        assert_eq!(report.evaluated_policy_ids, vec![DEVTOOLS_POLICY_ID.to_owned()]);
    }

    #[test]
    fn json_output_is_stable() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, valid_suppression_line(DEVTOOLS_POLICY_ID))],
            selection: FamilySelection::from_families(["devtools"]),
        });
        let json = render_json(&report).expect("report should serialize");

        assert!(json.contains("\"active_suppressions\""));
        assert!(json.contains(DEVTOOLS_POLICY_ID));
        assert!(!report.has_blocking_diagnostics());
    }

    #[test]
    fn markdown_output_is_stable() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, valid_suppression_line(DEVTOOLS_POLICY_ID))],
            selection: FamilySelection::from_families(["devtools"]),
        });
        let markdown = render_markdown(&report);

        assert!(markdown.starts_with("# Tiger Style Standards Report"));
        assert!(markdown.contains("## Active Suppressions"));
        assert!(markdown.contains(DEVTOOLS_POLICY_ID));
    }

    #[test]
    fn architecture_checks_accept_compliant_workspace_shape() {
        let sources = vec![
            SourceFile::new("crates/tigerstyle-core/src/lib.rs", "#![no_std]\n"),
            SourceFile::new("crates/tigerstyle-core/Cargo.toml", "[dependencies]\nsnafu = \"0.8\"\n"),
            SourceFile::new("crates/tigerstyle-std/src/lib.rs", "pub fn shell() {}\n"),
            SourceFile::new("src/lib.rs", "pub use tigerstyle_core as core;\n"),
        ];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_checks_reject_core_runtime_dependency() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/Cargo.toml",
            "[dependencies]\ntokio = \"1\"\n[dev-dependencies]\nclap = \"4\"\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, FCIS_POLICY_ID);
        assert!(
            diagnostics[0].message.contains("forbidden dependency `tokio`")
                || diagnostics[0].message.contains("runtime crate `tokio`")
        );
    }

    #[test]
    fn custom_boundary_checks_reject_core_no_std_with_domain_layout() {
        let sources = vec![SourceFile::new("domain-core/src/lib.rs", "pub fn core() {}\n")];
        let diagnostics = check_architecture(
            &sample_custom_boundary_registry(),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, FCIS_POLICY_ID);
        assert!(diagnostics[0].message.contains("missing #![no_std]"));
        assert!(diagnostics[0].message.contains("domain-core/src/lib.rs"));
    }

    #[test]
    fn custom_boundary_checks_reject_forbidden_dependency_with_domain_layout() {
        let sources = vec![SourceFile::new(
            "domain-core/Cargo.toml",
            "[dependencies]\ntokio = \"1\"\n",
        )];
        let diagnostics = check_architecture(
            &sample_custom_boundary_registry(),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, FCIS_POLICY_ID);
        assert!(diagnostics[0].message.contains("forbidden dependency `tokio`"));
    }

    #[test]
    fn custom_boundary_checks_reject_external_leakage_with_domain_layout() {
        let sources = vec![SourceFile::new(
            "domain-core/src/lib.rs",
            "#![no_std]\n\npub fn handle(input: axum::Json<u32>) -> u32 {\n    input.0\n}\n",
        )];
        let diagnostics = check_architecture(
            &sample_custom_boundary_registry(),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID);
        assert!(diagnostics[0].message.contains("axum::"));
    }

    #[test]
    fn custom_boundary_checks_accept_compliant_domain_layout() {
        let sources = vec![
            SourceFile::new("domain-core/src/lib.rs", "#![no_std]\n"),
            SourceFile::new("domain-core/Cargo.toml", "[dependencies]\nsnafu = \"0.8\"\n"),
            SourceFile::new(
                "infrastructure-shell/Cargo.toml",
                "[dependencies]\ndomain-core = { path = \"../domain-core\" }\n",
            ),
            SourceFile::new("infrastructure-shell/src/lib.rs", "pub fn shell() {}\n"),
        ];
        let diagnostics = check_architecture(
            &sample_custom_boundary_registry(),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_checks_reject_missing_core_no_std() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/lib.rs",
            "pub fn core() {}\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, FCIS_POLICY_ID);
        assert!(diagnostics[0].message.contains("missing #![no_std]"));
    }

    #[test]
    fn architecture_checks_reject_external_domain_leakage() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/lib.rs",
            "#![no_std]\n\npub fn accept(input: axum::Json<DomainCommand>) -> DomainCommand {\n    input.0\n}\n\npub struct DomainCommand;\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID);
        assert!(diagnostics[0].message.contains("axum::"));
        assert!(diagnostics[0].remediation.contains("translate at the boundary"));
    }

    #[test]
    fn architecture_checks_reject_multiple_core_fixture_shapes() {
        let sources = vec![
            SourceFile::new("missing/crates/tigerstyle-core/src/lib.rs", "pub fn core() {}\n"),
            SourceFile::new("leak/crates/tigerstyle-core/src/lib.rs", "#![no_std]\n\npub use tonic::Status;\n"),
        ];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.policy_id == FCIS_POLICY_ID));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.policy_id == EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID));
    }

    #[test]
    fn architecture_checks_reject_thick_root_file() {
        let thick_root = (0..THICK_ROOT_LINE_COUNT).map(|_| "fn helper() {}\n").collect::<String>();
        let sources = vec![SourceFile::new("src/lib.rs", thick_root)];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, ROOT_THINNESS_POLICY_ID);
        assert!(diagnostics[0].message.contains("thin-shell limit"));
    }

    #[test]
    fn architecture_checks_reject_artifact_backed_ambient_capability() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/lib.rs",
            "#![no_std]\n\npub fn process_id() -> u32 { std::process::id() }\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, EFFECT_CAPABILITY_POLICY_ID);
        assert!(diagnostics[0].message.contains("std::process::"));
        assert!(diagnostics[0].message.contains("process capability"));
        assert!(diagnostics[0].remediation.contains("Clock"));
    }

    #[test]
    fn architecture_checks_name_direct_effect_capability_families() {
        let source = "#![allow(dead_code)]\n\
fn filesystem() { let _ = std::fs::read(\"config\"); }\n\
fn network() { let _ = std::net::TcpStream::connect(\"127.0.0.1:1\"); }\n\
fn time() { let _ = SystemTime::now(); }\n\
fn randomness() { let _ = rand::random::<u64>(); }\n\
fn process() { let _ = std::process::id(); }\n\
fn environment() { let _ = std::env::var(\"HOME\"); }\n\
fn secrets() { let _ = Secret::load(); }\n\
static mut VALUE: u8 = 0;\n\
fn concurrency() { let _ = std::thread::spawn(|| ()); }\n";
        let sources = vec![SourceFile::new("crates/tigerstyle-core/src/lib.rs", source)];
        let diagnostics = check_architecture(
            &sample_all_effect_capability_registry(),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );
        let messages = diagnostics.iter().map(|diagnostic| diagnostic.message.as_str()).collect::<Vec<_>>().join("\n");

        assert!(messages.contains("filesystem capability"));
        assert!(messages.contains("network capability"));
        assert!(messages.contains("time capability"));
        assert!(messages.contains("randomness capability"));
        assert!(messages.contains("process capability"));
        assert!(messages.contains("environment capability"));
        assert!(messages.contains("secrets capability"));
        assert!(messages.contains("global-state capability"));
        assert!(messages.contains("concurrency capability"));
    }

    #[test]
    fn architecture_checks_accept_artifact_backed_shell_constructor_scope() {
        let sources = vec![
            SourceFile::new(
                "crates/tigerstyle-core/src/lib.rs",
                "#![no_std]\n\npub trait Clock { fn now_ms(&self) -> u64; }\n",
            ),
            SourceFile::new(
                "crates/tigerstyle-std/src/lib.rs",
                "pub fn now_ms() -> u64 { std::time::SystemTime::now().elapsed().map_or(0, |_| 1) }\n",
            ),
        ];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn capability_manifest_report_records_direct_and_injected_observations() {
        let sources = vec![
            SourceFile::new(
                "crates/tigerstyle-core/src/lib.rs",
                "#![no_std]\n\npub trait Clock { fn now_ms(&self) -> u64; }\npub fn process_id() -> u32 { std::process::id() }\n",
            ),
            SourceFile::new(
                "standards/check-fixtures/capability/clock-handler.txt",
                "success path: fake clock returns deterministic millis\nfailure path: fake clock returns configured error\n",
            ),
        ];
        let check = check_capability_manifest(
            &sample_capability_handler_registry(),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(check.diagnostics.is_empty(), "unexpected diagnostics: {:?}", check.diagnostics);
        assert_eq!(check.reports.len(), 1);
        let report = &check.reports[0];
        assert!(report.observations.iter().any(|observation| {
            observation.observation_kind == "direct-platform-effect"
                && observation.capability_family == "process"
                && observation.status == "diagnostic"
        }));
        assert!(report.observations.iter().any(|observation| {
            observation.observation_kind == "injected-capability"
                && observation.matched == "Clock"
                && observation.status == "accepted"
        }));
        assert!(report.handler_evidence.iter().any(|evidence| {
            evidence.handler == "FakeClock"
                && evidence.success_path_coverage
                && evidence.failure_path_coverage
                && !evidence.blake3.is_empty()
        }));
    }

    #[test]
    fn capability_manifest_rejects_missing_handler_failure_coverage() {
        let sources = vec![SourceFile::new(
            "standards/check-fixtures/capability/clock-handler.txt",
            "success path: fake clock returns deterministic millis\n",
        )];
        let check = check_capability_manifest(
            &sample_capability_handler_registry(),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(
            check
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("must cover success and failure paths") })
        );
    }

    #[test]
    fn capability_manifest_rejects_ambiguous_strict_scope_resolution() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/generated/model.rs",
            "pub trait Clock { fn now_ms(&self) -> u64; }\n",
        )];
        let check = check_capability_manifest(
            &sample_ambiguous_capability_registry(),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(
            check
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("capability scope resolution is ambiguous") })
        );
    }

    #[test]
    fn capability_manifest_report_preserves_non_claim_boundaries() {
        let check =
            check_capability_manifest(&sample_registry(), &[], &FamilySelection::from_families(["architecture"]));
        let report = &check.reports[0];
        let joined = report.non_claims.join("\n");

        assert!(joined.contains("observed source shapes only"));
        assert!(joined.contains("does not add Rust effect types"));
        assert!(joined.contains("does not prove transitive effect inference"));
    }

    #[test]
    fn architecture_port_mechanism_noops_without_declared_ports() {
        let sources = vec![SourceFile::new(
            "domain/src/lib.rs",
            "pub trait Clock { fn now_ms(&self) -> u64; }\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_port_mechanism_rejects_missing_core_port_trait() {
        let sources = vec![SourceFile::new(
            "adapters/std/src/lib.rs",
            "impl Clock for AdapterClock { fn now_ms(&self) -> u64 { 0 } }\n",
        )];
        let diagnostics = check_architecture(
            &sample_port_registry(false),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, HEXAGONAL_PORT_POLICY_ID);
        assert!(diagnostics[0].message.contains("must be defined as a trait in core"));
    }

    #[test]
    fn architecture_port_mechanism_rejects_missing_adapter_impl() {
        let sources = vec![SourceFile::new(
            "domain/src/lib.rs",
            "pub trait Clock { fn now_ms(&self) -> u64; }\n",
        )];
        let diagnostics = check_architecture(
            &sample_port_registry(false),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, HEXAGONAL_PORT_POLICY_ID);
        assert!(diagnostics[0].message.contains("no implementation"));
    }

    #[test]
    fn architecture_port_mechanism_accepts_adapter_impl() {
        let sources = vec![
            SourceFile::new("domain/src/lib.rs", "pub trait Clock { fn now_ms(&self) -> u64; }\n"),
            SourceFile::new(
                "adapters/std/src/lib.rs",
                "impl Clock for SystemClock { fn now_ms(&self) -> u64 { 0 } }\n",
            ),
        ];
        let diagnostics = check_architecture(
            &sample_port_registry(false),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_port_mechanism_rejects_core_concrete_impl() {
        let sources = vec![
            SourceFile::new(
                "domain/src/lib.rs",
                "pub trait Clock { fn now_ms(&self) -> u64; }\nimpl Clock for SystemClock { fn now_ms(&self) -> u64 { 0 } }\n",
            ),
            SourceFile::new(
                "adapters/std/src/lib.rs",
                "impl Clock for AdapterClock { fn now_ms(&self) -> u64 { 0 } }\n",
            ),
        ];
        let diagnostics = check_architecture(
            &sample_port_registry(false),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, HEXAGONAL_PORT_POLICY_ID);
        assert!(diagnostics[0].message.contains("SystemClock"));
    }

    #[test]
    fn architecture_port_mechanism_reserved_suppresses_missing_impl() {
        let sources = vec![SourceFile::new(
            "domain/src/lib.rs",
            "pub trait Clock { fn now_ms(&self) -> u64; }\n",
        )];
        let diagnostics = check_architecture(
            &sample_port_registry(true),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_port_mechanism_reserved_keeps_core_purity_check() {
        let sources = vec![SourceFile::new(
            "domain/src/lib.rs",
            "pub trait Clock { fn now_ms(&self) -> u64; }\nimpl Clock for SystemClock { fn now_ms(&self) -> u64 { 0 } }\n",
        )];
        let diagnostics = check_architecture(
            &sample_port_registry(true),
            &sources,
            &FamilySelection::from_families(["architecture"]),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, HEXAGONAL_PORT_POLICY_ID);
        assert!(diagnostics[0].message.contains("SystemClock"));
    }

    #[test]
    fn architecture_dependency_direction_accepts_clean_metadata_fixture() {
        let diagnostics = check_architecture(
            &sample_dependency_direction_registry(vec!["normal", "build"]),
            &[metadata_source(DEPENDENCY_DIRECTION_CLEAN_METADATA)],
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_dependency_direction_rejects_core_to_adapter() {
        let diagnostics = check_architecture(
            &sample_dependency_direction_registry(vec!["normal", "build"]),
            &[metadata_source(DEPENDENCY_DIRECTION_CORE_TO_ADAPTER_METADATA)],
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic.message.contains("core crate `domain-core` depends on adapter crate `http-adapter`")
                && diagnostic.message.contains("direct")
        }));
    }

    #[test]
    fn architecture_dependency_direction_rejects_core_to_external_infrastructure() {
        let diagnostics = check_architecture(
            &sample_dependency_direction_registry(vec!["normal", "build"]),
            &[metadata_source(DEPENDENCY_DIRECTION_CORE_TO_EXTERNAL_METADATA)],
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic.message.contains("external infrastructure crate `sqlx`")
        }));
    }

    #[test]
    fn architecture_dependency_direction_rejects_shell_outward_edge() {
        let diagnostics = check_architecture(
            &sample_dependency_direction_registry(vec!["normal", "build"]),
            &[metadata_source(DEPENDENCY_DIRECTION_SHELL_OUTWARD_METADATA)],
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic
                    .message
                    .contains("shell crate `app-shell` depends outward on adapter crate `http-adapter`")
        }));
    }

    #[test]
    fn architecture_dependency_direction_rejects_adapter_missing_core_edge() {
        let diagnostics = check_architecture(
            &sample_dependency_direction_registry(vec!["normal", "build"]),
            &[metadata_source(DEPENDENCY_DIRECTION_ADAPTER_MISSING_CORE_METADATA)],
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic.message.contains("adapter crate `http-adapter` must depend inward")
        }));
    }

    #[test]
    fn architecture_dependency_direction_reports_transitive_leakage_distinctly() {
        let diagnostics = check_architecture(
            &sample_dependency_direction_registry(vec!["normal", "build"]),
            &[metadata_source(DEPENDENCY_DIRECTION_TRANSITIVE_METADATA)],
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic.message.contains("transitive dependency leakage")
                && diagnostic.message.contains("domain-helper")
        }));
    }

    #[test]
    fn architecture_dependency_direction_excludes_dev_dependencies_by_default() {
        let diagnostics = check_architecture(
            &sample_dependency_direction_registry(Vec::new()),
            &[metadata_source(dev_dependency_metadata())],
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_dependency_direction_includes_configured_dev_dependencies() {
        let diagnostics = check_architecture(
            &sample_dependency_direction_registry(vec!["normal", "build", "dev"]),
            &[metadata_source(dev_dependency_metadata())],
            &FamilySelection::from_families(["architecture"]),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic.message.contains("core crate `domain-core` depends on adapter crate `http-adapter`")
        }));
    }

    #[test]
    fn idioms_selection_skips_architecture_family_diagnostics() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/lib.rs",
            "pub fn core() {}\n",
        )];
        let diagnostics = check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["idioms"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_checks_reject_artifact_backed_raw_type_state() {
        let sources = vec![SourceFile::new(
            "standards/check-fixtures/architecture/noncompliant/typestate-raw/src/lib.rs",
            "type ConnectionState = bool;

pub fn send_raw(_state: ConnectionState, _bytes: &[u8]) {}
",
        )];
        let diagnostics = check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["idioms"]));

        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.policy_id == TYPE_STATE_POLICY_ID));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("type ConnectionState = bool")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("send_raw(")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("missing required marker")));
    }

    #[test]
    fn architecture_checks_accept_artifact_backed_type_state_api() {
        let sources = vec![SourceFile::new(
            "standards/check-fixtures/architecture/compliant/typestate-protocol/src/lib.rs",
            "pub struct Disconnected;
pub struct Connected;
pub struct Authenticated;
impl Session<Disconnected> { pub fn connect(self) -> Session<Connected> { todo!() } }
impl Session<Connected> { pub fn authenticate(self) -> Session<Authenticated> { todo!() } }
impl Session<Authenticated> { pub fn send(self) {} }
",
        )];
        let diagnostics = check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["idioms"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn proof_rail_checks_accept_passing_receipt() {
        let sources = vec![
            SourceFile::new(
                "standards/check-fixtures/verification/compliant/src/lib.rs",
                "pub enum Effect { ReadEnv }
",
            ),
            SourceFile::new(
                "standards/check-fixtures/verification/compliant/proof-receipt.txt",
                "proof-result: pass deterministic-effect-handlers
",
            ),
        ];
        let diagnostics =
            check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::from_families(["verification"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn proof_rail_checks_reject_missing_pass_signal() {
        let sources = vec![
            SourceFile::new(
                "standards/check-fixtures/verification/noncompliant/src/lib.rs",
                "pub enum Effect { ReadEnv }
",
            ),
            SourceFile::new(
                "standards/check-fixtures/verification/noncompliant/proof-receipt.txt",
                "proof-result: fail deterministic-effect-handlers
",
            ),
        ];
        let diagnostics =
            check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::from_families(["verification"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, VERIFIED_POLICY_ID);
        assert!(diagnostics[0].message.contains("missing pass signal"));
        assert!(diagnostics[0].remediation.contains("rerun the configured proof command"));
    }

    #[test]
    fn proof_rail_default_selection_rejects_relevant_noncompliant_receipt() {
        let sources = vec![
            SourceFile::new(
                "standards/check-fixtures/verification/noncompliant/src/lib.rs",
                "pub enum Effect { ReadEnv }\n",
            ),
            SourceFile::new(
                "standards/check-fixtures/verification/noncompliant/proof-receipt.txt",
                "proof-result: fail deterministic-effect-handlers\n",
            ),
        ];
        let diagnostics = check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::all());

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, VERIFIED_POLICY_ID);
        assert!(diagnostics[0].message.contains("missing pass signal"));
    }

    #[test]
    fn proof_rail_family_isolation_skips_verification_diagnostics() {
        let sources = vec![SourceFile::new(
            "standards/check-fixtures/verification/noncompliant/proof-receipt.txt",
            "proof-result: fail deterministic-effect-handlers
",
        )];
        let diagnostics =
            check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::from_families(["embedded"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn verified_module_coverage_accepts_matching_verus_receipt() {
        let sources = sample_verus_sources(PROOF_STATUS_PASSED);
        let diagnostics = check_verified_proof_rails(
            &sample_verus_registry(),
            &sources,
            &FamilySelection::from_families(["verification"]),
        );

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn verified_module_coverage_reports_manual_review_for_unbacked_claim() {
        let sources = vec![SourceFile::new(
            "src/verified/addition.rs",
            "// octet-formal-proof-claim: addition bound\npub fn checked_add_one(value: u64) -> u64 { value + 1 }\n",
        )];
        let diagnostics =
            check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::from_families(["verification"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, "octet.verus_proof_rails.verified_module_coverage");
        assert!(diagnostics[0].message.contains("manual_review_required"));
    }

    #[test]
    fn verified_module_coverage_rejects_matching_verus_receipt_without_pass_signal() {
        let sources = sample_verus_sources(crate::proof::PROOF_STATUS_FAILED);
        let diagnostics = check_verified_proof_rails(
            &sample_verus_registry(),
            &sources,
            &FamilySelection::from_families(["verification"]),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, "octet.verus_proof_rails.addition-bound");
        assert!(diagnostics[0].message.contains("field `status`"));
    }

    #[test]
    fn resource_checks_reject_unbounded_channel() {
        let sources = vec![SourceFile::new(
            "src/device.rs",
            "fn start() { let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel(); }",
        )];
        let diagnostics = check_resources(&sample_registry(), &sources, &FamilySelection::from_families(["embedded"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, EMBEDDED_POLICY_ID);
        assert!(diagnostics[0].message.contains("unbounded_channel"));
    }

    #[test]
    fn resource_checks_accept_bounded_channel() {
        let sources = vec![SourceFile::new(
            "src/device.rs",
            format!("fn start() {{ let (_tx, _rx) = tokio::sync::mpsc::channel({BOUNDED_CHANNEL_CAPACITY}); }}"),
        )];
        let diagnostics = check_resources(&sample_registry(), &sources, &FamilySelection::from_families(["embedded"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn fixture_manifest_accepts_covered_hard_policies() {
        let diagnostics = validate_hard_policy_fixture_coverage(&sample_registry(), &covered_fixture_manifest());

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn advisory_rule_reports_through_run_checks_without_blocking() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(
                PUBLIC_API_BUDGET_FIXTURE_PATH,
                PUBLIC_API_BUDGET_FIXTURE_SOURCE,
            )],
            selection: FamilySelection::from_families(["architecture"]),
        });

        assert_eq!(report.diagnostics.len(), 1);
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.policy_id, PUBLIC_API_BUDGET_POLICY_ID);
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert!(diagnostic.source_section.contains("when-boundaries-are-too-tight"));
        assert!(diagnostic.message.contains("advisory budget"));
        assert!(report.is_advisory_only());
        assert!(!report.has_blocking_diagnostics());
        assert!(report.evaluated_policy_ids.iter().any(|policy_id| policy_id == PUBLIC_API_BUDGET_POLICY_ID));
    }

    #[test]
    fn advisory_rule_accepts_small_public_surface() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, "pub fn one_public_item() {}\n")],
            selection: FamilySelection::from_families(["architecture"]),
        });

        assert!(report.diagnostics.is_empty());
        assert!(!report.has_blocking_diagnostics());
    }

    #[test]
    fn fixture_manifest_rejects_missing_noncompliant_fixture() {
        let manifest = FixtureManifest {
            policy_coverage: vec![PolicyFixtureCoverage {
                policy_id: DEVTOOLS_POLICY_ID.to_owned(),
                compliant: vec!["ok.rs".to_owned()],
                noncompliant: Vec::new(),
            }],
        };
        let diagnostics = validate_hard_policy_fixture_coverage(&sample_registry(), &manifest);

        assert_eq!(diagnostics.len(), MISSING_FIXTURE_COVERAGE_DIAGNOSTICS);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("no noncompliant fixture")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("missing fixture coverage")));
    }

    fn sample_registry() -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![
                sample_policy(DEVTOOLS_POLICY_ID, "devtools", DEVTOOLS_SOURCE),
                sample_policy(EMBEDDED_POLICY_ID, "embedded", EMBEDDED_SOURCE),
                sample_policy(
                    FCIS_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
                ),
                sample_policy(
                    EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-as-architecture",
                ),
                sample_policy(
                    ROOT_THINNESS_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
                ),
                sample_advisory_policy(
                    PUBLIC_API_BUDGET_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#when-boundaries-are-too-tight",
                ),
                sample_effect_capability_policy(),
                sample_type_state_policy(),
                sample_verified_policy(),
            ],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_dependency_direction_registry(edge_kinds: Vec<&str>) -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![sample_dependency_direction_policy(edge_kinds)],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_dependency_direction_policy(edge_kinds: Vec<&str>) -> PolicyEntry {
        PolicyEntry {
            mechanisms: vec![Mechanism {
                kind: MechanismKind::CrateBoundary,
                description: "test dependency direction mechanism".to_owned(),
                core_crates: vec!["crates/domain-core/src/lib.rs".to_owned()],
                shell_crates: vec!["crates/app-shell/Cargo.toml".to_owned()],
                adapter_crates: vec!["crates/http-adapter/Cargo.toml".to_owned()],
                dependency_edge_kinds: edge_kinds.into_iter().map(str::to_owned).collect(),
                ..Mechanism::default()
            }],
            current_enforcement_mode: EnforcementMode::Gate,
            target_enforcement_mode: EnforcementMode::Structural,
            ..sample_policy(
                FCIS_POLICY_ID,
                "architecture",
                "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
            )
        }
    }

    fn metadata_source(text: &str) -> SourceFile {
        SourceFile::new(CARGO_METADATA_SOURCE_PATH, text)
    }

    fn dev_dependency_metadata() -> &'static str {
        r#"{
  "packages": [
    { "id": "core", "name": "domain-core", "manifest_path": "/workspace/crates/domain-core/Cargo.toml" },
    { "id": "adapter", "name": "http-adapter", "manifest_path": "/workspace/crates/http-adapter/Cargo.toml" }
  ],
  "resolve": {
    "nodes": [
      { "id": "core", "deps": [{ "pkg": "adapter", "dep_kinds": [{ "kind": "dev" }] }] },
      { "id": "adapter", "deps": [{ "pkg": "core", "dep_kinds": [{ "kind": null }] }] }
    ]
  },
  "workspace_members": ["core", "adapter"]
}"#
    }

    fn sample_port_registry(reserved: bool) -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![sample_port_policy(reserved)],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_port_policy(reserved: bool) -> PolicyEntry {
        PolicyEntry {
            mechanisms: vec![Mechanism {
                kind: MechanismKind::Port,
                description: "test port mechanism".to_owned(),
                port_traits: vec!["Clock".to_owned()],
                core_owner_scope: "domain/src/".to_owned(),
                permitted_adapter_scopes: vec!["adapters/std/src/".to_owned()],
                forbidden_concrete_types: vec!["SystemClock".to_owned()],
                reserved,
                ..Mechanism::default()
            }],
            current_enforcement_mode: EnforcementMode::Gate,
            target_enforcement_mode: EnforcementMode::Structural,
            ..sample_policy(
                HEXAGONAL_PORT_POLICY_ID,
                "architecture",
                "docs/standards/source/architecture.adoc#boundaries-as-architecture",
            )
        }
    }

    fn sample_advisory_policy(policy_id: &str, pillar: &str, source_anchor: &str) -> PolicyEntry {
        PolicyEntry {
            classification: Classification::Advisory,
            enforcement_level: EnforcementLevel::Advisory,
            default_severity: Severity::Warning,
            suppression_requirements: advisory_suppression_requirements(),
            fixture_coverage: FixtureCoverage {
                status: FixtureStatus::NotApplicable,
                compliant: Vec::new(),
                noncompliant: Vec::new(),
            },
            verification_evidence_requirements: vec![
                "deterministic sample input".to_owned(),
                "stable advisory report".to_owned(),
            ],
            classification_rationale: "sample advisory policy".to_owned(),
            false_positive_risk: "medium".to_owned(),
            ..sample_policy(policy_id, pillar, source_anchor)
        }
    }

    fn sample_effect_capability_policy() -> PolicyEntry {
        PolicyEntry {
            mechanisms: vec![sample_effect_capability_mechanism()],
            current_enforcement_mode: EnforcementMode::Gate,
            target_enforcement_mode: EnforcementMode::Structural,
            ..sample_policy(
                EFFECT_CAPABILITY_POLICY_ID,
                "architecture",
                "docs/standards/source/architecture.adoc#effect-capability-boundaries",
            )
        }
    }

    fn sample_effect_capability_mechanism() -> Mechanism {
        Mechanism {
            kind: MechanismKind::CapabilityApi,
            description: "test capability mechanism".to_owned(),
            forbidden_ambient_apis: vec!["std::process::".to_owned(), "SystemTime::now(".to_owned()],
            core_source_scopes: vec!["crates/tigerstyle-core/src/".to_owned()],
            shell_constructor_scopes: vec!["crates/tigerstyle-std/src/".to_owned()],
            approved_capability_names: vec!["Clock".to_owned()],
            capability_enforcement_mode: "strict".to_owned(),
            capability_scope_classes: vec!["core".to_owned(), "shell".to_owned(), "test".to_owned()],
            capability_families: vec!["time".to_owned(), "process".to_owned()],
            capability_injected_traits: vec!["Clock".to_owned()],
            capability_tokens: vec!["ClockToken".to_owned()],
            capability_shell_interpreters: vec!["SystemClock".to_owned()],
            ..Mechanism::default()
        }
    }

    fn sample_all_effect_capability_registry() -> PolicyRegistry {
        let mut mechanism = sample_effect_capability_mechanism();
        mechanism.forbidden_ambient_apis = vec![
            "std::fs::".to_owned(),
            "std::net::".to_owned(),
            "SystemTime::now(".to_owned(),
            "rand::".to_owned(),
            "std::process::".to_owned(),
            "std::env::".to_owned(),
            "Secret".to_owned(),
            "static mut".to_owned(),
            "thread::spawn".to_owned(),
        ];
        mechanism.capability_families = vec![
            "filesystem".to_owned(),
            "network".to_owned(),
            "time".to_owned(),
            "randomness".to_owned(),
            "process".to_owned(),
            "environment".to_owned(),
            "secrets".to_owned(),
            "global-state".to_owned(),
            "concurrency".to_owned(),
        ];
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![PolicyEntry {
                mechanisms: vec![mechanism],
                current_enforcement_mode: EnforcementMode::Gate,
                target_enforcement_mode: EnforcementMode::Structural,
                ..sample_policy(
                    EFFECT_CAPABILITY_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#effect-capability-boundaries",
                )
            }],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_capability_handler_registry() -> PolicyRegistry {
        let mut mechanism = sample_effect_capability_mechanism();
        mechanism.capability_deterministic_handlers = vec!["FakeClock".to_owned()];
        mechanism.capability_handler_replay_commands = vec!["cargo test fake_clock_handler".to_owned()];
        mechanism.capability_handler_artifacts =
            vec!["standards/check-fixtures/capability/clock-handler.txt".to_owned()];
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![PolicyEntry {
                mechanisms: vec![mechanism],
                current_enforcement_mode: EnforcementMode::Gate,
                target_enforcement_mode: EnforcementMode::Structural,
                ..sample_policy(
                    EFFECT_CAPABILITY_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#effect-capability-boundaries",
                )
            }],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_ambiguous_capability_registry() -> PolicyRegistry {
        let mut mechanism = sample_effect_capability_mechanism();
        mechanism.capability_generated_scopes = vec!["generated/".to_owned()];
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![PolicyEntry {
                mechanisms: vec![mechanism],
                current_enforcement_mode: EnforcementMode::Gate,
                target_enforcement_mode: EnforcementMode::Structural,
                ..sample_policy(
                    EFFECT_CAPABILITY_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#effect-capability-boundaries",
                )
            }],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_custom_boundary_registry() -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![
                sample_policy_with_boundary_mechanism(
                    FCIS_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
                ),
                sample_policy(
                    EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-as-architecture",
                ),
                sample_policy(
                    ROOT_THINNESS_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
                ),
                sample_advisory_policy(
                    PUBLIC_API_BUDGET_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#when-boundaries-are-too-tight",
                ),
                sample_effect_capability_policy(),
            ],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_policy_with_boundary_mechanism(policy_id: &str, pillar: &str, source_anchor: &str) -> PolicyEntry {
        PolicyEntry {
            mechanisms: vec![Mechanism {
                kind: MechanismKind::CrateBoundary,
                description: "custom-layout test mechanism".to_owned(),
                core_crates: vec!["domain-core/src/lib.rs".to_owned()],
                shell_crates: vec!["infrastructure-shell/Cargo.toml".to_owned()],
                adapter_crates: vec!["adapters/http-api/src/".to_owned()],
                forbidden_core_deps: vec!["tokio".to_owned(), "reqwest".to_owned()],
                external_domain_patterns: vec!["axum::".to_owned(), "sqlx::".to_owned()],
                core_no_std: true,
                ..Mechanism::default()
            }],
            current_enforcement_mode: EnforcementMode::Gate,
            target_enforcement_mode: EnforcementMode::Structural,
            ..sample_policy(policy_id, pillar, source_anchor)
        }
    }

    fn sample_type_state_policy() -> PolicyEntry {
        PolicyEntry {
            mechanisms: vec![Mechanism {
                kind: MechanismKind::TypeState,
                description: "test type-state mechanism".to_owned(),
                type_state_source_scopes: vec!["typestate-protocol/src/".to_owned(), "typestate-raw/src/".to_owned()],
                raw_state_alias_patterns: vec!["type ConnectionState = bool".to_owned()],
                raw_transition_patterns: vec!["send_raw(".to_owned()],
                required_state_markers: vec![
                    "Disconnected".to_owned(),
                    "Connected".to_owned(),
                    "Authenticated".to_owned(),
                ],
                required_transition_methods: vec![
                    "connect(".to_owned(),
                    "authenticate(".to_owned(),
                    "send(".to_owned(),
                ],
                ..Mechanism::default()
            }],
            current_enforcement_mode: EnforcementMode::Structural,
            target_enforcement_mode: EnforcementMode::Structural,
            ..sample_policy(
                TYPE_STATE_POLICY_ID,
                "idioms",
                "docs/standards/source/idioms.adoc#nominal-and-structural-type-boundaries",
            )
        }
    }

    fn sample_verified_policy() -> PolicyEntry {
        PolicyEntry {
            current_enforcement_mode: EnforcementMode::Verified,
            target_enforcement_mode: EnforcementMode::Verified,
            proof_rails: vec![ProofRail {
                backend: "local-receipt".to_owned(),
                command: "cargo test proof_rail".to_owned(),
                command_argv: vec!["cargo".to_owned(), "test".to_owned(), "proof_rail".to_owned()],
                source_paths: vec!["standards/check-fixtures/verification/*/src/lib.rs".to_owned()],
                proof_paths: vec!["standards/check-fixtures/verification/*/proof-receipt.txt".to_owned()],
                pass_signal: "proof-result: pass deterministic-effect-handlers".to_owned(),
                trust_boundaries: vec!["fixture receipt only".to_owned()],
                receipt_binding: "none".to_owned(),
                receipt_provenance: ReceiptProvenance::default(),
                receipt_verifier: ReceiptVerifier::default(),
                ..ProofRail::default()
            }],
            ..sample_policy(
                VERIFIED_POLICY_ID,
                "verification",
                "docs/standards/source/verification.adoc#deterministic-effect-handler-tests",
            )
        }
    }

    fn sample_verus_registry() -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![sample_verus_policy()],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_verus_sources(status: &str) -> Vec<SourceFile> {
        let registry = sample_verus_registry();
        let rail = &registry.policies[0].proof_rails[0];
        let sources = vec![SourceFile::new(
            "src/verified/addition.rs",
            "pub fn checked_add_one(value: u64) -> u64 { value + 1 }\n",
        )];
        let proof_sources = vec![SourceFile::new(
            "verus/proofs/addition_bound.rs",
            "verus! { proof fn addition_bound() ensures true {} }\n",
        )];
        let receipt = build_verus_proof_receipt(&VerusProofReceiptInput {
            policy_id: "octet.verus_proof_rails.addition-bound",
            rail_index: 0,
            rail,
            sources: &sources,
            proof_sources: &proof_sources,
            checked_scope: "formal-proof:fixture-only:octet.verus_proof_rails.addition-bound#rail-0",
            status,
            command_blake3: "fixture-command-digest",
            proof_artifact_identity: "verus/proofs/addition.receipt",
            stdout_artifact: sample_log_artifact(
                "verus/proofs/addition.stdout",
                "proof-result: pass octet-addition-bound",
            ),
            stderr_artifact: sample_log_artifact("verus/proofs/addition.stderr", ""),
        })
        .expect("fixture receipt");
        let receipt_text = serde_json::to_string_pretty(&receipt).expect("receipt json");
        sources
            .into_iter()
            .chain(proof_sources)
            .chain([SourceFile::new("verus/proofs/addition.receipt", receipt_text)])
            .collect()
    }

    fn sample_log_artifact(locator: &str, text: &str) -> VerusLogArtifact {
        VerusLogArtifact {
            locator: locator.to_owned(),
            blake3: blake3::hash(text.as_bytes()).to_hex().to_string(),
            bytes: text.len(),
            truncated: false,
        }
    }

    fn sample_verus_policy() -> PolicyEntry {
        PolicyEntry {
            current_enforcement_mode: EnforcementMode::Verified,
            target_enforcement_mode: EnforcementMode::Verified,
            proof_rails: vec![ProofRail {
                backend: "verus".to_owned(),
                command: "octet-verus verus/proofs/addition_bound.rs".to_owned(),
                command_argv: vec!["octet-verus".to_owned(), "verus/proofs/addition_bound.rs".to_owned()],
                source_paths: vec!["src/verified/addition.rs".to_owned()],
                proof_paths: vec!["verus/proofs/addition_bound.rs".to_owned()],
                receipt_paths: vec!["verus/proofs/addition.receipt".to_owned()],
                pass_signal: "proof-result: pass octet-addition-bound".to_owned(),
                trust_boundaries: vec!["fixture-scoped Verus proof rail".to_owned()],
                receipt_binding: "blake3-command-and-source".to_owned(),
                receipt_provenance: ReceiptProvenance {
                    schema_version: crate::proof::VERUS_PROOF_RECEIPT_SCHEMA_VERSION.to_owned(),
                    runner: "octet-standards".to_owned(),
                    environment: "cargo-octet-prove".to_owned(),
                    requires_backend: true,
                    requires_trust_boundary: true,
                },
                receipt_verifier: ReceiptVerifier {
                    verifier_id: "octet-verus@fixture-v1".to_owned(),
                    verifier_digest: "fixture-digest".to_owned(),
                    verifier_binary_path: "octet-verus".to_owned(),
                    verifier_binary_digest: "fixture-binary-digest".to_owned(),
                    verifier_nix_closure: "fixture-closure".to_owned(),
                    verifier_nix_closure_digest: "fixture-closure-digest".to_owned(),
                    toolchain_id: "verus-fixture-wrapper/v1".to_owned(),
                    toolchain_digest: "fixture-toolchain-digest".to_owned(),
                    ..ReceiptVerifier::default()
                },
                verifier_mode: crate::proof::VERIFIER_MODE_FIXTURE.to_owned(),
                checked_scope: "formal-proof:fixture-only:octet.verus_proof_rails.addition-bound#rail-0".to_owned(),
                timeout_ms: 1_000,
                stdout_max_bytes: 1_000,
                stderr_max_bytes: 1_000,
                environment_allowlist: vec!["PATH".to_owned()],
            }],
            ..sample_policy(
                "octet.verus_proof_rails.addition-bound",
                "verification",
                "docs/standards/source/verification.adoc#verus-proof-rails",
            )
        }
    }

    fn sample_policy(policy_id: &str, pillar: &str, source_anchor: &str) -> PolicyEntry {
        PolicyEntry {
            policy_id: policy_id.to_owned(),
            pillar: pillar.to_owned(),
            source_path: "docs/standards/source/devtools.adoc".to_owned(),
            source_heading: "Enforcing, not advising".to_owned(),
            source_anchor: source_anchor.to_owned(),
            summary: "sample policy".to_owned(),
            classification: Classification::Hard,
            enforcement_level: EnforcementLevel::Hard,
            current_enforcement_mode: EnforcementMode::Gate,
            target_enforcement_mode: EnforcementMode::Gate,
            escalation_criteria: "already gated".to_owned(),
            mechanisms: Vec::new(),
            deterministic_checker: EvidenceCommand::default(),
            proof_rails: Vec::new(),
            checker_owner: "test".to_owned(),
            checker_id: "octet-standards".to_owned(),
            default_severity: Severity::Error,
            suppression_requirements: hard_suppression_requirements(),
            fixture_coverage: FixtureCoverage {
                status: FixtureStatus::Planned,
                compliant: Vec::new(),
                noncompliant: Vec::new(),
            },
            verification_evidence_requirements: vec!["positive fixture".to_owned(), "negative fixture".to_owned()],
            classification_rationale: "deterministic test policy".to_owned(),
            false_positive_risk: "low".to_owned(),
            existing_enforcement: Vec::new(),
            custom_checker_justification: "test".to_owned(),
        }
    }

    fn advisory_suppression_requirements() -> SuppressionRequirements {
        SuppressionRequirements {
            requires_policy_id: true,
            requires_source_section: true,
            requires_owner: false,
            requires_rationale: true,
            requires_review_trigger: false,
            requires_safer_default: false,
        }
    }

    fn hard_suppression_requirements() -> SuppressionRequirements {
        SuppressionRequirements {
            requires_policy_id: true,
            requires_source_section: true,
            requires_owner: true,
            requires_rationale: true,
            requires_review_trigger: true,
            requires_safer_default: true,
        }
    }

    fn valid_suppression_line(policy_id: &str) -> String {
        format!(
            "// tigerstyle-allow: policy={policy_id}; source={DEVTOOLS_SOURCE}; {OWNER}; {RATIONALE}; {REVIEW}; {SAFER_DEFAULT}"
        )
    }

    fn covered_fixture_manifest() -> FixtureManifest {
        FixtureManifest {
            policy_coverage: vec![
                PolicyFixtureCoverage {
                    policy_id: DEVTOOLS_POLICY_ID.to_owned(),
                    compliant: vec!["devtools/ok.rs".to_owned()],
                    noncompliant: vec!["devtools/bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: EMBEDDED_POLICY_ID.to_owned(),
                    compliant: vec!["embedded/ok.rs".to_owned()],
                    noncompliant: vec!["embedded/bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: FCIS_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/ok.rs".to_owned()],
                    noncompliant: vec!["architecture/bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/domain-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/domain-bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: ROOT_THINNESS_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/root-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/root-bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: EFFECT_CAPABILITY_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/capability-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/capability-bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: TYPE_STATE_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/typestate-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/typestate-bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: VERIFIED_POLICY_ID.to_owned(),
                    compliant: vec!["verification/proof-ok.txt".to_owned()],
                    noncompliant: vec!["verification/proof-bad.txt".to_owned()],
                },
            ],
        }
    }
}
