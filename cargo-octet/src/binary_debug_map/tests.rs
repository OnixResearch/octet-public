//! Focused positive and negative fixtures for the binary debug-map rail.

use std::path::Path;
use std::path::PathBuf;

use gimli::LineEncoding;
use gimli::write::Address;
use gimli::write::AttributeValue;
use gimli::write::Dwarf;
use gimli::write::EndianVec;
use gimli::write::LineProgram;
use gimli::write::LineString;
use gimli::write::Sections;
use gimli::write::Unit;
use object::write::Object;
use object::write::Symbol;
use object::write::SymbolSection;

use super::BinaryDebugMapStatus;
use super::Profile;
use super::REQUIRED_NON_CLAIM;
use super::RequirementRefSet;
use super::core;
use super::evaluate_envelope;
use super::identity;
use super::linkage::FunctionAddressArtifact;
use super::linkage::parse_function_address_artifact;
use super::parse_requirement_refs;
use super::profile_for_tests;
use super::validate_handoff_role;
use super::validate_receipt_details;
use super::validate_registry;

const COMP_DIR: &str = "/build/octet-binary-debug-map";
const SOURCE_FILE: &str = "src/lib.rs";
const REPOSITORY_PATH: &str = "crates/binary-debug-map-fixture/src/lib.rs";
const BUILD_ID: &[u8] = &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];

#[derive(Clone, Copy)]
struct FixtureFunction {
    name: &'static str,
    linkage: Option<&'static str>,
    begin: u64,
    end: u64,
    line: u64,
}

fn functions() -> Vec<FixtureFunction> {
    vec![
        FixtureFunction {
            name: "alpha",
            linkage: Some("_ZN5fixture5alpha17h0123456789abcdefE"),
            begin: 0x1000,
            end: 0x1100,
            line: 10,
        },
        FixtureFunction {
            name: "beta",
            linkage: None,
            begin: 0x1200,
            end: 0x1300,
            line: 40,
        },
    ]
}

fn build_id_note(build_id: &[u8]) -> Vec<u8> {
    let mut note = Vec::new();
    note.extend_from_slice(&4_u32.to_le_bytes());
    note.extend_from_slice(&u32::try_from(build_id.len()).unwrap_or(0).to_le_bytes());
    note.extend_from_slice(&3_u32.to_le_bytes());
    note.extend_from_slice(b"GNU\0");
    note.extend_from_slice(build_id);
    while note.len() % 4 != 0 {
        note.push(0);
    }
    note
}

fn build_fixture(
    comp_dir: &str,
    build_id: Option<&[u8]>,
    functions: &[FixtureFunction],
    unit_count: usize,
    with_debug: bool,
    with_text: bool,
) -> Vec<u8> {
    let mut object = Object::new(object::BinaryFormat::Elf, object::Architecture::X86_64, object::Endianness::Little);
    let text = if with_text {
        let text = object.add_section(Vec::new(), b".text".to_vec(), object::SectionKind::Text);
        object.append_section_data(text, &vec![0_u8; 0x2000], 16);
        Some(text)
    } else {
        None
    };
    if let Some(build_id) = build_id {
        let section = object.add_section(Vec::new(), b".note.gnu.build-id".to_vec(), object::SectionKind::Note);
        object.append_section_data(section, &build_id_note(build_id), 4);
    }
    if let Some(text) = text {
        for function in functions {
            object.add_symbol(Symbol {
                name: function.linkage.unwrap_or(function.name).as_bytes().to_vec(),
                value: function.begin,
                size: function.end.saturating_sub(function.begin),
                kind: object::SymbolKind::Text,
                scope: object::SymbolScope::Linkage,
                weak: false,
                section: SymbolSection::Section(text),
                flags: object::SymbolFlags::None,
            });
        }
    }
    if with_debug {
        add_dwarf_sections(&mut object, comp_dir, functions, unit_count);
    }
    object.write().unwrap_or_else(|error| panic!("write ELF fixture: {error}"))
}

fn add_dwarf_sections(object: &mut Object<'_>, comp_dir: &str, functions: &[FixtureFunction], unit_count: usize) {
    let encoding = gimli::Encoding {
        format: gimli::Format::Dwarf32,
        version: 4,
        address_size: 8,
    };
    let mut dwarf = Dwarf::new();
    for _ in 0..unit_count {
        let mut program = LineProgram::new(
            encoding,
            LineEncoding::default(),
            LineString::String(comp_dir.as_bytes().to_vec()),
            None,
            LineString::String(SOURCE_FILE.as_bytes().to_vec()),
            None,
        );
        let directory = program.add_directory(LineString::String(comp_dir.as_bytes().to_vec()));
        let file = program.add_file(LineString::String(SOURCE_FILE.as_bytes().to_vec()), directory, None);
        for function in functions {
            program.begin_sequence(Some(Address::Constant(function.begin)));
            program.row().file = file;
            program.row().line = function.line;
            program.row().column = 1;
            program.row().address_offset = 0;
            program.generate_row();
            program.row().address_offset = function.end.saturating_sub(function.begin).saturating_sub(1);
            program.row().line = function.line + 1;
            program.generate_row();
            program.end_sequence(function.end.saturating_sub(function.begin));
        }
        let unit_id = dwarf.units.add(Unit::new(encoding, program));
        let unit = dwarf.units.get_mut(unit_id);
        let root = unit.root();
        unit.get_mut(root).set(gimli::DW_AT_name, AttributeValue::String(SOURCE_FILE.as_bytes().to_vec()));
        unit.get_mut(root).set(gimli::DW_AT_comp_dir, AttributeValue::String(comp_dir.as_bytes().to_vec()));
        for function in functions {
            let child = unit.add(root, gimli::DW_TAG_subprogram);
            let entry = unit.get_mut(child);
            entry.set(gimli::DW_AT_name, AttributeValue::String(function.name.as_bytes().to_vec()));
            if let Some(linkage) = function.linkage {
                entry.set(gimli::DW_AT_linkage_name, AttributeValue::String(linkage.as_bytes().to_vec()));
            }
            entry.set(gimli::DW_AT_decl_file, AttributeValue::FileIndex(Some(file)));
            entry.set(gimli::DW_AT_decl_line, AttributeValue::Udata(function.line));
            entry.set(gimli::DW_AT_low_pc, AttributeValue::Address(Address::Constant(function.begin)));
            entry.set(gimli::DW_AT_high_pc, AttributeValue::Udata(function.end.saturating_sub(function.begin)));
        }
    }
    let mut sections = Sections::new(EndianVec::new(gimli::LittleEndian));
    dwarf.write(&mut sections).unwrap_or_else(|error| panic!("write DWARF fixture: {error}"));
    sections
        .for_each(|id, data| -> Result<(), ()> {
            let section = object.add_section(Vec::new(), id.name().as_bytes().to_vec(), object::SectionKind::Debug);
            object.append_section_data(section, data.slice(), 1);
            Ok(())
        })
        .unwrap_or_else(|()| panic!("collect DWARF sections"));
}

fn function_address_artifact(records: &[(&str, u64, &str)]) -> FunctionAddressArtifact {
    let rendered = serde_json::json!({
        "records": records.iter().map(|(name, line, address)| serde_json::json!({
            "path": REPOSITORY_PATH,
            "name": name,
            "line": line,
            "column": 1,
            "start_byte": 0,
            "end_byte": 128,
            "function_address": address,
            "source_identity": format!("src:{name}"),
        })).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec(&rendered).unwrap_or_default();
    parse_function_address_artifact(&bytes).unwrap_or_else(|error| panic!("function addresses: {error}"))
}

fn requirement_refs() -> RequirementRefSet {
    let rendered = serde_json::json!({
        "schema_version": super::REQUIREMENT_REFS_SCHEMA,
        "repository": "github:OnixResearch/octet",
        "revision": "0123456789abcdef0123456789abcdef01234567",
        "registry_identity": "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "references": [{
            "requirement_id": "r[octet.evidence_provenance.binary_debug_map_linkage]",
            "function_address": "fa-alpha",
            "legacy_requirement_ids": ["legacy:alpha"],
        }],
    });
    let bytes = serde_json::to_vec(&rendered).unwrap_or_default();
    parse_requirement_refs(std::str::from_utf8(&bytes).unwrap_or(""))
        .unwrap_or_else(|error| panic!("requirement references: {error}"))
}

fn analyze_fixture(profile: &Profile, with_companion: bool) -> super::BinaryDebugMapEnvelope {
    let binary = build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 1, !with_companion, true);
    let companion = with_companion.then(|| build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 1, true, false));
    let addresses = function_address_artifact(&[("alpha", 10, "fa-alpha"), ("beta", 40, "fa-beta")]);
    core::analyze(core::AnalysisInput {
        binary: &binary,
        companion: companion.as_deref(),
        profile,
        registry_identity: identity(b"registry"),
        function_addresses: Some(&addresses),
        requirement_refs: Some(&requirement_refs()),
        requirement_refs_identity: Some(identity(b"requirement-refs")),
    })
}

#[test]
fn checked_in_profile_is_valid_and_binds_every_limit() {
    let profile = profile_for_tests();
    let registry = super::checked_in_registry().expect("checked-in registry parses");
    assert!(validate_registry(&registry).is_empty(), "{:?}", validate_registry(&registry));
    assert_eq!(profile.profile_id, super::PROFILE_ID);
    assert_eq!(profile.verification_role, super::RECORDED_ONLY_ROLE);
    assert!(profile.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM));
    assert!(!profile.source_roots.is_empty());
    let identity = super::profile_identity(&profile).expect("profile identity");
    assert!(super::is_b3_identity(&identity));
    assert_eq!(identity, super::profile_identity(&profile).expect("stable profile identity"));
}

#[test]
fn profile_validation_rejects_missing_limits_unsupported_formats_and_promotion() {
    let registry = super::checked_in_registry().expect("checked-in registry parses");
    let mut profile = profile_for_tests();

    profile.debug_format = "pdb".to_owned();
    let diagnostics = validate_registry(&super::ProfileRegistry {
        schema_version: registry.schema_version.clone(),
        profiles: vec![profile.clone()],
    });
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported debug format")));

    profile = profile_for_tests();
    profile.source_roots.clear();
    let diagnostics = validate_registry(&super::ProfileRegistry {
        schema_version: registry.schema_version.clone(),
        profiles: vec![profile.clone()],
    });
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("source-root mapping")));

    profile = profile_for_tests();
    profile.verification_role = "formal_proof".to_owned();
    let diagnostics = validate_registry(&super::ProfileRegistry {
        schema_version: registry.schema_version.clone(),
        profiles: vec![profile.clone()],
    });
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("verification role")));

    profile = profile_for_tests();
    profile.non_claims.push("this proves behavioral correctness".to_owned());
    let diagnostics = validate_registry(&super::ProfileRegistry {
        schema_version: registry.schema_version.clone(),
        profiles: vec![profile],
    });
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("promoted claim")));

    let missing_limit = super::PROFILE_JSON.replace("\"max_frames\": 4096,", "");
    assert!(super::parse_registry(&missing_limit).is_err());
    let unknown_field = super::PROFILE_JSON
        .replace("\"profile_id\": \"elf-dwarf-v1\",", "\"profile_id\": \"elf-dwarf-v1\", \"ambient_limit\": 1,");
    assert!(super::parse_registry(&unknown_field).is_err());
}

#[test]
fn inline_dwarf_fixture_emits_deterministic_linked_rows() {
    let profile = profile_for_tests();
    let first = analyze_fixture(&profile, false);
    let second = analyze_fixture(&profile, false);
    assert_eq!(
        serde_json::to_string(&first).expect("serialize first"),
        serde_json::to_string(&second).expect("serialize second"),
        "inline receipt must be deterministic"
    );
    let details = &first.details;
    assert_eq!(details.debug_source, super::DEBUG_SOURCE_INLINE);
    assert_eq!(details.rows.len(), 2, "{:?}", details.findings);
    assert!(details.findings.is_empty(), "{:?}", details.findings);
    assert_eq!(details.excluded_rows, 0);
    assert_eq!(details.verification_role, super::RECORDED_ONLY_ROLE);

    let alpha = details.rows.iter().find(|row| row.demangled_name.contains("alpha")).expect("alpha row");
    assert_eq!(alpha.repository_path, REPOSITORY_PATH);
    assert_eq!(alpha.address_start, 0x1000);
    assert_eq!(alpha.address_end, 0x1100);
    assert_eq!(alpha.line_start, 10);
    assert_eq!(alpha.line_end, 11);
    assert_eq!(alpha.function_address.as_deref(), Some("fa-alpha"));
    assert_eq!(alpha.source_identity.as_deref(), Some("src:alpha"));
    let requirement = alpha.requirement.as_ref().expect("typed requirement");
    assert_eq!(requirement.repository, "github:OnixResearch/octet");
    assert_eq!(requirement.registry_identity, "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(requirement.requirement_id, "r[octet.evidence_provenance.binary_debug_map_linkage]");
    assert_eq!(alpha.legacy_requirement_ids, vec!["legacy:alpha".to_owned()]);
    assert!(alpha.path_hash.starts_with("b3:"));
    assert!(alpha.raw_symbol_name_hash.starts_with("b3:"));

    let beta = details.rows.iter().find(|row| row.demangled_name.contains("beta")).expect("beta row");
    assert_eq!(beta.function_address.as_deref(), Some("fa-beta"));
    assert!(beta.requirement.is_none());

    let mut diagnostics = Vec::new();
    validate_receipt_details(details, &first.non_claims, &mut diagnostics);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(super::status_for(Some(details), &[]), BinaryDebugMapStatus::Passed);
}

#[test]
fn split_debug_fixture_binds_binary_and_companion_identities() {
    let profile = profile_for_tests();
    let envelope = analyze_fixture(&profile, true);
    let details = &envelope.details;
    assert_eq!(details.debug_source, super::DEBUG_SOURCE_SPLIT);
    let companion = details.debug_companion.as_ref().expect("companion identity");
    assert_eq!(companion.role, "split-debug");
    assert_ne!(companion.identity, details.binary_identity);
    assert_eq!(companion.build_identity.as_deref(), Some("1122334455667788"));
    assert_eq!(details.rows.len(), 2, "{:?}", details.findings);
    assert!(details.findings.is_empty(), "{:?}", details.findings);
}

#[test]
fn wrong_companion_and_malformed_inputs_fail_closed() {
    let profile = profile_for_tests();
    let binary = build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 1, false, true);
    let wrong = build_fixture(COMP_DIR, Some(&[0x99, 0x88]), &functions(), 1, true, false);
    let envelope = core::analyze(core::AnalysisInput {
        binary: &binary,
        companion: Some(&wrong),
        profile: &profile,
        registry_identity: identity(b"registry"),
        function_addresses: None,
        requirement_refs: None,
        requirement_refs_identity: None,
    });
    assert!(envelope.details.findings.iter().any(|finding| finding.issue_code == "wrong-companion"));
    assert_eq!(super::status_for(Some(&envelope.details), &[]), BinaryDebugMapStatus::Failed);

    let malformed = core::analyze(core::AnalysisInput {
        binary: b"not an elf",
        companion: None,
        profile: &profile,
        registry_identity: identity(b"registry"),
        function_addresses: None,
        requirement_refs: None,
        requirement_refs_identity: None,
    });
    assert!(malformed.details.findings.iter().any(|finding| finding.issue_code == "malformed-binary"));

    let architecture_mismatch = build_aarch64_fixture();
    let envelope = core::analyze(core::AnalysisInput {
        binary: &architecture_mismatch,
        companion: None,
        profile: &profile,
        registry_identity: identity(b"registry"),
        function_addresses: None,
        requirement_refs: None,
        requirement_refs_identity: None,
    });
    assert!(envelope.details.findings.iter().any(|finding| finding.issue_code == "unsupported-format"));
}

fn build_aarch64_fixture() -> Vec<u8> {
    let mut object = Object::new(object::BinaryFormat::Elf, object::Architecture::Aarch64, object::Endianness::Little);
    let text = object.add_section(Vec::new(), b".text".to_vec(), object::SectionKind::Text);
    object.append_section_data(text, &[0_u8; 8], 4);
    object.write().unwrap_or_else(|error| panic!("write AArch64 fixture: {error}"))
}

#[test]
fn unmapped_absolute_paths_do_not_disclose_build_roots() {
    let profile = profile_for_tests();
    let binary = build_fixture("/private/build/secret", Some(BUILD_ID), &functions(), 1, true, true);
    let envelope = core::analyze(core::AnalysisInput {
        binary: &binary,
        companion: None,
        profile: &profile,
        registry_identity: identity(b"registry"),
        function_addresses: None,
        requirement_refs: None,
        requirement_refs_identity: None,
    });
    let unmapped: Vec<_> =
        envelope.details.findings.iter().filter(|finding| finding.issue_code == "unmapped-path").collect();
    assert!(!unmapped.is_empty());
    assert!(envelope.details.rows.is_empty());
    for finding in &envelope.details.findings {
        assert!(!finding.message.contains("/private/build"));
    }
    assert_eq!(super::status_for(Some(&envelope.details), &[]), BinaryDebugMapStatus::Failed);

    let mut excluded_profile = profile_for_tests();
    excluded_profile.unmapped_path_policy = "exclude".to_owned();
    let excluded = core::analyze(core::AnalysisInput {
        binary: &binary,
        companion: None,
        profile: &excluded_profile,
        registry_identity: identity(b"registry"),
        function_addresses: None,
        requirement_refs: None,
        requirement_refs_identity: None,
    });
    assert!(excluded.details.findings.is_empty());
    assert!(excluded.details.rows.is_empty());
    assert!(excluded.details.excluded_rows >= 1);
}

#[test]
fn stale_spans_and_ambiguous_candidates_fail_closed() {
    let profile = profile_for_tests();
    let binary = build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 1, true, true);
    let stale = function_address_artifact(&[("alpha", 99, "fa-alpha")]);
    let envelope = core::analyze(core::AnalysisInput {
        binary: &binary,
        companion: None,
        profile: &profile,
        registry_identity: identity(b"registry"),
        function_addresses: Some(&stale),
        requirement_refs: None,
        requirement_refs_identity: None,
    });
    assert!(envelope.details.findings.iter().any(|finding| finding.issue_code == "stale-span"));
    assert!(
        envelope
            .details
            .rows
            .iter()
            .filter(|row| row.demangled_name.contains("alpha"))
            .all(|row| row.function_address.is_none())
    );

    let ambiguous = function_address_artifact(&[("alpha", 10, "fa-alpha-1"), ("alpha", 11, "fa-alpha-2")]);
    let envelope = core::analyze(core::AnalysisInput {
        binary: &binary,
        companion: None,
        profile: &profile,
        registry_identity: identity(b"registry"),
        function_addresses: Some(&ambiguous),
        requirement_refs: None,
        requirement_refs_identity: None,
    });
    assert!(envelope.details.findings.iter().any(|finding| finding.issue_code == "ambiguous-function"));
    assert!(
        envelope
            .details
            .rows
            .iter()
            .filter(|row| row.demangled_name.contains("alpha"))
            .all(|row| row.function_address.is_none())
    );
    assert_eq!(super::status_for(Some(&envelope.details), &[]), BinaryDebugMapStatus::Failed);
}

#[test]
fn every_resource_limit_class_fails_without_complete_evidence() {
    let cases = [
        ("max_sections", 1_u64),
        ("max_symbols", 1),
        ("max_ranges", 1),
        ("max_line_rows", 1),
        ("max_frames", 1),
        ("max_path_bytes", 1),
        ("max_rows", 1),
    ];
    for (limit, value) in cases {
        let mut profile = profile_for_tests();
        match limit {
            "max_sections" => profile.resource_limits.max_sections = value,
            "max_symbols" => profile.resource_limits.max_symbols = value,
            "max_ranges" => profile.resource_limits.max_ranges = value,
            "max_line_rows" => profile.resource_limits.max_line_rows = value,
            "max_frames" => profile.resource_limits.max_frames = value,
            "max_path_bytes" => profile.resource_limits.max_path_bytes = value,
            "max_rows" => profile.resource_limits.max_rows = value,
            _ => unreachable!(),
        }
        let binary = build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 1, true, true);
        let envelope = core::analyze(core::AnalysisInput {
            binary: &binary,
            companion: None,
            profile: &profile,
            registry_identity: identity(b"registry"),
            function_addresses: None,
            requirement_refs: None,
            requirement_refs_identity: None,
        });
        assert!(
            envelope.details.findings.iter().any(|finding| finding.issue_code == "resource-limit"),
            "{limit} did not fail: {:?}",
            envelope.details.findings
        );
        assert_eq!(envelope.details.truncation.limit.as_deref(), Some(limit));
        assert!(envelope.details.truncation.truncated);
        assert_eq!(super::status_for(Some(&envelope.details), &[]), BinaryDebugMapStatus::Failed);
    }

    let mut profile = profile_for_tests();
    profile.resource_limits.max_units = 1;
    let binary = build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 2, true, true);
    let envelope = core::analyze(core::AnalysisInput {
        binary: &binary,
        companion: None,
        profile: &profile,
        registry_identity: identity(b"registry"),
        function_addresses: None,
        requirement_refs: None,
        requirement_refs_identity: None,
    });
    assert!(envelope.details.findings.iter().any(|finding| finding.issue_code == "resource-limit"));
    assert_eq!(envelope.details.truncation.limit.as_deref(), Some("max_units"));
}

#[test]
fn requirement_linkage_rejects_claim_promotion_and_stale_registries() {
    assert!(validate_handoff_role("recorded_only").is_empty());
    for role in [
        "formal_proof",
        "semantic_equivalence",
        "requirement_satisfaction",
        "behavioral_correctness",
        "release_eligible",
    ] {
        assert!(!validate_handoff_role(role).is_empty(), "{role} must be rejected");
    }

    let profile = profile_for_tests();
    let envelope = analyze_fixture(&profile, false);
    let details = &envelope.details;
    let mut promoted = details.clone();
    promoted.verification_role = "formal_proof".to_owned();
    let mut diagnostics = Vec::new();
    validate_receipt_details(&promoted, &promoted.non_claims, &mut diagnostics);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded_only")));

    let mut stale = details.clone();
    stale.rows[0].requirement.as_mut().expect("typed requirement").registry_identity =
        "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned();
    let mut diagnostics = Vec::new();
    validate_receipt_details(&stale, &stale.non_claims, &mut diagnostics);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("requirement registry identity")));

    let mut promoted_claim = details.clone();
    promoted_claim.non_claims.push("this proves requirement satisfaction".to_owned());
    let mut diagnostics = Vec::new();
    validate_receipt_details(&promoted_claim, &promoted_claim.non_claims, &mut diagnostics);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("promoted claim")));
}

#[test]
fn envelope_evaluation_rejects_identity_confusion() {
    let profile = profile_for_tests();
    let envelope = analyze_fixture(&profile, false);
    let text = serde_json::to_string(&envelope).expect("serialize envelope");
    let evaluation = evaluate_envelope(&text);
    assert_eq!(evaluation.status, BinaryDebugMapStatus::Passed);
    assert!(evaluation.diagnostics.is_empty());

    let mut confused = envelope.clone();
    confused.details.binary_identity = "not-a-blake3".to_owned();
    let text = serde_json::to_string(&confused).expect("serialize confused envelope");
    let evaluation = evaluate_envelope(&text);
    assert_eq!(evaluation.status, BinaryDebugMapStatus::Failed);
    assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("binary_identity")));

    let malformed = evaluate_envelope("{ not json");
    assert_eq!(malformed.status, BinaryDebugMapStatus::Failed);
    assert!(!malformed.diagnostics.is_empty());
}

fn temp_dir() -> PathBuf {
    tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}")).keep()
}

fn write_registry(dir: &Path, name: &str, profile: &Profile) -> PathBuf {
    let registry = super::ProfileRegistry {
        schema_version: super::PROFILE_SCHEMA.to_owned(),
        profiles: vec![profile.clone()],
    };
    let path = dir.join(name);
    std::fs::write(&path, serde_json::to_vec_pretty(&registry).expect("serialize registry"))
        .unwrap_or_else(|error| panic!("write registry: {error}"));
    path
}

fn write_fixture(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap_or_else(|error| panic!("write fixture {name}: {error}"));
    path
}

#[test]
fn shell_enforces_profile_byte_limits_before_parsing() {
    let dir = temp_dir();
    let profile = profile_for_tests();
    let config = write_registry(&dir, "profiles.json", &profile);
    let binary =
        write_fixture(&dir, "fixture.elf", &build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 1, true, true));

    let mut limited = profile.clone();
    limited.resource_limits.max_binary_bytes = 16;
    let limited_config = write_registry(&dir, "profiles-binary-limit.json", &limited);
    let error = super::shell::run(&super::shell::ShellRequest {
        workspace_root: &dir,
        binary: &binary,
        companion: None,
        config: &limited_config,
        profile_id: super::PROFILE_ID,
        function_addresses: None,
        requirement_refs: None,
    })
    .expect_err("binary byte limit must fail before parsing");
    assert!(error.contains("exceeds"), "{error}");

    let companion =
        write_fixture(&dir, "fixture.debug", &build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 1, true, false));
    let mut limited_debug = profile.clone();
    limited_debug.resource_limits.max_debug_bytes = 16;
    let limited_debug_config = write_registry(&dir, "profiles-debug-limit.json", &limited_debug);
    let error = super::shell::run(&super::shell::ShellRequest {
        workspace_root: &dir,
        binary: &binary,
        companion: Some(&companion),
        config: &limited_debug_config,
        profile_id: super::PROFILE_ID,
        function_addresses: None,
        requirement_refs: None,
    })
    .expect_err("debug byte limit must fail before parsing");
    assert!(error.contains("exceeds"), "{error}");

    let output = super::shell::run(&super::shell::ShellRequest {
        workspace_root: &dir,
        binary: &binary,
        companion: Some(&companion),
        config: &config,
        profile_id: super::PROFILE_ID,
        function_addresses: None,
        requirement_refs: None,
    })
    .expect("shell collection succeeds");
    assert!(output.registry_identity.starts_with("b3:"));
    assert!(output.envelope_json.contains("\"debug_source\":\"split\""));
}

#[test]
fn collected_bundle_verifies_and_exports_recorded_only_handoff() {
    let dir = temp_dir();
    let profile = profile_for_tests();
    let config = write_registry(&dir, "profiles.json", &profile);
    let binary =
        write_fixture(&dir, "fixture.elf", &build_fixture(COMP_DIR, Some(BUILD_ID), &functions(), 1, true, true));
    let addresses = write_fixture(
        &dir,
        "function-addresses.json",
        &serde_json::to_vec(&serde_json::json!({
            "records": [
                {"path": REPOSITORY_PATH, "name": "alpha", "line": 10, "column": 1,
                 "start_byte": 0, "end_byte": 128, "function_address": "fa-alpha", "source_identity": "src:alpha"},
                {"path": REPOSITORY_PATH, "name": "beta", "line": 40, "column": 1,
                 "start_byte": 0, "end_byte": 128, "function_address": "fa-beta", "source_identity": "src:beta"}
            ]
        }))
        .expect("serialize function addresses"),
    );
    let references = write_fixture(
        &dir,
        "requirement-refs.json",
        &serde_json::to_vec(&serde_json::json!({
            "schema_version": super::REQUIREMENT_REFS_SCHEMA,
            "repository": "github:OnixResearch/octet",
            "revision": "0123456789abcdef0123456789abcdef01234567",
            "registry_identity": "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "references": [{
                "requirement_id": "r[octet.evidence_provenance.binary_debug_map_linkage]",
                "function_address": "fa-alpha",
            }],
        }))
        .expect("serialize requirement references"),
    );
    let artifact_dir = dir.join("bundle");
    let args = crate::cli::EvidenceRailArgs {
        rail: crate::evidence_rails::EvidenceRailKind::BinaryDebugMap,
        input: Some(binary),
        artifact_dir: artifact_dir.clone(),
        workspace_root: Some(dir.clone()),
        config: Some(config),
        debug_companion: None,
        function_address_artifact: Some(addresses),
        requirement_refs: Some(references),
        baseline: None,
        profile: Some(super::PROFILE_ID.to_owned()),
        command: None,
        parent_artifacts: Vec::new(),
        external_reports: Vec::new(),
        output_format: crate::cli::OutputFormat::Human,
        locked: false,
        update_advisory_database: false,
        run: false,
        command_line: "cargo octet evidence collect --rail binary-debug-map".to_owned(),
    };
    let exit = crate::evidence_rails::run(&args).expect("collect binary debug-map bundle");
    assert_eq!(exit, 0);

    let input = read_bundle(&artifact_dir);
    let report = crate::artifact::verify_bundle(input.clone());
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);

    let export =
        crate::artifact::build_valence_provenance_export(input.clone()).expect("Valence provenance export validates");
    let record = export
        .provenance_records
        .iter()
        .find(|record| record.source_id == "octet.artifact.binary-debug-map")
        .expect("binary debug-map provenance record");
    let details = record.receipt.binary_debug_map.as_ref().expect("binary debug-map details");
    assert_eq!(details.verification_role, super::RECORDED_ONLY_ROLE);
    assert_eq!(record.evidence_tier, "transcript_assertion");

    let mut promoted = input;
    let receipt = promoted
        .files
        .iter_mut()
        .find(|file| file.name == crate::evidence_rails::RECEIPT_ARTIFACT_NAME)
        .expect("receipt file");
    let mut receipt: crate::evidence_rails::EvidenceRailReceipt =
        serde_json::from_slice(&receipt.contents).expect("parse receipt");
    receipt.binary_debug_map.as_mut().expect("binary debug-map details").verification_role = "formal_proof".to_owned();
    receipt_json_replace(&mut promoted, &receipt);
    assert!(crate::artifact::build_valence_provenance_export(promoted).is_err());
}

fn read_bundle(dir: &Path) -> crate::artifact::ArtifactBundleInput {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap_or_else(|error| panic!("read bundle: {error}")) {
        let entry = entry.unwrap_or_else(|error| panic!("read bundle entry: {error}"));
        if entry.file_type().is_ok_and(|kind| kind.is_file()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let contents = std::fs::read(entry.path()).unwrap_or_else(|error| panic!("read {name}: {error}"));
            files.push(crate::artifact::BundleFile { name, contents });
        }
    }
    crate::artifact::ArtifactBundleInput {
        artifact_dir: dir.display().to_string(),
        files,
    }
}

fn receipt_json_replace(
    input: &mut crate::artifact::ArtifactBundleInput,
    receipt: &crate::evidence_rails::EvidenceRailReceipt,
) {
    let bytes = serde_json::to_vec_pretty(receipt).expect("serialize receipt");
    if let Some(file) = input.files.iter_mut().find(|file| file.name == crate::evidence_rails::RECEIPT_ARTIFACT_NAME) {
        file.contents = bytes;
    }
}
