//! Capability manifest checking and deterministic capability reports.

use serde::Deserialize;
use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::FamilySelection;
use crate::model::Mechanism;
use crate::model::MechanismKind;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::model::Severity;
use crate::source::SourceFile;

const ARCHITECTURE_FAMILY: &str = "architecture";
const EFFECT_CAPABILITY_POLICY_ID: &str = "tigerstyle.architecture.effect-capability-boundaries";
const RUST_SOURCE_EXTENSION: &str = ".rs";
const FIRST_LINE_NUMBER_OFFSET: usize = 1;
const UNKNOWN_CAPABILITY_FAMILY: &str = "unknown";
const DEFAULT_REPLAY_COMMAND: &str = "not-declared";
const POLICY_CHECK_EVIDENCE_TIER: &str = "policy_check";
const SOURCE_SHAPE_EVIDENCE_TIER: &str = "source_shape";
const HANDLER_EVIDENCE_TIER: &str = "deterministic_handler";
const OBSERVATION_DIRECT_EFFECT: &str = "direct-platform-effect";
const OBSERVATION_INJECTED_CAPABILITY: &str = "injected-capability";
const STATUS_ACCEPTED: &str = "accepted";
const STATUS_DIAGNOSTIC: &str = "diagnostic";
const SCOPE_CORE: &str = "core";
const SCOPE_SHELL: &str = "shell";
const SCOPE_ADAPTER: &str = "adapter";
const SCOPE_TEST: &str = "test";
const SCOPE_GENERATED: &str = "generated";
const SCOPE_REMOTE_EXECUTION: &str = "remote-execution";
const MODE_ADVISORY: &str = "advisory";
const MODE_STRICT: &str = "strict";
const SUCCESS_COVERAGE_MARKERS: &[&str] = &["success", "succeeds", "ok", "happy path"];
const FAILURE_COVERAGE_MARKERS: &[&str] = &["failure", "fails", "error", "sad path"];
const NON_CLAIMS: &[&str] = &[
    "validates declared policy and observed source shapes only",
    "does not add Rust effect types or ability syntax",
    "does not prove transitive effect inference or whole-program purity",
    "does not prove complete runtime-effect absence or semantic correctness",
];
const KNOWN_SCOPE_CLASSES: &[&str] = &[
    SCOPE_CORE,
    SCOPE_SHELL,
    SCOPE_ADAPTER,
    SCOPE_TEST,
    SCOPE_GENERATED,
    SCOPE_REMOTE_EXECUTION,
];
const KNOWN_CAPABILITY_FAMILIES: &[&str] = &[
    "time",
    "randomness",
    "filesystem",
    "network",
    "process",
    "environment",
    "secrets",
    "remote-execution",
    "unsafe-code",
    "global-state",
    "concurrency",
    "observability",
    UNKNOWN_CAPABILITY_FAMILY,
];
const TIME_LINT_IDS: &[&str] = &["ambient_clock", "contradictory_time"];
const RANDOMNESS_LINT_IDS: &[&str] = &["ambient_random"];
const FILESYSTEM_LINT_IDS: &[&str] = &["impure_call_in_core", "resource_return_from_core"];
const NETWORK_LINT_IDS: &[&str] = &["impure_call_in_core", "resource_return_from_core"];
const PROCESS_LINT_IDS: &[&str] = &["impure_call_in_core", "spawn_in_core"];
const ENVIRONMENT_LINT_IDS: &[&str] = &["ambient_env"];
const SECRETS_LINT_IDS: &[&str] = &["ambient_env"];
const REMOTE_EXECUTION_LINT_IDS: &[&str] = &["impure_call_in_core", "spawn_in_core"];
const UNSAFE_CODE_LINT_IDS: &[&str] = &["public_unsafe_api"];
const GLOBAL_STATE_LINT_IDS: &[&str] = &["global_state_in_core"];
const CONCURRENCY_LINT_IDS: &[&str] = &["spawn_in_core", "unbounded_channel", "ignored_join_handle"];
const OBSERVABILITY_LINT_IDS: &[&str] = &["logging_in_core"];
const UNKNOWN_LINT_IDS: &[&str] = &["unknown-capability"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EffectPattern {
    pattern: &'static str,
    family: &'static str,
    lint_ids: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EffectMatch {
    family: &'static str,
    lint_ids: &'static [&'static str],
}

const EFFECT_PATTERNS: &[EffectPattern] = &[
    EffectPattern {
        pattern: "SystemTime::now(",
        family: "time",
        lint_ids: TIME_LINT_IDS,
    },
    EffectPattern {
        pattern: "Instant::now(",
        family: "time",
        lint_ids: TIME_LINT_IDS,
    },
    EffectPattern {
        pattern: "std::time::",
        family: "time",
        lint_ids: TIME_LINT_IDS,
    },
    EffectPattern {
        pattern: "rand::",
        family: "randomness",
        lint_ids: RANDOMNESS_LINT_IDS,
    },
    EffectPattern {
        pattern: "getrandom::",
        family: "randomness",
        lint_ids: RANDOMNESS_LINT_IDS,
    },
    EffectPattern {
        pattern: "std::fs::",
        family: "filesystem",
        lint_ids: FILESYSTEM_LINT_IDS,
    },
    EffectPattern {
        pattern: "File::open(",
        family: "filesystem",
        lint_ids: FILESYSTEM_LINT_IDS,
    },
    EffectPattern {
        pattern: "std::net::",
        family: "network",
        lint_ids: NETWORK_LINT_IDS,
    },
    EffectPattern {
        pattern: "reqwest::",
        family: "network",
        lint_ids: NETWORK_LINT_IDS,
    },
    EffectPattern {
        pattern: "std::process::",
        family: "process",
        lint_ids: PROCESS_LINT_IDS,
    },
    EffectPattern {
        pattern: "std::env::",
        family: "environment",
        lint_ids: ENVIRONMENT_LINT_IDS,
    },
    EffectPattern {
        pattern: "Secret",
        family: "secrets",
        lint_ids: SECRETS_LINT_IDS,
    },
    EffectPattern {
        pattern: "remote_exec",
        family: "remote-execution",
        lint_ids: REMOTE_EXECUTION_LINT_IDS,
    },
    EffectPattern {
        pattern: "unsafe {",
        family: "unsafe-code",
        lint_ids: UNSAFE_CODE_LINT_IDS,
    },
    EffectPattern {
        pattern: "static mut",
        family: "global-state",
        lint_ids: GLOBAL_STATE_LINT_IDS,
    },
    EffectPattern {
        pattern: "thread::spawn",
        family: "concurrency",
        lint_ids: CONCURRENCY_LINT_IDS,
    },
    EffectPattern {
        pattern: "tokio::spawn",
        family: "concurrency",
        lint_ids: CONCURRENCY_LINT_IDS,
    },
    EffectPattern {
        pattern: "tracing::",
        family: "observability",
        lint_ids: OBSERVABILITY_LINT_IDS,
    },
    EffectPattern {
        pattern: "log::",
        family: "observability",
        lint_ids: OBSERVABILITY_LINT_IDS,
    },
];

/// Result of evaluating capability manifests for one checker run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilityManifestCheck {
    /// Blocking or advisory diagnostics produced by manifest validation.
    pub diagnostics: Vec<Diagnostic>,
    /// Deterministic human/JSON capability reports.
    pub reports: Vec<CapabilityReport>,
}

/// Capability report emitted by standards checks.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityReport {
    /// Owning policy ID.
    pub policy_id: String,
    /// Manifest enforcement mode, usually `strict` or `advisory`.
    pub enforcement_mode: String,
    /// Declared capability families in stable order.
    pub declared_capability_families: Vec<String>,
    /// Declared scope classes in stable order.
    pub declared_scope_classes: Vec<String>,
    /// Direct effects and injected capabilities observed in source shapes.
    pub observations: Vec<CapabilityObservation>,
    /// Deterministic handler artifacts declared by the manifest.
    pub handler_evidence: Vec<CapabilityHandlerEvidence>,
    /// Explicit report non-claims.
    pub non_claims: Vec<String>,
}

/// One capability observation in a report.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityObservation {
    /// Source path for the observation.
    pub path: String,
    /// One-based line number when available.
    pub line: Option<u32>,
    /// Resolved capability scope class.
    pub scope_class: String,
    /// Capability family associated with the observation.
    pub capability_family: String,
    /// Observation kind, such as `direct-platform-effect` or `injected-capability`.
    pub observation_kind: String,
    /// Matched pattern, trait, token, or capability name.
    pub matched: String,
    /// Lint IDs that inspect the same effect family when available.
    pub linked_lint_ids: Vec<String>,
    /// Evidence tier for this observation.
    pub evidence_tier: String,
    /// Replayable source or artifact locators associated with this observation.
    pub replay_artifacts: Vec<String>,
    /// Acceptance status for this observation.
    pub status: String,
}

/// One deterministic handler evidence record in a capability report.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityHandlerEvidence {
    /// Handler, fixture, or interpreter name.
    pub handler: String,
    /// Declared handler artifact path.
    pub artifact: String,
    /// Replay command declared by the manifest.
    pub replay_command: String,
    /// Checked source scope for the handler.
    pub checked_scope: String,
    /// True when the artifact text contains an explicit success-path marker.
    pub success_path_coverage: bool,
    /// True when the artifact text contains an explicit failure-path marker.
    pub failure_path_coverage: bool,
    /// BLAKE3 digest for the handler artifact text, if present.
    pub blake3: String,
    /// Evidence tier for this handler record.
    pub evidence_tier: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapabilityManifest {
    enforcement_mode: String,
    scope_classes: Vec<String>,
    capability_families: Vec<String>,
    forbidden_ambient_apis: Vec<String>,
    core_source_scopes: Vec<String>,
    shell_constructor_scopes: Vec<String>,
    adapter_scopes: Vec<String>,
    test_scopes: Vec<String>,
    generated_scopes: Vec<String>,
    remote_execution_scopes: Vec<String>,
    injected_traits: Vec<String>,
    tokens: Vec<String>,
    shell_interpreters: Vec<String>,
    deterministic_handlers: Vec<String>,
    handler_replay_commands: Vec<String>,
    handler_artifacts: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceScopeMatch {
    class: &'static str,
}

/// Evaluates capability manifest diagnostics and report data for one checker run.
// r[impl octet.capability_manifest_checker.manifest]
// r[impl octet.capability_manifest_checker.scope_resolution]
// r[impl octet.capability_manifest_checker.effect_matching]
#[must_use]
pub fn check_capability_manifest(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> CapabilityManifestCheck {
    if !selection.includes(ARCHITECTURE_FAMILY) {
        return CapabilityManifestCheck::default();
    }
    let Some(policy) = registry.policy_by_id(EFFECT_CAPABILITY_POLICY_ID) else {
        return CapabilityManifestCheck::default();
    };
    if capability_mechanism(policy).is_none() {
        return CapabilityManifestCheck::default();
    }
    let manifest = CapabilityManifest::from_policy(policy);
    let mut diagnostics = manifest_validation_diagnostics(policy, &manifest, sources);
    diagnostics.extend(handler_validation_diagnostics(policy, &manifest, sources));
    let report = capability_report(policy, &manifest, sources);
    CapabilityManifestCheck {
        diagnostics,
        reports: vec![report],
    }
}

impl CapabilityManifest {
    fn from_policy(policy: &PolicyEntry) -> Self {
        let mechanism = capability_mechanism(policy);
        let enforcement_mode = mechanism
            .map(|mechanism| mechanism.capability_enforcement_mode.as_str())
            .filter(|mode| !mode.is_empty())
            .unwrap_or(MODE_STRICT)
            .to_owned();
        let approved_names = mechanism_field(mechanism, |mechanism| &mechanism.approved_capability_names);
        let injected_traits =
            merge_unique(mechanism_field(mechanism, |mechanism| &mechanism.capability_injected_traits), approved_names);
        Self {
            enforcement_mode,
            scope_classes: mechanism_field(mechanism, |mechanism| &mechanism.capability_scope_classes),
            capability_families: mechanism_field(mechanism, |mechanism| &mechanism.capability_families),
            forbidden_ambient_apis: mechanism_field(mechanism, |mechanism| &mechanism.forbidden_ambient_apis),
            core_source_scopes: mechanism_field(mechanism, |mechanism| &mechanism.core_source_scopes),
            shell_constructor_scopes: mechanism_field(mechanism, |mechanism| &mechanism.shell_constructor_scopes),
            adapter_scopes: mechanism_field(mechanism, |mechanism| &mechanism.capability_adapter_scopes),
            test_scopes: mechanism_field(mechanism, |mechanism| &mechanism.capability_test_scopes),
            generated_scopes: mechanism_field(mechanism, |mechanism| &mechanism.capability_generated_scopes),
            remote_execution_scopes: mechanism_field(mechanism, |mechanism| {
                &mechanism.capability_remote_execution_scopes
            }),
            injected_traits,
            tokens: mechanism_field(mechanism, |mechanism| &mechanism.capability_tokens),
            shell_interpreters: mechanism_field(mechanism, |mechanism| &mechanism.capability_shell_interpreters),
            deterministic_handlers: mechanism_field(mechanism, |mechanism| {
                &mechanism.capability_deterministic_handlers
            }),
            handler_replay_commands: mechanism_field(mechanism, |mechanism| {
                &mechanism.capability_handler_replay_commands
            }),
            handler_artifacts: mechanism_field(mechanism, |mechanism| &mechanism.capability_handler_artifacts),
        }
    }

    fn is_strict(&self) -> bool {
        self.enforcement_mode == MODE_STRICT
    }

    fn declared_scope_classes(&self) -> Vec<String> {
        if !self.scope_classes.is_empty() {
            return sorted_unique(self.scope_classes.clone());
        }
        let mut classes = Vec::new();
        push_class_if_scopes_exist(&mut classes, SCOPE_CORE, &self.core_source_scopes);
        push_class_if_scopes_exist(&mut classes, SCOPE_SHELL, &self.shell_constructor_scopes);
        push_class_if_scopes_exist(&mut classes, SCOPE_ADAPTER, &self.adapter_scopes);
        push_class_if_scopes_exist(&mut classes, SCOPE_TEST, &self.test_scopes);
        push_class_if_scopes_exist(&mut classes, SCOPE_GENERATED, &self.generated_scopes);
        push_class_if_scopes_exist(&mut classes, SCOPE_REMOTE_EXECUTION, &self.remote_execution_scopes);
        sorted_unique(classes)
    }

    fn declared_capability_families(&self) -> Vec<String> {
        if self.capability_families.is_empty() {
            return observed_families(&self.forbidden_ambient_apis);
        }
        sorted_unique(self.capability_families.clone())
    }
}

fn capability_mechanism(policy: &PolicyEntry) -> Option<&Mechanism> {
    policy.mechanisms.iter().find(|mechanism| mechanism.kind == MechanismKind::CapabilityApi)
}

fn mechanism_field(mechanism: Option<&Mechanism>, field: impl FnOnce(&Mechanism) -> &Vec<String>) -> Vec<String> {
    mechanism.map_or_else(Vec::new, |mechanism| field(mechanism).clone())
}

fn merge_unique(left: Vec<String>, right: Vec<String>) -> Vec<String> {
    let mut values = left;
    values.extend(right);
    sorted_unique(values)
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn push_class_if_scopes_exist(classes: &mut Vec<String>, class: &str, scopes: &[String]) {
    if !scopes.is_empty() {
        classes.push(class.to_owned());
    }
}

fn observed_families(patterns: &[String]) -> Vec<String> {
    let families = patterns.iter().map(|pattern| effect_pattern(pattern).family.to_owned()).collect();
    sorted_unique(families)
}

fn manifest_validation_diagnostics(
    policy: &PolicyEntry,
    manifest: &CapabilityManifest,
    sources: &[SourceFile],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(unknown_scope_class_diagnostics(policy, manifest));
    diagnostics.extend(unknown_capability_family_diagnostics(policy, manifest));
    diagnostics.extend(unknown_enforcement_mode_diagnostics(policy, manifest));
    if manifest.is_strict() && manifest.core_source_scopes.is_empty() {
        diagnostics.push(capability_diagnostic(
            policy,
            fallback_source(sources),
            None,
            "strict capability manifest declares no core source scopes",
            "declare at least one core_source_scopes entry or switch the manifest to advisory mode",
        ));
    }
    if manifest.is_strict() {
        diagnostics.extend(ambiguous_scope_diagnostics(policy, manifest, sources));
    }
    diagnostics
}

fn unknown_scope_class_diagnostics(policy: &PolicyEntry, manifest: &CapabilityManifest) -> Vec<Diagnostic> {
    manifest
        .scope_classes
        .iter()
        .filter(|scope_class| !KNOWN_SCOPE_CLASSES.contains(&scope_class.as_str()))
        .map(|scope_class| {
            policy_level_diagnostic(
                policy,
                &format!("capability manifest names unknown scope class `{scope_class}`"),
                "use core, shell, adapter, test, generated, or remote-execution scope classes",
            )
        })
        .collect()
}

fn unknown_capability_family_diagnostics(policy: &PolicyEntry, manifest: &CapabilityManifest) -> Vec<Diagnostic> {
    manifest
        .capability_families
        .iter()
        .filter(|family| !KNOWN_CAPABILITY_FAMILIES.contains(&family.as_str()))
        .map(|family| {
            policy_level_diagnostic(
                policy,
                &format!("capability manifest names unknown capability family `{family}`"),
                "use the Octet capability vocabulary or classify the gap as unknown",
            )
        })
        .collect()
}

fn unknown_enforcement_mode_diagnostics(policy: &PolicyEntry, manifest: &CapabilityManifest) -> Vec<Diagnostic> {
    if matches!(manifest.enforcement_mode.as_str(), MODE_ADVISORY | MODE_STRICT) {
        return Vec::new();
    }
    vec![policy_level_diagnostic(
        policy,
        &format!("capability manifest names unknown enforcement mode `{}`", manifest.enforcement_mode),
        "use strict or advisory capability manifest enforcement mode",
    )]
}

fn ambiguous_scope_diagnostics(
    policy: &PolicyEntry,
    manifest: &CapabilityManifest,
    sources: &[SourceFile],
) -> Vec<Diagnostic> {
    sources
        .iter()
        .filter_map(|source| {
            let matches = source_scope_matches(&source.path, manifest);
            (matches.len() > FIRST_LINE_NUMBER_OFFSET).then(|| {
                let classes = matches.iter().map(|scope_match| scope_match.class).collect::<Vec<_>>().join(", ");
                capability_diagnostic(
                    policy,
                    Some(source),
                    None,
                    &format!("capability scope resolution is ambiguous: `{}` matches {classes}", source.path),
                    "make strict capability scope path patterns disjoint before using the manifest as a gate",
                )
            })
        })
        .collect()
}

fn handler_validation_diagnostics(
    policy: &PolicyEntry,
    manifest: &CapabilityManifest,
    sources: &[SourceFile],
) -> Vec<Diagnostic> {
    if !manifest.is_strict() || manifest.deterministic_handlers.is_empty() {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    if manifest.handler_artifacts.is_empty() {
        diagnostics.push(policy_level_diagnostic(
            policy,
            "strict capability manifest declares deterministic handlers without handler artifacts",
            "declare capability_handler_artifacts for each required deterministic handler",
        ));
    }
    if manifest.handler_replay_commands.is_empty() {
        diagnostics.push(policy_level_diagnostic(
            policy,
            "strict capability manifest declares deterministic handlers without replay commands",
            "declare capability_handler_replay_commands that replay success and failure paths",
        ));
    }
    for evidence in handler_evidence(manifest, sources) {
        if evidence.blake3.is_empty() {
            diagnostics.push(policy_level_diagnostic(
                policy,
                &format!("deterministic handler artifact `{}` is missing", evidence.artifact),
                "check in the handler artifact or correct the manifest path",
            ));
            continue;
        }
        if !evidence.success_path_coverage || !evidence.failure_path_coverage {
            diagnostics.push(policy_level_diagnostic(
                policy,
                &format!("deterministic handler `{}` must cover success and failure paths", evidence.handler),
                "record explicit success and failure markers in the handler artifact",
            ));
        }
    }
    diagnostics
}

fn capability_report(policy: &PolicyEntry, manifest: &CapabilityManifest, sources: &[SourceFile]) -> CapabilityReport {
    CapabilityReport {
        policy_id: policy.policy_id.clone(),
        enforcement_mode: manifest.enforcement_mode.clone(),
        declared_capability_families: manifest.declared_capability_families(),
        declared_scope_classes: manifest.declared_scope_classes(),
        observations: capability_observations(manifest, sources),
        handler_evidence: handler_evidence(manifest, sources),
        non_claims: NON_CLAIMS.iter().map(|claim| (*claim).to_owned()).collect(),
    }
}

fn capability_observations(manifest: &CapabilityManifest, sources: &[SourceFile]) -> Vec<CapabilityObservation> {
    let mut observations = Vec::new();
    for source in sources.iter().filter(|source| source.path.ends_with(RUST_SOURCE_EXTENSION)) {
        let Some(scope_match) = resolved_source_scope(&source.path, manifest) else {
            continue;
        };
        append_direct_effect_observations(manifest, source, &scope_match, &mut observations);
        append_injected_capability_observations(manifest, source, &scope_match, &mut observations);
    }
    observations.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.observation_kind.cmp(&right.observation_kind))
            .then_with(|| left.matched.cmp(&right.matched))
    });
    observations
}

fn append_direct_effect_observations(
    manifest: &CapabilityManifest,
    source: &SourceFile,
    scope_match: &SourceScopeMatch,
    observations: &mut Vec<CapabilityObservation>,
) {
    for (line_index, line) in source.text.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        let Some(pattern) = manifest.forbidden_ambient_apis.iter().find(|pattern| line.contains(pattern.as_str()))
        else {
            continue;
        };
        let effect = effect_pattern(pattern);
        observations.push(CapabilityObservation {
            path: source.path.clone(),
            line: Some(line_number_from_index(line_index)),
            scope_class: scope_match.class.to_owned(),
            capability_family: effect.family.to_owned(),
            observation_kind: OBSERVATION_DIRECT_EFFECT.to_owned(),
            matched: pattern.clone(),
            linked_lint_ids: effect.lint_ids.iter().map(|lint| (*lint).to_owned()).collect(),
            evidence_tier: SOURCE_SHAPE_EVIDENCE_TIER.to_owned(),
            replay_artifacts: vec![source.path.clone()],
            status: direct_effect_status(scope_match).to_owned(),
        });
    }
}

fn append_injected_capability_observations(
    manifest: &CapabilityManifest,
    source: &SourceFile,
    scope_match: &SourceScopeMatch,
    observations: &mut Vec<CapabilityObservation>,
) {
    if scope_match.class != SCOPE_CORE {
        return;
    }
    for name in manifest.injected_traits.iter().chain(manifest.tokens.iter()) {
        let Some(line) = line_for_text(&source.text, name) else {
            continue;
        };
        let family = capability_name_family(name, manifest);
        observations.push(CapabilityObservation {
            path: source.path.clone(),
            line: Some(line),
            scope_class: scope_match.class.to_owned(),
            capability_family: family.clone(),
            observation_kind: OBSERVATION_INJECTED_CAPABILITY.to_owned(),
            matched: name.clone(),
            linked_lint_ids: lint_ids_for_family(&family).iter().map(|lint| (*lint).to_owned()).collect(),
            evidence_tier: POLICY_CHECK_EVIDENCE_TIER.to_owned(),
            replay_artifacts: vec![source.path.clone()],
            status: STATUS_ACCEPTED.to_owned(),
        });
    }
}

fn direct_effect_status(scope_match: &SourceScopeMatch) -> &'static str {
    if scope_match.class == SCOPE_CORE {
        STATUS_DIAGNOSTIC
    } else {
        STATUS_ACCEPTED
    }
}

fn handler_evidence(manifest: &CapabilityManifest, sources: &[SourceFile]) -> Vec<CapabilityHandlerEvidence> {
    let mut records = manifest
        .deterministic_handlers
        .iter()
        .enumerate()
        .map(|(index, handler)| handler_evidence_record(manifest, sources, index, handler))
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.handler.cmp(&right.handler).then_with(|| left.artifact.cmp(&right.artifact)));
    records
}

fn handler_evidence_record(
    manifest: &CapabilityManifest,
    sources: &[SourceFile],
    index: usize,
    handler: &str,
) -> CapabilityHandlerEvidence {
    let artifact = manifest.handler_artifacts.get(index).cloned().unwrap_or_default();
    let replay_command = manifest
        .handler_replay_commands
        .get(index)
        .or_else(|| manifest.handler_replay_commands.first())
        .cloned()
        .unwrap_or_else(|| DEFAULT_REPLAY_COMMAND.to_owned());
    let checked_scope = manifest.core_source_scopes.first().cloned().unwrap_or_else(|| SCOPE_CORE.to_owned());
    let artifact_source = source_for_artifact(sources, &artifact);
    let artifact_text = artifact_source.map_or("", |source| source.text.as_str());
    let blake3 = if artifact_text.is_empty() {
        String::new()
    } else {
        blake3::hash(artifact_text.as_bytes()).to_hex().to_string()
    };
    CapabilityHandlerEvidence {
        handler: handler.to_owned(),
        artifact,
        replay_command,
        checked_scope,
        success_path_coverage: contains_any_marker(artifact_text, SUCCESS_COVERAGE_MARKERS),
        failure_path_coverage: contains_any_marker(artifact_text, FAILURE_COVERAGE_MARKERS),
        blake3,
        evidence_tier: HANDLER_EVIDENCE_TIER.to_owned(),
    }
}

fn source_for_artifact<'a>(sources: &'a [SourceFile], artifact: &str) -> Option<&'a SourceFile> {
    if artifact.is_empty() {
        return None;
    }
    sources
        .iter()
        .find(|source| source.path == artifact || source.path.ends_with(artifact) || artifact.ends_with(&source.path))
}

fn contains_any_marker(text: &str, markers: &[&str]) -> bool {
    let lowercase = text.to_ascii_lowercase();
    markers.iter().any(|marker| lowercase.contains(marker))
}

fn capability_name_family(name: &str, manifest: &CapabilityManifest) -> String {
    let lowercase = name.to_ascii_lowercase();
    if lowercase.contains("clock") || lowercase.contains("time") {
        return "time".to_owned();
    }
    if lowercase.contains("file") || lowercase.contains("store") {
        return "filesystem".to_owned();
    }
    if lowercase.contains("env") {
        return "environment".to_owned();
    }
    if lowercase.contains("random") || lowercase.contains("rng") {
        return "randomness".to_owned();
    }
    if lowercase.contains("net") || lowercase.contains("http") {
        return "network".to_owned();
    }
    manifest
        .capability_families
        .first()
        .cloned()
        .unwrap_or_else(|| UNKNOWN_CAPABILITY_FAMILY.to_owned())
}

fn resolved_source_scope(path: &str, manifest: &CapabilityManifest) -> Option<SourceScopeMatch> {
    source_scope_matches(path, manifest).into_iter().next()
}

fn source_scope_matches(path: &str, manifest: &CapabilityManifest) -> Vec<SourceScopeMatch> {
    let mut matches = Vec::new();
    push_scope_match(&mut matches, path, SCOPE_CORE, &manifest.core_source_scopes);
    push_scope_match(&mut matches, path, SCOPE_SHELL, &manifest.shell_constructor_scopes);
    push_scope_match(&mut matches, path, SCOPE_ADAPTER, &manifest.adapter_scopes);
    push_scope_match(&mut matches, path, SCOPE_TEST, &manifest.test_scopes);
    push_scope_match(&mut matches, path, SCOPE_GENERATED, &manifest.generated_scopes);
    push_scope_match(&mut matches, path, SCOPE_REMOTE_EXECUTION, &manifest.remote_execution_scopes);
    matches
}

fn push_scope_match(matches: &mut Vec<SourceScopeMatch>, path: &str, class: &'static str, scopes: &[String]) {
    if scopes.iter().any(|scope| path.contains(scope)) {
        matches.push(SourceScopeMatch { class });
    }
}

fn effect_pattern(pattern: &str) -> EffectMatch {
    let effect = EFFECT_PATTERNS
        .iter()
        .find(|effect| pattern.contains(effect.pattern) || effect.pattern.contains(pattern));
    effect.map_or(
        EffectMatch {
            family: UNKNOWN_CAPABILITY_FAMILY,
            lint_ids: UNKNOWN_LINT_IDS,
        },
        |effect| EffectMatch {
            family: effect.family,
            lint_ids: effect.lint_ids,
        },
    )
}

fn lint_ids_for_family(family: &str) -> &'static [&'static str] {
    match family {
        "time" => TIME_LINT_IDS,
        "randomness" => RANDOMNESS_LINT_IDS,
        "filesystem" => FILESYSTEM_LINT_IDS,
        "network" => NETWORK_LINT_IDS,
        "process" => PROCESS_LINT_IDS,
        "environment" => ENVIRONMENT_LINT_IDS,
        "secrets" => SECRETS_LINT_IDS,
        "remote-execution" => REMOTE_EXECUTION_LINT_IDS,
        "unsafe-code" => UNSAFE_CODE_LINT_IDS,
        "global-state" => GLOBAL_STATE_LINT_IDS,
        "concurrency" => CONCURRENCY_LINT_IDS,
        "observability" => OBSERVABILITY_LINT_IDS,
        _ => UNKNOWN_LINT_IDS,
    }
}

fn is_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//") || trimmed.starts_with("///")
}

fn line_for_text(text: &str, needle: &str) -> Option<u32> {
    for (line_index, line) in text.lines().enumerate() {
        if line.contains(needle) {
            return Some(line_number_from_index(line_index));
        }
    }
    None
}

fn line_number_from_index(index: usize) -> u32 {
    u32::try_from(index.saturating_add(FIRST_LINE_NUMBER_OFFSET)).unwrap_or(u32::MAX)
}

fn fallback_source(sources: &[SourceFile]) -> Option<&SourceFile> {
    sources.first()
}

fn policy_level_diagnostic(policy: &PolicyEntry, message: &str, remediation: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: "standards/policies/registry.ncl".to_owned(),
        line: None,
        severity: Severity::Error,
        message: message.to_owned(),
        remediation: remediation.to_owned(),
    })
}

fn capability_diagnostic(
    policy: &PolicyEntry,
    source: Option<&SourceFile>,
    line: Option<u32>,
    message: &str,
    remediation: &str,
) -> Diagnostic {
    let path = source.map_or_else(|| "standards/policies/registry.ncl".to_owned(), |source| source.path.clone());
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path,
        line,
        severity: policy.default_severity,
        message: message.to_owned(),
        remediation: remediation.to_owned(),
    })
}
