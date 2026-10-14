use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;
use syn::visit::Visit;

pub const CATALOG_SCHEMA_VERSION: &str = "octet-effect-closure-catalog/v1";
pub const WORKSPACE_POLICY_SCHEMA_VERSION: &str = "octet-effect-closure-workspace-policy/v1";
pub const RECEIPT_SCHEMA_VERSION: &str = "octet-effect-closure-receipt/v1";
pub const DETERMINISTIC_CORE_PROFILE: &str = "deterministic-core";
pub const CLAIM_SCOPE: &str = "selected-source-token-effect-closure";
pub const WORKSPACE_POLICY_FILE: &str = "octet-effect-closure.json";
pub const RECEIPT_FILE: &str = "effect-closure.json";

const CATALOG_JSON: &str = include_str!("../../standards/effect-closure/generated/effect-closure-catalog.json");
const CALL_SITE_DOMAIN: &str = "octet-effect-closure-call-site/v1";
const RECEIPT_DOMAIN: &str = "octet-effect-closure-receipt/v1";
const MISSING_FUNCTION_ADDRESS: &str = "b3:missing-function-address";
const DIAGNOSTIC_CATALOG: &str = "catalog_invalid";
const DIAGNOSTIC_POLICY: &str = "workspace_policy_invalid";
const DIAGNOSTIC_SOURCE_PARSE: &str = "source_parse_failed";
const DIAGNOSTIC_ROLE: &str = "module_role_invalid";
const DIAGNOSTIC_ROLE_EDGE: &str = "module_role_edge_denied";
const DIAGNOSTIC_ADDRESS_MISSING: &str = "function_address_missing";
const DIAGNOSTIC_ADDRESS_DUPLICATE: &str = "function_address_duplicate";
const DIAGNOSTIC_ADDRESS_UNMATCHED: &str = "function_address_unmatched";
const DIAGNOSTIC_CALL_UNKNOWN: &str = "call_edge_unknown";
const DIAGNOSTIC_SUMMARY_DUPLICATE: &str = "call_summary_duplicate";
const DIAGNOSTIC_SUMMARY_STALE: &str = "call_summary_stale";
const DIAGNOSTIC_PROVIDER_MISMATCH: &str = "dependency_provider_mismatch";
const DIAGNOSTIC_EFFECT_DENIED: &str = "closure_effect_denied";
const DIAGNOSTIC_FIXED_POINT: &str = "closure_fixed_point_failed";
const DIAGNOSTIC_RECEIPT_IDENTITY: &str = "receipt_identity_mismatch";

const EFFECT_ORDER: &[Effect] = &[
    Effect::Alloc,
    Effect::Panic,
    Effect::Unsafe,
    Effect::Clock,
    Effect::Random,
    Effect::Fs,
    Effect::Net,
    Effect::Spawn,
    Effect::Lock,
    Effect::Ffi,
    Effect::Unknown,
];
const ROLE_ORDER: &[ModuleRole] = &[
    ModuleRole::Core,
    ModuleRole::Shell,
    ModuleRole::Adapter,
    ModuleRole::Ffi,
    ModuleRole::Proof,
];
const HASH_HEX_LENGTH: usize = 64;
const BLAKE3_PREFIX: &str = "b3:";
const EXTRA_FIXED_POINT_ITERATION: usize = 1;
const DETERMINISTIC_ALLOWED_EFFECT_COUNT: usize = 1;
const UNIQUE_LOCAL_CANDIDATE_COUNT: usize = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    Alloc,
    Panic,
    Unsafe,
    Clock,
    Random,
    Fs,
    Net,
    Spawn,
    Lock,
    Ffi,
    Unknown,
}

impl Effect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Alloc => "alloc",
            Self::Panic => "panic",
            Self::Unsafe => "unsafe",
            Self::Clock => "clock",
            Self::Random => "random",
            Self::Fs => "fs",
            Self::Net => "net",
            Self::Spawn => "spawn",
            Self::Lock => "lock",
            Self::Ffi => "ffi",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleRole {
    Core,
    Shell,
    Adapter,
    Ffi,
    Proof,
}

impl ModuleRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Shell => "shell",
            Self::Adapter => "adapter",
            Self::Ffi => "ffi",
            Self::Proof => "proof",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    CargoPackage,
    Toolchain,
    Workspace,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RoleEdge {
    pub caller: ModuleRole,
    pub callee: ModuleRole,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EffectProfile {
    pub profile_id: String,
    pub allowed_effects: Vec<Effect>,
    pub denied_effects: Vec<Effect>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EffectCatalog {
    pub schema_version: String,
    pub effects: Vec<Effect>,
    pub roles: Vec<ModuleRole>,
    pub role_edges: Vec<RoleEdge>,
    pub profiles: Vec<EffectProfile>,
    pub resolver: String,
    pub canonicalizer: String,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ModuleRoleAssignment {
    pub path: String,
    pub role: ModuleRole,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProviderIdentity {
    pub kind: ProviderKind,
    pub name: String,
    pub version: String,
    pub source: String,
    pub target: String,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EvidenceReference {
    pub kind: String,
    pub locator: String,
    pub blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct CallSummary {
    pub call_site_id: String,
    pub review_label: String,
    pub callee: String,
    pub provider: ProviderIdentity,
    pub effects: Vec<Effect>,
    pub evidence: EvidenceReference,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkspacePolicy {
    pub schema_version: String,
    pub profile_id: String,
    pub function_address_baseline: String,
    pub module_roles: Vec<ModuleRoleAssignment>,
    pub call_summaries: Vec<CallSummary>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceEnvelope {
    pub path: String,
    pub source: String,
    pub source_identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FunctionAddressRecord {
    pub path: String,
    pub name: String,
    pub function_address: String,
    pub source_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureInput {
    pub profile_id: String,
    pub profile_hash: String,
    pub config_hash: String,
    pub command_blake3: String,
    pub check_status_blake3: String,
    pub function_address_artifact_blake3: String,
    pub function_address_record_root: String,
    pub workspace_policy_blake3: String,
    pub cargo_graph_blake3: String,
    pub cargo_lock_blake3: String,
    pub target: String,
    pub toolchain: String,
    pub sources: Vec<SourceEnvelope>,
    pub function_addresses: Vec<FunctionAddressRecord>,
    pub resolved_providers: Vec<ProviderIdentity>,
    pub catalog: EffectCatalog,
    pub policy: WorkspacePolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallResolution {
    Local,
    Intrinsic,
    ReviewedSummary,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ClosureCallEdge {
    pub call_site_id: String,
    pub occurrence: usize,
    pub shape: String,
    pub resolution: CallResolution,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callee_function_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderIdentity>,
    pub effects: Vec<Effect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceReference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FunctionClosureNode {
    pub path: String,
    pub name: String,
    pub function_address: String,
    pub role: ModuleRole,
    pub direct_effects: Vec<Effect>,
    pub closure_effects: Vec<Effect>,
    pub calls: Vec<ClosureCallEdge>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ClosureDiagnostic {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_site_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AdmissionStatus {
    Admitted,
    Denied,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClosureInputIdentities {
    pub profile_id: String,
    pub profile_hash: String,
    pub config_hash: String,
    pub command_blake3: String,
    pub check_status_blake3: String,
    pub function_address_artifact_blake3: String,
    pub function_address_record_root: String,
    pub workspace_policy_blake3: String,
    pub cargo_graph_blake3: String,
    pub cargo_lock_blake3: String,
    pub target: String,
    pub toolchain: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ClosureSourceIdentity {
    pub path: String,
    pub source_identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EffectClosureReceipt {
    pub schema_version: String,
    pub status: AdmissionStatus,
    pub claim_scope: String,
    pub resolver: String,
    pub canonicalizer: String,
    pub inputs: ClosureInputIdentities,
    pub sources: Vec<ClosureSourceIdentity>,
    pub providers: Vec<ProviderIdentity>,
    pub nodes: Vec<FunctionClosureNode>,
    pub diagnostics: Vec<ClosureDiagnostic>,
    pub non_claims: Vec<String>,
    pub receipt_identity: String,
}

#[derive(Clone)]
struct FunctionDraft {
    path: String,
    name: String,
    owner: Option<String>,
    attrs: Vec<syn::Attribute>,
    signature: syn::Signature,
    block: syn::Block,
    source_identity: String,
}

#[derive(Clone)]
struct AddressedFunction {
    draft: FunctionDraft,
    function_address: String,
    role: ModuleRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RawCall {
    shape: String,
    kind: RawCallKind,
    effects: BTreeSet<Effect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RawCallKind {
    Function,
    Method,
    Macro,
}

#[derive(Default)]
struct FunctionCollector {
    functions: Vec<CollectedFunction>,
}

#[derive(Clone)]
struct CollectedFunction {
    name: String,
    owner: Option<String>,
    attrs: Vec<syn::Attribute>,
    signature: syn::Signature,
    block: syn::Block,
}

struct BodyScanner<'a> {
    foreign_functions: &'a BTreeSet<String>,
    direct_effects: BTreeSet<Effect>,
    calls: Vec<RawCall>,
}

impl<'ast> Visit<'ast> for FunctionCollector {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.functions.push(CollectedFunction {
            name: node.sig.ident.to_string(),
            owner: None,
            attrs: node.attrs.clone(),
            signature: node.sig.clone(),
            block: (*node.block).clone(),
        });
        syn::visit::visit_block(self, &node.block);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let owner = owner_from_type(&node.self_ty);
        for item in &node.items {
            if let syn::ImplItem::Fn(function) = item {
                self.functions.push(CollectedFunction {
                    name: function.sig.ident.to_string(),
                    owner: owner.clone(),
                    attrs: function.attrs.clone(),
                    signature: function.sig.clone(),
                    block: function.block.clone(),
                });
                syn::visit::visit_block(self, &function.block);
            }
        }
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            self.functions.push(CollectedFunction {
                name: node.sig.ident.to_string(),
                owner: None,
                attrs: node.attrs.clone(),
                signature: node.sig.clone(),
                block: block.clone(),
            });
            syn::visit::visit_block(self, block);
        }
    }
}

impl<'a> BodyScanner<'a> {
    fn new(foreign_functions: &'a BTreeSet<String>) -> Self {
        Self {
            foreign_functions,
            direct_effects: BTreeSet::new(),
            calls: Vec::new(),
        }
    }

    fn record_call(&mut self, shape: String, kind: RawCallKind, effects: BTreeSet<Effect>) {
        self.direct_effects.extend(effects.iter().copied());
        self.calls.push(RawCall { shape, kind, effects });
    }
}

impl<'ast> Visit<'ast> for BodyScanner<'_> {
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if node.path().is_ident("cfg") || node.path().is_ident("cfg_attr") {
            self.direct_effects.insert(Effect::Unknown);
        }
        syn::visit::visit_attribute(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        let (shape, effects) = match node.func.as_ref() {
            syn::Expr::Path(path) => {
                let path = path_text(&path.path);
                let effects = classify_path_call(&path, self.foreign_functions).unwrap_or_default();
                (format!("call:{path}"), effects)
            }
            _ => ("call:<opaque-expression>".to_owned(), BTreeSet::from([Effect::Unknown])),
        };
        self.record_call(shape, RawCallKind::Function, effects);
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        let receiver = match node.receiver.as_ref() {
            syn::Expr::Path(path) if path.path.is_ident("self") => "self",
            _ => "value",
        };
        let effects = classify_method_call(&method);
        self.record_call(format!("method:{receiver}:{method}"), RawCallKind::Method, effects);
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let path = path_text(&node.path);
        let mut effects = classify_macro(&path).unwrap_or_default();
        effects.insert(Effect::Unknown);
        self.record_call(format!("macro:{path}"), RawCallKind::Macro, effects);
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.direct_effects.insert(Effect::Unsafe);
        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_expr_index(&mut self, node: &'ast syn::ExprIndex) {
        self.direct_effects.insert(Effect::Panic);
        syn::visit::visit_expr_index(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if matches!(
            node.op,
            syn::BinOp::Add(_)
                | syn::BinOp::Sub(_)
                | syn::BinOp::Mul(_)
                | syn::BinOp::Div(_)
                | syn::BinOp::Rem(_)
                | syn::BinOp::Shl(_)
                | syn::BinOp::Shr(_)
        ) {
            self.direct_effects.insert(Effect::Panic);
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_unary(&mut self, node: &'ast syn::ExprUnary) {
        if matches!(node.op, syn::UnOp::Neg(_)) {
            self.direct_effects.insert(Effect::Panic);
        }
        syn::visit::visit_expr_unary(self, node);
    }

    fn visit_expr_await(&mut self, node: &'ast syn::ExprAwait) {
        self.direct_effects.insert(Effect::Unknown);
        syn::visit::visit_expr_await(self, node);
    }

    fn visit_expr_async(&mut self, node: &'ast syn::ExprAsync) {
        self.direct_effects.insert(Effect::Unknown);
        syn::visit::visit_expr_async(self, node);
    }
}

// r[impl octet.effect_closure.catalog]
pub fn embedded_catalog() -> Result<EffectCatalog, String> {
    let catalog = serde_json::from_str::<EffectCatalog>(CATALOG_JSON)
        .map_err(|error| format!("parse embedded effect-closure catalog: {error}"))?;
    let diagnostics = validate_catalog(&catalog);
    if diagnostics.is_empty() {
        Ok(catalog)
    } else {
        Err(format!("invalid embedded effect-closure catalog: {}", diagnostics.join("; ")))
    }
}

pub fn validate_catalog(catalog: &EffectCatalog) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if catalog.schema_version != CATALOG_SCHEMA_VERSION {
        diagnostics.push(format!(
            "{DIAGNOSTIC_CATALOG}: schema_version must be `{CATALOG_SCHEMA_VERSION}`, got `{}`",
            catalog.schema_version
        ));
    }
    if catalog.effects != EFFECT_ORDER {
        diagnostics.push(format!("{DIAGNOSTIC_CATALOG}: effect vocabulary/order does not match v1"));
    }
    if catalog.roles != ROLE_ORDER {
        diagnostics.push(format!("{DIAGNOSTIC_CATALOG}: module role vocabulary/order does not match v1"));
    }
    if catalog.claim_scope != CLAIM_SCOPE {
        diagnostics.push(format!("{DIAGNOSTIC_CATALOG}: claim_scope must be `{CLAIM_SCOPE}`"));
    }
    validate_non_claims(DIAGNOSTIC_CATALOG, &catalog.non_claims, &mut diagnostics);

    let mut profile_ids = BTreeSet::new();
    for profile in &catalog.profiles {
        if !profile_ids.insert(profile.profile_id.as_str()) {
            diagnostics.push(format!("{DIAGNOSTIC_CATALOG}: duplicate profile `{}`", profile.profile_id));
        }
        if !is_sorted_unique(&profile.allowed_effects) || !is_sorted_unique(&profile.denied_effects) {
            diagnostics.push(format!(
                "{DIAGNOSTIC_CATALOG}: profile `{}` effects must be sorted and unique",
                profile.profile_id
            ));
        }
        let allowed = profile.allowed_effects.iter().copied().collect::<BTreeSet<_>>();
        let denied = profile.denied_effects.iter().copied().collect::<BTreeSet<_>>();
        if !allowed.is_disjoint(&denied) || allowed.union(&denied).copied().collect::<Vec<_>>() != EFFECT_ORDER {
            diagnostics.push(format!(
                "{DIAGNOSTIC_CATALOG}: profile `{}` must partition the complete effect vocabulary",
                profile.profile_id
            ));
        }
    }
    match catalog.profiles.iter().find(|profile| profile.profile_id == DETERMINISTIC_CORE_PROFILE) {
        Some(profile)
            if profile.allowed_effects == [Effect::Alloc]
                && profile.denied_effects == EFFECT_ORDER[DETERMINISTIC_ALLOWED_EFFECT_COUNT..] => {}
        Some(_) => {
            diagnostics.push(format!("{DIAGNOSTIC_CATALOG}: `{DETERMINISTIC_CORE_PROFILE}` must allow only `alloc`"))
        }
        None => diagnostics.push(format!("{DIAGNOSTIC_CATALOG}: missing `{DETERMINISTIC_CORE_PROFILE}` profile")),
    }

    if !is_sorted_unique(&catalog.role_edges) {
        diagnostics.push(format!("{DIAGNOSTIC_CATALOG}: role edges must be sorted and unique"));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

// r[impl octet.effect_closure.catalog.invalid]
// r[impl octet.effect_closure.roles]
// r[impl octet.effect_closure.dependency_summaries]
pub fn validate_workspace_policy(policy: &WorkspacePolicy) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if policy.schema_version != WORKSPACE_POLICY_SCHEMA_VERSION {
        diagnostics.push(format!(
            "{DIAGNOSTIC_POLICY}: schema_version must be `{WORKSPACE_POLICY_SCHEMA_VERSION}`, got `{}`",
            policy.schema_version
        ));
    }
    if policy.profile_id != DETERMINISTIC_CORE_PROFILE {
        diagnostics.push(format!(
            "{DIAGNOSTIC_POLICY}: profile_id must be `{DETERMINISTIC_CORE_PROFILE}`, got `{}`",
            policy.profile_id
        ));
    }
    if policy.function_address_baseline.trim().is_empty() {
        diagnostics.push(format!("{DIAGNOSTIC_POLICY}: function_address_baseline must not be empty"));
    }
    validate_non_claims(DIAGNOSTIC_POLICY, &policy.non_claims, &mut diagnostics);

    let mut role_paths = BTreeSet::new();
    for assignment in &policy.module_roles {
        if assignment.path.trim().is_empty() {
            diagnostics.push(format!("{DIAGNOSTIC_POLICY}: module role path must not be empty"));
        }
        if !role_paths.insert(assignment.path.as_str()) {
            diagnostics.push(format!("{DIAGNOSTIC_POLICY}: duplicate module role path `{}`", assignment.path));
        }
    }
    if policy.module_roles.is_empty() {
        diagnostics.push(format!("{DIAGNOSTIC_POLICY}: module_roles must not be empty"));
    }
    if !is_sorted_unique(&policy.module_roles) {
        diagnostics.push(format!("{DIAGNOSTIC_POLICY}: module_roles must be sorted and unique"));
    }
    if !is_sorted_unique(&policy.call_summaries) {
        diagnostics.push(format!("{DIAGNOSTIC_POLICY}: call_summaries must be sorted by exact call-site identity"));
    }

    let mut call_site_ids = BTreeSet::new();
    for summary in &policy.call_summaries {
        if !call_site_ids.insert(summary.call_site_id.as_str()) {
            diagnostics
                .push(format!("{DIAGNOSTIC_SUMMARY_DUPLICATE}: duplicate call_site_id `{}`", summary.call_site_id));
        }
        if !is_blake3_identity(&summary.call_site_id) {
            diagnostics.push(format!(
                "{DIAGNOSTIC_POLICY}: call_site_id `{}` is not a full BLAKE3 identity",
                summary.call_site_id
            ));
        }
        if summary.review_label.trim().is_empty() || summary.callee.trim().is_empty() {
            diagnostics.push(format!(
                "{DIAGNOSTIC_POLICY}: call summary `{}` requires review_label and callee",
                summary.call_site_id
            ));
        }
        if summary.effects.contains(&Effect::Unknown) {
            diagnostics.push(format!(
                "{DIAGNOSTIC_POLICY}: call summary `{}` must not declare `unknown`",
                summary.call_site_id
            ));
        }
        if !is_sorted_unique(&summary.effects) {
            diagnostics.push(format!(
                "{DIAGNOSTIC_POLICY}: call summary `{}` effects must be sorted and unique",
                summary.call_site_id
            ));
        }
        validate_provider(&summary.call_site_id, &summary.provider, &mut diagnostics);
        if summary.evidence.kind.trim().is_empty()
            || summary.evidence.locator.trim().is_empty()
            || !is_blake3_identity(&summary.evidence.blake3)
        {
            diagnostics.push(format!(
                "{DIAGNOSTIC_POLICY}: call summary `{}` requires an exact evidence reference",
                summary.call_site_id
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

// r[impl octet.effect_closure.inference]
// r[impl octet.effect_closure.transitive]
// r[impl octet.effect_closure.evidence]
pub fn compute_closure(input: ClosureInput) -> EffectClosureReceipt {
    let mut diagnostics = validate_catalog(&input.catalog)
        .into_iter()
        .map(|message| diagnostic(DIAGNOSTIC_CATALOG, message, None, None, None))
        .chain(
            validate_workspace_policy(&input.policy)
                .into_iter()
                .map(|message| diagnostic(DIAGNOSTIC_POLICY, message, None, None, None)),
        )
        .collect::<Vec<_>>();

    if input.profile_id != DETERMINISTIC_CORE_PROFILE || input.policy.profile_id != input.profile_id {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_POLICY,
            format!(
                "closure v1 requires matching `{DETERMINISTIC_CORE_PROFILE}` profile, got run `{}` and policy `{}`",
                input.profile_id, input.policy.profile_id
            ),
            None,
            None,
            None,
        ));
    }

    let roles = role_map(&input.sources, &input.policy, &mut diagnostics);
    let drafts = parse_function_drafts(&input.sources, &mut diagnostics);
    let addressed = address_functions(drafts, &input.function_addresses, &roles, &mut diagnostics);
    let mut nodes = build_nodes(&addressed, &input, &mut diagnostics);
    propagate_closure_effects(&mut nodes, &mut diagnostics);
    apply_profile_effect_gate(&nodes, &input.catalog, &input.profile_id, &mut diagnostics);

    diagnostics.sort();
    diagnostics.dedup();
    nodes.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.name.cmp(&right.name))
            .then(left.function_address.cmp(&right.function_address))
    });

    let mut sources = input
        .sources
        .iter()
        .map(|source| ClosureSourceIdentity {
            path: source.path.clone(),
            source_identity: source.source_identity.clone(),
        })
        .collect::<Vec<_>>();
    sources.sort();
    sources.dedup();

    let used_providers = nodes
        .iter()
        .flat_map(|node| node.calls.iter().filter_map(|call| call.provider.clone()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut non_claims = input.catalog.non_claims.iter().chain(&input.policy.non_claims).cloned().collect::<Vec<_>>();
    non_claims.sort();
    non_claims.dedup();

    let status = if diagnostics.is_empty() {
        AdmissionStatus::Admitted
    } else {
        AdmissionStatus::Denied
    };
    finalize_receipt(EffectClosureReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
        status,
        claim_scope: CLAIM_SCOPE.to_owned(),
        resolver: input.catalog.resolver,
        canonicalizer: input.catalog.canonicalizer,
        inputs: ClosureInputIdentities {
            profile_id: input.profile_id,
            profile_hash: input.profile_hash,
            config_hash: input.config_hash,
            command_blake3: input.command_blake3,
            check_status_blake3: input.check_status_blake3,
            function_address_artifact_blake3: input.function_address_artifact_blake3,
            function_address_record_root: input.function_address_record_root,
            workspace_policy_blake3: input.workspace_policy_blake3,
            cargo_graph_blake3: input.cargo_graph_blake3,
            cargo_lock_blake3: input.cargo_lock_blake3,
            target: input.target,
            toolchain: input.toolchain,
        },
        sources,
        providers: used_providers,
        nodes,
        diagnostics,
        non_claims,
        receipt_identity: String::new(),
    })
}

pub fn verify_receipt_identity(receipt: &EffectClosureReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
        diagnostics.push(format!("unsupported effect-closure receipt schema `{}`", receipt.schema_version));
    }
    if receipt.claim_scope != CLAIM_SCOPE {
        diagnostics.push(format!("effect-closure claim_scope must be `{CLAIM_SCOPE}`"));
    }
    if receipt.non_claims.is_empty() {
        diagnostics.push("effect-closure receipt must preserve explicit non-claims".to_owned());
    }
    if !is_sorted_unique(&receipt.sources)
        || !is_sorted_unique(&receipt.providers)
        || !is_sorted_unique(&receipt.diagnostics)
        || !is_sorted_unique(&receipt.non_claims)
    {
        diagnostics.push("effect-closure receipt canonical collections are not sorted and unique".to_owned());
    }
    diagnostics.extend(validate_receipt_structure(receipt));

    let expected = receipt_identity(receipt);
    if receipt.receipt_identity != expected {
        diagnostics
            .push(format!("{DIAGNOSTIC_RECEIPT_IDENTITY}: expected `{expected}`, got `{}`", receipt.receipt_identity));
    }
    let expected_status = if receipt.diagnostics.is_empty() {
        AdmissionStatus::Admitted
    } else {
        AdmissionStatus::Denied
    };
    if receipt.status != expected_status {
        diagnostics.push("effect-closure receipt status does not match diagnostics".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_receipt_structure(receipt: &EffectClosureReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let bound_identities = [
        ("profile_hash", receipt.inputs.profile_hash.as_str()),
        ("config_hash", receipt.inputs.config_hash.as_str()),
        ("function_address_record_root", receipt.inputs.function_address_record_root.as_str()),
    ];
    for (label, identity) in bound_identities {
        if !is_blake3_identity(identity) {
            diagnostics.push(format!("effect-closure receipt input `{label}` is not a full BLAKE3 identity"));
        }
    }
    let bound_digests = [
        ("command_blake3", receipt.inputs.command_blake3.as_str()),
        ("check_status_blake3", receipt.inputs.check_status_blake3.as_str()),
        ("function_address_artifact_blake3", receipt.inputs.function_address_artifact_blake3.as_str()),
        ("workspace_policy_blake3", receipt.inputs.workspace_policy_blake3.as_str()),
        ("cargo_graph_blake3", receipt.inputs.cargo_graph_blake3.as_str()),
        ("cargo_lock_blake3", receipt.inputs.cargo_lock_blake3.as_str()),
    ];
    for (label, digest) in bound_digests {
        if !is_blake3_digest(digest) {
            diagnostics.push(format!("effect-closure receipt input `{label}` is not a full BLAKE3 digest"));
        }
    }
    if receipt.inputs.profile_id != DETERMINISTIC_CORE_PROFILE
        || receipt.inputs.target.trim().is_empty()
        || receipt.inputs.toolchain.trim().is_empty()
        || receipt.resolver.trim().is_empty()
        || receipt.canonicalizer.trim().is_empty()
    {
        diagnostics
            .push("effect-closure receipt has an invalid profile, target, toolchain, or resolver identity".to_owned());
    }

    let node_keys = receipt
        .nodes
        .iter()
        .map(|node| (node.path.as_str(), node.name.as_str(), node.function_address.as_str()))
        .collect::<Vec<_>>();
    if !is_sorted_unique(&node_keys) {
        diagnostics.push("effect-closure receipt nodes are not sorted and unique".to_owned());
    }
    let local_addresses = receipt.nodes.iter().map(|node| node.function_address.as_str()).collect::<BTreeSet<_>>();
    let source_paths = receipt.sources.iter().map(|source| source.path.as_str()).collect::<BTreeSet<_>>();
    let mut used_providers = BTreeSet::new();

    for source in &receipt.sources {
        if source.path.trim().is_empty() || !is_blake3_identity(&source.source_identity) {
            diagnostics.push(format!("effect-closure source `{}` has an invalid path or identity", source.path));
        }
    }
    for provider in &receipt.providers {
        validate_provider("receipt provider", provider, &mut diagnostics);
    }
    for node in &receipt.nodes {
        if node.path.trim().is_empty()
            || node.name.trim().is_empty()
            || !is_blake3_identity(&node.function_address)
            || !source_paths.contains(node.path.as_str())
        {
            diagnostics.push(format!(
                "effect-closure node `{}::{}` has an invalid path, name, address, or source link",
                node.path, node.name
            ));
        }
        if !is_sorted_unique(&node.direct_effects)
            || !is_sorted_unique(&node.closure_effects)
            || !is_sorted_unique(&node.calls)
        {
            diagnostics
                .push(format!("effect-closure node `{}::{}` collections are not canonical", node.path, node.name));
        }
        let closure = node.closure_effects.iter().copied().collect::<BTreeSet<_>>();
        if node.direct_effects.iter().any(|effect| !closure.contains(effect)) {
            diagnostics.push(format!(
                "effect-closure node `{}::{}` closure does not contain every direct effect",
                node.path, node.name
            ));
        }
        for call in &node.calls {
            if call.shape.trim().is_empty()
                || !is_blake3_identity(&call.call_site_id)
                || call.call_site_id != call_site_identity(&node.function_address, &call.shape, call.occurrence)
                || !is_sorted_unique(&call.effects)
            {
                diagnostics.push(format!(
                    "effect-closure call `{}` has invalid canonical identity or effects",
                    call.call_site_id
                ));
            }
            match call.resolution {
                CallResolution::Local => {
                    if call.callee_function_address.as_deref().is_none_or(|address| !local_addresses.contains(address))
                        || call.provider.is_some()
                        || call.review_label.is_some()
                        || call.evidence.is_some()
                    {
                        diagnostics.push(format!("local call `{}` has invalid linkage fields", call.call_site_id));
                    }
                }
                CallResolution::ReviewedSummary => {
                    let Some(provider) = call.provider.as_ref() else {
                        diagnostics.push(format!("reviewed call `{}` is missing its provider", call.call_site_id));
                        continue;
                    };
                    used_providers.insert(provider.clone());
                    if call.callee_function_address.is_some()
                        || call.review_label.as_deref().is_none_or(str::is_empty)
                        || call.evidence.as_ref().is_none_or(|evidence| {
                            evidence.kind.trim().is_empty()
                                || evidence.locator.trim().is_empty()
                                || !is_blake3_identity(&evidence.blake3)
                        })
                    {
                        diagnostics.push(format!("reviewed call `{}` has invalid summary linkage", call.call_site_id));
                    }
                }
                CallResolution::Unknown => {
                    if call.callee_function_address.is_some()
                        || call.provider.is_some()
                        || call.review_label.is_some()
                        || call.evidence.is_some()
                        || call.effects != [Effect::Unknown]
                    {
                        diagnostics
                            .push(format!("unknown call `{}` has invalid fail-closed fields", call.call_site_id));
                    }
                }
                CallResolution::Intrinsic => diagnostics.push(format!(
                    "call `{}` uses unsupported name-based intrinsic resolution in closure v1",
                    call.call_site_id
                )),
            }
        }
    }
    if used_providers.into_iter().collect::<Vec<_>>() != receipt.providers {
        diagnostics.push("effect-closure receipt provider set does not exactly match reviewed calls".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn call_site_identity(function_address: &str, shape: &str, occurrence: usize) -> String {
    full_hash(&format!(
        "domain={CALL_SITE_DOMAIN}\ncaller={function_address}\nshape={shape}\noccurrence={occurrence}\n"
    ))
}

fn role_map(
    sources: &[SourceEnvelope],
    policy: &WorkspacePolicy,
    diagnostics: &mut Vec<ClosureDiagnostic>,
) -> BTreeMap<String, ModuleRole> {
    let source_paths = sources.iter().map(|source| source.path.as_str()).collect::<BTreeSet<_>>();
    let mut roles = BTreeMap::new();
    for assignment in &policy.module_roles {
        if !source_paths.contains(assignment.path.as_str()) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_ROLE,
                format!("module role path `{}` is outside the selected source envelopes", assignment.path),
                Some(assignment.path.clone()),
                None,
                None,
            ));
            continue;
        }
        if assignment.role != ModuleRole::Core {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_ROLE,
                format!(
                    "selected deterministic-core path `{}` must have role `core`, got `{}`",
                    assignment.path,
                    assignment.role.as_str()
                ),
                Some(assignment.path.clone()),
                None,
                None,
            ));
        }
        if roles.insert(assignment.path.clone(), assignment.role).is_some() {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_ROLE,
                format!("selected path `{}` has duplicate role assignments", assignment.path),
                Some(assignment.path.clone()),
                None,
                None,
            ));
        }
    }
    for source in sources {
        if !roles.contains_key(&source.path) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_ROLE,
                format!("selected path `{}` has no explicit module role", source.path),
                Some(source.path.clone()),
                None,
                None,
            ));
        }
    }
    roles
}

fn parse_function_drafts(sources: &[SourceEnvelope], diagnostics: &mut Vec<ClosureDiagnostic>) -> Vec<FunctionDraft> {
    let mut drafts = Vec::new();
    for source in sources {
        match syn::parse_file(&source.source) {
            Ok(file) => {
                let mut collector = FunctionCollector::default();
                collector.visit_file(&file);
                drafts.extend(collector.functions.into_iter().map(|function| FunctionDraft {
                    path: source.path.clone(),
                    name: function.name,
                    owner: function.owner,
                    attrs: function.attrs,
                    signature: function.signature,
                    block: function.block,
                    source_identity: source.source_identity.clone(),
                }));
            }
            Err(error) => diagnostics.push(diagnostic(
                DIAGNOSTIC_SOURCE_PARSE,
                format!("parse selected Rust source `{}`: {error}", source.path),
                Some(source.path.clone()),
                None,
                None,
            )),
        }
    }
    drafts.sort_by(|left, right| left.path.cmp(&right.path).then(left.name.cmp(&right.name)));
    drafts
}

fn address_functions(
    drafts: Vec<FunctionDraft>,
    records: &[FunctionAddressRecord],
    roles: &BTreeMap<String, ModuleRole>,
    diagnostics: &mut Vec<ClosureDiagnostic>,
) -> Vec<AddressedFunction> {
    let mut by_locator = BTreeMap::<(&str, &str), Vec<&FunctionAddressRecord>>::new();
    for record in records {
        by_locator.entry((record.path.as_str(), record.name.as_str())).or_default().push(record);
    }
    let mut used = BTreeSet::new();
    let mut addressed = Vec::new();
    for draft in drafts {
        let locator = (draft.path.as_str(), draft.name.as_str());
        let matching = by_locator.get(&locator).cloned().unwrap_or_default();
        let function_address = match matching.as_slice() {
            [record] if record.source_identity == draft.source_identity => {
                used.insert((record.path.clone(), record.name.clone(), record.function_address.clone()));
                record.function_address.clone()
            }
            [record] => {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_ADDRESS_MISSING,
                    format!(
                        "function-address source identity for `{}::{}` does not match selected source",
                        draft.path, draft.name
                    ),
                    Some(draft.path.clone()),
                    Some(record.function_address.clone()),
                    None,
                ));
                MISSING_FUNCTION_ADDRESS.to_owned()
            }
            [] => {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_ADDRESS_MISSING,
                    format!("selected function `{}::{}` has no function-address record", draft.path, draft.name),
                    Some(draft.path.clone()),
                    None,
                    None,
                ));
                MISSING_FUNCTION_ADDRESS.to_owned()
            }
            _ => {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_ADDRESS_DUPLICATE,
                    format!(
                        "selected function `{}::{}` has duplicate function-address records",
                        draft.path, draft.name
                    ),
                    Some(draft.path.clone()),
                    None,
                    None,
                ));
                MISSING_FUNCTION_ADDRESS.to_owned()
            }
        };
        let role = roles.get(&draft.path).copied().unwrap_or(ModuleRole::Core);
        addressed.push(AddressedFunction {
            draft,
            function_address,
            role,
        });
    }
    for record in records {
        let key = (record.path.clone(), record.name.clone(), record.function_address.clone());
        if !used.contains(&key) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_ADDRESS_UNMATCHED,
                format!(
                    "function-address record `{}::{}` does not match one supported selected function",
                    record.path, record.name
                ),
                Some(record.path.clone()),
                Some(record.function_address.clone()),
                None,
            ));
        }
    }
    addressed
}

fn build_nodes(
    functions: &[AddressedFunction],
    input: &ClosureInput,
    diagnostics: &mut Vec<ClosureDiagnostic>,
) -> Vec<FunctionClosureNode> {
    let foreign_functions = foreign_function_names(&input.sources);
    let local_names = local_function_index(functions);
    let allowed_role_edges = input.catalog.role_edges.iter().cloned().collect::<BTreeSet<_>>();
    let providers = input.resolved_providers.iter().cloned().collect::<BTreeSet<_>>();
    let mut summaries = BTreeMap::new();
    for summary in &input.policy.call_summaries {
        if summaries.insert(summary.call_site_id.clone(), summary).is_some() {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SUMMARY_DUPLICATE,
                format!("duplicate summary for call site `{}`", summary.call_site_id),
                None,
                None,
                Some(summary.call_site_id.clone()),
            ));
        }
    }
    let mut used_summaries = BTreeSet::new();
    let mut nodes = Vec::new();

    for function in functions {
        let mut scanner = BodyScanner::new(&foreign_functions);
        if function.draft.signature.asyncness.is_some() {
            scanner.direct_effects.insert(Effect::Unknown);
        }
        if function.draft.signature.unsafety.is_some() {
            scanner.direct_effects.insert(Effect::Unsafe);
        }
        if function.draft.signature.abi.is_some() {
            scanner.direct_effects.insert(Effect::Ffi);
        }
        for attribute in &function.draft.attrs {
            scanner.visit_attribute(attribute);
        }
        scanner.visit_block(&function.draft.block);

        let mut calls = Vec::new();
        for (occurrence, raw_call) in scanner.calls.into_iter().enumerate() {
            let call_site_id = call_site_identity(&function.function_address, &raw_call.shape, occurrence);
            let local = resolve_local_call(&raw_call, function, &local_names);
            let edge = if let Some(callee) = local {
                let role_edge = RoleEdge {
                    caller: function.role,
                    callee: callee.role,
                };
                if !allowed_role_edges.contains(&role_edge) {
                    diagnostics.push(diagnostic(
                        DIAGNOSTIC_ROLE_EDGE,
                        format!(
                            "role edge `{}` -> `{}` is denied for call `{}`",
                            function.role.as_str(),
                            callee.role.as_str(),
                            raw_call.shape
                        ),
                        Some(function.draft.path.clone()),
                        Some(function.function_address.clone()),
                        Some(call_site_id.clone()),
                    ));
                }
                ClosureCallEdge {
                    call_site_id,
                    occurrence,
                    shape: raw_call.shape,
                    resolution: CallResolution::Local,
                    callee_function_address: Some(callee.function_address.clone()),
                    provider: None,
                    effects: raw_call.effects.into_iter().collect(),
                    review_label: None,
                    evidence: None,
                }
            } else if raw_call.kind != RawCallKind::Macro {
                match summaries.get(&call_site_id).copied() {
                    Some(summary)
                        if providers.contains(&summary.provider) && summary_evidence_matches(summary, input) =>
                    {
                        used_summaries.insert(call_site_id.clone());
                        scanner.direct_effects.extend(summary.effects.iter().copied());
                        ClosureCallEdge {
                            call_site_id,
                            occurrence,
                            shape: raw_call.shape,
                            resolution: CallResolution::ReviewedSummary,
                            callee_function_address: None,
                            provider: Some(summary.provider.clone()),
                            effects: summary.effects.clone(),
                            review_label: Some(summary.review_label.clone()),
                            evidence: Some(summary.evidence.clone()),
                        }
                    }
                    Some(summary) => {
                        used_summaries.insert(call_site_id.clone());
                        scanner.direct_effects.insert(Effect::Unknown);
                        diagnostics.push(diagnostic(
                            DIAGNOSTIC_PROVIDER_MISMATCH,
                            format!(
                                "call summary `{}` provider/evidence identity for `{}` {} does not match the resolved graph",
                                call_site_id, summary.provider.name, summary.provider.version
                            ),
                            Some(function.draft.path.clone()),
                            Some(function.function_address.clone()),
                            Some(call_site_id.clone()),
                        ));
                        unknown_edge(call_site_id, occurrence, raw_call.shape)
                    }
                    None => {
                        scanner.direct_effects.insert(Effect::Unknown);
                        diagnostics.push(diagnostic(
                            DIAGNOSTIC_CALL_UNKNOWN,
                            format!("call `{}` has no selected local or exact reviewed summary", raw_call.shape),
                            Some(function.draft.path.clone()),
                            Some(function.function_address.clone()),
                            Some(call_site_id.clone()),
                        ));
                        unknown_edge(call_site_id, occurrence, raw_call.shape)
                    }
                }
            } else {
                scanner.direct_effects.insert(Effect::Unknown);
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_CALL_UNKNOWN,
                    format!("macro call `{}` is opaque to the v1 resolver", raw_call.shape),
                    Some(function.draft.path.clone()),
                    Some(function.function_address.clone()),
                    Some(call_site_id.clone()),
                ));
                unknown_edge(call_site_id, occurrence, raw_call.shape)
            };
            calls.push(edge);
        }
        calls.sort();
        let direct_effects = scanner.direct_effects.into_iter().collect::<Vec<_>>();
        nodes.push(FunctionClosureNode {
            path: function.draft.path.clone(),
            name: function.draft.name.clone(),
            function_address: function.function_address.clone(),
            role: function.role,
            closure_effects: direct_effects.clone(),
            direct_effects,
            calls,
        });
    }

    for summary in &input.policy.call_summaries {
        if !used_summaries.contains(&summary.call_site_id) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_SUMMARY_STALE,
                format!("call summary `{}` is stale or targets a resolved call", summary.call_site_id),
                None,
                None,
                Some(summary.call_site_id.clone()),
            ));
        }
    }
    nodes
}

// r[impl octet.effect_closure.transitive.propagates]
// r[impl octet.effect_closure.transitive.cycle]
fn propagate_closure_effects(nodes: &mut [FunctionClosureNode], diagnostics: &mut Vec<ClosureDiagnostic>) {
    let address_to_index = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.function_address.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let iteration_bound = nodes.len().saturating_mul(EFFECT_ORDER.len()).saturating_add(EXTRA_FIXED_POINT_ITERATION);
    let mut converged = false;
    for _iteration in 0..iteration_bound {
        let previous = nodes.iter().map(|node| node.closure_effects.clone()).collect::<Vec<_>>();
        let mut changed = false;
        for node in nodes.iter_mut() {
            let mut closure = node.closure_effects.iter().copied().collect::<BTreeSet<_>>();
            for call in &node.calls {
                if call.resolution != CallResolution::Local {
                    continue;
                }
                let Some(callee_address) = call.callee_function_address.as_ref() else {
                    continue;
                };
                let Some(callee_index) = address_to_index.get(callee_address).copied() else {
                    closure.insert(Effect::Unknown);
                    continue;
                };
                closure.extend(previous[callee_index].iter().copied());
            }
            let next = closure.into_iter().collect::<Vec<_>>();
            if next != node.closure_effects {
                node.closure_effects = next;
                changed = true;
            }
        }
        if !changed {
            converged = true;
            break;
        }
    }
    if !converged && !nodes.is_empty() {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_FIXED_POINT,
            "effect closure did not converge within the derived finite-lattice bound".to_owned(),
            None,
            None,
            None,
        ));
    }
}

fn apply_profile_effect_gate(
    nodes: &[FunctionClosureNode],
    catalog: &EffectCatalog,
    profile_id: &str,
    diagnostics: &mut Vec<ClosureDiagnostic>,
) {
    let allowed = catalog
        .profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
        .map(|profile| profile.allowed_effects.iter().copied().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    for node in nodes {
        for effect in node.closure_effects.iter().copied().filter(|effect| !allowed.contains(effect)) {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_EFFECT_DENIED,
                format!("function `{}::{}` closure contains denied effect `{}`", node.path, node.name, effect.as_str()),
                Some(node.path.clone()),
                Some(node.function_address.clone()),
                None,
            ));
        }
    }
}

fn local_function_index(functions: &[AddressedFunction]) -> BTreeMap<String, Vec<&AddressedFunction>> {
    let mut index = BTreeMap::<String, Vec<&AddressedFunction>>::new();
    for function in functions {
        index.entry(function.draft.name.clone()).or_default().push(function);
    }
    index
}

fn resolve_local_call<'a>(
    raw_call: &RawCall,
    caller: &AddressedFunction,
    local_names: &'a BTreeMap<String, Vec<&'a AddressedFunction>>,
) -> Option<&'a AddressedFunction> {
    let name = raw_call.shape.rsplit(':').next()?;
    let candidates = local_names.get(name)?;
    if candidates.len() != UNIQUE_LOCAL_CANDIDATE_COUNT {
        return None;
    }
    let candidate = candidates[0];
    match raw_call.kind {
        RawCallKind::Method => {
            if raw_call.shape.starts_with("method:self:") && candidate.draft.owner == caller.draft.owner {
                Some(candidate)
            } else {
                None
            }
        }
        RawCallKind::Macro => None,
        RawCallKind::Function => {
            let path = raw_call.shape.strip_prefix("call:")?;
            let segments = path.split("::").collect::<Vec<_>>();
            if segments.len() == EXTRA_FIXED_POINT_ITERATION
                || matches!(segments.first().copied(), Some("self" | "super" | "crate"))
                || candidate
                    .draft
                    .owner
                    .as_ref()
                    .is_some_and(|owner| segments.first().copied() == Some(owner.as_str()))
            {
                Some(candidate)
            } else {
                None
            }
        }
    }
}

fn classify_path_call(path: &str, foreign_functions: &BTreeSet<String>) -> Option<BTreeSet<Effect>> {
    let segments = path.split("::").collect::<Vec<_>>();
    let first = segments.first().copied().unwrap_or_default();
    let last = segments.last().copied().unwrap_or_default();
    if foreign_functions.contains(last) {
        return Some(BTreeSet::from([Effect::Ffi]));
    }
    if matches!(first, "Vec" | "String" | "Box" | "BTreeMap" | "BTreeSet" | "HashMap" | "HashSet" | "Rc" | "Arc")
        || matches!(segments.get(1).copied(), Some("vec" | "string" | "boxed" | "collections" | "rc" | "sync"))
    {
        return Some(BTreeSet::from([Effect::Alloc]));
    }
    if path.starts_with("std::fs::") || path.starts_with("fs::") {
        return Some(BTreeSet::from([Effect::Fs]));
    }
    if path.starts_with("std::net::") || path.starts_with("net::") {
        return Some(BTreeSet::from([Effect::Net]));
    }
    if path.contains("SystemTime::now") || path.contains("Instant::now") {
        return Some(BTreeSet::from([Effect::Clock]));
    }
    if matches!(first, "rand" | "getrandom") || path.contains("Uuid::new_v4") {
        return Some(BTreeSet::from([Effect::Random]));
    }
    if path.contains("thread::spawn") || path.contains("tokio::spawn") || path.contains("task::spawn") {
        return Some(BTreeSet::from([Effect::Spawn]));
    }
    if matches!(last, "panic" | "unwrap" | "expect") {
        return Some(BTreeSet::from([Effect::Panic]));
    }
    if matches!(last, "Some" | "Ok" | "Err" | "drop")
        || path.starts_with("core::cmp::")
        || path.starts_with("std::cmp::")
        || path.starts_with("core::mem::")
        || path.starts_with("std::mem::")
        || primitive_type(first)
    {
        return Some(BTreeSet::new());
    }
    None
}

fn classify_method_call(method: &str) -> BTreeSet<Effect> {
    match method {
        "unwrap" | "expect" => BTreeSet::from([Effect::Panic]),
        "clone" | "collect" | "extend" | "push" | "sort" | "sort_by" | "to_ascii_lowercase" | "to_owned"
        | "to_string" | "into_boxed_slice" | "into_boxed_str" => BTreeSet::from([Effect::Alloc]),
        "lock" | "try_lock" => BTreeSet::from([Effect::Lock]),
        "spawn" | "spawn_blocking" => BTreeSet::from([Effect::Spawn]),
        _ => BTreeSet::new(),
    }
}

fn classify_macro(path: &str) -> Option<BTreeSet<Effect>> {
    let last = path.rsplit("::").next().unwrap_or(path);
    match last {
        "format" | "vec" => Some(BTreeSet::from([Effect::Alloc])),
        "panic" | "assert" | "assert_eq" | "assert_ne" | "debug_assert" | "debug_assert_eq" | "debug_assert_ne"
        | "todo" | "unimplemented" | "unreachable" => Some(BTreeSet::from([Effect::Panic])),
        "matches" | "cfg" => Some(BTreeSet::new()),
        _ => None,
    }
}

fn primitive_type(name: &str) -> bool {
    matches!(
        name,
        "bool"
            | "char"
            | "str"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
    )
}

fn foreign_function_names(sources: &[SourceEnvelope]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for source in sources {
        let Ok(file) = syn::parse_file(&source.source) else {
            continue;
        };
        collect_foreign_names(&file.items, &mut names);
    }
    names
}

fn collect_foreign_names(items: &[syn::Item], names: &mut BTreeSet<String>) {
    for item in items {
        match item {
            syn::Item::ForeignMod(module) => {
                for foreign in &module.items {
                    if let syn::ForeignItem::Fn(function) = foreign {
                        names.insert(function.sig.ident.to_string());
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_brace, nested)) = &module.content {
                    collect_foreign_names(nested, names);
                }
            }
            _ => {}
        }
    }
}

fn owner_from_type(node: &syn::Type) -> Option<String> {
    match node {
        syn::Type::Path(path) => path.path.segments.last().map(|segment| segment.ident.to_string()),
        syn::Type::Reference(reference) => owner_from_type(&reference.elem),
        _ => None,
    }
}

fn path_text(path: &syn::Path) -> String {
    path.segments.iter().map(|segment| segment.ident.to_string()).collect::<Vec<_>>().join("::")
}

fn unknown_edge(call_site_id: String, occurrence: usize, shape: String) -> ClosureCallEdge {
    ClosureCallEdge {
        call_site_id,
        occurrence,
        shape,
        resolution: CallResolution::Unknown,
        callee_function_address: None,
        provider: None,
        effects: vec![Effect::Unknown],
        review_label: None,
        evidence: None,
    }
}

fn summary_evidence_matches(summary: &CallSummary, input: &ClosureInput) -> bool {
    match summary.provider.kind {
        ProviderKind::CargoPackage | ProviderKind::Workspace => {
            summary.evidence.blake3 == canonical_blake3_identity(&input.cargo_lock_blake3)
                && summary.evidence.locator.starts_with("Cargo.lock")
        }
        ProviderKind::Toolchain => summary.evidence.blake3 == provider_identity_blake3(&summary.provider),
    }
}

fn canonical_blake3_identity(value: &str) -> String {
    if value.starts_with(BLAKE3_PREFIX) {
        value.to_owned()
    } else {
        format!("{BLAKE3_PREFIX}{value}")
    }
}

pub fn provider_identity_blake3(provider: &ProviderIdentity) -> String {
    let canonical = serde_json::to_vec(provider).expect("provider identity serialization is infallible");
    full_hash_bytes(b"octet-effect-closure-provider/v1", &canonical)
}

fn validate_provider(call_site_id: &str, provider: &ProviderIdentity, diagnostics: &mut Vec<String>) {
    if provider.name.trim().is_empty()
        || provider.version.trim().is_empty()
        || provider.source.trim().is_empty()
        || provider.target.trim().is_empty()
        || !is_sorted_unique(&provider.features)
    {
        diagnostics.push(format!(
            "{DIAGNOSTIC_POLICY}: call summary `{call_site_id}` provider identity is incomplete or non-canonical"
        ));
    }
}

fn validate_non_claims(prefix: &str, non_claims: &[String], diagnostics: &mut Vec<String>) {
    if non_claims.is_empty() || non_claims.iter().any(|claim| claim.trim().is_empty()) {
        diagnostics.push(format!("{prefix}: non_claims must contain non-empty text"));
    }
}

fn diagnostic(
    code: &str,
    message: String,
    path: Option<String>,
    function_address: Option<String>,
    call_site_id: Option<String>,
) -> ClosureDiagnostic {
    ClosureDiagnostic {
        code: code.to_owned(),
        message,
        path,
        function_address,
        call_site_id,
    }
}

fn finalize_receipt(mut receipt: EffectClosureReceipt) -> EffectClosureReceipt {
    receipt.receipt_identity = receipt_identity(&receipt);
    receipt
}

fn receipt_identity(receipt: &EffectClosureReceipt) -> String {
    let mut canonical = receipt.clone();
    canonical.receipt_identity.clear();
    let bytes = serde_json::to_vec(&canonical).expect("effect-closure receipt serialization is infallible");
    full_hash_bytes(RECEIPT_DOMAIN.as_bytes(), &bytes)
}

fn full_hash(input: &str) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(input.as_bytes()).to_hex())
}

fn full_hash_bytes(domain: &[u8], input: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[0]);
    hasher.update(input);
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

fn is_blake3_identity(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(is_lower_blake3_hex)
}

fn is_blake3_digest(value: &str) -> bool {
    is_lower_blake3_hex(value.strip_prefix(BLAKE3_PREFIX).unwrap_or(value))
}

fn is_lower_blake3_hex(hex: &str) -> bool {
    hex.len() == HASH_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_sorted_unique<T: Ord>(items: &[T]) -> bool {
    items.windows(2).all(|window| window[0] < window[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PATH: &str = "crates/domain/src/core.rs";
    const TEST_SOURCE_ID: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const TEST_PROFILE_HASH: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TEST_CONFIG_HASH: &str = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const TEST_ARTIFACT_HASH: &str = "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const TEST_RECORD_ROOT: &str = "b3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const TEST_POLICY_HASH: &str = "b3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    const TEST_GRAPH_HASH: &str = "b3:1111111111111111111111111111111111111111111111111111111111111111";
    const TEST_LOCK_HASH: &str = "b3:2222222222222222222222222222222222222222222222222222222222222222";
    const TEST_EVIDENCE_HASH: &str = "b3:3333333333333333333333333333333333333333333333333333333333333333";
    const TEST_TARGET: &str = "x86_64-unknown-linux-gnu";
    const TEST_TOOLCHAIN: &str = "rustc-test";

    fn address(name: &str, fill: char) -> FunctionAddressRecord {
        address_at(TEST_PATH, name, fill)
    }

    fn address_at(path: &str, name: &str, fill: char) -> FunctionAddressRecord {
        FunctionAddressRecord {
            path: path.to_owned(),
            name: name.to_owned(),
            function_address: format!("b3:{}", fill.to_string().repeat(HASH_HEX_LENGTH)),
            source_identity: TEST_SOURCE_ID.to_owned(),
        }
    }

    fn policy(call_summaries: Vec<CallSummary>) -> WorkspacePolicy {
        WorkspacePolicy {
            schema_version: WORKSPACE_POLICY_SCHEMA_VERSION.to_owned(),
            profile_id: DETERMINISTIC_CORE_PROFILE.to_owned(),
            function_address_baseline: "evidence/functions.json".to_owned(),
            module_roles: vec![ModuleRoleAssignment {
                path: TEST_PATH.to_owned(),
                role: ModuleRole::Core,
            }],
            call_summaries,
            non_claims: vec!["selected source-token closure only".to_owned()],
        }
    }

    fn input(source: &str, addresses: Vec<FunctionAddressRecord>, policy: WorkspacePolicy) -> ClosureInput {
        ClosureInput {
            profile_id: DETERMINISTIC_CORE_PROFILE.to_owned(),
            profile_hash: TEST_PROFILE_HASH.to_owned(),
            config_hash: TEST_CONFIG_HASH.to_owned(),
            command_blake3: TEST_ARTIFACT_HASH.to_owned(),
            check_status_blake3: TEST_ARTIFACT_HASH.to_owned(),
            function_address_artifact_blake3: TEST_ARTIFACT_HASH.to_owned(),
            function_address_record_root: TEST_RECORD_ROOT.to_owned(),
            workspace_policy_blake3: TEST_POLICY_HASH.to_owned(),
            cargo_graph_blake3: TEST_GRAPH_HASH.to_owned(),
            cargo_lock_blake3: TEST_LOCK_HASH.to_owned(),
            target: TEST_TARGET.to_owned(),
            toolchain: TEST_TOOLCHAIN.to_owned(),
            sources: vec![SourceEnvelope {
                path: TEST_PATH.to_owned(),
                source: source.to_owned(),
                source_identity: TEST_SOURCE_ID.to_owned(),
            }],
            function_addresses: addresses,
            resolved_providers: Vec::new(),
            catalog: embedded_catalog().expect("embedded catalog"),
            policy,
        }
    }

    fn toolchain_summary(
        function_address: &str,
        shape: &str,
        occurrence: usize,
        provider_name: &str,
        effects: Vec<Effect>,
    ) -> (CallSummary, ProviderIdentity) {
        let provider = ProviderIdentity {
            kind: ProviderKind::Toolchain,
            name: provider_name.to_owned(),
            version: TEST_TOOLCHAIN.to_owned(),
            source: "rustc-toolchain".to_owned(),
            target: TEST_TARGET.to_owned(),
            features: Vec::new(),
        };
        let summary = CallSummary {
            call_site_id: call_site_identity(function_address, shape, occurrence),
            review_label: format!("reviewed `{shape}` toolchain call"),
            callee: shape.trim_start_matches("call:").to_owned(),
            provider: provider.clone(),
            effects,
            evidence: EvidenceReference {
                kind: "toolchain-api-review-v1".to_owned(),
                locator: format!("rustc-version#{provider_name}"),
                blake3: provider_identity_blake3(&provider),
            },
        };
        (summary, provider)
    }

    // r[verify octet.effect_closure.catalog.valid]
    // r[verify octet.effect_closure.catalog.invalid]
    #[test]
    fn embedded_catalog_and_workspace_policy_validate() {
        let catalog = embedded_catalog().expect("valid embedded catalog");
        assert!(validate_catalog(&catalog).is_empty());
        assert!(validate_workspace_policy(&policy(Vec::new())).is_empty());

        let mut invalid = policy(Vec::new());
        invalid.non_claims.clear();
        invalid.module_roles.push(invalid.module_roles[0].clone());
        invalid.call_summaries.push(CallSummary {
            call_site_id: full_hash("invalid unknown summary"),
            review_label: "invalid unknown summary".to_owned(),
            callee: "opaque".to_owned(),
            provider: ProviderIdentity {
                kind: ProviderKind::Toolchain,
                name: "std".to_owned(),
                version: TEST_TOOLCHAIN.to_owned(),
                source: "rustc-toolchain".to_owned(),
                target: TEST_TARGET.to_owned(),
                features: Vec::new(),
            },
            effects: vec![Effect::Unknown],
            evidence: EvidenceReference {
                kind: "toolchain-api-review-v1".to_owned(),
                locator: "rustc-version#std".to_owned(),
                blake3: TEST_EVIDENCE_HASH.to_owned(),
            },
        });
        let diagnostics = validate_workspace_policy(&invalid);
        assert!(diagnostics.iter().any(|message| message.contains("non_claims")));
        assert!(diagnostics.iter().any(|message| message.contains("duplicate module role")));
        assert!(diagnostics.iter().any(|message| message.contains("must not declare `unknown`")));
    }

    // r[verify octet.effect_closure.inference.direct]
    // r[verify octet.effect_closure.transitive.propagates]
    // r[verify octet.effect_closure.evidence.visible]
    #[test]
    fn selected_local_closure_propagates_deterministically() {
        let source = "fn leaf() { let _values: Vec<u8> = Vec::new(); }\nfn root() { leaf(); }\n";
        let leaf_address = address("leaf", '4');
        let (summary, provider) =
            toolchain_summary(&leaf_address.function_address, "call:Vec::new", 0, "alloc", vec![Effect::Alloc]);
        let mut closure_input = input(source, vec![leaf_address, address("root", '5')], policy(vec![summary]));
        closure_input.resolved_providers.push(provider);

        let first = compute_closure(closure_input.clone());
        let second = compute_closure(closure_input);

        assert_eq!(first, second);
        assert_eq!(first.status, AdmissionStatus::Admitted);
        assert!(first.diagnostics.is_empty());
        let root = first.nodes.iter().find(|node| node.name == "root").expect("root node");
        assert_eq!(root.closure_effects, vec![Effect::Alloc]);
        assert!(verify_receipt_identity(&first).is_empty());
    }

    // r[verify octet.effect_closure.transitive.cycle]
    #[test]
    fn recursive_cycle_converges_without_termination_claim() {
        let source = "fn left() { right(); }\nfn right() { let _values: Vec<u8> = Vec::new(); left(); }\n";
        let right_address = address("right", '7');
        let (summary, provider) =
            toolchain_summary(&right_address.function_address, "call:Vec::new", 0, "alloc", vec![Effect::Alloc]);
        let mut closure_input = input(source, vec![address("left", '6'), right_address], policy(vec![summary]));
        closure_input.resolved_providers.push(provider);
        let receipt = compute_closure(closure_input);

        assert_eq!(receipt.status, AdmissionStatus::Admitted);
        assert!(receipt.nodes.iter().all(|node| node.closure_effects == [Effect::Alloc]));
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.inference.unknown]
    #[test]
    fn unknown_and_stale_dependency_edges_fail_closed() {
        let source = "fn root() { mystery(); }\n";
        let function_address = address("root", '8');
        let missing = compute_closure(input(source, vec![function_address.clone()], policy(Vec::new())));
        assert_eq!(missing.status, AdmissionStatus::Denied);
        assert!(missing.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_CALL_UNKNOWN));
        assert!(missing.nodes[0].closure_effects.contains(&Effect::Unknown));

        let stale_summary = CallSummary {
            call_site_id: full_hash("stale"),
            review_label: "stale summary".to_owned(),
            callee: "mystery".to_owned(),
            provider: ProviderIdentity {
                kind: ProviderKind::CargoPackage,
                name: "mystery".to_owned(),
                version: "1.0.0".to_owned(),
                source: "registry+test".to_owned(),
                target: TEST_TARGET.to_owned(),
                features: Vec::new(),
            },
            effects: Vec::new(),
            evidence: EvidenceReference {
                kind: "reviewed-source-summary-v1".to_owned(),
                locator: "Cargo.lock#mystery".to_owned(),
                blake3: TEST_EVIDENCE_HASH.to_owned(),
            },
        };
        let stale = compute_closure(input(source, vec![function_address], policy(vec![stale_summary])));
        assert!(stale.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_SUMMARY_STALE));
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    #[test]
    fn apparent_toolchain_intrinsic_requires_exact_summary() {
        let source = "fn root() -> Result<(), ()> { Ok(()) }\n";
        let receipt = compute_closure(input(source, vec![address("root", '3')], policy(Vec::new())));

        assert_eq!(receipt.status, AdmissionStatus::Denied);
        assert_eq!(receipt.nodes[0].calls[0].shape, "call:Ok");
        assert_eq!(receipt.nodes[0].calls[0].resolution, CallResolution::Unknown);
    }

    // r[verify octet.effect_closure.dependency_summaries.valid]
    #[test]
    fn exact_dependency_summary_is_identity_bound() {
        let source = "fn root() { mystery(); }\n";
        let function_address = address("root", '9');
        let call_site_id = call_site_identity(&function_address.function_address, "call:mystery", 0);
        let provider = ProviderIdentity {
            kind: ProviderKind::CargoPackage,
            name: "mystery".to_owned(),
            version: "1.0.0".to_owned(),
            source: "registry+test".to_owned(),
            target: TEST_TARGET.to_owned(),
            features: vec!["std".to_owned()],
        };
        let summary = CallSummary {
            call_site_id,
            review_label: "reviewed pure mystery call".to_owned(),
            callee: "mystery::mystery".to_owned(),
            provider: provider.clone(),
            effects: Vec::new(),
            evidence: EvidenceReference {
                kind: "reviewed-source-summary-v1".to_owned(),
                locator: "Cargo.lock#mystery".to_owned(),
                blake3: TEST_LOCK_HASH.to_owned(),
            },
        };
        let mut closure_input = input(source, vec![function_address.clone()], policy(vec![summary.clone()]));
        closure_input.resolved_providers.push(provider.clone());

        let receipt = compute_closure(closure_input);

        assert_eq!(receipt.status, AdmissionStatus::Admitted);
        assert_eq!(receipt.nodes[0].calls[0].resolution, CallResolution::ReviewedSummary);

        let mut mismatch = input(source, vec![function_address], policy(vec![summary]));
        let mut mismatched_provider = provider;
        mismatched_provider.version = "2.0.0".to_owned();
        mismatch.resolved_providers.push(mismatched_provider);
        let denied = compute_closure(mismatch);
        assert!(denied.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_PROVIDER_MISMATCH));
    }

    // r[verify octet.effect_closure.roles.valid]
    // r[verify octet.effect_closure.roles.invalid]
    #[test]
    fn role_edges_and_missing_function_addresses_fail_closed() {
        const HELPER_PATH: &str = "crates/domain/src/helper.rs";
        let mut closure_input = input(
            "fn root() { helper(); }\n",
            vec![address("root", 'c'), address_at(HELPER_PATH, "helper", 'd')],
            policy(Vec::new()),
        );
        closure_input.sources.push(SourceEnvelope {
            path: HELPER_PATH.to_owned(),
            source: "fn helper() {}\n".to_owned(),
            source_identity: TEST_SOURCE_ID.to_owned(),
        });
        closure_input.policy.module_roles.push(ModuleRoleAssignment {
            path: HELPER_PATH.to_owned(),
            role: ModuleRole::Adapter,
        });

        let role_denied = compute_closure(closure_input);
        assert!(role_denied.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_ROLE));
        assert!(role_denied.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_ROLE_EDGE));

        let address_denied = compute_closure(input("fn root() {}\n", Vec::new(), policy(Vec::new())));
        assert!(address_denied.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_ADDRESS_MISSING));
    }

    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn trait_dispatch_is_unknown_without_exact_summary() {
        let source = "trait Run { fn run(&self); }\nfn root(value: &dyn Run) { value.run(); }\n";
        let receipt = compute_closure(input(source, vec![address("root", 'e')], policy(Vec::new())));

        assert_eq!(receipt.status, AdmissionStatus::Denied);
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_CALL_UNKNOWN));
        assert!(receipt.nodes[0].closure_effects.contains(&Effect::Unknown));
    }

    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn known_macro_tokens_remain_unknown_without_expansion() {
        let source = "fn root() { let _values = vec![1_u8]; }\n";
        let receipt = compute_closure(input(source, vec![address("root", 'f')], policy(Vec::new())));

        assert_eq!(receipt.status, AdmissionStatus::Denied);
        assert!(receipt.nodes[0].direct_effects.contains(&Effect::Alloc));
        assert!(receipt.nodes[0].direct_effects.contains(&Effect::Unknown));
    }

    // r[verify octet.effect_closure.inference.direct]
    // r[verify octet.effect_closure.inference.unknown]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn hidden_panic_unsafe_cfg_ffi_and_unknown_macro_are_denied() {
        let source = r#"
extern "C" { fn external_call(); }
#[cfg(feature = "special")]
unsafe fn risky(values: &[u8]) -> u8 {
    opaque!();
    external_call();
    values[1] + 1
}
"#;
        let receipt = compute_closure(input(source, vec![address("risky", 'a')], policy(Vec::new())));
        let effects = &receipt.nodes[0].closure_effects;

        assert_eq!(receipt.status, AdmissionStatus::Denied);
        assert!(effects.contains(&Effect::Panic));
        assert!(effects.contains(&Effect::Unsafe));
        assert!(effects.contains(&Effect::Ffi));
        assert!(effects.contains(&Effect::Unknown));
    }

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn receipt_identity_rejects_tampering() {
        let source = "fn root() { let _values: Vec<u8> = Vec::new(); }\n";
        let root_address = address("root", 'b');
        let (summary, provider) =
            toolchain_summary(&root_address.function_address, "call:Vec::new", 0, "alloc", vec![Effect::Alloc]);
        let mut closure_input = input(source, vec![root_address], policy(vec![summary]));
        closure_input.resolved_providers.push(provider);
        let mut receipt = compute_closure(closure_input);
        assert!(verify_receipt_identity(&receipt).is_empty());

        let mut coherently_rewritten = receipt.clone();
        coherently_rewritten.nodes[0].calls[0].resolution = CallResolution::Intrinsic;
        let coherently_rewritten = finalize_receipt(coherently_rewritten);
        let structural_diagnostics = verify_receipt_identity(&coherently_rewritten);
        assert!(
            structural_diagnostics
                .iter()
                .any(|message| message.contains("unsupported name-based intrinsic resolution"))
        );
        assert!(!structural_diagnostics.iter().any(|message| message.contains(DIAGNOSTIC_RECEIPT_IDENTITY)));

        receipt.nodes[0].closure_effects.clear();
        let diagnostics = verify_receipt_identity(&receipt);
        assert!(diagnostics.iter().any(|message| message.contains(DIAGNOSTIC_RECEIPT_IDENTITY)));
    }
}
