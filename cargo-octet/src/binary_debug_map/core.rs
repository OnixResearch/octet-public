//! Pure bounded ELF/DWARF normalization over supplied bytes.
//!
//! No file access, process execution, environment access, clock access, or output
//! writing happens here. Inputs are immutable byte slices and a validated profile.

use std::collections::BTreeMap;

use gimli::AttributeValue;
use gimli::Dwarf;
use gimli::EndianSlice;
use gimli::LittleEndian;
use gimli::Reader as _;
use gimli::Unit;
use object::Object;
use object::ObjectSection;
use object::ObjectSymbol;
use object::SymbolKind;

use super::BinaryDebugMapEnvelope;
use super::BinaryDebugMapFinding;
use super::BinaryDebugMapReceiptDetails;
use super::BinaryDebugRow;
use super::COMPANION_ROLE;
use super::DEBUG_FORMAT_DWARF;
use super::DEBUG_SOURCE_INLINE;
use super::DEBUG_SOURCE_SPLIT;
use super::DebugCompanionIdentity;
use super::ENVELOPE_SCHEMA;
use super::OBJECT_FORMAT_ELF;
use super::PROFILE_ID;
use super::Profile;
use super::RECEIPT_SCHEMA;
use super::RECORDED_ONLY_ROLE;
use super::RequirementRefSet;
use super::ResourceLimits;
use super::SEVERITY_MANUAL_REVIEW;
use super::TruncationState;
use super::display_name;
use super::identity;
use super::is_path_prefix;
use super::is_safe_relative_path;
use super::linkage::FunctionAddressArtifact;
use super::linkage::LinkIssue;
use super::linkage::LinkRequest;
use super::linkage::link_row;
use super::path_hash;
use super::profile_identity;
use super::symbol_name_hash;

const BLAKE3_PREFIX: &str = "b3:";
const BUILD_ID_NOTE_SECTION: &str = ".note.gnu.build-id";
const GNU_NOTE_NAME: &str = "GNU";
const NT_GNU_BUILD_ID: u32 = 3;

const MALFORMED_BINARY_ISSUE: &str = "malformed-binary";
const MALFORMED_DEBUG_ISSUE: &str = "malformed-debug";
const UNSUPPORTED_FORMAT_ISSUE: &str = "unsupported-format";
const MISSING_DEBUG_SECTIONS_ISSUE: &str = "missing-debug-sections";
const WRONG_COMPANION_ISSUE: &str = "wrong-companion";
const UNMAPPED_PATH_ISSUE: &str = "unmapped-path";
const PATH_TRAVERSAL_ISSUE: &str = "path-traversal";
const STALE_SPAN_ISSUE: &str = "stale-span";
const AMBIGUOUS_FUNCTION_ISSUE: &str = "ambiguous-function";
const AMBIGUOUS_REQUIREMENT_ISSUE: &str = "ambiguous-requirement";
const RESOURCE_LIMIT_ISSUE: &str = "resource-limit";
const MISSING_SYMBOL_ISSUE: &str = "missing-symbol";
const MISSING_LINE_ROW_ISSUE: &str = "missing-line-row";
const MIXED_SOURCE_RANGE_ISSUE: &str = "mixed-source-range";

pub(crate) struct AnalysisInput<'a> {
    pub(crate) binary: &'a [u8],
    pub(crate) companion: Option<&'a [u8]>,
    pub(crate) profile: &'a Profile,
    pub(crate) registry_identity: String,
    pub(crate) function_addresses: Option<&'a FunctionAddressArtifact>,
    pub(crate) requirement_refs: Option<&'a RequirementRefSet>,
    pub(crate) requirement_refs_identity: Option<String>,
}

#[derive(Clone, Debug)]
struct CoreFailure {
    code: &'static str,
    message: String,
    limit: Option<&'static str>,
}

impl CoreFailure {
    fn malformed(code: &'static str, message: String) -> Self {
        Self {
            code,
            message,
            limit: None,
        }
    }

    fn limit(name: &'static str) -> Self {
        Self {
            code: RESOURCE_LIMIT_ISSUE,
            message: format!("binary debug-map profile limit `{name}` was exceeded; evidence is truncated"),
            limit: Some(name),
        }
    }
}

#[derive(Default)]
struct Facts {
    rows: Vec<BinaryDebugRow>,
    findings: Vec<BinaryDebugMapFinding>,
    excluded_rows: u64,
    build_identity: Option<String>,
}

struct SymbolEntry {
    end: u64,
    name: String,
}

struct LinePoint {
    address: u64,
    path: Option<String>,
    line: u64,
    column: u64,
}

pub(crate) fn analyze(input: AnalysisInput<'_>) -> BinaryDebugMapEnvelope {
    let binary_identity = binary_identity(input.binary);
    let companion_identity = input.companion.map(identity);
    let profile_id = profile_identity(input.profile).unwrap_or_default();
    let mut facts = Facts::default();
    let mut truncation = TruncationState::default();
    let debug_source = if input.companion.is_some() {
        DEBUG_SOURCE_SPLIT
    } else {
        DEBUG_SOURCE_INLINE
    };

    if let Err(failure) = collect(&mut facts, &input) {
        let severity = severity_for(input.profile, failure.code, false);
        facts.findings.push(BinaryDebugMapFinding {
            issue_code: failure.code.to_owned(),
            severity,
            message: failure.message,
            address: None,
        });
        if let Some(limit) = failure.limit {
            truncation.truncated = true;
            truncation.limit = Some(limit.to_owned());
        }
    }

    facts.rows.sort_by(|left, right| {
        left.address_start
            .cmp(&right.address_start)
            .then(left.range_index.cmp(&right.range_index))
            .then(left.repository_path.cmp(&right.repository_path))
            .then(left.index.cmp(&right.index))
    });
    for (index, row) in facts.rows.iter_mut().enumerate() {
        row.index = u64::try_from(index).unwrap_or(u64::MAX);
    }
    facts.findings.sort_by(|left, right| {
        left.issue_code
            .cmp(&right.issue_code)
            .then(left.address.cmp(&right.address))
            .then(left.message.cmp(&right.message))
    });

    let details = BinaryDebugMapReceiptDetails {
        schema_version: RECEIPT_SCHEMA.to_owned(),
        profile_id: PROFILE_ID.to_owned(),
        profile_identity: profile_id,
        registry_identity: input.registry_identity,
        object_format: OBJECT_FORMAT_ELF.to_owned(),
        architecture: input.profile.architecture.clone(),
        debug_format: DEBUG_FORMAT_DWARF.to_owned(),
        binary_identity,
        debug_companion: companion_identity.map(|identity| DebugCompanionIdentity {
            identity,
            role: COMPANION_ROLE.to_owned(),
            build_identity: facts.build_identity.clone(),
        }),
        build_identity: facts.build_identity,
        debug_source: debug_source.to_owned(),
        function_address_identity: input.function_addresses.map(|artifact| artifact.identity.clone()),
        requirement_refs_identity: input.requirement_refs_identity.clone(),
        requirement_registry_identity: input.requirement_refs.map(|references| references.registry_identity.clone()),
        source_roots: input.profile.source_roots.clone(),
        limits: input.profile.resource_limits.clone(),
        rows: facts.rows,
        excluded_rows: facts.excluded_rows,
        findings: facts.findings,
        truncation,
        verification_role: RECORDED_ONLY_ROLE.to_owned(),
        non_claims: input.profile.non_claims.clone(),
    };

    BinaryDebugMapEnvelope {
        schema_version: ENVELOPE_SCHEMA.to_owned(),
        non_claims: details.non_claims.clone(),
        details,
    }
}

fn collect(facts: &mut Facts, input: &AnalysisInput<'_>) -> Result<(), CoreFailure> {
    let binary = object::File::parse(input.binary)
        .map_err(|error| CoreFailure::malformed(MALFORMED_BINARY_ISSUE, format!("parse binary ELF: {error}")))?;
    validate_object(&binary, input.profile)?;
    check_sections(&binary, input.profile, MALFORMED_BINARY_ISSUE)?;
    let binary_build_id = build_identity(&binary);
    facts.build_identity.clone_from(&binary_build_id);

    let debug_file = match input.companion {
        Some(companion_bytes) => {
            let companion = object::File::parse(companion_bytes).map_err(|error| {
                CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("parse debug companion ELF: {error}"))
            })?;
            validate_object(&companion, input.profile)?;
            check_sections(&companion, input.profile, MALFORMED_DEBUG_ISSUE)?;
            let companion_build_id = build_identity(&companion);
            match (binary_build_id.as_deref(), companion_build_id.as_deref()) {
                (Some(expected), Some(observed)) if expected == observed => {}
                (Some(_), Some(_)) => {
                    return Err(CoreFailure::malformed(
                        WRONG_COMPANION_ISSUE,
                        "split-debug companion build identity does not match the selected binary".to_owned(),
                    ));
                }
                _ => {
                    return Err(CoreFailure::malformed(
                        WRONG_COMPANION_ISSUE,
                        "split-debug companion cannot be matched without a binary build identity".to_owned(),
                    ));
                }
            }
            companion
        }
        None => binary,
    };
    require_debug_sections(&debug_file, input.profile)?;
    let dwarf = load_dwarf(&debug_file)?;
    let symbols = collect_symbols(&debug_file, input.profile)?;
    collect_units(&dwarf, &symbols, facts, input)?;
    Ok(())
}

fn validate_object(file: &object::File<'_>, profile: &Profile) -> Result<(), CoreFailure> {
    if file.format() != object::BinaryFormat::Elf {
        return Err(CoreFailure::malformed(
            UNSUPPORTED_FORMAT_ISSUE,
            format!("binary debug-map admits ELF only, observed `{:?}`", file.format()),
        ));
    }
    let observed = match file.architecture() {
        object::Architecture::X86_64 => "x86_64",
        object::Architecture::Aarch64 => "aarch64",
        other => {
            return Err(CoreFailure::malformed(
                UNSUPPORTED_FORMAT_ISSUE,
                format!("binary debug-map does not admit architecture `{other:?}`"),
            ));
        }
    };
    if observed != profile.architecture {
        return Err(CoreFailure::malformed(
            UNSUPPORTED_FORMAT_ISSUE,
            format!("binary debug-map profile selects architecture `{}`, observed `{observed}`", profile.architecture),
        ));
    }
    Ok(())
}

fn check_sections(file: &object::File<'_>, profile: &Profile, code: &'static str) -> Result<(), CoreFailure> {
    let count = u64::try_from(file.sections().count()).unwrap_or(u64::MAX);
    if count > profile.resource_limits.max_sections {
        return Err(CoreFailure::limit("max_sections"));
    }
    if count == 0 {
        return Err(CoreFailure::malformed(code, "ELF object declares no sections".to_owned()));
    }
    Ok(())
}

fn require_debug_sections(file: &object::File<'_>, profile: &Profile) -> Result<(), CoreFailure> {
    let mut missing = Vec::new();
    for name in &profile.required_sections {
        if section_bytes(file, name).is_none() {
            missing.push(name.as_str());
        }
    }
    if !missing.is_empty() {
        return Err(CoreFailure::malformed(
            MISSING_DEBUG_SECTIONS_ISSUE,
            format!(
                "binary debug-map requires DWARF sections [{}]; supply a matching split-debug companion",
                missing.join(", ")
            ),
        ));
    }
    Ok(())
}

fn section_bytes<'a>(file: &object::File<'a>, name: &str) -> Option<&'a [u8]> {
    let section = file.section_by_name(name)?;
    let compressed = section.compressed_file_range().is_ok_and(|range| range.format != object::CompressionFormat::None);
    if compressed {
        return None;
    }
    section.data().ok()
}

fn load_dwarf<'a>(file: &object::File<'a>) -> Result<Dwarf<EndianSlice<'a, LittleEndian>>, CoreFailure> {
    Dwarf::load(|id| -> Result<EndianSlice<'a, LittleEndian>, CoreFailure> {
        let bytes = section_bytes(file, id.name()).unwrap_or(&[]);
        Ok(EndianSlice::new(bytes, LittleEndian))
    })
}

fn build_identity(file: &object::File<'_>) -> Option<String> {
    let bytes = section_bytes(file, BUILD_ID_NOTE_SECTION)?;
    parse_build_id_note(bytes).map(hex)
}

fn parse_build_id_note(bytes: &[u8]) -> Option<&[u8]> {
    let header = bytes.get(..12)?;
    let name_size = u32::from_le_bytes(header.get(0..4)?.try_into().ok()?);
    let desc_size = u32::from_le_bytes(header.get(4..8)?.try_into().ok()?);
    let note_type = u32::from_le_bytes(header.get(8..12)?.try_into().ok()?);
    if note_type != NT_GNU_BUILD_ID {
        return None;
    }
    let name_size = usize::try_from(name_size).ok()?;
    let desc_size = usize::try_from(desc_size).ok()?;
    let name_start: usize = 12;
    let name_end = name_start.checked_add(name_size)?;
    let name = bytes.get(name_start..name_end)?;
    if name.strip_suffix(&[0]) != Some(GNU_NOTE_NAME.as_bytes()) {
        return None;
    }
    let desc_start = align4(name_end)?;
    let desc_end = desc_start.checked_add(desc_size)?;
    bytes.get(desc_start..desc_end)
}

fn align4(value: usize) -> Option<usize> {
    value.checked_add(3).map(|value| value & !3)
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut rendered = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        let _ = write!(&mut rendered, "{byte:02x}");
    }
    rendered
}

fn collect_symbols(file: &object::File<'_>, profile: &Profile) -> Result<BTreeMap<u64, SymbolEntry>, CoreFailure> {
    let mut symbols = BTreeMap::new();
    let mut count = 0_u64;
    for symbol in file.symbols() {
        count = count.saturating_add(1);
        if count > profile.resource_limits.max_symbols {
            return Err(CoreFailure::limit("max_symbols"));
        }
        if symbol.kind() != SymbolKind::Text || !symbol.is_definition() {
            continue;
        }
        let Ok(name) = symbol.name() else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let address = symbol.address();
        let end = address.saturating_add(symbol.size());
        symbols.entry(address).or_insert_with(|| SymbolEntry {
            end,
            name: name.to_owned(),
        });
    }
    Ok(symbols)
}

fn collect_units(
    dwarf: &Dwarf<EndianSlice<'_, LittleEndian>>,
    symbols: &BTreeMap<u64, SymbolEntry>,
    facts: &mut Facts,
    input: &AnalysisInput<'_>,
) -> Result<(), CoreFailure> {
    let mut units = dwarf.units();
    let mut unit_count = 0_u64;
    let mut range_count = 0_u64;
    let mut line_row_count = 0_u64;
    let mut frame_count = 0_u64;
    let mut symbol_count = u64::try_from(symbols.len()).unwrap_or(u64::MAX);
    while let Some(header) = units
        .next()
        .map_err(|error| CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("read DWARF unit header: {error}")))?
    {
        unit_count = unit_count.saturating_add(1);
        if unit_count > input.profile.resource_limits.max_units {
            return Err(CoreFailure::limit("max_units"));
        }
        let unit = dwarf
            .unit(header)
            .map_err(|error| CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("read DWARF unit: {error}")))?;
        let comp_dir = match unit.comp_dir {
            Some(reader) => Some(reader_text(reader)?),
            None => None,
        };
        let mut line_points = Vec::new();
        if let Some(program) = unit.line_program.clone() {
            let mut rows = program.rows();
            while let Some((header, row)) = rows.next_row().map_err(|error| {
                CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("read DWARF line row: {error}"))
            })? {
                line_row_count = line_row_count.saturating_add(1);
                if line_row_count > input.profile.resource_limits.max_line_rows {
                    return Err(CoreFailure::limit("max_line_rows"));
                }
                if row.end_sequence() {
                    frame_count = frame_count.saturating_add(1);
                    if frame_count > input.profile.resource_limits.max_frames {
                        return Err(CoreFailure::limit("max_frames"));
                    }
                    continue;
                }
                let raw_path = match row.file(header) {
                    Some(file) => file_path(dwarf, &unit, file, header)?,
                    None => None,
                };
                let path = match raw_path {
                    Some(raw) => match normalize_source_path(&raw, comp_dir.as_deref(), input.profile) {
                        PathOutcome::Admitted(path) => Some(path),
                        PathOutcome::Excluded => None,
                        PathOutcome::Failed(failure) => {
                            if failure.code == RESOURCE_LIMIT_ISSUE {
                                return Err(failure);
                            }
                            let severity = severity_for(input.profile, failure.code, false);
                            facts.findings.push(BinaryDebugMapFinding {
                                issue_code: failure.code.to_owned(),
                                severity,
                                message: failure.message,
                                address: Some(row.address()),
                            });
                            None
                        }
                    },
                    None => None,
                };
                line_points.push(LinePoint {
                    address: row.address(),
                    path,
                    line: row.line().map_or(0, std::num::NonZeroU64::get),
                    column: match row.column() {
                        gimli::ColumnType::LeftEdge => 0,
                        gimli::ColumnType::Column(column) => column.get(),
                    },
                });
            }
        }
        line_points.sort_by_key(|point| (point.address, point.line, point.column));
        collect_subprograms(dwarf, &unit, &line_points, symbols, facts, input, &mut range_count, &mut symbol_count)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn collect_subprograms(
    dwarf: &Dwarf<EndianSlice<'_, LittleEndian>>,
    unit: &Unit<EndianSlice<'_, LittleEndian>>,
    line_points: &[LinePoint],
    symbols: &BTreeMap<u64, SymbolEntry>,
    facts: &mut Facts,
    input: &AnalysisInput<'_>,
    range_count: &mut u64,
    symbol_count: &mut u64,
) -> Result<(), CoreFailure> {
    let mut entries = unit.entries();
    while let Some(entry) = entries
        .next_dfs()
        .map_err(|error| CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("read DWARF entry: {error}")))?
    {
        if entry.tag() != gimli::DW_TAG_subprogram {
            continue;
        }
        *symbol_count = symbol_count.saturating_add(1);
        if *symbol_count > input.profile.resource_limits.max_symbols {
            return Err(CoreFailure::limit("max_symbols"));
        }
        let (raw_name, source_name) = subprogram_names(dwarf, unit, entry)?;
        let mut ranges = dwarf
            .die_ranges(unit, entry)
            .map_err(|error| CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("read DWARF ranges: {error}")))?;
        let mut range_index = 0_u64;
        while let Some(range) = ranges
            .next()
            .map_err(|error| CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("read DWARF range: {error}")))?
        {
            if range.end <= range.begin {
                continue;
            }
            *range_count = range_count.saturating_add(1);
            if *range_count > input.profile.resource_limits.max_ranges {
                return Err(CoreFailure::limit("max_ranges"));
            }
            emit_row(
                facts,
                input,
                line_points,
                symbols,
                raw_name.as_deref(),
                source_name.as_deref(),
                range.begin,
                range.end,
                range_index,
            )?;
            range_index = range_index.saturating_add(1);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(
    clippy::too_many_lines,
    reason = "one row emission keeps the profile, linkage, and resource facts visible together"
)]
fn emit_row(
    facts: &mut Facts,
    input: &AnalysisInput<'_>,
    line_points: &[LinePoint],
    symbols: &BTreeMap<u64, SymbolEntry>,
    die_name: Option<&str>,
    die_source_name: Option<&str>,
    begin: u64,
    end: u64,
    range_index: u64,
) -> Result<(), CoreFailure> {
    if u64::try_from(facts.rows.len()).unwrap_or(u64::MAX) >= input.profile.resource_limits.max_rows {
        return Err(CoreFailure::limit("max_rows"));
    }
    let raw_name = match die_name {
        Some(name) => Some(name.to_owned()),
        None => symbol_at(symbols, begin).map(|entry| entry.name.clone()),
    };
    let Some(raw_name) = raw_name else {
        facts.excluded_rows = facts.excluded_rows.saturating_add(1);
        push_finding(
            facts,
            input.profile,
            MISSING_SYMBOL_ISSUE,
            format!("DWARF subprogram at address {begin} has no admitted symbol name"),
            Some(begin),
            false,
        );
        return Ok(());
    };
    let start = line_points.partition_point(|point| point.address < begin);
    let end_index = line_points.partition_point(|point| point.address < end);
    let span = line_points.get(start..end_index).unwrap_or(&[]);
    let admitted: Vec<&LinePoint> = span.iter().filter(|point| point.path.is_some()).collect();
    let Some(first) = admitted.first() else {
        facts.excluded_rows = facts.excluded_rows.saturating_add(1);
        // A span that only held policy-excluded paths is an explicit exclusion, not
        // a missing line row; an empty span is missing DWARF line information.
        if span.is_empty() {
            push_finding(
                facts,
                input.profile,
                MISSING_LINE_ROW_ISSUE,
                format!("DWARF subprogram at address {begin} has no source line row"),
                Some(begin),
                false,
            );
        }
        return Ok(());
    };
    let Some(last) = admitted.last() else {
        return Ok(());
    };
    let Some(repository_path) = first.path.as_deref() else {
        return Ok(());
    };
    if last.path.as_deref() != Some(repository_path) || first.line == 0 {
        facts.excluded_rows = facts.excluded_rows.saturating_add(1);
        push_finding(
            facts,
            input.profile,
            MIXED_SOURCE_RANGE_ISSUE,
            format!("DWARF subprogram at address {begin} spans inconsistent source positions"),
            Some(begin),
            false,
        );
        return Ok(());
    }
    let source_name = die_source_name.map_or_else(|| source_symbol_name(&raw_name), str::to_owned);
    let link = link_row(&LinkRequest {
        repository_path,
        source_name: &source_name,
        line_start: first.line,
        line_end: last.line.max(first.line),
        column_start: first.column,
        column_end: last.column,
        function_addresses: input.function_addresses,
        requirement_refs: input.requirement_refs,
        strict: input.profile.strict_linkage,
    });
    if let Some(issue) = link.issue {
        let (code, message) = match issue {
            LinkIssue::AmbiguousFunction => (
                AMBIGUOUS_FUNCTION_ISSUE,
                format!(
                    "DWARF subprogram `{source_name}` at `{repository_path}` matches several function-address candidates"
                ),
            ),
            LinkIssue::StaleSpan => (
                STALE_SPAN_ISSUE,
                format!(
                    "function-address candidate for `{source_name}` at `{repository_path}` is outside the DWARF line span"
                ),
            ),
            LinkIssue::AmbiguousRequirement => (
                AMBIGUOUS_REQUIREMENT_ISSUE,
                format!("several typed requirement references bind function `{source_name}`"),
            ),
        };
        push_finding(facts, input.profile, code, message, Some(begin), !link.strict);
    }
    facts.rows.push(BinaryDebugRow {
        index: u64::try_from(facts.rows.len()).unwrap_or(u64::MAX),
        raw_symbol_name_hash: symbol_name_hash(&raw_name),
        demangled_name: display_name(&raw_name),
        address_start: begin,
        address_end: end,
        range_index,
        repository_path: repository_path.to_owned(),
        path_hash: path_hash(repository_path),
        line_start: first.line,
        column_start: first.column,
        line_end: last.line.max(first.line),
        column_end: last.column,
        source_identity: link.source_identity,
        function_address: link.function_address,
        requirement: link.requirement,
        legacy_requirement_ids: link.legacy_requirement_ids,
    });
    Ok(())
}

fn push_finding(
    facts: &mut Facts,
    profile: &Profile,
    code: &'static str,
    message: String,
    address: Option<u64>,
    downgrade: bool,
) {
    let severity = if downgrade {
        SEVERITY_MANUAL_REVIEW.to_owned()
    } else {
        severity_for(profile, code, false)
    };
    facts.findings.push(BinaryDebugMapFinding {
        issue_code: code.to_owned(),
        severity,
        message,
        address,
    });
}

fn symbol_at(symbols: &BTreeMap<u64, SymbolEntry>, address: u64) -> Option<&SymbolEntry> {
    let (start, entry) = symbols.range(..=address).next_back()?;
    if address < entry.end || entry.end == *start {
        Some(entry)
    } else {
        None
    }
}

fn subprogram_names(
    dwarf: &Dwarf<EndianSlice<'_, LittleEndian>>,
    unit: &Unit<EndianSlice<'_, LittleEndian>>,
    entry: &gimli::DebuggingInformationEntry<EndianSlice<'_, LittleEndian>>,
) -> Result<(Option<String>, Option<String>), CoreFailure> {
    let name = attribute_text(dwarf, unit, entry, gimli::DW_AT_name)?;
    let linkage_name = attribute_text(dwarf, unit, entry, gimli::DW_AT_linkage_name)?;
    let raw = linkage_name.clone().or_else(|| name.clone());
    Ok((raw, name))
}

fn attribute_text(
    dwarf: &Dwarf<EndianSlice<'_, LittleEndian>>,
    unit: &Unit<EndianSlice<'_, LittleEndian>>,
    entry: &gimli::DebuggingInformationEntry<EndianSlice<'_, LittleEndian>>,
    attribute: gimli::DwAt,
) -> Result<Option<String>, CoreFailure> {
    let Some(value) = entry.attr_value(attribute) else {
        return Ok(None);
    };
    let text = attr_text(dwarf, unit, value)?;
    if text.is_empty() { Ok(None) } else { Ok(Some(text)) }
}

fn file_path(
    dwarf: &Dwarf<EndianSlice<'_, LittleEndian>>,
    unit: &Unit<EndianSlice<'_, LittleEndian>>,
    file: &gimli::FileEntry<EndianSlice<'_, LittleEndian>>,
    header: &gimli::LineProgramHeader<EndianSlice<'_, LittleEndian>>,
) -> Result<Option<String>, CoreFailure> {
    let name = attr_text(dwarf, unit, file.path_name())?;
    if name.is_empty() {
        return Ok(None);
    }
    if name.starts_with('/') {
        return Ok(Some(name));
    }
    let directory = match file.directory(header) {
        Some(value) => Some(attr_text(dwarf, unit, value)?),
        None => None,
    };
    Ok(Some(match directory {
        Some(directory) if !directory.is_empty() => format!("{directory}/{name}"),
        _ => name,
    }))
}

fn attr_text(
    dwarf: &Dwarf<EndianSlice<'_, LittleEndian>>,
    unit: &Unit<EndianSlice<'_, LittleEndian>>,
    value: AttributeValue<EndianSlice<'_, LittleEndian>>,
) -> Result<String, CoreFailure> {
    let reader = dwarf
        .attr_string(unit, value)
        .map_err(|error| CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("read DWARF string: {error}")))?;
    reader_text(reader)
}

fn reader_text(reader: EndianSlice<'_, LittleEndian>) -> Result<String, CoreFailure> {
    let bytes = reader
        .to_slice()
        .map_err(|error| CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("read DWARF bytes: {error}")))?;
    std::str::from_utf8(bytes.as_ref())
        .map(str::to_owned)
        .map_err(|error| CoreFailure::malformed(MALFORMED_DEBUG_ISSUE, format!("DWARF text is not UTF-8: {error}")))
}

fn source_symbol_name(raw_name: &str) -> String {
    // Only `DW_AT_linkage_name` was present. Demangling is display metadata, so
    // strip a legacy trailing `::h<hex>` hash segment and keep the path text.
    let demangled = display_name(raw_name);
    if let Some(index) = demangled.rfind("::h") {
        let suffix = &demangled[index + 2..];
        if !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_hexdigit()) {
            return demangled[..index].to_owned();
        }
    }
    demangled
}

enum PathOutcome {
    Admitted(String),
    Excluded,
    Failed(CoreFailure),
}

fn normalize_source_path(raw: &str, comp_dir: Option<&str>, profile: &Profile) -> PathOutcome {
    let limit = profile.resource_limits.max_path_bytes;
    if u64::try_from(raw.len()).unwrap_or(u64::MAX) > limit {
        return PathOutcome::Failed(CoreFailure::limit("max_path_bytes"));
    }
    if raw.contains('\0') {
        return PathOutcome::Failed(CoreFailure::malformed(
            MALFORMED_DEBUG_ISSUE,
            "DWARF source path contains a null byte".to_owned(),
        ));
    }
    let absolute = if raw.starts_with('/') {
        raw.to_owned()
    } else if let Some(comp_dir) = comp_dir.filter(|dir| dir.starts_with('/')) {
        format!("{comp_dir}/{raw}")
    } else {
        return match canonical_relative(raw) {
            Some(path) => PathOutcome::Admitted(path),
            None => PathOutcome::Failed(CoreFailure::malformed(
                PATH_TRAVERSAL_ISSUE,
                "DWARF source path escapes the admitted repository scope".to_owned(),
            )),
        };
    };
    let mut best: Option<&super::SourceRoot> = None;
    for mapping in &profile.source_roots {
        if is_path_prefix(&mapping.build_root, &absolute)
            && best.is_none_or(|current| mapping.build_root.len() > current.build_root.len())
        {
            best = Some(mapping);
        }
    }
    let Some(mapping) = best else {
        return match profile.unmapped_path_policy.as_str() {
            super::UNMAPPED_EXCLUDE => PathOutcome::Excluded,
            _ => PathOutcome::Failed(CoreFailure::malformed(
                UNMAPPED_PATH_ISSUE,
                "DWARF source path is outside every admitted source-root mapping".to_owned(),
            )),
        };
    };
    let remainder = absolute.strip_prefix(&mapping.build_root).unwrap_or("").trim_start_matches('/');
    let joined = if remainder.is_empty() {
        mapping.repository_root.clone()
    } else {
        format!("{}/{}", mapping.repository_root, remainder)
    };
    match canonical_relative(&joined) {
        Some(path) => PathOutcome::Admitted(path),
        None => PathOutcome::Failed(CoreFailure::malformed(
            PATH_TRAVERSAL_ISSUE,
            "mapped DWARF source path is not a canonical repository-relative path".to_owned(),
        )),
    }
}

fn canonical_relative(path: &str) -> Option<String> {
    if !is_safe_relative_path(path) {
        return None;
    }
    Some(path.to_owned())
}

#[allow(
    clippy::match_same_arms,
    reason = "the explicit malformed-code inventory is kept visible and the closed default intentionally repeats it"
)]
fn severity_for(profile: &Profile, code: &str, downgrade: bool) -> String {
    if downgrade {
        return SEVERITY_MANUAL_REVIEW.to_owned();
    }
    let severity = match code {
        MALFORMED_BINARY_ISSUE
        | MALFORMED_DEBUG_ISSUE
        | MISSING_SYMBOL_ISSUE
        | MISSING_LINE_ROW_ISSUE
        | MIXED_SOURCE_RANGE_ISSUE
        | PATH_TRAVERSAL_ISSUE => &profile.severity.malformed,
        UNSUPPORTED_FORMAT_ISSUE | MISSING_DEBUG_SECTIONS_ISSUE => &profile.severity.unsupported,
        WRONG_COMPANION_ISSUE => &profile.severity.wrong_companion,
        UNMAPPED_PATH_ISSUE => &profile.severity.unmapped_path,
        STALE_SPAN_ISSUE => &profile.severity.stale_span,
        AMBIGUOUS_FUNCTION_ISSUE | AMBIGUOUS_REQUIREMENT_ISSUE => &profile.severity.ambiguous_function,
        RESOURCE_LIMIT_ISSUE => &profile.severity.resource_limit,
        _ => &profile.severity.malformed,
    };
    severity.clone()
}

/// Bounded resource limits are re-exported for shell-side byte admission.
pub(crate) fn byte_limits(profile: &Profile) -> &ResourceLimits {
    &profile.resource_limits
}

/// The canonical binary identity for the supplied exact bytes.
pub(crate) fn binary_identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}
