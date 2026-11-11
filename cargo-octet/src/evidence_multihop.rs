//! Bounded typed multi-hop evidence-query experiment.
//!
//! This module is opt-in. It does not change `cargo octet check`, the existing
//! `query_evidence_index` one-expansion query, artifact discovery, process execution,
//! or publication. Callers hand it an already-built evidence codebase index and a
//! bounded reachability request.
//!
//! The experiment keeps a typed-edge breadth-first reference implementation and a
//! fixed-rule Datalog-equivalent evaluator over one finite snapshot. Both return the
//! same typed reachability facts when given the same request. Relation semantics stay
//! distinct: explicit parent links, provenance references, asserted dependency/support,
//! shared requirement/policy/lint labels, and content identity. Shared labels are
//! relatedness only; content identity is not proof; an asserted-support relation that
//! the evidence index does not carry is reported unavailable instead of invented.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use serde::Serialize;

use crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex;
use crate::valence_core_shim::evidence_codebase_index::EvidenceIndexIssueSeverity;

pub const MULTIHOP_SCHEMA_VERSION: &str = "octet-evidence-multihop-experiment/v1";

/// Claim boundary for every experiment report.
pub const MULTIHOP_BOUNDARY: &str = "Typed multi-hop query output proves only that the admitted relation facts in the named snapshot reach the listed record occurrences. It does not prove causal dependency, whole-program correctness, semantic equivalence, complete dependency closure, distributed execution correctness, remote code mobility, release eligibility, or formal proof. Shared requirement, policy, and lint labels are relatedness only; content identity is not proof; an unavailable asserted dependency/support relation is not implied.";

pub const DEFAULT_MAX_DEPTH: usize = 8;
pub const DEFAULT_MAX_FACTS: usize = 1024;
pub const DEFAULT_MAX_TUPLES: usize = 512;
pub const DEFAULT_MAX_WORK: usize = 8192;
pub const DEFAULT_MAX_MEMORY_BYTES: usize = 8 * 1024 * 1024;
pub const DEFAULT_MAX_OUTPUT: usize = 256;

const OCCURRENCE_MEMORY_BYTES: usize = 160;
const RELATION_MEMORY_BYTES: usize = 96;
const REACHED_MEMORY_BYTES: usize = 128;
const WITNESS_MEMORY_BYTES: usize = 192;

/// Relation categories admitted by the experiment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    /// Explicit `parent_links` value naming a parent artifact, stable ID, or record ID.
    ParentLink,
    /// Explicit `provenance_source_ids` reference to another record's stable source ID.
    ProvenanceSource,
    /// Asserted dependency/support relation. The base index model has no such field.
    AssertedSupport,
    /// Shared requirement ID.
    SharedRequirement,
    /// Shared policy ID.
    SharedPolicy,
    /// Shared lint ID.
    SharedLint,
    /// Shared artifact hash or fingerprint.
    ContentIdentity,
}

impl RelationKind {
    pub const ALL: [Self; 7] = [
        Self::ParentLink,
        Self::ProvenanceSource,
        Self::AssertedSupport,
        Self::SharedRequirement,
        Self::SharedPolicy,
        Self::SharedLint,
        Self::ContentIdentity,
    ];

    #[must_use]
    pub fn semantics(self) -> RelationSemantics {
        match self {
            Self::ParentLink => RelationSemantics::ParentLink,
            Self::ProvenanceSource => RelationSemantics::ProvenanceReference,
            Self::AssertedSupport => RelationSemantics::AssertedSupport,
            Self::SharedRequirement | Self::SharedPolicy | Self::SharedLint => RelationSemantics::SharedLabel,
            Self::ContentIdentity => RelationSemantics::ContentIdentity,
        }
    }
}

/// The meaning an admitted relation may carry in an explanation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationSemantics {
    /// Explicit parent/child containment recorded by the source artifact.
    ParentLink,
    /// Explicit provenance source reference recorded by the source artifact.
    ProvenanceReference,
    /// Asserted dependency/support edge. No base-index source supplies this today.
    AssertedSupport,
    /// Shared label. Relatedness only, never causal dependency.
    SharedLabel,
    /// Shared content identity. Identity only, never proof support.
    ContentIdentity,
}

impl RelationSemantics {
    /// Only an asserted dependency/support edge may establish a dependency claim.
    #[must_use]
    pub fn establishes_dependency(self) -> bool {
        matches!(self, Self::AssertedSupport)
    }

    /// Shared labels describe relatedness, not causality.
    #[must_use]
    pub fn is_relatedness(self) -> bool {
        matches!(self, Self::SharedLabel)
    }

    /// No relation family in this experiment supplies proof support.
    #[must_use]
    pub fn proof_support(self) -> bool {
        let _ = self;
        false
    }
}

/// Exactly which snapshot field and value produced a relation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RelationProvenance {
    pub field: String,
    pub value: String,
}

/// One indexed record occurrence in the finite snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SnapshotOccurrence {
    pub occurrence: usize,
    pub record_id: String,
    pub stable_id: String,
    pub artifact_family: String,
    pub artifact_locator: String,
    pub artifact_blake3: String,
}

/// One exact artifact revision observed in the snapshot.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SourceRevision {
    pub artifact_locator: String,
    pub artifact_blake3: String,
}

/// One typed relation occurrence.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SnapshotRelation {
    pub from: usize,
    pub to: usize,
    pub kind: RelationKind,
    pub provenance: RelationProvenance,
}

/// Whether the snapshot carries facts for a relation family.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RelationAvailability {
    Available { fact_count: usize },
    Unavailable { reason: String },
}

pub const ASSERTED_SUPPORT_UNAVAILABLE: &str = "evidence index records expose parent links, source references, shared labels, and content identity but no asserted dependency/support field; an asserted-support query is unavailable rather than inferred";

/// Explicit snapshot completeness declaration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SnapshotCompleteness {
    DeclaredComplete,
    DeclaredPartial { reason: String },
    Undeclared,
}

/// Snapshot-level facts that limit what the explanation can claim.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SnapshotGap {
    MissingIndexHash,
    SnapshotCompletenessUndeclared,
    DeclaredPartial {
        reason: String,
    },
    IndexIssue {
        severity: EvidenceIndexIssueSeverity,
        code: String,
        artifact_locator: String,
        detail: String,
    },
    MissingSourceRevision {
        occurrence: usize,
        artifact_locator: String,
    },
    ConflictingSourceRevision {
        artifact_locator: String,
        revisions: Vec<String>,
    },
    DuplicateStableId {
        stable_id: String,
        occurrences: Vec<usize>,
    },
    UnknownEndpoint {
        from: usize,
        relation: RelationKind,
        reference: String,
    },
    AmbiguousEndpoint {
        from: usize,
        relation: RelationKind,
        reference: String,
        candidates: Vec<usize>,
    },
    UnknownStart {
        reference: String,
    },
    AmbiguousStart {
        reference: String,
        candidates: Vec<usize>,
    },
}

/// A finite adapter over one evidence codebase index revision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EvidenceSnapshot {
    pub schema_version: String,
    pub snapshot_id: String,
    pub index_hash: String,
    pub index_schema_version: String,
    pub normalization_version: String,
    pub index_boundary: String,
    pub completeness: SnapshotCompleteness,
    pub source_revisions: Vec<SourceRevision>,
    pub occurrences: Vec<SnapshotOccurrence>,
    pub relations: Vec<SnapshotRelation>,
    pub availability: Vec<RelationAvailability>,
    pub gaps: Vec<SnapshotGap>,
    #[serde(skip)]
    endpoint_index: BTreeMap<String, Vec<usize>>,
}

enum EndpointResolution {
    Resolved(usize),
    Unknown,
    Ambiguous(Vec<usize>),
}

impl EvidenceSnapshot {
    /// Builds a snapshot without a completeness declaration.
    #[must_use]
    pub fn from_index(index: &EvidenceCodebaseIndex) -> Self {
        Self::from_index_declared(index, SnapshotCompleteness::Undeclared)
    }

    /// Builds a snapshot with an explicit completeness declaration.
    #[must_use]
    pub fn from_index_declared(index: &EvidenceCodebaseIndex, completeness: SnapshotCompleteness) -> Self {
        let mut gaps = snapshot_declaration_gaps(index, &completeness);
        let occurrences = index
            .records
            .iter()
            .enumerate()
            .map(|(occurrence, record)| SnapshotOccurrence {
                occurrence,
                record_id: record.record_id.clone(),
                stable_id: record.stable_id.clone(),
                artifact_family: record.artifact_family.clone(),
                artifact_locator: record.artifact_locator.clone(),
                artifact_blake3: record.artifact_blake3.clone(),
            })
            .collect::<Vec<_>>();
        let endpoint_index = build_endpoint_index(&occurrences);
        let stable_endpoint_index = build_stable_endpoint_index(&occurrences);
        collect_duplicate_stable_ids(&occurrences, &mut gaps);
        let source_revisions = source_revisions(&occurrences, &mut gaps);
        let mut relations = Vec::new();
        collect_parent_relations(index, &endpoint_index, &mut relations, &mut gaps);
        collect_provenance_relations(index, &stable_endpoint_index, &mut relations, &mut gaps);
        collect_identity_relations(index, &mut relations);
        collect_shared_label_relations(index, &mut relations);
        relations.sort();
        relations.dedup();
        let availability = relation_availability(&relations);
        gaps.sort();
        gaps.dedup();

        Self {
            schema_version: MULTIHOP_SCHEMA_VERSION.to_owned(),
            snapshot_id: if index.index_hash.is_empty() {
                derived_snapshot_id(index)
            } else {
                index.index_hash.clone()
            },
            index_hash: index.index_hash.clone(),
            index_schema_version: index.schema_version.clone(),
            normalization_version: index.normalization_version.clone(),
            index_boundary: index.boundary.clone(),
            completeness,
            source_revisions,
            occurrences,
            relations,
            availability,
            gaps,
            endpoint_index,
        }
    }

    fn resolve(&self, reference: &str) -> EndpointResolution {
        resolve_in(&self.endpoint_index, reference)
    }
}

fn derived_snapshot_id(index: &EvidenceCodebaseIndex) -> String {
    let payload = serde_json::to_vec(&index.records).unwrap_or_default();
    format!("derived-snapshot-{}", blake3::hash(&payload).to_hex())
}

fn snapshot_declaration_gaps(index: &EvidenceCodebaseIndex, completeness: &SnapshotCompleteness) -> Vec<SnapshotGap> {
    let mut gaps = Vec::new();
    if index.index_hash.is_empty() {
        gaps.push(SnapshotGap::MissingIndexHash);
    }
    match completeness {
        SnapshotCompleteness::DeclaredComplete => {}
        SnapshotCompleteness::DeclaredPartial { reason } => {
            gaps.push(SnapshotGap::DeclaredPartial { reason: reason.clone() });
        }
        SnapshotCompleteness::Undeclared => gaps.push(SnapshotGap::SnapshotCompletenessUndeclared),
    }
    for issue in &index.issues {
        gaps.push(SnapshotGap::IndexIssue {
            severity: issue.severity,
            code: issue.code.clone(),
            artifact_locator: issue.artifact_locator.clone(),
            detail: issue.detail.clone(),
        });
    }
    gaps
}

fn build_endpoint_index(occurrences: &[SnapshotOccurrence]) -> BTreeMap<String, Vec<usize>> {
    let mut index = BTreeMap::<String, BTreeSet<usize>>::new();
    for occurrence in occurrences {
        for identity in [
            &occurrence.record_id,
            &occurrence.stable_id,
            &occurrence.artifact_locator,
        ] {
            if !identity.is_empty() {
                index.entry(identity.clone()).or_default().insert(occurrence.occurrence);
            }
        }
    }
    index.into_iter().map(|(identity, positions)| (identity, positions.into_iter().collect())).collect()
}

fn build_stable_endpoint_index(occurrences: &[SnapshotOccurrence]) -> BTreeMap<String, Vec<usize>> {
    let mut index = BTreeMap::<String, BTreeSet<usize>>::new();
    for occurrence in occurrences {
        if !occurrence.stable_id.is_empty() {
            index.entry(occurrence.stable_id.clone()).or_default().insert(occurrence.occurrence);
        }
    }
    index.into_iter().map(|(identity, positions)| (identity, positions.into_iter().collect())).collect()
}

fn resolve_in(index: &BTreeMap<String, Vec<usize>>, reference: &str) -> EndpointResolution {
    match index.get(reference) {
        Some(candidates) if candidates.len() == 1 => EndpointResolution::Resolved(candidates[0]),
        Some(candidates) => EndpointResolution::Ambiguous(candidates.clone()),
        None => EndpointResolution::Unknown,
    }
}

fn collect_duplicate_stable_ids(occurrences: &[SnapshotOccurrence], gaps: &mut Vec<SnapshotGap>) {
    let mut groups = BTreeMap::<&str, Vec<usize>>::new();
    for occurrence in occurrences {
        if !occurrence.stable_id.is_empty() {
            groups.entry(occurrence.stable_id.as_str()).or_default().push(occurrence.occurrence);
        }
    }
    for (stable_id, positions) in groups {
        if positions.len() > 1 {
            gaps.push(SnapshotGap::DuplicateStableId {
                stable_id: stable_id.to_owned(),
                occurrences: positions,
            });
        }
    }
}

fn source_revisions(occurrences: &[SnapshotOccurrence], gaps: &mut Vec<SnapshotGap>) -> Vec<SourceRevision> {
    let mut revisions = BTreeMap::<String, BTreeSet<String>>::new();
    for occurrence in occurrences {
        if occurrence.artifact_blake3.is_empty() {
            gaps.push(SnapshotGap::MissingSourceRevision {
                occurrence: occurrence.occurrence,
                artifact_locator: occurrence.artifact_locator.clone(),
            });
        } else {
            revisions
                .entry(occurrence.artifact_locator.clone())
                .or_default()
                .insert(occurrence.artifact_blake3.clone());
        }
    }
    let mut output = Vec::new();
    for (artifact_locator, hashes) in revisions {
        if hashes.len() > 1 {
            gaps.push(SnapshotGap::ConflictingSourceRevision {
                artifact_locator: artifact_locator.clone(),
                revisions: hashes.iter().cloned().collect(),
            });
        }
        for artifact_blake3 in hashes {
            output.push(SourceRevision {
                artifact_locator: artifact_locator.clone(),
                artifact_blake3,
            });
        }
    }
    output
}

fn collect_parent_relations(
    index: &EvidenceCodebaseIndex,
    endpoint_index: &BTreeMap<String, Vec<usize>>,
    relations: &mut Vec<SnapshotRelation>,
    gaps: &mut Vec<SnapshotGap>,
) {
    for (from, record) in index.records.iter().enumerate() {
        for reference in &record.parent_links {
            match resolve_in(endpoint_index, reference) {
                EndpointResolution::Resolved(to) => relations.push(SnapshotRelation {
                    from,
                    to,
                    kind: RelationKind::ParentLink,
                    provenance: RelationProvenance {
                        field: "parent_links".to_owned(),
                        value: reference.clone(),
                    },
                }),
                EndpointResolution::Unknown => gaps.push(SnapshotGap::UnknownEndpoint {
                    from,
                    relation: RelationKind::ParentLink,
                    reference: reference.clone(),
                }),
                EndpointResolution::Ambiguous(candidates) => gaps.push(SnapshotGap::AmbiguousEndpoint {
                    from,
                    relation: RelationKind::ParentLink,
                    reference: reference.clone(),
                    candidates,
                }),
            }
        }
    }
}

fn collect_provenance_relations(
    index: &EvidenceCodebaseIndex,
    stable_endpoint_index: &BTreeMap<String, Vec<usize>>,
    relations: &mut Vec<SnapshotRelation>,
    gaps: &mut Vec<SnapshotGap>,
) {
    for (from, record) in index.records.iter().enumerate() {
        for reference in &record.provenance_source_ids {
            match resolve_in(stable_endpoint_index, reference) {
                EndpointResolution::Resolved(to) => relations.push(SnapshotRelation {
                    from,
                    to,
                    kind: RelationKind::ProvenanceSource,
                    provenance: RelationProvenance {
                        field: "provenance_source_ids".to_owned(),
                        value: reference.clone(),
                    },
                }),
                EndpointResolution::Unknown => gaps.push(SnapshotGap::UnknownEndpoint {
                    from,
                    relation: RelationKind::ProvenanceSource,
                    reference: reference.clone(),
                }),
                EndpointResolution::Ambiguous(candidates) => gaps.push(SnapshotGap::AmbiguousEndpoint {
                    from,
                    relation: RelationKind::ProvenanceSource,
                    reference: reference.clone(),
                    candidates,
                }),
            }
        }
    }
}

fn collect_identity_relations(index: &EvidenceCodebaseIndex, relations: &mut Vec<SnapshotRelation>) {
    let mut hashes = BTreeMap::<String, Vec<usize>>::new();
    let mut fingerprints = BTreeMap::<String, Vec<usize>>::new();
    for (occurrence, record) in index.records.iter().enumerate() {
        for value in &record.artifact_hashes {
            hashes.entry(value.clone()).or_default().push(occurrence);
        }
        for value in &record.fingerprints {
            fingerprints.entry(value.clone()).or_default().push(occurrence);
        }
    }
    push_pairwise(&hashes, RelationKind::ContentIdentity, "artifact_hashes", relations);
    push_pairwise(&fingerprints, RelationKind::ContentIdentity, "fingerprints", relations);
}

fn collect_shared_label_relations(index: &EvidenceCodebaseIndex, relations: &mut Vec<SnapshotRelation>) {
    let mut requirements = BTreeMap::<String, Vec<usize>>::new();
    let mut policies = BTreeMap::<String, Vec<usize>>::new();
    let mut lints = BTreeMap::<String, Vec<usize>>::new();
    for (occurrence, record) in index.records.iter().enumerate() {
        for value in &record.requirement_ids {
            requirements.entry(value.clone()).or_default().push(occurrence);
        }
        for value in &record.policy_ids {
            policies.entry(value.clone()).or_default().push(occurrence);
        }
        for value in &record.lint_ids {
            lints.entry(value.clone()).or_default().push(occurrence);
        }
    }
    push_pairwise(&requirements, RelationKind::SharedRequirement, "requirement_ids", relations);
    push_pairwise(&policies, RelationKind::SharedPolicy, "policy_ids", relations);
    push_pairwise(&lints, RelationKind::SharedLint, "lint_ids", relations);
}

fn push_pairwise(
    groups: &BTreeMap<String, Vec<usize>>,
    kind: RelationKind,
    field: &'static str,
    relations: &mut Vec<SnapshotRelation>,
) {
    for (value, occurrences) in groups {
        for &from in occurrences {
            for &to in occurrences {
                if from != to {
                    relations.push(SnapshotRelation {
                        from,
                        to,
                        kind,
                        provenance: RelationProvenance {
                            field: field.to_owned(),
                            value: value.clone(),
                        },
                    });
                }
            }
        }
    }
}

fn relation_availability(relations: &[SnapshotRelation]) -> Vec<RelationAvailability> {
    let mut counts = BTreeMap::<RelationKind, usize>::new();
    for relation in relations {
        *counts.entry(relation.kind).or_default() += 1;
    }
    RelationKind::ALL
        .iter()
        .map(|kind| {
            if *kind == RelationKind::AssertedSupport {
                RelationAvailability::Unavailable {
                    reason: ASSERTED_SUPPORT_UNAVAILABLE.to_owned(),
                }
            } else {
                RelationAvailability::Available {
                    fact_count: counts.get(kind).copied().unwrap_or(0),
                }
            }
        })
        .collect()
}

/// Bounded evaluator choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluatorKind {
    BfsReference,
    FixedRuleDatalog,
}

/// Fixed-rule identities instrumented by the Datalog-equivalent evaluator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleId {
    StartSeed,
    ReachExtension,
}

/// Fixed-rule premise and derivation counters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuleInstrumentation {
    pub rule: RuleId,
    pub premises: Vec<String>,
    pub evaluations: usize,
    pub derivations: usize,
}

/// Limits applied to one query.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct QueryLimits {
    pub max_depth: usize,
    pub max_facts: usize,
    pub max_tuples: usize,
    pub max_work: usize,
    pub max_memory_bytes: usize,
    pub max_output: usize,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            max_facts: DEFAULT_MAX_FACTS,
            max_tuples: DEFAULT_MAX_TUPLES,
            max_work: DEFAULT_MAX_WORK,
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            max_output: DEFAULT_MAX_OUTPUT,
        }
    }
}

/// One bounded query request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReachabilityRequest {
    pub start_identities: Vec<String>,
    pub admitted_relations: BTreeSet<RelationKind>,
    pub limits: QueryLimits,
}

impl ReachabilityRequest {
    #[must_use]
    pub fn new(start_identity: impl Into<String>) -> Self {
        Self {
            start_identities: vec![start_identity.into()],
            admitted_relations: BTreeSet::from([RelationKind::ParentLink]),
            limits: QueryLimits::default(),
        }
    }

    #[must_use]
    pub fn with_admitted_relations(mut self, relations: BTreeSet<RelationKind>) -> Self {
        self.admitted_relations = relations;
        self
    }

    #[must_use]
    pub fn with_limits(mut self, limits: QueryLimits) -> Self {
        self.limits = limits;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitKind {
    Depth,
    Facts,
    Tuples,
    Work,
    Memory,
    Output,
}

/// One observed bound. Only analysis-stopping bounds make reachability incomplete.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Truncation {
    pub limit: LimitKind,
    pub limit_value: usize,
    pub observed: usize,
    pub analysis_stopped: bool,
}

/// Why one record occurrence was reached.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReachReason {
    Origin,
    Typed {
        relation: RelationKind,
        semantics: RelationSemantics,
        provenance: RelationProvenance,
        depth: usize,
        establishes_dependency: bool,
        proof_support: bool,
    },
}

/// One reached record occurrence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReachedRecord {
    pub occurrence: usize,
    pub record_id: String,
    pub stable_id: String,
    pub artifact_locator: String,
    pub depth: usize,
    pub reason: ReachReason,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WitnessNode {
    pub occurrence: usize,
    pub stable_id: String,
    pub depth: usize,
    pub reason: ReachReason,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WitnessEdge {
    pub from: usize,
    pub to: usize,
    pub relation: RelationKind,
    pub semantics: RelationSemantics,
    pub provenance: RelationProvenance,
    pub depth: usize,
}

/// Why an edge closed back onto an already-reached occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosureKind {
    /// The target is an ancestor of the source in the selected witness tree.
    Cycle,
    /// The source and target are the same occurrence.
    SelfLoop,
    /// The edge closes a diamond or cross link without forming a cycle.
    DiamondClosure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClosureEdge {
    pub from: usize,
    pub to: usize,
    pub relation: RelationKind,
    pub semantics: RelationSemantics,
    pub provenance: RelationProvenance,
    pub depth: usize,
    pub closure: ClosureKind,
}

/// Deterministic selected witness, not an enumeration of every path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExplanationWitness {
    pub nodes: Vec<WitnessNode>,
    pub selected_edges: Vec<WitnessEdge>,
    pub closures: Vec<ClosureEdge>,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EvaluationCounters {
    pub facts_considered: usize,
    pub tuples_derived: usize,
    pub work_steps: usize,
    pub output_entries: usize,
    pub estimated_memory_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    Complete,
    Truncated,
    IncompleteSnapshot,
    TruncatedAndIncompleteSnapshot,
}

/// One explanation report. Every fact in it comes from the named snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExplanationReport {
    pub schema_version: String,
    pub snapshot_id: String,
    pub index_hash: String,
    pub index_boundary: String,
    pub boundary: String,
    pub evaluator: EvaluatorKind,
    pub query: ReachabilityRequest,
    pub status: AnalysisStatus,
    pub truncations: Vec<Truncation>,
    pub output_truncated: bool,
    pub gaps: Vec<SnapshotGap>,
    pub availability: Vec<RelationAvailability>,
    pub reached: Vec<ReachedRecord>,
    pub witness: ExplanationWitness,
    pub counters: EvaluationCounters,
    pub rules: Vec<RuleInstrumentation>,
}

/// Reuse of a runtime against a different snapshot is rejected instead of returning stale facts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeError {
    StaleSnapshot {
        expected_snapshot_id: String,
        actual_snapshot_id: String,
    },
}

/// Fixed-rule runtime bound to exactly one snapshot revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundedDatalogRuntime {
    snapshot_id: String,
    index_hash: String,
}

impl BoundedDatalogRuntime {
    /// Creates a fresh runtime for one snapshot.
    #[must_use]
    pub fn for_snapshot(snapshot: &EvidenceSnapshot) -> Self {
        Self {
            snapshot_id: snapshot.snapshot_id.clone(),
            index_hash: snapshot.index_hash.clone(),
        }
    }

    /// Evaluates one request. A different snapshot is rejected before any facts are read.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::StaleSnapshot`] when the runtime's bound snapshot identity
    /// differs from the supplied snapshot.
    pub fn evaluate(
        &mut self,
        snapshot: &EvidenceSnapshot,
        request: &ReachabilityRequest,
    ) -> Result<ExplanationReport, RuntimeError> {
        if self.snapshot_id != snapshot.snapshot_id || self.index_hash != snapshot.index_hash {
            return Err(RuntimeError::StaleSnapshot {
                expected_snapshot_id: self.snapshot_id.clone(),
                actual_snapshot_id: snapshot.snapshot_id.clone(),
            });
        }
        Ok(evaluate_fixed_rule_datalog(snapshot, request))
    }
}

/// Evaluates with the selected evaluator. The Datalog path always uses a fresh runtime.
///
/// # Errors
///
/// Returns [`RuntimeError::StaleSnapshot`] when the selected evaluator rejects a stale
/// snapshot.
pub fn evaluate(
    snapshot: &EvidenceSnapshot,
    request: &ReachabilityRequest,
    evaluator: EvaluatorKind,
) -> Result<ExplanationReport, RuntimeError> {
    match evaluator {
        EvaluatorKind::BfsReference => Ok(evaluate_typed_reference(snapshot, request)),
        EvaluatorKind::FixedRuleDatalog => {
            let mut runtime = BoundedDatalogRuntime::for_snapshot(snapshot);
            runtime.evaluate(snapshot, request)
        }
    }
}

/// Typed-edge breadth-first reference implementation.
#[must_use]
pub fn evaluate_typed_reference(snapshot: &EvidenceSnapshot, request: &ReachabilityRequest) -> ExplanationReport {
    run_traversal(snapshot, request, EvaluatorKind::BfsReference)
}

/// Fixed-rule Datalog-equivalent implementation with a fresh runtime.
#[must_use]
pub fn evaluate_fixed_rule_datalog(snapshot: &EvidenceSnapshot, request: &ReachabilityRequest) -> ExplanationReport {
    run_traversal(snapshot, request, EvaluatorKind::FixedRuleDatalog)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReachedNode {
    depth: usize,
    reason: ReachReason,
}

struct TraversalState<'a> {
    snapshot: &'a EvidenceSnapshot,
    request: &'a ReachabilityRequest,
    relations_from: BTreeMap<usize, Vec<usize>>,
    reached: BTreeMap<usize, ReachedNode>,
    witness_nodes: Vec<WitnessNode>,
    witness_edges: Vec<WitnessEdge>,
    closures: Vec<ClosureEdge>,
    closure_seen: BTreeSet<(usize, usize, RelationKind, String, String)>,
    predecessor: BTreeMap<usize, usize>,
    truncations: Vec<Truncation>,
    output_truncated: bool,
    output_entries: usize,
    counters: EvaluationCounters,
    gaps: Vec<SnapshotGap>,
    stopped: bool,
}

enum Discovery {
    Discovered(usize),
    AlreadyReached,
    NotAdmitted,
    Stopped,
}

impl<'a> TraversalState<'a> {
    fn new(snapshot: &'a EvidenceSnapshot, request: &'a ReachabilityRequest) -> Self {
        let mut relations_from = BTreeMap::<usize, Vec<usize>>::new();
        for (position, relation) in snapshot.relations.iter().enumerate() {
            relations_from.entry(relation.from).or_default().push(position);
        }
        let mut state = Self {
            snapshot,
            request,
            relations_from,
            reached: BTreeMap::new(),
            witness_nodes: Vec::new(),
            witness_edges: Vec::new(),
            closures: Vec::new(),
            closure_seen: BTreeSet::new(),
            predecessor: BTreeMap::new(),
            truncations: Vec::new(),
            output_truncated: false,
            output_entries: 0,
            counters: EvaluationCounters {
                facts_considered: 0,
                tuples_derived: 0,
                work_steps: 0,
                output_entries: 0,
                estimated_memory_bytes: 0,
            },
            gaps: Vec::new(),
            stopped: false,
        };
        state.counters.estimated_memory_bytes = state.memory_estimate();
        state.check_memory();
        state
    }

    fn memory_estimate(&self) -> usize {
        (self.snapshot.occurrences.len() * OCCURRENCE_MEMORY_BYTES)
            + (self.snapshot.relations.len() * RELATION_MEMORY_BYTES)
            + (self.reached.len() * REACHED_MEMORY_BYTES)
            + ((self.witness_nodes.len() + self.witness_edges.len() + self.closures.len()) * WITNESS_MEMORY_BYTES)
    }

    fn seed_starts(&mut self) -> Vec<usize> {
        if self.stopped {
            return Vec::new();
        }
        let mut starts = BTreeSet::new();
        for reference in &self.request.start_identities {
            match self.snapshot.resolve(reference) {
                EndpointResolution::Resolved(occurrence) => {
                    starts.insert(occurrence);
                }
                EndpointResolution::Unknown => {
                    self.gaps.push(SnapshotGap::UnknownStart {
                        reference: reference.clone(),
                    });
                }
                EndpointResolution::Ambiguous(candidates) => {
                    self.gaps.push(SnapshotGap::AmbiguousStart {
                        reference: reference.clone(),
                        candidates,
                    });
                }
            }
        }
        let mut origins = Vec::new();
        for occurrence in starts {
            if self.stopped {
                break;
            }
            if self.reached.len() >= self.request.limits.max_facts {
                self.truncate(LimitKind::Facts, self.reached.len() + 1);
                break;
            }
            self.reached.insert(occurrence, ReachedNode {
                depth: 0,
                reason: ReachReason::Origin,
            });
            self.counters.facts_considered = self.reached.len();
            origins.push(occurrence);
            self.push_witness_node(occurrence, 0, ReachReason::Origin);
            self.check_memory();
        }
        origins
    }

    fn ordered_relation_indices(&self, node: usize) -> Vec<usize> {
        self.relations_from.get(&node).cloned().unwrap_or_default()
    }

    fn note_work(&mut self) -> bool {
        self.counters.work_steps += 1;
        if self.counters.work_steps > self.request.limits.max_work {
            self.truncate(LimitKind::Work, self.counters.work_steps);
            return false;
        }
        true
    }

    fn try_discover(&mut self, from: usize, depth: usize, relation_index: usize) -> Discovery {
        let relation = &self.snapshot.relations[relation_index];
        let to = relation.to;
        let kind = relation.kind;
        let semantics = kind.semantics();
        let provenance = relation.provenance.clone();
        if self.reached.contains_key(&to) {
            self.note_closure(from, to, depth + 1, kind, semantics, provenance);
            return Discovery::AlreadyReached;
        }
        if depth + 1 > self.request.limits.max_depth {
            self.truncate(LimitKind::Depth, depth + 1);
            return Discovery::NotAdmitted;
        }
        if self.reached.len() >= self.request.limits.max_facts {
            self.truncate(LimitKind::Facts, self.reached.len() + 1);
            return Discovery::Stopped;
        }
        self.counters.tuples_derived += 1;
        if self.counters.tuples_derived > self.request.limits.max_tuples {
            self.truncate(LimitKind::Tuples, self.counters.tuples_derived);
            return Discovery::Stopped;
        }
        let reason = ReachReason::Typed {
            relation: kind,
            semantics,
            provenance: provenance.clone(),
            depth: depth + 1,
            establishes_dependency: semantics.establishes_dependency(),
            proof_support: semantics.proof_support(),
        };
        self.reached.insert(to, ReachedNode {
            depth: depth + 1,
            reason: reason.clone(),
        });
        self.predecessor.insert(to, from);
        self.counters.facts_considered = self.reached.len();
        self.push_witness_node(to, depth + 1, reason);
        self.push_witness_edge(WitnessEdge {
            from,
            to,
            relation: kind,
            semantics,
            provenance,
            depth: depth + 1,
        });
        self.check_memory();
        if self.stopped {
            Discovery::Stopped
        } else {
            Discovery::Discovered(to)
        }
    }

    fn push_witness_node(&mut self, occurrence: usize, depth: usize, reason: ReachReason) {
        if self.output_entries >= self.request.limits.max_output {
            self.truncate(LimitKind::Output, self.output_entries + 1);
            return;
        }
        let stable_id = self.snapshot.occurrences[occurrence].stable_id.clone();
        self.witness_nodes.push(WitnessNode {
            occurrence,
            stable_id,
            depth,
            reason,
        });
        self.output_entries += 1;
        self.counters.output_entries = self.output_entries;
    }

    fn push_witness_edge(&mut self, edge: WitnessEdge) {
        if self.output_entries >= self.request.limits.max_output {
            self.truncate(LimitKind::Output, self.output_entries + 1);
            return;
        }
        self.witness_edges.push(edge);
        self.output_entries += 1;
        self.counters.output_entries = self.output_entries;
    }

    fn note_closure(
        &mut self,
        from: usize,
        to: usize,
        depth: usize,
        relation: RelationKind,
        semantics: RelationSemantics,
        provenance: RelationProvenance,
    ) {
        let key = (from, to, relation, provenance.field.clone(), provenance.value.clone());
        if !self.closure_seen.insert(key) {
            return;
        }
        if self.output_entries >= self.request.limits.max_output {
            self.truncate(LimitKind::Output, self.output_entries + 1);
            return;
        }
        self.closures.push(ClosureEdge {
            from,
            to,
            relation,
            semantics,
            provenance,
            depth,
            closure: self.classify_closure(from, to),
        });
        self.output_entries += 1;
        self.counters.output_entries = self.output_entries;
    }

    fn classify_closure(&self, from: usize, to: usize) -> ClosureKind {
        if from == to {
            return ClosureKind::SelfLoop;
        }
        if self.is_ancestor(to, from) {
            ClosureKind::Cycle
        } else {
            ClosureKind::DiamondClosure
        }
    }

    fn is_ancestor(&self, candidate: usize, node: usize) -> bool {
        let mut cursor = Some(node);
        let mut steps = 0;
        while let Some(current) = cursor {
            if current == candidate {
                return true;
            }
            cursor = self.predecessor.get(&current).copied();
            steps += 1;
            if steps > self.reached.len() {
                return false;
            }
        }
        false
    }

    fn check_memory(&mut self) {
        self.counters.estimated_memory_bytes = self.memory_estimate();
        if self.counters.estimated_memory_bytes > self.request.limits.max_memory_bytes {
            self.truncate(LimitKind::Memory, self.counters.estimated_memory_bytes);
        }
    }

    fn truncate(&mut self, limit: LimitKind, observed: usize) {
        let analysis_stopped = limit != LimitKind::Output;
        if limit == LimitKind::Output {
            self.output_truncated = true;
        }
        if self.truncations.iter().any(|truncation| truncation.limit == limit) {
            if analysis_stopped {
                self.stopped = true;
            }
            return;
        }
        let limit_value = match limit {
            LimitKind::Depth => self.request.limits.max_depth,
            LimitKind::Facts => self.request.limits.max_facts,
            LimitKind::Tuples => self.request.limits.max_tuples,
            LimitKind::Work => self.request.limits.max_work,
            LimitKind::Memory => self.request.limits.max_memory_bytes,
            LimitKind::Output => self.request.limits.max_output,
        };
        self.truncations.push(Truncation {
            limit,
            limit_value,
            observed,
            analysis_stopped,
        });
        if analysis_stopped {
            self.stopped = true;
        }
    }

    fn finish(mut self, evaluator: EvaluatorKind, rules: Vec<RuleInstrumentation>) -> ExplanationReport {
        self.truncations.sort_by_key(|truncation| (truncation.limit, truncation.observed));
        let mut gaps = self.snapshot.gaps.clone();
        gaps.append(&mut self.gaps);
        gaps.sort();
        gaps.dedup();

        let mut reached = self
            .reached
            .iter()
            .map(|(occurrence, node)| {
                let record = &self.snapshot.occurrences[*occurrence];
                ReachedRecord {
                    occurrence: *occurrence,
                    record_id: record.record_id.clone(),
                    stable_id: record.stable_id.clone(),
                    artifact_locator: record.artifact_locator.clone(),
                    depth: node.depth,
                    reason: node.reason.clone(),
                }
            })
            .collect::<Vec<_>>();
        reached.sort_by_key(|record| (record.depth, record.occurrence));

        let mut nodes = self.witness_nodes;
        nodes.sort_by_key(|node| (node.depth, node.occurrence));
        let mut selected_edges = self.witness_edges;
        selected_edges.sort_by_key(|edge| (edge.depth, edge.from, edge.to, edge.relation));
        let mut closures = self.closures;
        closures.sort_by_key(|edge| (edge.depth, edge.from, edge.to, edge.relation));

        let incomplete = !gaps.is_empty();
        let analysis_truncated = self.truncations.iter().any(|truncation| truncation.analysis_stopped);
        let status = match (analysis_truncated, incomplete) {
            (false, false) => AnalysisStatus::Complete,
            (true, false) => AnalysisStatus::Truncated,
            (false, true) => AnalysisStatus::IncompleteSnapshot,
            (true, true) => AnalysisStatus::TruncatedAndIncompleteSnapshot,
        };

        ExplanationReport {
            schema_version: MULTIHOP_SCHEMA_VERSION.to_owned(),
            snapshot_id: self.snapshot.snapshot_id.clone(),
            index_hash: self.snapshot.index_hash.clone(),
            index_boundary: self.snapshot.index_boundary.clone(),
            boundary: MULTIHOP_BOUNDARY.to_owned(),
            evaluator,
            query: self.request.clone(),
            status,
            truncations: self.truncations,
            output_truncated: self.output_truncated,
            gaps,
            availability: self.snapshot.availability.clone(),
            reached,
            witness: ExplanationWitness {
                nodes,
                selected_edges,
                closures,
                truncated: self.output_truncated,
            },
            counters: self.counters,
            rules,
        }
    }
}

fn run_traversal(
    snapshot: &EvidenceSnapshot,
    request: &ReachabilityRequest,
    evaluator: EvaluatorKind,
) -> ExplanationReport {
    match evaluator {
        EvaluatorKind::BfsReference => run_bfs(snapshot, request),
        EvaluatorKind::FixedRuleDatalog => run_datalog(snapshot, request),
    }
}

fn run_bfs(snapshot: &EvidenceSnapshot, request: &ReachabilityRequest) -> ExplanationReport {
    let mut state = TraversalState::new(snapshot, request);
    let origins = state.seed_starts();
    let mut queue = VecDeque::<(usize, usize)>::new();
    for origin in origins {
        queue.push_back((origin, 0));
    }
    while let Some((node, depth)) = queue.pop_front() {
        if state.stopped {
            break;
        }
        for relation_index in state.ordered_relation_indices(node) {
            if state.stopped {
                break;
            }
            let kind = snapshot.relations[relation_index].kind;
            if !request.admitted_relations.contains(&kind) {
                continue;
            }
            if !state.note_work() {
                break;
            }
            match state.try_discover(node, depth, relation_index) {
                Discovery::Discovered(occurrence) => queue.push_back((occurrence, depth + 1)),
                Discovery::Stopped => break,
                Discovery::AlreadyReached | Discovery::NotAdmitted => {}
            }
        }
    }
    state.finish(EvaluatorKind::BfsReference, Vec::new())
}

fn run_datalog(snapshot: &EvidenceSnapshot, request: &ReachabilityRequest) -> ExplanationReport {
    let mut state = TraversalState::new(snapshot, request);
    let origins = state.seed_starts();
    let seed_derivations = origins.len();
    let mut frontier = origins;
    let mut depth = 0;
    while !frontier.is_empty() && !state.stopped {
        let mut next = Vec::new();
        for node in frontier {
            if state.stopped {
                break;
            }
            for relation_index in state.ordered_relation_indices(node) {
                if state.stopped {
                    break;
                }
                let kind = snapshot.relations[relation_index].kind;
                if !request.admitted_relations.contains(&kind) {
                    continue;
                }
                if !state.note_work() {
                    break;
                }
                match state.try_discover(node, depth, relation_index) {
                    Discovery::Discovered(occurrence) => next.push(occurrence),
                    Discovery::Stopped => break,
                    Discovery::AlreadyReached | Discovery::NotAdmitted => {}
                }
            }
        }
        frontier = next;
        depth += 1;
    }
    let rules = vec![
        RuleInstrumentation {
            rule: RuleId::StartSeed,
            premises: vec!["start(occurrence)".to_owned()],
            evaluations: request.start_identities.len(),
            derivations: seed_derivations,
        },
        RuleInstrumentation {
            rule: RuleId::ReachExtension,
            premises: vec![
                "reach(occurrence, depth)".to_owned(),
                "admitted_edge(kind, occurrence, next)".to_owned(),
                "depth + 1 <= max_depth".to_owned(),
            ],
            evaluations: state.counters.work_steps,
            derivations: state.counters.tuples_derived,
        },
    ];
    state.finish(EvaluatorKind::FixedRuleDatalog, rules)
}

#[cfg(test)]
mod tests {
    use valence_core::evidence_codebase_index::EvidenceIndexRecord;
    use valence_core::evidence_codebase_index::INDEX_BOUNDARY;
    use valence_core::evidence_codebase_index::INDEX_SCHEMA_VERSION;
    use valence_core::evidence_codebase_index::NORMALIZATION_VERSION;
    use valence_core::evidence_codebase_index::query_evidence_index;

    use super::*;

    #[derive(Clone, Default)]
    struct RecordSpec {
        family: &'static str,
        stable_id: &'static str,
        locator: &'static str,
        blake3: &'static str,
        parents: Vec<&'static str>,
        provenance: Vec<&'static str>,
        policies: Vec<&'static str>,
        requirements: Vec<&'static str>,
        fingerprints: Vec<&'static str>,
        hashes: Vec<&'static str>,
    }

    fn record(spec: &RecordSpec) -> EvidenceIndexRecord {
        EvidenceIndexRecord {
            record_id: format!("{}:{}:{}", spec.family, spec.stable_id, spec.locator),
            artifact_family: spec.family.to_owned(),
            stable_id: spec.stable_id.to_owned(),
            artifact_locator: spec.locator.to_owned(),
            artifact_blake3: spec.blake3.to_owned(),
            evidence_tier: "fixture".to_owned(),
            source_ids: Vec::new(),
            requirement_ids: spec.requirements.iter().map(|value| (*value).to_owned()).collect(),
            lint_ids: Vec::new(),
            policy_ids: spec.policies.iter().map(|value| (*value).to_owned()).collect(),
            fingerprints: spec.fingerprints.iter().map(|value| (*value).to_owned()).collect(),
            artifact_hashes: spec.hashes.iter().map(|value| (*value).to_owned()).collect(),
            provenance_source_ids: spec.provenance.iter().map(|value| (*value).to_owned()).collect(),
            replay_commands: Vec::new(),
            parent_links: spec.parents.iter().map(|value| (*value).to_owned()).collect(),
            caveats: Vec::new(),
            summary: String::new(),
        }
    }

    fn index(records: Vec<EvidenceIndexRecord>) -> EvidenceCodebaseIndex {
        let source_artifact_count =
            records.iter().map(|record| record.artifact_locator.as_str()).collect::<BTreeSet<_>>().len();
        let mut index = EvidenceCodebaseIndex {
            schema_version: INDEX_SCHEMA_VERSION.to_owned(),
            normalization_version: NORMALIZATION_VERSION.to_owned(),
            index_hash: String::new(),
            boundary: INDEX_BOUNDARY.to_owned(),
            source_artifact_count,
            record_count: records.len(),
            records,
            issues: Vec::new(),
        };
        let payload = serde_json::to_string(&index.records).expect("serialize fixture records");
        index.index_hash = blake3::hash(payload.as_bytes()).to_hex().to_string();
        index
    }

    fn complete_snapshot(index: &EvidenceCodebaseIndex) -> EvidenceSnapshot {
        EvidenceSnapshot::from_index_declared(index, SnapshotCompleteness::DeclaredComplete)
    }

    fn chain_index() -> EvidenceCodebaseIndex {
        index(vec![
            record(&RecordSpec {
                family: "octet-evidence-rail-finding",
                stable_id: "rail-1#rule",
                locator: "artifacts/rail-findings.jsonl",
                blake3: "hash-a",
                parents: vec!["rail-1"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-evidence-rail-receipt",
                stable_id: "rail-1",
                locator: "artifacts/rail-receipt.json",
                blake3: "hash-b",
                parents: vec!["artifacts/evidence-log.json"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-evidence-log",
                stable_id: "artifacts/evidence-log.json",
                locator: "artifacts/evidence-log.json",
                blake3: "hash-c",
                parents: vec!["prov-1"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-provenance",
                stable_id: "prov-1",
                locator: "artifacts/provenance.jsonl",
                blake3: "hash-d",
                ..RecordSpec::default()
            }),
        ])
    }

    fn long_chain_index(length: usize) -> EvidenceCodebaseIndex {
        const NAMES: [&str; 6] = ["a", "b", "c", "d", "e", "f"];
        const LOCATORS: [&str; 6] = [
            "artifacts/a.json",
            "artifacts/b.json",
            "artifacts/c.json",
            "artifacts/d.json",
            "artifacts/e.json",
            "artifacts/f.json",
        ];
        const HASHES: [&str; 6] = ["hash-a", "hash-b", "hash-c", "hash-d", "hash-e", "hash-f"];
        let records = (0..length)
            .map(|position| {
                let parents = if position + 1 < length {
                    vec![NAMES[position + 1]]
                } else {
                    Vec::new()
                };
                record(&RecordSpec {
                    family: "octet-noise",
                    stable_id: NAMES[position],
                    locator: LOCATORS[position],
                    blake3: HASHES[position],
                    parents,
                    ..RecordSpec::default()
                })
            })
            .collect();
        index(records)
    }

    fn reached_ids(report: &ExplanationReport) -> Vec<&str> {
        report.reached.iter().map(|record| record.stable_id.as_str()).collect()
    }

    fn selected_edges(report: &ExplanationReport) -> Vec<(usize, usize, RelationKind, String)> {
        report
            .witness
            .selected_edges
            .iter()
            .map(|edge| (edge.from, edge.to, edge.relation, edge.provenance.value.clone()))
            .collect()
    }

    fn assert_reports_agree(left: &ExplanationReport, right: &ExplanationReport) {
        assert_eq!(reached_ids(left), reached_ids(right));
        assert_eq!(selected_edges(left), selected_edges(right));
        assert_eq!(left.witness.closures, right.witness.closures);
        assert_eq!(left.status, right.status);
    }

    #[test]
    fn multi_hop_chain_extends_beyond_the_one_expansion_query() {
        let index = chain_index();
        let baseline = query_evidence_index(&index, "rail-1#rule");
        assert_eq!(baseline.match_count, 2);
        assert_eq!(
            baseline.records.iter().map(|record| record.stable_id.as_str()).collect::<Vec<_>>(),
            vec!["rail-1#rule", "rail-1"],
            "the existing query must remain a one-expansion compatibility baseline"
        );

        let snapshot = complete_snapshot(&index);
        let report = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("rail-1#rule"));
        assert_eq!(report.status, AnalysisStatus::Complete);
        assert_eq!(reached_ids(&report), vec!["rail-1#rule", "rail-1", "artifacts/evidence-log.json", "prov-1"]);

        let expected = vec![
            (0, 1, RelationKind::ParentLink, "rail-1".to_owned()),
            (1, 2, RelationKind::ParentLink, "artifacts/evidence-log.json".to_owned()),
            (2, 3, RelationKind::ParentLink, "prov-1".to_owned()),
        ];
        assert_eq!(selected_edges(&report), expected);
        for edge in &report.witness.selected_edges {
            let source = &index.records[edge.from];
            let target = &index.records[edge.to];
            assert!(source.parent_links.contains(&edge.provenance.value));
            assert_eq!(edge.provenance.field, "parent_links");
            assert_eq!(edge.provenance.value, target.stable_id.clone());
        }
    }

    #[test]
    fn fixed_rule_datalog_agrees_with_the_typed_reference_on_a_chain() {
        let index = chain_index();
        let snapshot = complete_snapshot(&index);
        let request = ReachabilityRequest::new("rail-1#rule");
        let reference = evaluate_typed_reference(&snapshot, &request);
        let datalog = evaluate_fixed_rule_datalog(&snapshot, &request);
        assert_eq!(datalog.evaluator, EvaluatorKind::FixedRuleDatalog);
        assert_reports_agree(&reference, &datalog);
        assert_eq!(datalog.rules.len(), 2);
        assert_eq!(datalog.rules[0].rule, RuleId::StartSeed);
        assert_eq!(datalog.rules[1].rule, RuleId::ReachExtension);
        assert!(datalog.rules[1].premises.iter().any(|premise| premise.contains("admitted_edge")));
        assert!(datalog.rules[1].derivations >= 3);
        assert_eq!(reference.counters.work_steps, datalog.counters.work_steps);
        assert_eq!(reference.counters.tuples_derived, datalog.counters.tuples_derived);
        assert_eq!(reference.counters.estimated_memory_bytes, datalog.counters.estimated_memory_bytes);
        assert_eq!(reference.counters.work_steps, 3);
        assert_eq!(reference.counters.tuples_derived, 3);
    }

    #[test]
    fn multiple_relation_families_agree_between_evaluators() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.jsonl",
                blake3: "hash-a",
                parents: vec!["b"],
                policies: vec!["policy.shared"],
                hashes: vec!["hash.shared"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-evidence-log",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                provenance: vec!["prov-1"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-provenance",
                stable_id: "prov-1",
                locator: "artifacts/provenance.jsonl",
                blake3: "hash-c",
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-fingerprint-review",
                stable_id: "d",
                locator: "artifacts/d.json",
                blake3: "hash-d",
                hashes: vec!["hash.shared"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octettranscript",
                stable_id: "e",
                locator: "artifacts/e.md",
                blake3: "hash-e",
                policies: vec!["policy.shared"],
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        assert!(snapshot.gaps.is_empty());
        let request = ReachabilityRequest::new("a").with_admitted_relations(BTreeSet::from([
            RelationKind::ParentLink,
            RelationKind::ProvenanceSource,
            RelationKind::SharedPolicy,
            RelationKind::ContentIdentity,
        ]));
        let reference = evaluate_typed_reference(&snapshot, &request);
        let datalog = evaluate_fixed_rule_datalog(&snapshot, &request);
        assert_reports_agree(&reference, &datalog);
        assert!(reached_ids(&reference).contains(&"prov-1"));
        assert!(reached_ids(&reference).contains(&"d"));
        assert!(reached_ids(&reference).contains(&"e"));
        assert!(reference.reached.iter().any(|record| {
            matches!(&record.reason, ReachReason::Typed {
                semantics: RelationSemantics::SharedLabel,
                ..
            })
        }));
    }

    #[test]
    fn diamond_selects_one_witness_and_classifies_the_closure() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                parents: vec!["b", "c"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                parents: vec!["d"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "c",
                locator: "artifacts/c.json",
                blake3: "hash-c",
                parents: vec!["d"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "d",
                locator: "artifacts/d.json",
                blake3: "hash-d",
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        let request = ReachabilityRequest::new("a");
        let first = evaluate_typed_reference(&snapshot, &request);
        let second = evaluate_typed_reference(&snapshot, &request);
        assert_eq!(first, second);
        assert_eq!(reached_ids(&first), vec!["a", "b", "c", "d"]);
        assert_eq!(first.witness.selected_edges.len(), 3);
        assert_eq!(first.witness.closures.len(), 1);
        assert_eq!(first.witness.closures[0].from, 2);
        assert_eq!(first.witness.closures[0].to, 3);
        assert_eq!(first.witness.closures[0].closure, ClosureKind::DiamondClosure);
        let datalog = evaluate_fixed_rule_datalog(&snapshot, &request);
        assert_reports_agree(&first, &datalog);
    }

    #[test]
    fn cycles_self_loops_and_self_reachability_are_explicit_and_bounded() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                parents: vec!["b"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                parents: vec!["c"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "c",
                locator: "artifacts/c.json",
                blake3: "hash-c",
                parents: vec!["a"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "self",
                locator: "artifacts/self.json",
                blake3: "hash-self",
                parents: vec!["self"],
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        let report = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("a"));
        assert_eq!(report.status, AnalysisStatus::Complete);
        assert_eq!(reached_ids(&report), vec!["a", "b", "c"]);
        assert_eq!(report.witness.closures.len(), 1);
        assert_eq!(report.witness.closures[0].closure, ClosureKind::Cycle);
        assert_eq!((report.witness.closures[0].from, report.witness.closures[0].to), (2, 0));

        let self_report = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("self"));
        assert_eq!(self_report.status, AnalysisStatus::Complete);
        assert_eq!(reached_ids(&self_report), vec!["self"]);
        assert_eq!(self_report.witness.nodes[0].reason, ReachReason::Origin);
        assert_eq!(self_report.witness.closures.len(), 1);
        assert_eq!(self_report.witness.closures[0].closure, ClosureKind::SelfLoop);
        assert_reports_agree(&report, &evaluate_fixed_rule_datalog(&snapshot, &ReachabilityRequest::new("a")));
        assert_reports_agree(&self_report, &evaluate_fixed_rule_datalog(&snapshot, &ReachabilityRequest::new("self")));
    }

    #[test]
    fn shared_policy_labels_are_relatedness_not_dependency() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                policies: vec!["policy.shared"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                policies: vec!["policy.shared"],
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        let impact = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("a"));
        assert_eq!(reached_ids(&impact), vec!["a"], "a shared policy label must not create a causal edge");
        assert!(impact.witness.selected_edges.is_empty());

        let related =
            ReachabilityRequest::new("a").with_admitted_relations(BTreeSet::from([RelationKind::SharedPolicy]));
        let related_report = evaluate_typed_reference(&snapshot, &related);
        assert_eq!(reached_ids(&related_report), vec!["a", "b"]);
        let reason = related_report
            .reached
            .iter()
            .find(|record| record.stable_id == "b")
            .map(|record| record.reason.clone())
            .expect("shared-label record");
        assert!(matches!(reason, ReachReason::Typed {
            relation: RelationKind::SharedPolicy,
            semantics: RelationSemantics::SharedLabel,
            establishes_dependency: false,
            proof_support: false,
            ..
        }));
        assert!(related_report.reached.iter().all(|record| !matches!(record.reason, ReachReason::Typed {
            establishes_dependency: true,
            ..
        })));
    }

    #[test]
    fn shared_content_identity_is_not_proof_support() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                hashes: vec!["hash.shared"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                hashes: vec!["hash.shared"],
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        let request =
            ReachabilityRequest::new("a").with_admitted_relations(BTreeSet::from([RelationKind::ContentIdentity]));
        let report = evaluate_typed_reference(&snapshot, &request);
        assert_eq!(reached_ids(&report), vec!["a", "b"]);
        assert!(matches!(report.reached[1].reason, ReachReason::Typed {
            relation: RelationKind::ContentIdentity,
            semantics: RelationSemantics::ContentIdentity,
            establishes_dependency: false,
            proof_support: false,
            ..
        }));
        assert_eq!(report.witness.closures.len(), 1, "the reverse identity edge closes once");
    }

    #[test]
    fn asserted_support_is_reported_unavailable_without_inventing_edges() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        let request =
            ReachabilityRequest::new("a").with_admitted_relations(BTreeSet::from([RelationKind::AssertedSupport]));
        let report = evaluate_typed_reference(&snapshot, &request);
        assert_eq!(reached_ids(&report), vec!["a"]);
        let availability = report
            .availability
            .iter()
            .find(|entry| matches!(entry, RelationAvailability::Unavailable { .. }))
            .expect("asserted support unavailable");
        if let RelationAvailability::Unavailable { reason } = availability {
            assert_eq!(reason, ASSERTED_SUPPORT_UNAVAILABLE);
        }
    }

    #[test]
    fn unknown_and_ambiguous_endpoints_are_explicit_gaps() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                parents: vec!["artifacts/missing.json"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                parents: vec!["artifacts/shared.json"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "c",
                locator: "artifacts/shared.json",
                blake3: "hash-shared",
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "d",
                locator: "artifacts/shared.json",
                blake3: "hash-shared",
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        let report = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("b"));
        assert_eq!(reached_ids(&report), vec!["b"]);
        assert_eq!(report.status, AnalysisStatus::IncompleteSnapshot);
        assert!(report.gaps.iter().any(
            |gap| matches!(gap, SnapshotGap::UnknownEndpoint { reference, .. } if reference == "artifacts/missing.json")
        ));
        assert!(report.gaps.iter().any(|gap| matches!(
            gap,
            SnapshotGap::AmbiguousEndpoint { reference, candidates, .. }
                if reference == "artifacts/shared.json" && candidates.len() == 2
        )));
    }

    #[test]
    fn duplicate_occurrences_and_unknown_starts_are_not_merged() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "dup",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "dup",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        assert!(snapshot.gaps.iter().any(|gap| matches!(gap, SnapshotGap::DuplicateStableId { stable_id, occurrences } if stable_id == "dup" && occurrences.len() == 2)));
        let report = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("dup"));
        assert!(report.reached.is_empty());
        assert_eq!(report.status, AnalysisStatus::IncompleteSnapshot);
        assert!(
            report
                .gaps
                .iter()
                .any(|gap| matches!(gap, SnapshotGap::AmbiguousStart { reference, .. } if reference == "dup"))
        );

        let unknown = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("not-present"));
        assert!(unknown.reached.is_empty());
        assert!(
            unknown
                .gaps
                .iter()
                .any(|gap| matches!(gap, SnapshotGap::UnknownStart { reference } if reference == "not-present"))
        );
    }

    #[test]
    fn partial_and_undeclared_snapshots_are_incomplete() {
        let index = index(vec![record(&RecordSpec {
            family: "octet-finding",
            stable_id: "a",
            locator: "artifacts/a.json",
            blake3: "hash-a",
            ..RecordSpec::default()
        })]);
        let undeclared = EvidenceSnapshot::from_index(&index);
        assert!(undeclared.gaps.contains(&SnapshotGap::SnapshotCompletenessUndeclared));
        let undeclared_report = evaluate_typed_reference(&undeclared, &ReachabilityRequest::new("a"));
        assert_eq!(undeclared_report.status, AnalysisStatus::IncompleteSnapshot);

        let partial = EvidenceSnapshot::from_index_declared(&index, SnapshotCompleteness::DeclaredPartial {
            reason: "fixture omits sibling artifacts".to_owned(),
        });
        assert!(
            partial
                .gaps
                .iter()
                .any(|gap| matches!(gap, SnapshotGap::DeclaredPartial { reason } if reason.contains("omits sibling")))
        );

        let complete = complete_snapshot(&index);
        assert!(complete.gaps.is_empty());
        let complete_report = evaluate_typed_reference(&complete, &ReachabilityRequest::new("a"));
        assert_eq!(complete_report.status, AnalysisStatus::Complete);
    }

    #[test]
    fn a_runtime_never_returns_stale_conclusions_for_a_replaced_revision() {
        let first_index = chain_index();
        let first_snapshot = complete_snapshot(&first_index);
        let mut runtime = BoundedDatalogRuntime::for_snapshot(&first_snapshot);
        let first_report = runtime
            .evaluate(&first_snapshot, &ReachabilityRequest::new("rail-1#rule"))
            .expect("first evaluation");
        assert!(reached_ids(&first_report).contains(&"prov-1"));

        let second_index = index(vec![
            record(&RecordSpec {
                family: "octet-evidence-rail-finding",
                stable_id: "rail-1#rule",
                locator: "artifacts/rail-findings.jsonl",
                blake3: "replaced-hash-a",
                parents: Vec::new(),
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-evidence-rail-receipt",
                stable_id: "rail-1",
                locator: "artifacts/rail-receipt.json",
                blake3: "hash-b",
                parents: Vec::new(),
                ..RecordSpec::default()
            }),
        ]);
        let second_snapshot = complete_snapshot(&second_index);
        let stale = runtime.evaluate(&second_snapshot, &ReachabilityRequest::new("rail-1#rule"));
        assert!(matches!(stale, Err(RuntimeError::StaleSnapshot { .. })));

        let mut fresh = BoundedDatalogRuntime::for_snapshot(&second_snapshot);
        let second_report = fresh
            .evaluate(&second_snapshot, &ReachabilityRequest::new("rail-1#rule"))
            .expect("fresh evaluation");
        assert_eq!(reached_ids(&second_report), vec!["rail-1#rule"]);
        assert_ne!(first_report.index_hash, second_report.index_hash);
        assert_ne!(first_report.snapshot_id, second_report.snapshot_id);
    }

    #[test]
    fn depth_facts_tuples_work_and_memory_limits_truncate_analysis() {
        let index = long_chain_index(6);
        let snapshot = complete_snapshot(&index);
        let base_request = ReachabilityRequest::new("a");

        let depth_limited = evaluate_typed_reference(
            &snapshot,
            &base_request.clone().with_limits(QueryLimits {
                max_depth: 2,
                ..QueryLimits::default()
            }),
        );
        assert_eq!(reached_ids(&depth_limited), vec!["a", "b", "c"]);
        assert_eq!(depth_limited.status, AnalysisStatus::Truncated);
        assert!(
            depth_limited
                .truncations
                .iter()
                .any(|truncation| truncation.limit == LimitKind::Depth && truncation.analysis_stopped)
        );

        let facts_limited = evaluate_typed_reference(
            &snapshot,
            &base_request.clone().with_limits(QueryLimits {
                max_facts: 2,
                ..QueryLimits::default()
            }),
        );
        assert_eq!(reached_ids(&facts_limited), vec!["a", "b"]);
        assert!(
            facts_limited
                .truncations
                .iter()
                .any(|truncation| truncation.limit == LimitKind::Facts && truncation.analysis_stopped)
        );

        let tuples_limited = evaluate_typed_reference(
            &snapshot,
            &base_request.clone().with_limits(QueryLimits {
                max_tuples: 1,
                ..QueryLimits::default()
            }),
        );
        assert_eq!(reached_ids(&tuples_limited), vec!["a", "b"]);
        assert!(
            tuples_limited
                .truncations
                .iter()
                .any(|truncation| truncation.limit == LimitKind::Tuples && truncation.analysis_stopped)
        );

        let work_limited = evaluate_typed_reference(
            &snapshot,
            &base_request.clone().with_limits(QueryLimits {
                max_work: 1,
                ..QueryLimits::default()
            }),
        );
        assert_eq!(reached_ids(&work_limited), vec!["a", "b"]);
        assert!(
            work_limited
                .truncations
                .iter()
                .any(|truncation| truncation.limit == LimitKind::Work && truncation.analysis_stopped)
        );

        let memory_limited = evaluate_typed_reference(
            &snapshot,
            &base_request.with_limits(QueryLimits {
                max_memory_bytes: 1,
                ..QueryLimits::default()
            }),
        );
        assert!(memory_limited.reached.is_empty());
        assert_eq!(memory_limited.status, AnalysisStatus::Truncated);
        assert!(
            memory_limited
                .truncations
                .iter()
                .any(|truncation| truncation.limit == LimitKind::Memory && truncation.analysis_stopped)
        );
    }

    #[test]
    fn output_truncation_is_distinct_from_incomplete_analysis() {
        let index = long_chain_index(4);
        let snapshot = complete_snapshot(&index);
        let request = ReachabilityRequest::new("a").with_limits(QueryLimits {
            max_output: 1,
            ..QueryLimits::default()
        });
        let report = evaluate_typed_reference(&snapshot, &request);
        assert_eq!(report.status, AnalysisStatus::Complete);
        assert!(report.output_truncated);
        assert!(report.witness.truncated);
        assert_eq!(reached_ids(&report), vec!["a", "b", "c", "d"]);
        assert_eq!(report.witness.nodes.len(), 1);
        assert!(
            report
                .truncations
                .iter()
                .any(|truncation| truncation.limit == LimitKind::Output && !truncation.analysis_stopped)
        );
        assert!(!report.truncations.iter().any(|truncation| truncation.analysis_stopped));
    }

    #[test]
    fn ordering_bindings_and_availability_are_stable() {
        let index = chain_index();
        let snapshot = complete_snapshot(&index);
        let request = ReachabilityRequest::new("rail-1#rule");
        let first = evaluate_typed_reference(&snapshot, &request);
        let second = evaluate_typed_reference(&snapshot, &request);
        assert_eq!(first, second);
        assert_eq!(first.schema_version, MULTIHOP_SCHEMA_VERSION);
        assert_eq!(first.snapshot_id, snapshot.snapshot_id);
        assert_eq!(first.index_hash, index.index_hash);
        assert_eq!(first.index_boundary, INDEX_BOUNDARY);
        assert_eq!(first.boundary, MULTIHOP_BOUNDARY);
        assert_eq!(first.query, request);
        assert_eq!(first.evaluator, EvaluatorKind::BfsReference);

        let parent_facts = first
            .availability
            .iter()
            .find(|entry| matches!(entry, RelationAvailability::Available { .. }))
            .expect("available relation");
        assert!(matches!(parent_facts, RelationAvailability::Available { fact_count: 3 }));
    }

    #[test]
    fn no_false_reach_without_an_admitted_relation() {
        let disconnected = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&disconnected);
        let negative = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("a"));
        assert_eq!(reached_ids(&negative), vec!["a"]);

        let connected = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/a.json",
                blake3: "hash-a",
                parents: vec!["b"],
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "b",
                locator: "artifacts/b.json",
                blake3: "hash-b",
                ..RecordSpec::default()
            }),
        ]);
        let positive_snapshot = complete_snapshot(&connected);
        let positive = evaluate_typed_reference(&positive_snapshot, &ReachabilityRequest::new("a"));
        assert_eq!(reached_ids(&positive), vec!["a", "b"]);
        assert_eq!(positive.witness.selected_edges.len(), 1);
    }

    #[test]
    fn conflicting_source_revisions_are_explicit() {
        let index = index(vec![
            record(&RecordSpec {
                family: "octet-finding",
                stable_id: "a",
                locator: "artifacts/shared.json",
                blake3: "hash-old",
                ..RecordSpec::default()
            }),
            record(&RecordSpec {
                family: "octet-provenance",
                stable_id: "b",
                locator: "artifacts/shared.json",
                blake3: "hash-new",
                ..RecordSpec::default()
            }),
        ]);
        let snapshot = complete_snapshot(&index);
        let conflicting = snapshot
            .gaps
            .iter()
            .find_map(|gap| match gap {
                SnapshotGap::ConflictingSourceRevision {
                    artifact_locator,
                    revisions,
                } => Some((artifact_locator.as_str(), revisions.clone())),
                _ => None,
            })
            .expect("conflicting revision gap");
        assert_eq!(conflicting.0, "artifacts/shared.json");
        assert_eq!(conflicting.1, vec!["hash-new".to_owned(), "hash-old".to_owned()]);
        let report = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("a"));
        assert_eq!(report.status, AnalysisStatus::IncompleteSnapshot);
    }

    #[test]
    fn every_witness_provenance_comes_from_the_selected_snapshot() {
        let index = chain_index();
        let snapshot = complete_snapshot(&index);
        let report = evaluate_typed_reference(&snapshot, &ReachabilityRequest::new("rail-1#rule"));
        let relation_keys = snapshot
            .relations
            .iter()
            .map(|relation| (relation.from, relation.to, relation.kind, relation.provenance.clone()))
            .collect::<BTreeSet<_>>();
        for edge in &report.witness.selected_edges {
            assert!(relation_keys.contains(&(edge.from, edge.to, edge.relation, edge.provenance.clone(),)));
        }
        assert_eq!(report.index_hash, snapshot.index_hash);
        assert_eq!(report.snapshot_id, snapshot.snapshot_id);
        assert!(report.index_boundary.contains("does not claim"));
    }

    #[test]
    fn an_unhashed_index_still_gets_a_distinct_snapshot_identity() {
        let mut first_index = index(vec![record(&RecordSpec {
            family: "octet-finding",
            stable_id: "a",
            locator: "artifacts/a.json",
            blake3: "hash-a",
            ..RecordSpec::default()
        })]);
        first_index.index_hash.clear();
        let first = EvidenceSnapshot::from_index(&first_index);
        assert!(first.snapshot_id.starts_with("derived-snapshot-"));
        assert!(first.gaps.contains(&SnapshotGap::MissingIndexHash));
        let mut runtime = BoundedDatalogRuntime::for_snapshot(&first);

        let mut second_index = first_index;
        second_index.records[0].summary = "changed bytes".to_owned();
        let second = EvidenceSnapshot::from_index(&second_index);
        assert_ne!(first.snapshot_id, second.snapshot_id);
        assert!(matches!(
            runtime.evaluate(&second, &ReachabilityRequest::new("a")),
            Err(RuntimeError::StaleSnapshot { .. })
        ));
    }

    #[test]
    fn output_truncation_does_not_hide_snapshot_incompleteness() {
        let index = long_chain_index(4);
        let partial = EvidenceSnapshot::from_index_declared(&index, SnapshotCompleteness::DeclaredPartial {
            reason: "only a chain was supplied".to_owned(),
        });
        let report = evaluate_typed_reference(
            &partial,
            &ReachabilityRequest::new("a").with_limits(QueryLimits {
                max_output: 1,
                ..QueryLimits::default()
            }),
        );
        assert_eq!(report.status, AnalysisStatus::IncompleteSnapshot);
        assert!(report.output_truncated);
        assert!(report.reached.len() == 4);
        assert!(report.gaps.iter().any(|gap| matches!(gap, SnapshotGap::DeclaredPartial { .. })));
    }
}
