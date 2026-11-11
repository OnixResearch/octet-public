use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use octet_architecture_ir::CompilerArchitectureCoverage;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::CoverageStatus;
use octet_architecture_ir::ItemKind;
use octet_standards::project_architecture_policy::parse_project_architecture_policy;

use super::SelectedProjectArchitecturePolicy;

mod body_owners;
mod core_scope_symlink;
mod forged_question;
mod freshness;
mod lint_freshness;
mod randomness;

const TOOLCHAIN: &str = "nightly-2026-03-21";
const FIXTURE_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "src/lib.rs",
    "src/body_owners.rs",
    "src/randomness.rs",
    "randomness.Cargo.toml",
    "randomness.Cargo.lock",
    "src/main.rs",
    "tests/integration.rs",
    "examples/sample.rs",
    "benches/sample.rs",
    "macro/Cargo.toml",
    "macro/src/lib.rs",
];
const SOURCE_SCOPES: &[&str] = &["src", "tests", "examples", "benches", "build.rs", "macro/src"];
const PACKAGES: &[&str] = &["architecture-target-fixture", "architecture-target-macro"];

fn policy(root: &Path, host: &str, require_no_std: bool) -> SelectedProjectArchitecturePolicy {
    policy_with_mode(
        root,
        host,
        require_no_std,
        octet_standards::project_architecture_policy::ArchitectureMode::Inventory,
    )
}

fn policy_with_mode(
    root: &Path,
    host: &str,
    require_no_std: bool,
    mode: octet_standards::project_architecture_policy::ArchitectureMode,
) -> SelectedProjectArchitecturePolicy {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../../../standards/project-architecture/octet-self-architecture.json"))
            .expect("template policy");
    value["project_id"] = "cargo-target-fixture".into();
    value["mode"] = serde_json::to_value(mode).expect("fixture mode");
    let mut source_scopes = SOURCE_SCOPES.to_vec();
    if require_no_std {
        source_scopes.retain(|scope| *scope != "src");
        source_scopes.extend(["src/lib.rs", "src/main.rs"]);
    }
    let production_sources = source_scopes.iter().copied().filter(|scope| *scope != "tests").collect::<Vec<_>>();
    value["production"] = serde_json::json!({
        "packages": PACKAGES, "source_scopes": production_sources,
        "targets": [{"triple": host, "features": ["extra"], "default_features": true}],
    });
    let mut roles = Vec::new();
    for package in PACKAGES {
        roles.push(serde_json::json!({"scope_kind":"package","scope":package,"role":"domain-core"}));
    }
    for scope in source_scopes {
        let role = if scope == "tests" { "test" } else { "domain-core" };
        roles.push(serde_json::json!({"scope_kind":"source","scope":scope,"role":role}));
    }
    value["roles"] = roles.into();
    value["dependency_rules"] = serde_json::json!([{
        "caller_role":"domain-core", "allowed_callee_roles":["domain-core"], "edge_kinds":["normal","build"],
    }]);
    for field in [
        "provider_classifications",
        "ports",
        "capability_classifications",
        "outbound_effects",
        "core_no_std",
        "waivers",
    ] {
        value[field] = serde_json::json!([]);
    }
    value["capability_classifications"] = serde_json::json!([{
        "family": "filesystem", "core_handling": "forbidden", "executor_role": "outbound-adapter",
        "rationale": "fixture filesystem effects must remain visible and forbidden in the core",
    }, {
        "family": "randomness", "core_handling": "forbidden", "executor_role": "outbound-adapter",
        "rationale": "fixture randomness effects must remain visible and forbidden in the core",
    }]);
    if require_no_std {
        value["core_no_std"] = serde_json::json!([{"scope":"src/lib.rs"}]);
    }
    value["admission"] = serde_json::Value::Null;
    let bytes = serde_json::to_vec(&value).expect("serialize fixture policy");
    let policy = parse_project_architecture_policy(&bytes).expect("validate typed fixture policy");
    let identity = blake3::hash(&bytes).to_hex().to_string();
    SelectedProjectArchitecturePolicy {
        source_path: root.join("fixture-policy.ncl"),
        export_path: root.join("fixture-policy.json"),
        manifest_path: root.join("fixture-policy.manifest.json"),
        source_blake3: identity.clone(),
        policy_blake3: identity,
        waiver_policy_blake3: octet_standards::project_architecture_policy::architecture_waiver_policy_blake3(&policy),
        policy,
    }
}

fn required_artifact(name: &str) -> PathBuf {
    let path = PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name} for the compiler control")));
    assert!(path.is_absolute() && path.is_file(), "missing compiler control artifact");
    path
}

#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_has_complete_compiler_coverage() {
    run_compiler_fixture(false, TestImport::None);
}

// r[verify octet.architecture.features.equivalence]
// r[verify octet.architecture.features.rejection]
// r[verify octet.architecture.features.evidence]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_has_complete_compiler_coverage_with_all_features() {
    run_compiler_fixture(true, TestImport::None);
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_rejects_production_import_of_test_source() {
    run_compiler_fixture(true, TestImport::ParentRelative);
}

// r[verify octet.project_architecture_gate.test_ownership]
#[cfg(unix)]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_rejects_symlink_alias_before_collection() {
    run_compiler_fixture(true, TestImport::Symlink);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TestImport {
    None,
    ParentRelative,
    Symlink,
}

fn run_compiler_fixture(all_features: bool, import_test_source: TestImport) {
    run_configured_compiler_fixture(all_features, import_test_source, NoStdMode::None);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NoStdMode {
    None,
    Always,
    ActiveCfgAttr,
    InactiveCfgAttr,
    NotTest,
    IntegrationOnly,
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_binds_no_std_to_library_units() {
    run_configured_compiler_fixture(true, TestImport::None, NoStdMode::Always);
}

#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_binds_active_no_std_cfg_attr() {
    run_configured_compiler_fixture(true, TestImport::None, NoStdMode::ActiveCfgAttr);
}

#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_rejects_inactive_no_std_cfg_attr() {
    run_configured_compiler_fixture(true, TestImport::None, NoStdMode::InactiveCfgAttr);
}

#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_rejects_no_std_disabled_in_library_test() {
    run_configured_compiler_fixture(true, TestImport::None, NoStdMode::NotTest);
}

#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_rejects_integration_only_no_std() {
    run_configured_compiler_fixture(true, TestImport::None, NoStdMode::IntegrationOnly);
}

fn run_configured_compiler_fixture(all_features: bool, import_test_source: TestImport, no_std: NoStdMode) {
    run_body_fixture(all_features, import_test_source, no_std, BodyFixture::Ordinary);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyFixture {
    Ordinary,
    NonFunctionOwners,
    Randomness,
}

// One real-Cargo fixture run asserts every body-fixture variant against the same collected IR.
#[allow(clippy::too_many_lines)]
fn run_body_fixture(all_features: bool, import_test_source: TestImport, no_std: NoStdMode, fixture: BodyFixture) {
    let include_body_owners = fixture == BodyFixture::NonFunctionOwners;
    let include_randomness = fixture == BodyFixture::Randomness;
    let library = required_artifact("OCTET_UI_LIBRARY");
    let driver = required_artifact("OCTET_UI_DRIVER");
    let owner = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("owner root");
    let source = owner.join("fixtures/architecture-targets");
    let temporary = tempfile::TempDir::new().expect("fixture workspace");
    let root = temporary.path();
    for relative in FIXTURE_FILES {
        let target = root.join(relative);
        fs::create_dir_all(target.parent().expect("fixture parent")).expect("create fixture parent");
        fs::copy(source.join(relative), target).expect("copy controlled fixture source");
    }
    if include_body_owners {
        let path = root.join("src/lib.rs");
        let source = fs::read_to_string(&path).expect("library fixture root");
        fs::write(path, format!("{source}\npub mod body_owners;\n")).expect("body-owner fixture module");
    }
    if include_randomness {
        fs::copy(root.join("randomness.Cargo.toml"), root.join("Cargo.toml")).expect("registry fixture manifest");
        fs::copy(root.join("randomness.Cargo.lock"), root.join("Cargo.lock")).expect("registry fixture lock");
        let path = root.join("src/lib.rs");
        let source = fs::read_to_string(&path).expect("library fixture root");
        fs::write(path, format!("{source}\npub mod randomness;\n")).expect("randomness fixture module");
    }
    if import_test_source != TestImport::None {
        let support = root.join("tests/support");
        fs::create_dir_all(&support).expect("test support directory");
        fs::write(support.join("production_import.rs"), "pub fn imported_value() -> u32 { 7 }\n")
            .expect("test-labelled source");
        let imported_path = if import_test_source == TestImport::Symlink {
            #[cfg(unix)]
            std::os::unix::fs::symlink("../tests/support/production_import.rs", root.join("src/import_alias.rs"))
                .expect("test source alias");
            "import_alias.rs"
        } else {
            "../tests/support/production_import.rs"
        };
        for entry in ["src/lib.rs", "src/main.rs"] {
            let path = root.join(entry);
            let source = fs::read_to_string(&path).expect("production root");
            fs::write(&path, format!("{source}\n#[path = \"{imported_path}\"]\npub mod production_import;\n"))
                .expect("production import fixture");
        }
    }
    if no_std != NoStdMode::None {
        let (entry, attribute) = match no_std {
            NoStdMode::Always => ("src/lib.rs", "#![no_std]"),
            NoStdMode::ActiveCfgAttr => ("src/lib.rs", "#![cfg_attr(feature = \"extra\", no_std)]"),
            NoStdMode::InactiveCfgAttr => ("src/lib.rs", "#![cfg_attr(not(feature = \"extra\"), no_std)]"),
            NoStdMode::NotTest => ("src/lib.rs", "#![cfg_attr(not(test), no_std)]"),
            NoStdMode::IntegrationOnly => ("tests/integration.rs", "#![no_std]"),
            NoStdMode::None => unreachable!("unmodified fixture handled outside this branch"),
        };
        let path = root.join(entry);
        let source = fs::read_to_string(&path).expect("configuration fixture source");
        // This tests the active crate attribute, not transitive freedom from std.
        // Explicit std keeps the original forbidden filesystem-effect controls intact.
        fs::write(&path, format!("{attribute}\nextern crate std;\n{source}")).expect("compiler configuration fixture");
    }
    let (_, host) = super::toolchain_identity(TOOLCHAIN).expect("pinned host");
    let policy = if include_body_owners || include_randomness {
        policy_with_mode(
            root,
            &host,
            no_std != NoStdMode::None,
            octet_standards::project_architecture_policy::ArchitectureMode::Gate,
        )
    } else {
        policy(root, &host, no_std != NoStdMode::None)
    };
    let scope = vec!["--workspace".to_owned()];
    let args = if all_features {
        vec!["--all-targets".to_owned(), "--all-features".to_owned()]
    } else {
        vec!["--all-targets".to_owned(), "--features".to_owned(), "extra".to_owned()]
    };
    if import_test_source == TestImport::Symlink {
        let rejected = super::prepare_collector(root, &policy, TOOLCHAIN, &library, &scope, &args);
        let error = rejected.err().expect("source symlink must reject before collection");
        assert!(error.contains("source envelope contains a source symlink"), "{error}");
        assert!(!root.join(".octet").exists(), "source alias must not publish a collector context");
        eprintln!("symlink alias rejected by the source envelope before compiler invocation");
        return;
    }
    if all_features {
        let mut incomplete = self::policy(root, &host, false);
        incomplete.policy.production.targets[0].features.clear();
        let rejected = super::prepare_collector(root, &incomplete, TOOLCHAIN, &library, &scope, &args);
        assert!(rejected.err().expect("under-declared row must fail").contains("does not match"));
        assert!(!root.join(".octet").exists(), "mismatch must not publish a collector context");
    }
    let mut conflicting = self::policy(root, &host, false);
    conflicting
        .policy
        .roles
        .iter_mut()
        .find(|role| role.scope == "src")
        .expect("production source scope")
        .role = octet_standards::project_architecture_policy::ArchitectureRole::Test;
    conflicting.policy.production.source_scopes.retain(|scope| scope != "src");
    parse_project_architecture_policy(&serde_json::to_vec(&conflicting.policy).expect("negative policy"))
        .expect("negative policy is structurally valid before Cargo root binding");
    let rejected = super::prepare_collector(root, &conflicting, TOOLCHAIN, &library, &scope, &args);
    assert!(
        rejected
            .err()
            .expect("production roots cannot become tests")
            .contains("conflicts with the declared compiler role")
    );
    assert!(!root.join(".octet").exists(), "invalid roots must not publish a collector context");
    let prepared = super::prepare_collector(root, &policy, TOOLCHAIN, &library, &scope, &args)
        .expect("plan actual Cargo compilation units");
    for kind in ["lib", "bin", "test", "example", "bench", "proc-macro", "custom-build"] {
        assert!(
            prepared
                .expected_units
                .iter()
                .any(|expected| expected.unit.target_kind.split('+').next() == Some(kind)),
            "missing {kind} unit"
        );
    }
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["check", "--locked", "--workspace"])
        .args(&args)
        .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env("RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("DYLINT_NO_DEPS", "1")
        .env("DYLINT_LIBS", serde_json::to_string(&[library]).expect("library selection"))
        .env(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV, &prepared.context_path)
        .output()
        .expect("compile fixture through real collector");
    assert!(output.status.success(), "compiler failed: {}", String::from_utf8_lossy(&output.stderr));
    let artifacts = super::normalize_collected_shards(&prepared).expect("normalize actual shards");
    assert!(artifacts.incomplete_error.is_none(), "{:?}", artifacts.incomplete_error);
    let coverage: CompilerArchitectureCoverage = serde_json::from_str(&artifacts.coverage).expect("coverage");
    assert_eq!(coverage.status, CoverageStatus::Complete);
    assert_eq!(coverage.expected_unit_ids.len(), prepared.expected_units.len());
    let ir: CompilerArchitectureIr = serde_json::from_str(&artifacts.architecture_ir).expect("IR");
    assert_selected_root_roles(&ir);
    assert_no_std_memberships(&ir, no_std);
    if include_body_owners {
        body_owners::assert_body_owner_facts(&ir);
    }
    assert!(octet_architecture_ir::verify_ir_unit_bindings(&ir).is_empty());
    assert!(octet_architecture_ir::verify_ir_unit_memberships(&ir).is_empty());
    for expected in &prepared.expected_units {
        assert!(ir.units.contains(&expected.unit), "compiler unit must retain its exact planned root and role");
    }
    assert!(
        ir.facts
            .iter()
            .any(|fact| matches!(fact, octet_architecture_ir::CompilerArchitectureFact::Effect { .. }))
    );
    assert_associated_item_classification(&ir);
    let compiler_receipt = serde_json::from_str(&artifacts.receipt).expect("compiler receipt");
    let command_id = blake3::hash(b"real Cargo fixture architecture evaluation").to_hex().to_string();
    let architecture = octet_standards::project_architecture_gate::evaluate_project_architecture(
        &octet_standards::project_architecture_gate::ProjectArchitectureGateInput {
            policy: &policy.policy,
            policy_source_blake3: &policy.source_blake3,
            policy_blake3: &policy.policy_blake3,
            waiver_policy_blake3: &policy.waiver_policy_blake3,
            // This compiler fixture does not run the consumer deny-all hook.
            lint_phase_status: "warning-only",
            command_blake3: &command_id,
            cargo_graph: &prepared.cargo_graph,
            architecture_ir: &ir,
            coverage: &coverage,
            compiler_receipt: &compiler_receipt,
            effect_closure: None,
        },
    );
    if include_body_owners {
        body_owners::assert_nested_effects(&ir, &architecture);
    }
    if include_randomness {
        randomness::assert_randomness_facts(&ir, &architecture);
    }
    if no_std != NoStdMode::None {
        let missing = architecture.findings.iter().any(|finding| finding.code == "no-std-obligation");
        let satisfied = matches!(no_std, NoStdMode::Always | NoStdMode::ActiveCfgAttr);
        assert_eq!(missing, !satisfied, "{no_std:?}: {:?}", architecture.findings);
        eprintln!("no-std obligation: mode={no_std:?} satisfied={satisfied}");
    }
    if import_test_source == TestImport::None {
        let disposition = architecture.test_only.as_ref().expect("valid compiler-owned production partition");
        assert!(!disposition.unit_ids.is_empty());
        let library_test = ir.units.iter().find(|unit| unit.target_kind == "lib+test").expect("library-test unit");
        assert!(
            architecture.findings.iter().any(|finding| finding.code == "domain-core-effect"
                && finding.unit_id.as_ref() == Some(&library_test.unit_id)),
            "library-test effects remain production findings: {:?}",
            architecture.findings
        );
        assert!(!architecture.findings.iter().any(|finding| finding.code == "source-role-unit-mismatch"));
        eprintln!(
            "production view retains library-test effects; independent test units={}",
            disposition.unit_ids.len()
        );
    } else {
        assert_eq!(
            architecture.status,
            octet_standards::project_architecture_gate::ArchitecturePhaseStatus::IntegrationFailure
        );
        for kind in ["lib", "lib+test", "bin", "bin+test"] {
            let unit = ir.units.iter().find(|unit| unit.target_kind == kind).expect("production observation");
            assert!(
                architecture.findings.iter().any(|finding| finding.code == "source-role-unit-mismatch"
                    && finding.message.contains("tests/support/production_import.rs")
                    && finding.message.contains(&unit.unit_id)),
                "{kind}: {:?}",
                architecture.findings
            );
        }
        eprintln!("production imports rejected in lib, lib+test, bin, and bin+test");
    }
    assert!(octet_standards::project_architecture_gate::verify_project_architecture_receipt(&architecture).is_empty());
    assert!(
        ir.unknowns
            .iter()
            .all(|unknown| unknown.kind != octet_architecture_ir::UnknownFactKind::UnsupportedExpansion),
        "fixture expansions must retain compiler-resolved provenance: {:?}",
        ir.unknowns
    );
    eprintln!(
        "actual Cargo target coverage: {} units, all_features={all_features}",
        coverage.observed_unit_ids.len()
    );
    let shard = fs::read_dir(&prepared.shard_directory)
        .expect("shards")
        .next()
        .expect("one shard")
        .expect("entry")
        .path();
    fs::remove_file(shard).expect("remove one controlled observation");
    let omitted = super::normalize_collected_shards(&prepared).expect("normalize omitted unit");
    assert!(omitted.incomplete_error.is_some(), "missing compiler unit must fail closed");
}

// r[verify octet.project_architecture_gate.test_ownership]
fn assert_selected_root_roles(ir: &CompilerArchitectureIr) {
    use octet_architecture_ir::CompilerArchitectureRole::DomainCore;
    use octet_architecture_ir::CompilerArchitectureRole::Test;
    for (kind, path, role) in [
        ("lib", "src/lib.rs", DomainCore),
        ("lib+test", "src/lib.rs", DomainCore),
        ("bin", "src/main.rs", DomainCore),
        ("bin+test", "src/main.rs", DomainCore),
        ("test+test", "tests/integration.rs", Test),
    ] {
        let unit = ir
            .units
            .iter()
            .find(|unit| unit.target_kind == kind && unit.source_path == path)
            .expect("selected target");
        assert_eq!(unit.role, role, "wrong role for {kind} root {path}");
        if kind.starts_with("bin") {
            assert_eq!(unit.target_name, "test", "test-named binary must remain production");
        }
        assert!(ir.unit_memberships.iter().any(|row| row.unit_id == unit.unit_id && !row.fact_ids.is_empty()));
        eprintln!("compiler-bound role: kind={kind} root={path} role={role:?}");
    }
    let library_test_items = ir
        .facts
        .iter()
        .filter(|fact| {
            matches!(fact,
                CompilerArchitectureFact::Item { qualified_path, .. } if qualified_path.ends_with("::test_effect")
            )
        })
        .collect::<Vec<_>>();
    assert!(!library_test_items.is_empty(), "library-test effect must remain visible");
    for fact in library_test_items {
        for row in ir.unit_memberships.iter().filter(|row| row.fact_ids.iter().any(|id| id == fact.fact_id())) {
            let unit = ir.units.iter().find(|unit| unit.unit_id == row.unit_id).expect("observing unit");
            assert_eq!(unit.role, DomainCore, "cfg(test) alone cannot remove production obligations");
        }
    }
}

fn assert_no_std_memberships(ir: &CompilerArchitectureIr, mode: NoStdMode) {
    for unit in &ir.units {
        let expected = match mode {
            NoStdMode::None | NoStdMode::InactiveCfgAttr => false,
            NoStdMode::Always | NoStdMode::ActiveCfgAttr => matches!(unit.target_kind.as_str(), "lib" | "lib+test"),
            NoStdMode::NotTest => unit.target_kind == "lib",
            NoStdMode::IntegrationOnly => unit.target_kind == "test+test",
        };
        let row = ir.unit_memberships.iter().find(|row| row.unit_id == unit.unit_id).expect("unit membership");
        let observed = ir.facts.iter().any(|fact| {
            matches!(fact, CompilerArchitectureFact::Configuration { predicate, active: true, .. }
                if predicate == &format!("crate:no-std={}", unit.package_name))
                && row.fact_ids.iter().any(|id| id == fact.fact_id())
        });
        assert_eq!(observed, expected, "active no_std membership: mode={mode:?} kind={}", unit.target_kind);
    }
    eprintln!("active no_std memberships verified for all {} units: {mode:?}", ir.units.len());
}

fn assert_associated_item_classification(ir: &CompilerArchitectureIr) {
    for (suffix, expected_kind) in [
        ("::Value", ItemKind::TypeAlias),
        ("::PRESENT", ItemKind::Constant),
        ("::Associated::identity", ItemKind::Function),
    ] {
        let mut observed = false;
        for fact in &ir.facts {
            let CompilerArchitectureFact::Item {
                qualified_path,
                item_kind,
                signature_id,
                ..
            } = fact
            else {
                continue;
            };
            if !qualified_path.ends_with(suffix) {
                continue;
            }
            observed = true;
            assert_eq!(*item_kind, expected_kind, "wrong classification for {qualified_path}");
            assert_eq!(signature_id.is_some(), expected_kind == ItemKind::Function);
        }
        assert!(observed, "missing associated item {suffix}");
    }
}
