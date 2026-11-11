use cairn_core::GateStage;

#[derive(Clone)]
pub(super) enum ReplayCaseKind {
    AgentCompletionValidate,
    AgentReceiptPreservesHash,
    AgentReceiptValidate,
    ArchiveDryRun,
    PreservesCorpusFreshness,
    Gate { stage: GateStage, change: String },
    MigrateDryRun,
    McpDescribe,
    McpStatus,
    ReceiptPreservesDiff,
    ReleaseReadiness,
    Status,
    SyncDryRun,
    Validate,
    View { view: String },
}

#[derive(Clone)]
pub(crate) struct ReplayGroup {
    pub(crate) id: String,
    pub(crate) cases: Vec<String>,
}

#[derive(Clone)]
pub(in crate::determinism) struct ReplayEntropyEntry {
    pub(super) id: String,
    pub(super) class: String,
    pub(super) source: String,
    pub(super) handling: String,
    pub(super) receipt_field: String,
    pub(super) description: String,
}

#[derive(Clone)]
pub(in crate::determinism) struct ReplayEnvironmentFact {
    pub(super) id: String,
    pub(super) class: String,
    pub(super) source: String,
    pub(super) handling: String,
    pub(super) receipt_field: String,
    pub(super) description: String,
}

pub(super) struct ReplaySelection {
    pub(super) cases: Vec<ReplayCase>,
    pub(super) selected_groups: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct ReplayCase {
    pub(crate) id: String,
    pub(super) command_kind: String,
    pub(super) args: Vec<String>,
    pub(super) fixture_root: String,
    pub(super) comparison_modes: Vec<String>,
    pub(super) kind: ReplayCaseKind,
    pub(super) hash_fields: Vec<String>,
    pub(super) display_only_paths: Vec<String>,
    pub(super) normalization_rules: Vec<String>,
}
