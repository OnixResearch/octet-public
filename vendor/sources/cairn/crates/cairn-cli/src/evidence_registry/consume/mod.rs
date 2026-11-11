use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::agent_receipt::AGENT_RECEIPT_MAX_BYTES;
use crate::support::{
    path_string, read_string_option, with_portable_receipt_hash, CapabilityRoot, CapabilityRootKind,
};
use cairn_core::canonical_json_hash;

use super::build_verify::evidence_registry_verify_receipt;
use super::common::{evidence_registry_issue, evidence_registry_resolve_index_path};
use super::{
    EvidenceRegistryFetchPlanOptions, EvidenceRegistrySelectOptions, EvidenceRegistryVerifyOptions,
};

mod fetch_plan;
mod matching;
mod receipts;
mod registry;
mod select;
mod selectors;

pub(super) use fetch_plan::evidence_registry_fetch_plan_receipt;
pub(super) use select::evidence_registry_select_receipt;
pub(super) use selectors::{evidence_registry_consume_selectors, EvidenceRegistryConsumeSelectors};

// r[impl cli_maintainability.evidence_registry_consume_submodule_split.focused_modules]
// r[impl cli_maintainability.evidence_registry_consume_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.evidence_registry_consume_submodule_split.focused_modules.narrow_bridges]
// r[verify cli_maintainability.evidence_registry_consume_submodule_split.behavior_preserved]
// r[verify cli_maintainability.evidence_registry_consume_submodule_split.behavior_preserved.focused_checks]
// r[verify cli_maintainability.evidence_registry_consume_submodule_split.behavior_preserved.repository_checks]
