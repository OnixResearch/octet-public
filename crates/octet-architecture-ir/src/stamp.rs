use std::path::Path;
use std::path::PathBuf;

use crate::identity::canonical_identity;
use crate::model::CompilationUnit;

/// Directory, beside the collector context file, that holds one freshness stamp per expected unit.
///
/// The collector records the selected unit's stamp in rustc dep-info, so Cargo recompiles that
/// unit, and the collector writes a new shard, whenever the runner rewrites or removes the stamp.
const UNIT_STAMP_DIRECTORY: &str = "unit-stamps";
const UNIT_STAMP_DOMAIN: &str = "octet.compiler-architecture-ir.unit-stamp/v1";
const UNIT_STAMP_EXTENSION: &str = "stamp";

/// Returns the unit-stamp directory that belongs to a collector context file.
#[must_use]
pub fn compilation_unit_stamp_directory(context_path: &Path) -> PathBuf {
    context_path.parent().unwrap_or_else(|| Path::new("")).join(UNIT_STAMP_DIRECTORY)
}

/// Returns the freshness-stamp path of a compilation unit for a collector context file.
#[must_use]
pub fn compilation_unit_stamp_path(context_path: &Path, unit: &CompilationUnit) -> PathBuf {
    compilation_unit_stamp_directory(context_path).join(compilation_unit_stamp_name(unit))
}

/// Names a unit's freshness stamp from its stable locator.
///
/// The locator is every unit field except `unit_id` and the volatile policy, source, and invocation
/// hashes, so one Cargo unit keeps one stamp file while its identity changes; the stamp content
/// carries the current `unit_id`.
///
/// # Panics
///
/// Panics only if the fixed, serializable locator fields cannot be serialized.
#[must_use]
pub fn compilation_unit_stamp_name(unit: &CompilationUnit) -> String {
    let locator = canonical_identity(
        UNIT_STAMP_DOMAIN,
        &(
            &unit.package_name,
            &unit.package_version,
            &unit.target_name,
            &unit.target_kind,
            &unit.source_path,
            &unit.crate_types,
            &unit.target_triple,
            &unit.features,
            unit.default_features,
            unit.role,
            &unit.toolchain_blake3,
            &unit.collector_blake3,
        ),
    )
    .expect("compilation-unit stamp locator fields must serialize");
    format!("{locator}.{UNIT_STAMP_EXTENSION}")
}
