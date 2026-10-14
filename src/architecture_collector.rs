use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::path::PathBuf;

use octet_architecture_ir::CapabilityFamily;
use octet_architecture_ir::CollectionStatus;
use octet_architecture_ir::CollectorCompilationUnit;
use octet_architecture_ir::CompilerArchitectureCollectorContext;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureRole;
use octet_architecture_ir::CompilerArchitectureShard;
use octet_architecture_ir::CompilerValueFlowSelection;
use octet_architecture_ir::DispatchKind;
use octet_architecture_ir::ItemKind;
use octet_architecture_ir::SourceOrigin;
use octet_architecture_ir::SourceSpan;
use octet_architecture_ir::TypeKind;
use octet_architecture_ir::UnknownCompilerFact;
use octet_architecture_ir::UnknownFactKind;
use octet_architecture_ir::ValueProvenanceStatus;
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_hir::def_id::LocalDefId;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::mir;
use rustc_middle::ty;
use rustc_middle::ty::Ty;
use rustc_session::impl_lint_pass;
use rustc_span::Span;
use rustc_span::Spanned;
use rustc_span::sym;

const CONTEXT_BYTE_LIMIT: u64 = 16 * 1024 * 1024;
const ITEM_ID_DOMAIN: &str = "octet.compiler-architecture-ir.item/v1";
const TYPE_ID_DOMAIN: &str = "octet.compiler-architecture-ir.type/v1";
const SIGNATURE_ID_DOMAIN: &str = "octet.compiler-architecture-ir.signature/v1";
const IMPLEMENTATION_ID_DOMAIN: &str = "octet.compiler-architecture-ir.implementation/v1";
const CONFIG_FACT_PREFIX: &str = "active";
const TEMP_FILE_SUFFIX: &str = "tmp";
const REMAPPED_WORKSPACE_PREFIX: &str = "<WORKSPACE>/";
const MAX_EXPANSION_CALLSITE_DEPTH: usize = 64;
const PARAMETER_ORIGIN_PREFIX: &str = "octet-parameter:";
const UNKNOWN_ORIGIN_PREFIX: &str = "octet-unknown:";
const VALUE_FLOW_LIMIT_TARGET: &str = "compiler-selected-value-flow";

type ValueOrigins = BTreeSet<String>;
type LocalValueOrigins = BTreeMap<mir::Local, ValueOrigins>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ProvenanceObservationKey {
    callee_definition: String,
    argument_position: usize,
    argument_type: String,
    span_lo: u32,
    span_hi: u32,
}

#[derive(Clone, Debug)]
struct ProvenanceObservation {
    key: ProvenanceObservationKey,
    origins: ValueOrigins,
    span: Span,
}

#[derive(Clone, Debug, Default)]
struct MirValueFlowResult {
    return_origins: ValueOrigins,
    observations: Vec<ProvenanceObservation>,
    argument_updates: Vec<((LocalDefId, usize), ValueOrigins)>,
}
const EFFECT_PATHS: &[(&str, CapabilityFamily)] = &[
    ("std::env::", CapabilityFamily::Environment),
    ("std::fs::", CapabilityFamily::Filesystem),
    ("std::net::", CapabilityFamily::Network),
    ("std::process::", CapabilityFamily::Process),
    ("std::thread::", CapabilityFamily::Concurrency),
    ("std::time::Instant::now", CapabilityFamily::Time),
    ("std::time::SystemTime::now", CapabilityFamily::Time),
];

pub(crate) fn active_collector() -> Result<Option<ArchitectureCollector>, String> {
    let Some(context_path) = std::env::var_os(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV)
    else {
        return Ok(None);
    };
    let context_path = PathBuf::from(context_path);
    let metadata = fs::metadata(&context_path)
        .map_err(|error| format!("read compiler architecture collector context {}: {error}", context_path.display()))?;
    if metadata.len() > CONTEXT_BYTE_LIMIT {
        return Err(format!(
            "compiler architecture collector context is {} bytes, limit is {CONTEXT_BYTE_LIMIT}",
            metadata.len()
        ));
    }
    let bytes = fs::read(&context_path)
        .map_err(|error| format!("read compiler architecture collector context {}: {error}", context_path.display()))?;
    let context = serde_json::from_slice::<CompilerArchitectureCollectorContext>(&bytes).map_err(|error| {
        format!("parse compiler architecture collector context {}: {error}", context_path.display())
    })?;
    if context.schema_version != octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_SCHEMA_VERSION {
        return Err(format!("unsupported compiler architecture collector context schema `{}`", context.schema_version));
    }
    let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| rustc_crate_name());
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let selected = select_unit(&context.selected_units, &crate_name, &manifest_dir)?;
    Ok(selected.map(|unit| ArchitectureCollector::new(context, unit)))
}

pub(crate) fn validate_compiler_session(
    sess: &rustc_session::Session,
    collector: &ArchitectureCollector,
) -> Result<(), String> {
    let observed_target = sess.target.llvm_target.as_ref();
    if observed_target != collector.selected.unit.target_triple {
        return Err(format!(
            "compiler target `{observed_target}` does not match selected architecture unit target `{}`",
            collector.selected.unit.target_triple
        ));
    }
    let mut observed_crate_types = sess.opts.crate_types.iter().map(ToString::to_string).collect::<Vec<_>>();
    observed_crate_types.sort();
    observed_crate_types.dedup();
    if !observed_crate_types.is_empty() && observed_crate_types != collector.selected.unit.crate_types {
        return Err(format!(
            "compiler crate types {:?} do not match selected architecture unit crate types {:?}",
            observed_crate_types, collector.selected.unit.crate_types
        ));
    }
    Ok(())
}

fn rustc_crate_name() -> String {
    std::env::var("CARGO_PKG_NAME").unwrap_or_default().replace('-', "_")
}

fn select_unit(
    selected_units: &[CollectorCompilationUnit],
    crate_name: &str,
    manifest_dir: &str,
) -> Result<Option<CollectorCompilationUnit>, String> {
    let mut matches = selected_units
        .iter()
        .filter(|selected| {
            selected.rustc_crate_name == crate_name && manifest_matches(manifest_dir, &selected.manifest_dir)
        })
        .cloned()
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        return Err(format!(
            "compiler architecture collector context has multiple units for crate `{crate_name}` and manifest `{manifest_dir}`"
        ));
    }
    Ok(matches.pop())
}

fn manifest_matches(observed: &str, selected_relative: &str) -> bool {
    if observed.is_empty() {
        return true;
    }
    if selected_relative.is_empty() {
        return Path::new(observed).join("Cargo.toml").is_file();
    }
    Path::new(observed).ends_with(selected_relative)
}

#[derive(Clone)]
pub(crate) struct ArchitectureCollector {
    context: CompilerArchitectureCollectorContext,
    selected: CollectorCompilationUnit,
    spans: BTreeMap<String, SourceSpan>,
    facts: BTreeMap<String, CompilerArchitectureFact>,
    unknowns: BTreeMap<String, UnknownCompilerFact>,
    item_ids: HashMap<LocalDefId, String>,
    type_ids: BTreeSet<String>,
    module_paths: BTreeSet<String>,
    selected_type_items: BTreeMap<String, (LocalDefId, String, String)>,
    incomplete: bool,
}

impl ArchitectureCollector {
    fn new(context: CompilerArchitectureCollectorContext, selected: CollectorCompilationUnit) -> Self {
        Self {
            spans: BTreeMap::new(),
            facts: BTreeMap::new(),
            unknowns: BTreeMap::new(),
            item_ids: HashMap::new(),
            type_ids: BTreeSet::new(),
            module_paths: BTreeSet::new(),
            selected_type_items: BTreeMap::new(),
            incomplete: false,
            context,
            selected,
        }
    }

    fn collect_item(
        &mut self,
        cx: &LateContext<'_>,
        def_id: LocalDefId,
        item_kind: ItemKind,
        span: Span,
        signature: bool,
    ) -> String {
        let qualified_path = qualified_definition(cx, def_id.to_def_id());
        let span_id = self.collect_span(cx, span);
        let signature_id = signature.then(|| signature_shape_identity(cx, def_id));
        let item_id = item_identity(
            &self.selected.unit.package_name,
            &qualified_path,
            item_kind,
            signature_id.as_deref(),
            &self.selected.unit.source_blake3,
        );
        self.insert_item_identity(def_id, &item_id);
        if let Some(signature_id) = &signature_id {
            self.collect_signature(cx, def_id, &item_id, signature_id);
        }
        let mut fact = CompilerArchitectureFact::Item {
            fact_id: String::new(),
            item_id: item_id.clone(),
            qualified_path: qualified_path.clone(),
            item_kind,
            origin_crate: self.selected.unit.package_name.clone(),
            visibility: if cx.tcx.visibility(def_id).is_public() {
                "public".to_owned()
            } else {
                "restricted".to_owned()
            },
            signature_id,
            span_id: span_id.clone(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);
        self.collect_function_safety(cx, def_id, item_kind, &item_id, &qualified_path, &span_id);
        self.remember_selected_type(cx, def_id, item_kind, &qualified_path, &span_id);
        self.collect_module(&qualified_path, item_kind, &span_id);
        if span.from_expansion() {
            self.collect_expansion(cx, span, &span_id);
        }
        item_id
    }

    fn collect_function_safety(
        &mut self,
        cx: &LateContext<'_>,
        def_id: LocalDefId,
        item_kind: ItemKind,
        item_id: &str,
        qualified_path: &str,
        span_id: &str,
    ) {
        if item_kind != ItemKind::Function {
            return;
        }
        let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
        let output_definition = type_definition_path(cx, signature.output());
        let selected = output_definition.as_ref().is_some_and(|definition| {
            self.context
                .value_flow_selections
                .iter()
                .any(|selection| selection.tracked_type_definitions.iter().any(|tracked| tracked == definition))
        });
        if !selected {
            return;
        }
        let output_type_id = self.collect_type(cx, signature.output());
        let mut fact = CompilerArchitectureFact::FunctionSafety {
            fact_id: String::new(),
            item_id: item_id.to_owned(),
            qualified_path: qualified_path.to_owned(),
            output_type_id,
            is_unsafe: signature.safety.is_unsafe(),
            span_id: span_id.to_owned(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);
    }

    fn remember_selected_type(
        &mut self,
        cx: &LateContext<'_>,
        def_id: LocalDefId,
        item_kind: ItemKind,
        qualified_path: &str,
        span_id: &str,
    ) {
        if !matches!(item_kind, ItemKind::Struct | ItemKind::Enum | ItemKind::Union) {
            return;
        }
        let selected = self
            .context
            .value_flow_selections
            .iter()
            .any(|selection| selection.tracked_type_definitions.iter().any(|tracked| tracked == qualified_path));
        if !selected {
            return;
        }
        let type_id = self.collect_type(cx, cx.tcx.type_of(def_id).instantiate_identity());
        self.selected_type_items.insert(qualified_path.to_owned(), (def_id, type_id, span_id.to_owned()));
    }

    fn collect_signature(&mut self, cx: &LateContext<'_>, def_id: LocalDefId, owner_item_id: &str, signature_id: &str) {
        let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
        let input_type_ids = signature.inputs().iter().map(|current| self.collect_type(cx, *current)).collect();
        let output_type_id = self.collect_type(cx, signature.output());
        let mut fact = CompilerArchitectureFact::Signature {
            fact_id: String::new(),
            signature_id: signature_id.to_owned(),
            owner_item_id: owner_item_id.to_owned(),
            input_type_ids,
            output_type_id,
            is_public: cx.tcx.visibility(def_id).is_public(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);
    }

    fn collect_type(&mut self, cx: &LateContext<'_>, resolved: Ty<'_>) -> String {
        let display = resolved.to_string();
        let (definition_path, origin_crate, type_kind) = type_metadata(cx, resolved);
        let type_id = octet_architecture_ir::canonical_identity(
            TYPE_ID_DOMAIN,
            &(&display, &definition_path, &origin_crate, type_kind, &self.selected.unit.source_blake3),
        )
        .expect("compiler type identity must serialize");
        if self.type_ids.contains(&type_id) {
            return type_id;
        }
        if self.type_ids.len() >= self.context.limits.max_types {
            self.incomplete = true;
            return type_id;
        }
        self.type_ids.insert(type_id.clone());
        {
            let mut fact = CompilerArchitectureFact::Type {
                fact_id: String::new(),
                type_id: type_id.clone(),
                display,
                definition_path,
                origin_crate,
                type_kind,
            };
            assign_fact_identity(&mut fact);
            self.push_fact(fact);
        }
        type_id
    }

    fn collect_module(&mut self, qualified_path: &str, item_kind: ItemKind, span_id: &str) {
        let module_path = if item_kind == ItemKind::Module {
            qualified_path
        } else {
            let Some((module_path, _)) = qualified_path.rsplit_once("::") else {
                return;
            };
            module_path
        };
        if self.module_paths.contains(module_path) {
            return;
        }
        if self.module_paths.len() >= self.context.limits.max_modules {
            self.incomplete = true;
            return;
        }
        self.module_paths.insert(module_path.to_owned());
        let mut fact = CompilerArchitectureFact::Module {
            fact_id: String::new(),
            module_path: module_path.to_owned(),
            span_id: span_id.to_owned(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);
    }

    fn collect_span(&mut self, cx: &LateContext<'_>, span: Span) -> String {
        let source_map = cx.sess().source_map();
        let workspace_span = workspace_source_span(cx, span);
        let resolved_span = workspace_span.unwrap_or(span);
        let filename = source_map.span_to_filename(resolved_span);
        let raw_path = filename.prefer_local_unconditionally().to_string();
        let external_source = workspace_span.is_none();
        let origin = normalized_workspace_source_path(&raw_path).map_or_else(
            || SourceOrigin::ExternalGenerated {
                provenance_blake3: octet_architecture_ir::framed_blake3(
                    "octet.compiler-architecture-ir.external-source/v1",
                    &[raw_path.as_bytes()],
                ),
            },
            |source_path| SourceOrigin::Workspace { source_path },
        );
        let location = source_map.lookup_char_pos(resolved_span.lo());
        let byte_start = resolved_span.lo().0.saturating_sub(location.file.start_pos.0);
        let byte_end = resolved_span.hi().0.saturating_sub(location.file.start_pos.0);
        let mut record = SourceSpan {
            span_id: String::new(),
            origin,
            byte_start: u64::from(byte_start),
            byte_end: u64::from(byte_end),
            line_start: bounded_source_line(location.line, self.context.limits.max_span_bytes, &mut self.incomplete),
            column_start: bounded_source_column(
                location.col.0,
                self.context.limits.max_span_bytes,
                &mut self.incomplete,
            ),
            from_expansion: span.from_expansion(),
        };
        record.span_id = octet_architecture_ir::source_span_identity(&record);
        if self.spans.len() < self.context.limits.max_spans {
            self.spans.entry(record.span_id.clone()).or_insert(record.clone());
        } else {
            self.incomplete = true;
        }
        if external_source {
            if let SourceOrigin::ExternalGenerated { provenance_blake3 } = &record.origin {
                self.push_unknown(
                    UnknownFactKind::OpaqueGeneratedCode,
                    None,
                    Some(provenance_blake3),
                    Some(&record.span_id),
                    "compiler span has no workspace call site; external source is content-redacted",
                );
            } else {
                self.incomplete = true;
            }
        }
        record.span_id
    }

    fn collect_expansion(&mut self, cx: &LateContext<'_>, span: Span, generated_span_id: &str) {
        let callsite = span.source_callsite();
        let callsite_span_id = self.collect_span(cx, callsite);
        let macro_definition = span
            .macro_backtrace()
            .find_map(|expansion| expansion.macro_def_id.map(|def_id| cx.tcx.def_path_str(def_id)));
        let mut fact = CompilerArchitectureFact::Expansion {
            fact_id: String::new(),
            generated_span_id: generated_span_id.to_owned(),
            callsite_span_id,
            macro_definition: macro_definition.clone(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);
        if macro_definition.is_none() {
            self.push_unknown(
                UnknownFactKind::UnsupportedExpansion,
                None,
                None,
                Some(generated_span_id),
                "expanded source has no compiler-resolved macro definition",
            );
        }
    }

    fn collect_call(
        &mut self,
        cx: &LateContext<'_>,
        expr: &hir::Expr<'_>,
        def_id: Option<DefId>,
        dispatch: DispatchKind,
    ) {
        let caller_item_id = self.caller_item_id(cx, expr.hir_id);
        let span_id = self.collect_span(cx, expr.span);
        let resolved_definition = def_id.map(|current| qualified_definition(cx, current));
        if def_id.is_none() {
            self.push_unknown(
                UnknownFactKind::UnresolvedDefinition,
                Some(&caller_item_id),
                None,
                Some(&span_id),
                "call expression has no compiler-resolved definition",
            );
        }
        let callee_item_id = def_id
            .filter(|current| current.is_local())
            .map(|current| self.local_item_identity(cx, current.expect_local()));
        let mut fact = CompilerArchitectureFact::Call {
            fact_id: String::new(),
            caller_item_id: caller_item_id.clone(),
            callee_item_id,
            resolved_definition: resolved_definition.clone(),
            dispatch,
            span_id: span_id.clone(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);

        if let Some(definition) = resolved_definition.as_deref()
            && let Some(family) = effect_family(definition)
        {
            let mut effect = CompilerArchitectureFact::Effect {
                fact_id: String::new(),
                caller_item_id: caller_item_id.clone(),
                capability_family: family,
                primitive_definition: definition.to_owned(),
                span_id: span_id.clone(),
            };
            assign_fact_identity(&mut effect);
            self.push_fact(effect);
        }
        if let Some(def_id) = def_id
            && definition_is_foreign(cx, def_id)
        {
            self.push_unknown(
                UnknownFactKind::Ffi,
                Some(&caller_item_id),
                resolved_definition.as_deref(),
                Some(&span_id),
                "foreign item call requires consumer-owned authority analysis",
            );
        }
        if dispatch == DispatchKind::Dynamic {
            self.push_unknown(
                UnknownFactKind::DynamicDispatch,
                Some(&caller_item_id),
                resolved_definition.as_deref(),
                Some(&span_id),
                "dynamic dispatch target is not statically unique",
            );
        }
        self.collect_constructor(cx, expr, &caller_item_id, def_id, &span_id);
        if expr.span.from_expansion() {
            self.collect_expansion(cx, expr.span, &span_id);
        }
    }

    fn collect_constructor(
        &mut self,
        cx: &LateContext<'_>,
        expr: &hir::Expr<'_>,
        caller_item_id: &str,
        def_id: Option<DefId>,
        span_id: &str,
    ) {
        let resolved = cx.typeck_results().expr_ty(expr);
        if !matches!(resolved.kind(), ty::Adt(..)) {
            return;
        }
        let constructed_type_id = self.collect_type(cx, resolved);
        let constructor_definition = def_id.map(|current| qualified_definition(cx, current));
        let mut fact = CompilerArchitectureFact::Constructor {
            fact_id: String::new(),
            caller_item_id: caller_item_id.to_owned(),
            constructed_type_id,
            constructor_definition,
            span_id: span_id.to_owned(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);
    }

    fn local_item_identity(&self, cx: &LateContext<'_>, def_id: LocalDefId) -> String {
        let qualified_path = qualified_definition(cx, def_id.to_def_id());
        let kind = def_kind_to_item_kind(cx.tcx.def_kind(def_id));
        let signature_id = matches!(kind, ItemKind::Function).then(|| signature_shape_identity(cx, def_id));
        item_identity(
            &self.selected.unit.package_name,
            &qualified_path,
            kind,
            signature_id.as_deref(),
            &self.selected.unit.source_blake3,
        )
    }

    fn caller_item_id(&mut self, cx: &LateContext<'_>, hir_id: hir::HirId) -> String {
        let owner = cx.tcx.hir_enclosing_body_owner(hir_id);
        if let Some(item_id) = self.item_ids.get(&owner) {
            return item_id.clone();
        }
        if cx.tcx.def_kind(owner) == DefKind::Closure {
            let span = cx.tcx.def_span(owner);
            let span_id = self.collect_span(cx, span);
            let qualified_path = closure_qualified_path(cx, span, &self.selected.unit.package_name);
            let item_id = item_identity(
                &self.selected.unit.package_name,
                &qualified_path,
                ItemKind::Other,
                None,
                &self.selected.unit.source_blake3,
            );
            let mut fact = CompilerArchitectureFact::Item {
                fact_id: String::new(),
                item_id: item_id.clone(),
                qualified_path,
                item_kind: ItemKind::Other,
                origin_crate: self.selected.unit.package_name.clone(),
                visibility: "private".to_owned(),
                signature_id: None,
                span_id,
            };
            assign_fact_identity(&mut fact);
            self.push_fact(fact);
            self.insert_item_identity(owner, &item_id);
            return item_id;
        }
        self.collect_item(cx, owner, ItemKind::Function, cx.tcx.def_span(owner), true)
    }

    fn insert_item_identity(&mut self, def_id: LocalDefId, item_id: &str) {
        if let std::collections::hash_map::Entry::Occupied(mut existing) = self.item_ids.entry(def_id) {
            existing.insert(item_id.to_owned());
            return;
        }
        if self.item_ids.len() >= self.context.limits.max_items {
            self.incomplete = true;
            return;
        }
        self.item_ids.insert(def_id, item_id.to_owned());
    }

    fn push_fact(&mut self, fact: CompilerArchitectureFact) {
        if self.fact_limit_reached(&fact) {
            self.incomplete = true;
            return;
        }
        self.facts.entry(fact.fact_id().to_owned()).or_insert(fact);
    }

    fn fact_limit_reached(&self, candidate: &CompilerArchitectureFact) -> bool {
        let (observed, limit) = match candidate {
            CompilerArchitectureFact::Module { .. } => (self.count_facts(is_module), self.context.limits.max_modules),
            CompilerArchitectureFact::Item { .. } => (self.count_facts(is_item), self.context.limits.max_items),
            CompilerArchitectureFact::Type { .. } => (self.count_facts(is_type), self.context.limits.max_types),
            CompilerArchitectureFact::Trait { .. } => (self.count_facts(is_trait), self.context.limits.max_traits),
            CompilerArchitectureFact::Implementation { .. } => {
                (self.count_facts(is_implementation), self.context.limits.max_implementations)
            }
            CompilerArchitectureFact::Signature { .. } => {
                (self.count_facts(is_signature), self.context.limits.max_signatures)
            }
            CompilerArchitectureFact::Call { .. } => (self.count_facts(is_call), self.context.limits.max_calls),
            CompilerArchitectureFact::Constructor { .. } => {
                (self.count_facts(is_constructor), self.context.limits.max_constructors)
            }
            CompilerArchitectureFact::Effect { .. } => (self.count_facts(is_effect), self.context.limits.max_effects),
            CompilerArchitectureFact::Expansion { .. } => {
                (self.count_facts(is_expansion), self.context.limits.max_expansions)
            }
            CompilerArchitectureFact::Configuration { .. } => {
                (self.count_facts(is_configuration), self.context.limits.max_configurations)
            }
            CompilerArchitectureFact::TypeShape { .. } => {
                (self.count_facts(is_type_shape), self.context.limits.max_type_shapes)
            }
            CompilerArchitectureFact::FunctionSafety { .. } => {
                (self.count_facts(is_function_safety), self.context.limits.max_function_safety_facts)
            }
            CompilerArchitectureFact::CallArgumentProvenance { .. } => {
                (self.count_facts(is_value_provenance), self.context.limits.max_value_provenance_facts)
            }
        };
        observed >= limit && !self.facts.contains_key(candidate.fact_id())
    }

    fn count_facts(&self, predicate: fn(&CompilerArchitectureFact) -> bool) -> usize {
        self.facts.values().filter(|fact| predicate(fact)).count()
    }

    fn push_unknown(
        &mut self,
        kind: UnknownFactKind,
        owner_item_id: Option<&str>,
        target_identity: Option<&str>,
        span_id: Option<&str>,
        detail: &str,
    ) {
        let mut unknown = UnknownCompilerFact {
            unknown_id: String::new(),
            kind,
            owner_item_id: owner_item_id.map(str::to_owned),
            target_identity: target_identity.map(str::to_owned),
            span_id: span_id.map(str::to_owned),
            detail: detail.to_owned(),
        };
        unknown.unknown_id = octet_architecture_ir::unknown_fact_identity(&unknown);
        if self.unknowns.contains_key(&unknown.unknown_id) {
            return;
        }
        if self.unknowns.len() >= self.context.limits.max_unknowns {
            self.incomplete = true;
            return;
        }
        self.unknowns.insert(unknown.unknown_id.clone(), unknown);
    }

    fn collect_role_fact(&mut self, cx: &LateContext<'_>, span: Span) {
        let span_id = self.collect_span(cx, span);
        let source_role = match self.spans.get(&span_id).map(|current| &current.origin) {
            Some(SourceOrigin::ExternalGenerated { .. }) => CompilerArchitectureRole::Generated,
            Some(SourceOrigin::Workspace { source_path }) => self
                .selected
                .source_roles
                .iter()
                .find(|scope| path_in_scope(source_path, &scope.source_scope))
                .map(|scope| scope.role)
                .unwrap_or_else(|| {
                    self.incomplete = true;
                    self.selected.unit.role
                }),
            None => {
                self.incomplete = true;
                self.selected.unit.role
            }
        };
        let mut fact = CompilerArchitectureFact::Configuration {
            fact_id: String::new(),
            predicate: format!("architecture-role={source_role:?}").to_ascii_lowercase(),
            active: true,
            span_id: Some(span_id),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);
    }

    fn collect_selected_type_shapes(&mut self, cx: &LateContext<'_>) {
        let selected_types = self.selected_type_items.clone();
        for (qualified_path, (def_id, type_id, span_id)) in selected_types {
            let adt_def = cx.tcx.adt_def(def_id);
            let mut field_visibilities = adt_def
                .all_fields()
                .map(|field| match field.vis {
                    ty::Visibility::Public => "public".to_owned(),
                    ty::Visibility::Restricted(module_def_id) => {
                        format!("restricted:{}", qualified_definition(cx, module_def_id))
                    }
                })
                .collect::<Vec<_>>();
            field_visibilities.sort();
            let mut field_type_definitions = adt_def
                .all_fields()
                .filter_map(|field| type_definition_path(cx, cx.tcx.type_of(field.did).instantiate_identity()))
                .collect::<Vec<_>>();
            field_type_definitions.sort();
            field_type_definitions.dedup();
            let mut implemented_traits = BTreeSet::new();
            for (trait_def_id, impl_def_ids) in cx.tcx.all_local_trait_impls(()).iter() {
                for impl_def_id in impl_def_ids {
                    let trait_ref = cx.tcx.impl_trait_ref(*impl_def_id).instantiate_identity();
                    let ty::Adt(self_adt, _) = trait_ref.self_ty().kind() else {
                        continue;
                    };
                    if self_adt.did() == def_id.to_def_id() {
                        implemented_traits.insert(qualified_definition(cx, *trait_def_id));
                    }
                }
            }
            let mut fact = CompilerArchitectureFact::TypeShape {
                fact_id: String::new(),
                type_id,
                definition_path: qualified_path,
                field_visibilities,
                field_type_definitions,
                implemented_traits: implemented_traits.into_iter().collect(),
                span_id,
            };
            assign_fact_identity(&mut fact);
            self.push_fact(fact);
        }
    }

    fn collect_selected_value_provenance(&mut self, cx: &LateContext<'_>) {
        if self.context.value_flow_selections.is_empty() {
            return;
        }
        let mut body_def_ids = self
            .item_ids
            .keys()
            .copied()
            .filter(|def_id| {
                matches!(cx.tcx.def_kind(*def_id), DefKind::Fn | DefKind::AssocFn | DefKind::Closure)
                    && cx.tcx.is_mir_available(def_id.to_def_id())
            })
            .collect::<Vec<_>>();
        body_def_ids.sort_by_key(|def_id| qualified_definition(cx, def_id.to_def_id()));
        body_def_ids.dedup();
        if body_def_ids.len() > self.context.limits.max_value_flow_bodies {
            self.push_unknown(
                UnknownFactKind::UnsupportedCompilerShape,
                None,
                Some(VALUE_FLOW_LIMIT_TARGET),
                None,
                "selected MIR body count exceeded the configured value-flow bound",
            );
            return;
        }
        let tracked_callees =
            selected_names(&self.context.value_flow_selections, |selection| &selection.tracked_callee_definitions);
        let tracked_types =
            selected_names(&self.context.value_flow_selections, |selection| &selection.tracked_type_definitions);
        let root_definitions =
            selected_names(&self.context.value_flow_selections, |selection| &selection.root_origin_definitions);
        let propagation_definitions =
            selected_names(&self.context.value_flow_selections, |selection| &selection.propagation_definitions);
        let mut argument_origins = HashMap::<(LocalDefId, usize), ValueOrigins>::new();
        let mut return_origins = HashMap::<LocalDefId, ValueOrigins>::new();
        let mut final_observations =
            BTreeMap::<(String, ProvenanceObservationKey), (LocalDefId, ProvenanceObservation)>::new();
        let mut steps = 0usize;
        let mut converged = false;
        for _ in 0..self.context.limits.max_value_flow_rounds {
            let mut next_argument_origins = HashMap::<(LocalDefId, usize), ValueOrigins>::new();
            let mut next_return_origins = HashMap::<LocalDefId, ValueOrigins>::new();
            let mut round_observations =
                BTreeMap::<(String, ProvenanceObservationKey), (LocalDefId, ProvenanceObservation)>::new();
            for def_id in &body_def_ids {
                let body = cx.tcx.optimized_mir(def_id.to_def_id());
                let result = analyze_selected_mir_body(
                    cx,
                    *def_id,
                    body,
                    &tracked_callees,
                    &tracked_types,
                    &root_definitions,
                    &propagation_definitions,
                    &argument_origins,
                    &return_origins,
                    self.context.limits.max_value_flow_steps,
                    &mut steps,
                );
                let Ok(result) = result else {
                    self.push_unknown(
                        UnknownFactKind::UnsupportedCompilerShape,
                        None,
                        Some(VALUE_FLOW_LIMIT_TARGET),
                        None,
                        "selected MIR transfer steps exceeded the configured value-flow bound",
                    );
                    return;
                };
                union_origin_map_entry(&mut next_return_origins, *def_id, &result.return_origins);
                for (key, origins) in result.argument_updates {
                    union_origin_map_entry(&mut next_argument_origins, key, &origins);
                }
                for observation in result.observations {
                    let key = (qualified_definition(cx, def_id.to_def_id()), observation.key.clone());
                    match round_observations.get_mut(&key) {
                        Some((_, existing)) => {
                            existing.origins.extend(observation.origins);
                        }
                        None => {
                            round_observations.insert(key, (*def_id, observation));
                        }
                    }
                }
            }
            let changed = next_argument_origins != argument_origins || next_return_origins != return_origins;
            argument_origins = next_argument_origins;
            return_origins = next_return_origins;
            final_observations = round_observations;
            if !changed {
                converged = true;
                break;
            }
        }
        if !converged {
            self.push_unknown(
                UnknownFactKind::AmbiguousValueProvenance,
                None,
                Some(VALUE_FLOW_LIMIT_TARGET),
                None,
                "selected MIR provenance did not converge within the configured round bound",
            );
            return;
        }
        for (_, (caller_def_id, observation)) in final_observations {
            let caller_item_id = self.local_item_identity(cx, caller_def_id);
            let argument_type_id = self.collect_type_from_definition(cx, &observation.key.argument_type);
            let span_id = self.collect_span(cx, observation.span);
            let (origin_definitions, status) = canonical_provenance(&observation.origins);
            let mut fact = CompilerArchitectureFact::CallArgumentProvenance {
                fact_id: String::new(),
                caller_item_id: caller_item_id.clone(),
                callee_definition: observation.key.callee_definition.clone(),
                argument_position: observation.key.argument_position,
                argument_type_id,
                origin_definitions,
                status,
                span_id: span_id.clone(),
            };
            assign_fact_identity(&mut fact);
            self.push_fact(fact);
            if status != ValueProvenanceStatus::Exact {
                let kind = if status == ValueProvenanceStatus::Merged {
                    UnknownFactKind::AmbiguousValueProvenance
                } else {
                    UnknownFactKind::MissingValueProvenance
                };
                self.push_unknown(
                    kind,
                    Some(&caller_item_id),
                    Some(&observation.key.callee_definition),
                    Some(&span_id),
                    "selected call argument provenance is not one exact compiler-resolved origin",
                );
            }
        }
    }

    fn collect_type_from_definition(&mut self, cx: &LateContext<'_>, definition: &str) -> String {
        if let Some((_, type_id, _)) = self.selected_type_items.get(definition) {
            return type_id.clone();
        }
        if let Some(type_id) = self.facts.values().find_map(|fact| match fact {
            CompilerArchitectureFact::Type {
                type_id,
                definition_path: Some(current),
                ..
            } if current == definition => Some(type_id.clone()),
            _ => None,
        }) {
            return type_id;
        }
        let fallback = octet_architecture_ir::canonical_identity(
            TYPE_ID_DOMAIN,
            &(
                definition,
                Some(definition),
                &self.selected.unit.package_name,
                TypeKind::Adt,
                &self.selected.unit.source_blake3,
            ),
        )
        .expect("selected compiler type identity must serialize");
        if !self.type_ids.contains(&fallback) {
            let selected = self.selected_type_items.get(definition).map(|(def_id, _, _)| *def_id);
            if let Some(def_id) = selected {
                return self.collect_type(cx, cx.tcx.type_of(def_id).instantiate_identity());
            }
        }
        fallback
    }

    fn collect_active_configuration(&mut self, cx: &LateContext<'_>) {
        if cx.tcx.hir_attrs(hir::CRATE_HIR_ID).iter().any(|attribute| attribute.has_name(sym::no_std)) {
            let mut no_std = CompilerArchitectureFact::Configuration {
                fact_id: String::new(),
                predicate: format!("crate:no-std={}", self.selected.unit.package_name),
                active: true,
                span_id: None,
            };
            assign_fact_identity(&mut no_std);
            self.push_fact(no_std);
        }
        for feature in self.selected.unit.features.clone() {
            let mut fact = CompilerArchitectureFact::Configuration {
                fact_id: String::new(),
                predicate: format!("{CONFIG_FACT_PREFIX}:feature={feature}"),
                active: true,
                span_id: None,
            };
            assign_fact_identity(&mut fact);
            self.push_fact(fact);
        }
        let mut target = CompilerArchitectureFact::Configuration {
            fact_id: String::new(),
            predicate: format!("{CONFIG_FACT_PREFIX}:target={}", self.selected.unit.target_triple),
            active: true,
            span_id: None,
        };
        assign_fact_identity(&mut target);
        self.push_fact(target);
        let mut active_cfgs = cx
            .sess()
            .psess
            .config
            .iter()
            .map(|(name, value)| match value {
                Some(value) => format!("{CONFIG_FACT_PREFIX}:cfg:{name}={value}"),
                None => format!("{CONFIG_FACT_PREFIX}:cfg:{name}"),
            })
            .collect::<Vec<_>>();
        active_cfgs.sort();
        active_cfgs.dedup();
        for predicate in active_cfgs {
            let mut fact = CompilerArchitectureFact::Configuration {
                fact_id: String::new(),
                predicate,
                active: true,
                span_id: None,
            };
            assign_fact_identity(&mut fact);
            self.push_fact(fact);
        }
    }

    fn emit_shard(&mut self, cx: &LateContext<'_>) -> Result<(), String> {
        self.collect_selected_type_shapes(cx);
        self.collect_selected_value_provenance(cx);
        self.collect_active_configuration(cx);
        let mut shard = CompilerArchitectureShard {
            schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION.to_owned(),
            shard_id: String::new(),
            unit: self.selected.unit.clone(),
            collection_status: if self.incomplete {
                CollectionStatus::Incomplete
            } else {
                CollectionStatus::Complete
            },
            spans: self.spans.values().cloned().collect(),
            facts: self.facts.values().cloned().collect(),
            unknowns: self.unknowns.values().cloned().collect(),
            non_claims: self.context.non_claims.clone(),
        };
        shard.shard_id = octet_architecture_ir::shard_identity(&shard);
        let mut bytes = serde_json::to_vec_pretty(&shard)
            .map_err(|error| format!("serialize compiler architecture shard: {error}"))?;
        bytes.push(b'\n');
        if bytes.len() > self.context.limits.max_shard_bytes {
            return Err(format!(
                "compiler architecture shard is {} bytes, policy limit is {}",
                bytes.len(),
                self.context.limits.max_shard_bytes
            ));
        }
        let output_directory = Path::new(&self.context.output_directory);
        let output_path = output_directory.join(format!("{}.json", shard.shard_id));
        let temporary_path = output_path.with_extension(TEMP_FILE_SUFFIX);
        fs::write(&temporary_path, bytes)
            .map_err(|error| format!("write compiler architecture shard {}: {error}", temporary_path.display()))?;
        fs::rename(&temporary_path, &output_path)
            .map_err(|error| format!("publish compiler architecture shard {}: {error}", output_path.display()))?;
        Ok(())
    }
}

fn bounded_source_line(line: usize, max_span_bytes: usize, incomplete: &mut bool) -> u32 {
    let one_based_line = line.max(1);
    bounded_source_coordinate(one_based_line, max_span_bytes, incomplete)
}

fn bounded_source_column(column: usize, max_span_bytes: usize, incomplete: &mut bool) -> u32 {
    let one_based_column = match column.checked_add(1) {
        Some(value) => value,
        None => {
            *incomplete = true;
            1
        }
    };
    bounded_source_coordinate(one_based_column, max_span_bytes, incomplete)
}

fn bounded_source_coordinate(coordinate: usize, max_span_bytes: usize, incomplete: &mut bool) -> u32 {
    let coordinate_limit = max_span_bytes.max(1);
    let bounded = coordinate.min(coordinate_limit);
    if bounded != coordinate {
        *incomplete = true;
    }
    match u32::try_from(bounded) {
        Ok(value) => value,
        Err(_) => {
            *incomplete = true;
            1
        }
    }
}

impl_lint_pass!(ArchitectureCollector => []);

impl<'tcx> LateLintPass<'tcx> for ArchitectureCollector {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let (kind, signature) = item_kind(&item.kind);
        let item_id = self.collect_item(cx, item.owner_id.def_id, kind, item.span, signature);
        self.collect_role_fact(cx, item.span);
        match &item.kind {
            hir::ItemKind::Trait(..) => {
                let mut fact = CompilerArchitectureFact::Trait {
                    fact_id: String::new(),
                    trait_id: item_id,
                    qualified_path: qualified_definition(cx, item.owner_id.def_id.to_def_id()),
                    origin_crate: self.selected.unit.package_name.clone(),
                    span_id: self.collect_span(cx, item.span),
                };
                assign_fact_identity(&mut fact);
                self.push_fact(fact);
            }
            hir::ItemKind::Impl(..) => {
                let self_type = cx.tcx.type_of(item.owner_id.def_id).instantiate_identity();
                let self_type_id = self.collect_type(cx, self_type);
                let trait_definition = match &item.kind {
                    hir::ItemKind::Impl(impl_block) => impl_block
                        .of_trait
                        .as_ref()
                        .and_then(|trait_ref| trait_ref.trait_ref.trait_def_id())
                        .map(|def_id| qualified_definition(cx, def_id)),
                    _ => None,
                };
                let trait_id = match &item.kind {
                    hir::ItemKind::Impl(impl_block) => impl_block
                        .of_trait
                        .as_ref()
                        .and_then(|trait_ref| trait_ref.trait_ref.trait_def_id())
                        .filter(|def_id| def_id.is_local())
                        .map(|def_id| self.local_item_identity(cx, def_id.expect_local())),
                    _ => None,
                };
                let implementation_id = octet_architecture_ir::canonical_identity(
                    IMPLEMENTATION_ID_DOMAIN,
                    &(&item_id, &trait_definition, &self_type_id, &self.selected.unit.source_blake3),
                )
                .expect("compiler implementation identity must serialize");
                let mut fact = CompilerArchitectureFact::Implementation {
                    fact_id: String::new(),
                    implementation_id,
                    trait_id,
                    trait_definition,
                    self_type_id,
                    span_id: self.collect_span(cx, item.span),
                };
                assign_fact_identity(&mut fact);
                self.push_fact(fact);
            }
            hir::ItemKind::ForeignMod { .. } => {
                let span_id = self.collect_span(cx, item.span);
                self.push_unknown(
                    UnknownFactKind::Ffi,
                    Some(&item_id),
                    None,
                    Some(&span_id),
                    "foreign module requires consumer-owned authority analysis",
                );
            }
            _ => {}
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::ImplItem<'tcx>) {
        let signature = matches!(item.kind, hir::ImplItemKind::Fn(..));
        self.collect_item(cx, item.owner_id.def_id, ItemKind::Function, item.span, signature);
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::TraitItem<'tcx>) {
        let signature = matches!(item.kind, hir::TraitItemKind::Fn(..));
        let item_id = self.collect_item(cx, item.owner_id.def_id, ItemKind::Function, item.span, signature);
        if matches!(item.kind, hir::TraitItemKind::Fn(_, hir::TraitFn::Required(_))) {
            let span_id = self.collect_span(cx, item.span);
            self.push_unknown(
                UnknownFactKind::MissingBody,
                Some(&item_id),
                None,
                Some(&span_id),
                "required trait method has no body in this compilation unit",
            );
        }
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        match &expr.kind {
            hir::ExprKind::Call(callee, _) => {
                let def_id = match &callee.kind {
                    hir::ExprKind::Path(qpath) => cx.qpath_res(qpath, callee.hir_id).opt_def_id(),
                    _ => None,
                };
                let dispatch = def_id.map_or(DispatchKind::Unknown, |current| dispatch_kind(cx, current, None));
                self.collect_call(cx, expr, def_id, dispatch);
            }
            hir::ExprKind::MethodCall(_, receiver, _, _) => {
                let def_id = cx.typeck_results().type_dependent_def_id(expr.hir_id);
                let receiver_ty = cx.typeck_results().expr_ty(receiver);
                let dispatch =
                    def_id.map_or(DispatchKind::Unknown, |current| dispatch_kind(cx, current, Some(receiver_ty)));
                self.collect_call(cx, expr, def_id, dispatch);
            }
            _ => {}
        }
    }

    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        if let Err(error) = self.emit_shard(cx) {
            cx.tcx.dcx().err(error);
        }
    }
}

fn qualified_definition(cx: &LateContext<'_>, def_id: DefId) -> String {
    let path = cx.tcx.def_path_str(def_id);
    if def_id.is_local() {
        let crate_name = cx.tcx.crate_name(def_id.krate).to_string();
        if path == crate_name || path.starts_with(&format!("{crate_name}::")) {
            path
        } else {
            format!("{crate_name}::{path}")
        }
    } else {
        path
    }
}

fn selected_names<'a>(
    selections: &'a [CompilerValueFlowSelection],
    select: impl Fn(&'a CompilerValueFlowSelection) -> &'a [String],
) -> BTreeSet<String> {
    selections.iter().flat_map(select).cloned().collect()
}

#[allow(clippy::too_many_arguments)]
fn analyze_selected_mir_body<'tcx>(
    cx: &LateContext<'tcx>,
    def_id: LocalDefId,
    body: &mir::Body<'tcx>,
    tracked_callees: &BTreeSet<String>,
    tracked_types: &BTreeSet<String>,
    root_definitions: &BTreeSet<String>,
    propagation_definitions: &BTreeSet<String>,
    argument_origins: &HashMap<(LocalDefId, usize), ValueOrigins>,
    return_origins: &HashMap<LocalDefId, ValueOrigins>,
    max_steps: usize,
    steps: &mut usize,
) -> Result<MirValueFlowResult, ()> {
    let mut result = MirValueFlowResult::default();
    let mut entry_states = BTreeMap::<mir::BasicBlock, LocalValueOrigins>::new();
    let mut start_state = LocalValueOrigins::new();
    let definition = qualified_definition(cx, def_id.to_def_id());
    for (position, local) in body.args_iter().enumerate() {
        if tracked_type_definition(cx, body.local_decls[local].ty, tracked_types).is_none() {
            continue;
        }
        let origins = argument_origins
            .get(&(def_id, position))
            .cloned()
            .unwrap_or_else(|| BTreeSet::from([parameter_origin(&definition, position)]));
        start_state.insert(local, origins);
    }
    entry_states.insert(mir::START_BLOCK, start_state);
    let mut queue = VecDeque::from([mir::START_BLOCK]);
    while let Some(block) = queue.pop_front() {
        let mut state = entry_states.get(&block).cloned().unwrap_or_default();
        let block_data = &body.basic_blocks[block];
        for statement in &block_data.statements {
            increment_value_flow_step(steps, max_steps)?;
            let mir::StatementKind::Assign(assignment) = &statement.kind else {
                continue;
            };
            let (destination, rvalue) = &**assignment;
            let destination_ty = destination.ty(&body.local_decls, cx.tcx).ty;
            if tracked_type_definition(cx, destination_ty, tracked_types).is_none() {
                continue;
            }
            let origins = rvalue_origins(cx, body, rvalue, &state, tracked_types);
            state.insert(destination.local, origins);
        }
        increment_value_flow_step(steps, max_steps)?;
        match &block_data.terminator().kind {
            mir::TerminatorKind::Call {
                func,
                args,
                destination,
                fn_span,
                ..
            } => {
                analyze_call_terminator(
                    cx,
                    def_id,
                    body,
                    func,
                    args,
                    Some(*destination),
                    *fn_span,
                    tracked_callees,
                    tracked_types,
                    root_definitions,
                    propagation_definitions,
                    argument_origins,
                    return_origins,
                    &mut state,
                    &mut result,
                );
            }
            mir::TerminatorKind::TailCall { func, args, fn_span } => {
                analyze_call_terminator(
                    cx,
                    def_id,
                    body,
                    func,
                    args,
                    None,
                    *fn_span,
                    tracked_callees,
                    tracked_types,
                    root_definitions,
                    propagation_definitions,
                    argument_origins,
                    return_origins,
                    &mut state,
                    &mut result,
                );
            }
            mir::TerminatorKind::Return => {
                if let Some(origins) = state.get(&mir::RETURN_PLACE) {
                    result.return_origins.extend(origins.iter().cloned());
                }
            }
            _ => {}
        }
        for successor in block_data.terminator().successors() {
            let successor_state = entry_states.entry(successor).or_default();
            if union_local_origins(successor_state, &state) {
                queue.push_back(successor);
            }
        }
    }
    result.return_origins.extend(state_origins(&entry_states, mir::RETURN_PLACE));
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn analyze_call_terminator<'tcx>(
    cx: &LateContext<'tcx>,
    caller_def_id: LocalDefId,
    body: &mir::Body<'tcx>,
    func: &mir::Operand<'tcx>,
    args: &[Spanned<mir::Operand<'tcx>>],
    destination: Option<mir::Place<'tcx>>,
    span: Span,
    tracked_callees: &BTreeSet<String>,
    tracked_types: &BTreeSet<String>,
    root_definitions: &BTreeSet<String>,
    propagation_definitions: &BTreeSet<String>,
    _argument_origins: &HashMap<(LocalDefId, usize), ValueOrigins>,
    return_origins: &HashMap<LocalDefId, ValueOrigins>,
    state: &mut LocalValueOrigins,
    result: &mut MirValueFlowResult,
) {
    let Some((callee_def_id, _)) = func.const_fn_def() else {
        for (position, argument) in args.iter().enumerate() {
            let argument_ty = argument.node.ty(&body.local_decls, cx.tcx);
            let Some(argument_type) = tracked_type_definition(cx, argument_ty, tracked_types) else {
                continue;
            };
            result.observations.push(provenance_observation(
                "unresolved-call-target".to_owned(),
                position,
                argument_type,
                BTreeSet::from([unknown_origin("unresolved-call-target")]),
                span,
            ));
        }
        return;
    };
    let callee_definition = qualified_definition(cx, callee_def_id);
    let argument_values = args.iter().map(|argument| operand_origins(&argument.node, state)).collect::<Vec<_>>();
    for (position, argument) in args.iter().enumerate() {
        let argument_ty = argument.node.ty(&body.local_decls, cx.tcx);
        let Some(argument_type) = tracked_type_definition(cx, argument_ty, tracked_types) else {
            continue;
        };
        let origins = argument_values.get(position).cloned().unwrap_or_default();
        if tracked_callees.contains(&callee_definition) {
            result.observations.push(provenance_observation(
                callee_definition.clone(),
                position,
                argument_type.clone(),
                origins.clone(),
                argument.span,
            ));
        }
        if let Some(local_callee) = callee_def_id.as_local() {
            result.argument_updates.push(((local_callee, position), origins));
        }
    }
    let Some(destination) = destination else {
        return;
    };
    let destination_ty = destination.ty(&body.local_decls, cx.tcx).ty;
    if tracked_type_definition(cx, destination_ty, tracked_types).is_none() {
        return;
    }
    let origins = if root_definitions.contains(&callee_definition) {
        BTreeSet::from([callee_definition.clone()])
    } else if propagation_definitions.contains(&callee_definition) {
        union_all_origins(&argument_values)
    } else if let Some(local_callee) = callee_def_id.as_local() {
        substitute_parameter_origins(
            return_origins.get(&local_callee).cloned().unwrap_or_default(),
            &callee_definition,
            &argument_values,
        )
    } else {
        BTreeSet::from([callee_definition])
    };
    state.insert(destination.local, origins);
    let _ = caller_def_id;
}

fn provenance_observation(
    callee_definition: String,
    argument_position: usize,
    argument_type: String,
    origins: ValueOrigins,
    span: Span,
) -> ProvenanceObservation {
    ProvenanceObservation {
        key: ProvenanceObservationKey {
            callee_definition,
            argument_position,
            argument_type,
            span_lo: span.lo().0,
            span_hi: span.hi().0,
        },
        origins,
        span,
    }
}

fn rvalue_origins<'tcx>(
    cx: &LateContext<'tcx>,
    _body: &mir::Body<'tcx>,
    rvalue: &mir::Rvalue<'tcx>,
    state: &LocalValueOrigins,
    tracked_types: &BTreeSet<String>,
) -> ValueOrigins {
    match rvalue {
        mir::Rvalue::Use(operand)
        | mir::Rvalue::Cast(_, operand, _)
        | mir::Rvalue::UnaryOp(_, operand)
        | mir::Rvalue::WrapUnsafeBinder(operand, _) => operand_origins(operand, state),
        mir::Rvalue::Ref(_, _, place) | mir::Rvalue::RawPtr(_, place) | mir::Rvalue::CopyForDeref(place) => {
            place_origins(place, state)
        }
        mir::Rvalue::Aggregate(kind, operands) => {
            let mut origins: ValueOrigins =
                operands.iter().flat_map(|operand| operand_origins(operand, state)).collect();
            if let mir::AggregateKind::Adt(def_id, ..) = &**kind {
                let definition = qualified_definition(cx, *def_id);
                if tracked_types.contains(&definition) {
                    origins.insert(definition);
                }
            }
            origins
        }
        mir::Rvalue::Repeat(operand, _) => operand_origins(operand, state),
        mir::Rvalue::BinaryOp(_, operands) => {
            let mut origins = operand_origins(&operands.0, state);
            origins.extend(operand_origins(&operands.1, state));
            origins
        }
        mir::Rvalue::Discriminant(place) => place_origins(place, state),
        mir::Rvalue::ThreadLocalRef(def_id) => BTreeSet::from([qualified_definition(cx, *def_id)]),
    }
}

fn operand_origins(operand: &mir::Operand<'_>, state: &LocalValueOrigins) -> ValueOrigins {
    match operand {
        mir::Operand::Copy(place) | mir::Operand::Move(place) => place_origins(place, state),
        mir::Operand::Constant(_) | mir::Operand::RuntimeChecks(_) => {
            BTreeSet::from([unknown_origin("constant-or-runtime-check")])
        }
    }
}

fn place_origins(place: &mir::Place<'_>, state: &LocalValueOrigins) -> ValueOrigins {
    state.get(&place.local).cloned().unwrap_or_default()
}

fn tracked_type_definition<'tcx>(
    cx: &LateContext<'tcx>,
    ty: Ty<'tcx>,
    tracked_types: &BTreeSet<String>,
) -> Option<String> {
    let definition = type_definition_path(cx, ty)?;
    tracked_types.contains(&definition).then_some(definition)
}

fn type_definition_path<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<String> {
    match ty.peel_refs().kind() {
        ty::Adt(definition, _) => Some(qualified_definition(cx, definition.did())),
        ty::Alias(_, alias) => Some(qualified_definition(cx, alias.def_id)),
        _ => None,
    }
}

fn union_local_origins(target: &mut LocalValueOrigins, source: &LocalValueOrigins) -> bool {
    let mut changed = false;
    for (local, origins) in source {
        let entry = target.entry(*local).or_default();
        let before = entry.len();
        entry.extend(origins.iter().cloned());
        changed |= entry.len() != before;
    }
    changed
}

fn union_origin_map_entry<K: Eq + Hash>(target: &mut HashMap<K, ValueOrigins>, key: K, origins: &ValueOrigins) -> bool {
    let entry = target.entry(key).or_default();
    let before = entry.len();
    entry.extend(origins.iter().cloned());
    entry.len() != before
}

fn state_origins(states: &BTreeMap<mir::BasicBlock, LocalValueOrigins>, local: mir::Local) -> ValueOrigins {
    states.values().filter_map(|state| state.get(&local)).flatten().cloned().collect()
}

fn union_all_origins(values: &[ValueOrigins]) -> ValueOrigins {
    values.iter().flatten().cloned().collect()
}

fn parameter_origin(definition: &str, position: usize) -> String {
    format!("{PARAMETER_ORIGIN_PREFIX}{definition}:{position}")
}

fn unknown_origin(reason: &str) -> String {
    format!("{UNKNOWN_ORIGIN_PREFIX}{reason}")
}

fn substitute_parameter_origins(
    origins: ValueOrigins,
    callee_definition: &str,
    argument_values: &[ValueOrigins],
) -> ValueOrigins {
    let parameter_prefix = format!("{PARAMETER_ORIGIN_PREFIX}{callee_definition}:");
    let mut resolved = ValueOrigins::new();
    for origin in origins {
        let Some(position) = origin.strip_prefix(&parameter_prefix).and_then(|value| value.parse::<usize>().ok())
        else {
            resolved.insert(origin);
            continue;
        };
        if let Some(argument_origins) = argument_values.get(position) {
            resolved.extend(argument_origins.iter().cloned());
        } else {
            resolved.insert(unknown_origin("missing-wrapper-argument"));
        }
    }
    resolved
}

fn canonical_provenance(origins: &ValueOrigins) -> (Vec<String>, ValueProvenanceStatus) {
    let mut canonical = origins
        .iter()
        .filter(|origin| !origin.starts_with(PARAMETER_ORIGIN_PREFIX) && !origin.starts_with(UNKNOWN_ORIGIN_PREFIX))
        .cloned()
        .collect::<Vec<_>>();
    canonical.sort();
    canonical.dedup();
    let has_unknown = origins
        .iter()
        .any(|origin| origin.starts_with(PARAMETER_ORIGIN_PREFIX) || origin.starts_with(UNKNOWN_ORIGIN_PREFIX));
    let status = if !has_unknown && canonical.len() == 1 {
        ValueProvenanceStatus::Exact
    } else if !has_unknown && canonical.len() > 1 {
        ValueProvenanceStatus::Merged
    } else {
        ValueProvenanceStatus::Unknown
    };
    (canonical, status)
}

fn increment_value_flow_step(steps: &mut usize, max_steps: usize) -> Result<(), ()> {
    *steps = steps.checked_add(1).ok_or(())?;
    (*steps <= max_steps).then_some(()).ok_or(())
}

fn item_kind(kind: &hir::ItemKind<'_>) -> (ItemKind, bool) {
    match kind {
        hir::ItemKind::Mod(..) => (ItemKind::Module, false),
        hir::ItemKind::Fn { .. } => (ItemKind::Function, true),
        hir::ItemKind::Struct(..) => (ItemKind::Struct, false),
        hir::ItemKind::Enum(..) => (ItemKind::Enum, false),
        hir::ItemKind::Union(..) => (ItemKind::Union, false),
        hir::ItemKind::Trait(..) => (ItemKind::Trait, false),
        hir::ItemKind::TyAlias(..) => (ItemKind::TypeAlias, false),
        hir::ItemKind::Impl(..) => (ItemKind::Implementation, false),
        hir::ItemKind::Const(..) => (ItemKind::Constant, false),
        hir::ItemKind::Static(..) => (ItemKind::Static, false),
        hir::ItemKind::ForeignMod { .. } => (ItemKind::Foreign, false),
        _ => (ItemKind::Other, false),
    }
}

fn item_identity(
    package_name: &str,
    qualified_path: &str,
    item_kind: ItemKind,
    signature_id: Option<&str>,
    source_blake3: &str,
) -> String {
    octet_architecture_ir::canonical_identity(
        ITEM_ID_DOMAIN,
        &(package_name, qualified_path, item_kind, signature_id, source_blake3),
    )
    .expect("compiler item identity must serialize")
}

fn signature_shape_identity(cx: &LateContext<'_>, def_id: LocalDefId) -> String {
    let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
    let inputs = signature.inputs().iter().map(ToString::to_string).collect::<Vec<_>>();
    let output = signature.output().to_string();
    octet_architecture_ir::canonical_identity(SIGNATURE_ID_DOMAIN, &(inputs, output))
        .expect("compiler signature shape identity must serialize")
}

fn def_kind_to_item_kind(kind: DefKind) -> ItemKind {
    match kind {
        DefKind::Mod => ItemKind::Module,
        DefKind::Struct => ItemKind::Struct,
        DefKind::Union => ItemKind::Union,
        DefKind::Enum => ItemKind::Enum,
        DefKind::Trait => ItemKind::Trait,
        DefKind::TyAlias => ItemKind::TypeAlias,
        DefKind::Fn | DefKind::AssocFn => ItemKind::Function,
        DefKind::Const { .. } | DefKind::AssocConst { .. } => ItemKind::Constant,
        DefKind::Static { .. } => ItemKind::Static,
        DefKind::ForeignMod | DefKind::ForeignTy => ItemKind::Foreign,
        _ => ItemKind::Other,
    }
}

fn type_metadata(cx: &LateContext<'_>, resolved: Ty<'_>) -> (Option<String>, String, TypeKind) {
    match resolved.kind() {
        ty::Adt(definition, _) => (
            Some(qualified_definition(cx, definition.did())),
            cx.tcx.crate_name(definition.did().krate).to_string(),
            TypeKind::Adt,
        ),
        ty::Dynamic(..) => (None, "dynamic".to_owned(), TypeKind::TraitObject),
        ty::Tuple(..) => (None, "core".to_owned(), TypeKind::Tuple),
        ty::Ref(..) => (None, "core".to_owned(), TypeKind::Reference),
        ty::RawPtr(..) => (None, "core".to_owned(), TypeKind::Pointer),
        ty::FnDef(def_id, _) => (
            Some(qualified_definition(cx, *def_id)),
            cx.tcx.crate_name(def_id.krate).to_string(),
            TypeKind::Function,
        ),
        ty::Alias(..) => (None, "opaque".to_owned(), TypeKind::Opaque),
        ty::Param(..) => (None, "generic".to_owned(), TypeKind::Generic),
        ty::Bool | ty::Char | ty::Int(..) | ty::Uint(..) | ty::Float(..) | ty::Str | ty::Never => {
            (None, "core".to_owned(), TypeKind::Primitive)
        }
        _ => (None, "unknown".to_owned(), TypeKind::Other),
    }
}

fn dispatch_kind(cx: &LateContext<'_>, def_id: DefId, receiver: Option<Ty<'_>>) -> DispatchKind {
    if definition_is_foreign(cx, def_id) {
        return DispatchKind::Foreign;
    }
    if receiver.is_some_and(|resolved| matches!(resolved.peel_refs().kind(), ty::Dynamic(..))) {
        return DispatchKind::Dynamic;
    }
    if matches!(cx.tcx.def_kind(def_id), DefKind::AssocFn | DefKind::AssocConst { .. } | DefKind::AssocTy) {
        DispatchKind::Trait
    } else {
        DispatchKind::Static
    }
}

fn definition_is_foreign(cx: &LateContext<'_>, def_id: DefId) -> bool {
    if cx.tcx.def_kind(def_id) == DefKind::ForeignTy {
        return true;
    }
    cx.tcx.opt_parent(def_id).is_some_and(|parent| cx.tcx.def_kind(parent) == DefKind::ForeignMod)
}

fn effect_family(definition: &str) -> Option<CapabilityFamily> {
    EFFECT_PATHS.iter().find_map(|(prefix, family)| definition.starts_with(prefix).then_some(*family))
}

fn path_in_scope(path: &str, scope: &str) -> bool {
    path == scope || path.strip_prefix(scope).is_some_and(|suffix| suffix.starts_with('/'))
}

fn closure_qualified_path(cx: &LateContext<'_>, span: Span, package_name: &str) -> String {
    let source_map = cx.sess().source_map();
    let resolved_span = workspace_source_span(cx, span).unwrap_or(span);
    let filename = source_map.span_to_filename(resolved_span);
    let raw_path = filename.prefer_local_unconditionally().to_string();
    let source_path = normalized_workspace_source_path(&raw_path).unwrap_or_else(|| {
        let provenance_blake3 =
            octet_architecture_ir::framed_blake3("octet.compiler-architecture-ir.external-source/v1", &[
                raw_path.as_bytes()
            ]);
        format!("generated@{provenance_blake3}")
    });
    let location = source_map.lookup_char_pos(resolved_span.lo());
    let byte_start = resolved_span.lo().0.saturating_sub(location.file.start_pos.0);
    let byte_end = resolved_span.hi().0.saturating_sub(location.file.start_pos.0);
    format!("{package_name}::closure@{source_path}:{byte_start}:{byte_end}")
}

fn workspace_source_span(cx: &LateContext<'_>, span: Span) -> Option<Span> {
    let mut candidate = span;
    for _ in 0..MAX_EXPANSION_CALLSITE_DEPTH {
        let filename = cx.sess().source_map().span_to_filename(candidate);
        if normalized_workspace_source_path(&filename.prefer_local_unconditionally().to_string()).is_some() {
            return Some(candidate);
        }
        let callsite = candidate.source_callsite();
        if callsite == candidate {
            return None;
        }
        candidate = callsite;
    }
    None
}

fn normalized_workspace_source_path(path: &str) -> Option<String> {
    if let Some(relative) = path.strip_prefix(REMAPPED_WORKSPACE_PREFIX) {
        return normalized_relative_path(Path::new(relative));
    }
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        let workspace = std::env::current_dir().ok()?;
        return candidate.strip_prefix(workspace).ok().and_then(normalized_relative_path);
    }
    normalized_relative_path(candidate)
}

fn normalized_relative_path(path: &Path) -> Option<String> {
    if path.as_os_str().is_empty() {
        return None;
    }
    let components = path
        .components()
        .map(|component| component.as_os_str().to_str().map(str::to_owned))
        .collect::<Option<Vec<_>>>()?;
    let normalized = components.join("/");
    (!normalized.is_empty() && !normalized.starts_with('/') && !normalized.contains("..")).then_some(normalized)
}

fn assign_fact_identity(fact: &mut CompilerArchitectureFact) {
    let identity = octet_architecture_ir::fact_identity(fact);
    match fact {
        CompilerArchitectureFact::Module { fact_id, .. }
        | CompilerArchitectureFact::Item { fact_id, .. }
        | CompilerArchitectureFact::Type { fact_id, .. }
        | CompilerArchitectureFact::Trait { fact_id, .. }
        | CompilerArchitectureFact::Implementation { fact_id, .. }
        | CompilerArchitectureFact::Signature { fact_id, .. }
        | CompilerArchitectureFact::Call { fact_id, .. }
        | CompilerArchitectureFact::Constructor { fact_id, .. }
        | CompilerArchitectureFact::Effect { fact_id, .. }
        | CompilerArchitectureFact::Expansion { fact_id, .. }
        | CompilerArchitectureFact::Configuration { fact_id, .. }
        | CompilerArchitectureFact::TypeShape { fact_id, .. }
        | CompilerArchitectureFact::FunctionSafety { fact_id, .. }
        | CompilerArchitectureFact::CallArgumentProvenance { fact_id, .. } => *fact_id = identity,
    }
}

fn is_module(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Module { .. })
}
fn is_item(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Item { .. })
}
fn is_type(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Type { .. })
}
fn is_trait(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Trait { .. })
}
fn is_implementation(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Implementation { .. })
}
fn is_signature(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Signature { .. })
}
fn is_call(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Call { .. })
}
fn is_constructor(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Constructor { .. })
}
fn is_effect(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Effect { .. })
}
fn is_expansion(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Expansion { .. })
}
fn is_configuration(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::Configuration { .. })
}

fn is_type_shape(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::TypeShape { .. })
}

fn is_function_safety(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::FunctionSafety { .. })
}

fn is_value_provenance(fact: &CompilerArchitectureFact) -> bool {
    matches!(fact, CompilerArchitectureFact::CallArgumentProvenance { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_context_does_not_activate_collection() {
        let original = std::env::var_os(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV);
        unsafe {
            std::env::remove_var(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV);
        }
        let result = active_collector();
        if let Some(value) = original {
            unsafe {
                std::env::set_var(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV, value);
            }
        }
        assert!(result.expect("absent collector context").is_none());
    }

    #[test]
    fn effect_classification_uses_resolved_definition_paths() {
        assert_eq!(effect_family("std::fs::read"), Some(CapabilityFamily::Filesystem));
        assert_eq!(effect_family("fixture::comment_named_std_fs_read"), None);
    }
}
