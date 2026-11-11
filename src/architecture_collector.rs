use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::collections::btree_map::Entry;
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
use rustc_hir::LangItem;
use rustc_hir::attrs::AttributeKind;
use rustc_hir::def::CtorKind;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_hir::def_id::DefId;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::find_attr;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::mir;
use rustc_middle::ty;
use rustc_middle::ty::Ty;
use rustc_middle::ty::adjustment::Adjust;
use rustc_middle::ty::adjustment::DerefAdjustKind;
use rustc_session::impl_lint_pass;
use rustc_span::Span;
use rustc_span::Spanned;
use rustc_span::hygiene::AstPass;
use rustc_span::hygiene::DesugaringKind;
use rustc_span::hygiene::ExpnKind;
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
const MUTABLE_ALIAS_REASON: &str = "mutable-alias";
const VALUE_FLOW_LIMIT_TARGET: &str = "compiler-selected-value-flow";
// One counter per `CompilerArchitectureFact` kind; see `fact_kind_limit`.
const FACT_KIND_COUNT: usize = 14;

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
struct ProvenanceObservation<'tcx> {
    key: ProvenanceObservationKey,
    // The compiler type of the argument; its definition path is `key.argument_type`.
    argument_ty: Ty<'tcx>,
    origins: ValueOrigins,
    span: Span,
}

#[derive(Clone, Debug, Default)]
struct MirValueFlowResult<'tcx> {
    return_origins: ValueOrigins,
    observations: Vec<ProvenanceObservation<'tcx>>,
    argument_updates: Vec<((LocalDefId, usize), ValueOrigins)>,
}
// Compiler-resolved definition prefixes. `print!`, `println!`, `eprint!`,
// `eprintln!`, and `dbg!` resolve to `std::io::_print` or `std::io::_eprint`.
const EFFECT_PATHS: &[(&str, CapabilityFamily)] = &[
    ("std::env::", CapabilityFamily::Environment),
    ("std::fs::", CapabilityFamily::Filesystem),
    ("std::net::", CapabilityFamily::Network),
    ("std::process::", CapabilityFamily::Process),
    ("std::thread::", CapabilityFamily::Concurrency),
    ("std::io::_eprint", CapabilityFamily::Observability),
    ("std::io::_print", CapabilityFamily::Observability),
    ("std::io::stderr", CapabilityFamily::Observability),
    ("std::io::stdin", CapabilityFamily::Observability),
    ("std::io::stdout", CapabilityFamily::Observability),
    ("std::time::Instant::now", CapabilityFamily::Time),
    ("std::time::SystemTime::now", CapabilityFamily::Time),
    // `elapsed` reads the clock; `duration_since` only compares two given times.
    ("std::time::Instant::elapsed", CapabilityFamily::Time),
    ("std::time::SystemTime::elapsed", CapabilityFamily::Time),
    // `Path` methods that query the filesystem. Path manipulation such as
    // `join`, `parent`, or `file_name` computes on the path alone.
    ("std::path::Path::canonicalize", CapabilityFamily::Filesystem),
    ("std::path::Path::exists", CapabilityFamily::Filesystem),
    ("std::path::Path::is_dir", CapabilityFamily::Filesystem),
    ("std::path::Path::is_file", CapabilityFamily::Filesystem),
    ("std::path::Path::is_symlink", CapabilityFamily::Filesystem),
    ("std::path::Path::metadata", CapabilityFamily::Filesystem),
    ("std::path::Path::read_dir", CapabilityFamily::Filesystem),
    ("std::path::Path::read_link", CapabilityFamily::Filesystem),
    ("std::path::Path::symlink_metadata", CapabilityFamily::Filesystem),
    ("std::path::Path::try_exists", CapabilityFamily::Filesystem),
];

// Synchronization covers locks, once cells, channels, and atomics; shared
// ownership through `Arc` is not itself a synchronization effect.
const SYNCHRONIZATION_PATHS: &[&str] = &[
    "std::sync::Barrier",
    "std::sync::Condvar",
    "std::sync::LazyLock",
    "std::sync::Mutex",
    "std::sync::Once",
    "std::sync::RwLock",
    "std::sync::atomic::",
    "std::sync::mpmc::",
    "std::sync::mpsc::",
    "core::sync::atomic::",
];

// No other thread can observe a constructor, which creates a value nobody else
// sees yet, or an exclusive-access operation, which takes ownership or `&mut`.
// Every other operation on these types, including ones this list does not
// name, counts.
const NON_SYNCHRONIZING_OPERATIONS: &[&str] = &["new", "get_mut", "into_inner"];

// Plain values defined under otherwise effectful paths. Their operations,
// including overloaded operators such as `==`, compute on the value alone, so
// none of them is an effect. Handles to processes, files, sockets, threads,
// and locks stay classified.
const EFFECT_FREE_VALUE_TYPES: &[&str] = &[
    "core::sync::atomic::Ordering",
    "std::env::JoinPathsError",
    "std::env::VarError",
    "std::fs::FileTimes",
    "std::fs::FileType",
    "std::fs::Metadata",
    "std::fs::Permissions",
    "std::net::AddrParseError",
    "std::net::IpAddr",
    "std::net::Ipv4Addr",
    "std::net::Ipv6Addr",
    "std::net::Shutdown",
    "std::net::SocketAddr",
    "std::net::SocketAddrV4",
    "std::net::SocketAddrV6",
    "std::process::ExitStatus",
    "std::process::ExitStatusError",
    "std::process::Output",
    "std::sync::BarrierWaitResult",
    "std::sync::OnceState",
    "std::sync::atomic::Ordering",
    "std::sync::mpsc::RecvError",
    "std::sync::mpsc::RecvTimeoutError",
    "std::sync::mpsc::SendError",
    "std::sync::mpsc::TryRecvError",
    "std::sync::mpsc::TrySendError",
    "std::thread::AccessError",
    "std::thread::ThreadId",
];

fn load_collector_context() -> Result<Option<(PathBuf, CompilerArchitectureCollectorContext)>, String> {
    load_collector_context_at(std::env::var_os(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV))
}

fn load_collector_context_at(
    context_path: Option<std::ffi::OsString>,
) -> Result<Option<(PathBuf, CompilerArchitectureCollectorContext)>, String> {
    let Some(context_path) = context_path else {
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
    octet_architecture_ir::validate_collector_units(&context.selected_units).map_err(|error| error.to_string())?;
    Ok(Some((context_path, context)))
}

/// Selects the architecture unit compiled by this session and records its freshness stamp in
/// rustc dep-info, so Cargo recompiles the unit whenever the runner rewrites or removes the stamp.
pub(crate) fn active_collector(sess: &rustc_session::Session) -> Result<Option<ArchitectureCollector>, String> {
    let Some((context_path, context)) = load_collector_context()? else {
        return Ok(None);
    };
    let workspace_root = fs::canonicalize(&context.workspace_root)
        .map_err(|error| format!("resolve architecture workspace: {error}"))?;
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .ok_or_else(|| "compiler architecture session has no manifest directory".to_owned())?;
    let manifest_dir =
        fs::canonicalize(manifest_dir).map_err(|error| format!("resolve compiler manifest directory: {error}"))?;
    let Ok(manifest_relative) = manifest_dir.strip_prefix(&workspace_root) else {
        return Ok(None);
    };
    let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| rustc_crate_name());
    let input = sess
        .io
        .input
        .opt_path()
        .ok_or_else(|| "compiler architecture session has no source path".to_owned())?;
    let input = fs::canonicalize(input).map_err(|error| format!("resolve compiler source root: {error}"))?;
    let source_relative = input
        .strip_prefix(&workspace_root)
        .map_err(|_| "selected compiler source root is outside the workspace".to_owned())?;
    let mut features = sess
        .psess
        .config
        .iter()
        .filter_map(|(name, value)| {
            (name.as_str() == "feature").then(|| value.map(|value| value.to_string())).flatten()
        })
        .collect::<Vec<_>>();
    features.sort();
    features.dedup();
    let test_configuration =
        sess.opts.test || sess.psess.config.iter().any(|(name, value)| name.as_str() == "test" && value.is_none());
    let selected = octet_architecture_ir::select_compilation_unit(
        &context.selected_units,
        &octet_architecture_ir::CompilerUnitObservation {
            crate_name: &crate_name,
            manifest_dir: &manifest_relative.to_string_lossy(),
            source_path: &source_relative.to_string_lossy(),
            test_configuration,
            target_triple: sess.opts.target_triple.tuple(),
            features: &features,
        },
    )
    .map_err(|error| error.to_string())?
    .cloned();
    let Some(selected) = selected else {
        return Ok(None);
    };
    let stamp_path = octet_architecture_ir::compilation_unit_stamp_path(&context_path, &selected.unit);
    let stamp_path = stamp_path
        .to_str()
        .ok_or_else(|| format!("compiler architecture unit stamp path is not UTF-8: {}", stamp_path.display()))?;
    sess.file_depinfo.lock().insert(rustc_span::Symbol::intern(stamp_path));
    Ok(Some(ArchitectureCollector::new(context, selected)))
}

pub(crate) fn validate_compiler_session(
    sess: &rustc_session::Session,
    collector: &ArchitectureCollector,
) -> Result<(), String> {
    // The planner records Rust target tuples (`rustc -vV` host, `--target`);
    // LLVM target names differ for targets such as `aarch64-apple-darwin`.
    let observed_target = sess.opts.target_triple.tuple();
    if observed_target != collector.selected.unit.target_triple {
        return Err(format!(
            "compiler target `{observed_target}` does not match selected architecture unit target `{}`",
            collector.selected.unit.target_triple
        ));
    }
    let mut observed_crate_types = if sess.opts.test {
        vec!["bin".to_owned()]
    } else {
        sess.opts.crate_types.iter().map(ToString::to_string).collect::<Vec<_>>()
    };
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

#[derive(Clone)]
pub(crate) struct ArchitectureCollector {
    context: CompilerArchitectureCollectorContext,
    selected: CollectorCompilationUnit,
    spans: BTreeMap<String, SourceSpan>,
    facts: BTreeMap<String, CompilerArchitectureFact>,
    // Stored facts per kind, indexed by `fact_kind_limit`, so limits never rescan `facts`.
    fact_counts: [usize; FACT_KIND_COUNT],
    unknowns: BTreeMap<String, UnknownCompilerFact>,
    item_ids: HashMap<LocalDefId, String>,
    type_ids: BTreeSet<String>,
    module_paths: BTreeSet<String>,
    selected_type_items: BTreeMap<String, (LocalDefId, String, String)>,
    // `None` for the package's library target; see `cargo_target_qualifier`.
    target_qualifier: Option<String>,
    // Local functions named outside a direct call, or called from a body value
    // flow does not analyze; their callers are not all visible to value flow.
    indirect_functions: HashSet<LocalDefId>,
    incomplete: bool,
}

impl ArchitectureCollector {
    fn new(context: CompilerArchitectureCollectorContext, selected: CollectorCompilationUnit) -> Self {
        Self {
            spans: BTreeMap::new(),
            facts: BTreeMap::new(),
            fact_counts: [0; FACT_KIND_COUNT],
            unknowns: BTreeMap::new(),
            item_ids: HashMap::new(),
            type_ids: BTreeSet::new(),
            module_paths: BTreeSet::new(),
            selected_type_items: BTreeMap::new(),
            target_qualifier: cargo_target_qualifier(
                &selected.unit.target_kind,
                &selected.unit.target_name,
                selected.test_configuration,
            ),
            indirect_functions: HashSet::new(),
            incomplete: false,
            context,
            selected,
        }
    }

    fn item_identity(&self, qualified_path: &str, item_kind: ItemKind, signature_id: Option<&str>) -> String {
        item_identity(
            &self.selected.unit.package_name,
            self.target_qualifier.as_deref(),
            qualified_path,
            item_kind,
            signature_id,
            &self.selected.unit.source_blake3,
        )
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
        let item_id = self.item_identity(&qualified_path, item_kind, signature_id.as_deref());
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
            visibility: if !definition_has_item_visibility(cx.tcx.def_kind(def_id)) {
                // Expression bodies have no independently visible item, and
                // rustc does not provide the visibility query for these keys.
                "private".to_owned()
            } else if cx.tcx.visibility(def_id).is_public() {
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
        self.collect_module(cx, def_id, item_kind, &span_id);
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

    // Inputs name their referent so `&Witness` and `Witness` share one definition, as value
    // provenance does, and keep a by-reference flag so a borrow stays distinct from a move;
    // the output keeps its reference because a borrow constructs nothing.
    fn collect_signature(&mut self, cx: &LateContext<'_>, def_id: LocalDefId, owner_item_id: &str, signature_id: &str) {
        let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
        let input_type_ids =
            signature.inputs().iter().map(|current| self.collect_type(cx, current.peel_refs())).collect();
        let input_by_reference = signature.inputs().iter().map(|current| current.is_ref()).collect();
        let output_type_id = self.collect_type(cx, signature.output());
        let mut fact = CompilerArchitectureFact::Signature {
            fact_id: String::new(),
            signature_id: signature_id.to_owned(),
            owner_item_id: owner_item_id.to_owned(),
            input_type_ids,
            input_by_reference,
            output_type_id,
            is_public: cx.tcx.visibility(def_id).is_public(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);
    }

    fn collect_type(&mut self, cx: &LateContext<'_>, resolved: Ty<'_>) -> String {
        let display = resolved.to_string();
        let (definition_path, origin_crate, type_kind) =
            type_metadata(cx, resolved, &self.selected.unit.package_name, &self.context.selected_units);
        // Types reached from the library crate keep their library identity in
        // every unit; only definitions local to a non-library target are qualified.
        let target_qualifier = self.target_qualifier.as_deref().filter(|_| mentions_local_definition(resolved));
        let type_id = match target_qualifier {
            None => octet_architecture_ir::canonical_identity(
                TYPE_ID_DOMAIN,
                &(&display, &definition_path, &origin_crate, type_kind, &self.selected.unit.source_blake3),
            ),
            Some(target) => octet_architecture_ir::canonical_identity(
                TYPE_ID_DOMAIN,
                &(target, &display, &definition_path, &origin_crate, type_kind, &self.selected.unit.source_blake3),
            ),
        }
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

    // Items nested in impl blocks, functions, or derive-generated anonymous
    // constants are not modules. Use the compiler's containing module rather
    // than truncating the item path, which turned each such parent into a
    // pseudo-module and exhausted `max_modules` on derive-heavy crates.
    fn collect_module(&mut self, cx: &LateContext<'_>, def_id: LocalDefId, item_kind: ItemKind, span_id: &str) {
        let module_def_id = if item_kind == ItemKind::Module {
            def_id
        } else {
            cx.tcx.parent_module_from_def_id(def_id).to_local_def_id()
        };
        let module_path = qualified_definition(cx, module_def_id.to_def_id());
        let module_path = module_path.as_str();
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
        let workspace_span = workspace_source_span(cx, span, &self.context.workspace_root);
        let resolved_span = workspace_span.unwrap_or(span);
        let filename = source_map.span_to_filename(resolved_span);
        let raw_path = filename.prefer_local_unconditionally().to_string();
        let external_source = workspace_span.is_none();
        let origin = normalized_workspace_source_path(&raw_path, &self.context.workspace_root).map_or_else(
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
        self.record_span(&record);
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

    // A span already recorded costs nothing; only a new span can exceed the limit.
    fn record_span(&mut self, record: &SourceSpan) {
        if self.spans.contains_key(&record.span_id) {
            return;
        }
        if self.spans.len() >= self.context.limits.max_spans {
            self.incomplete = true;
            return;
        }
        self.spans.insert(record.span_id.clone(), record.clone());
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
        if macro_definition.is_none() && !compiler_owned_expansion(&span.ctxt().outer_expn_data().kind) {
            self.push_unknown(
                UnknownFactKind::UnsupportedExpansion,
                None,
                None,
                Some(generated_span_id),
                &format!(
                    "expanded source has no compiler-resolved macro definition: {:?}",
                    span.ctxt().outer_expn_data().kind
                ),
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
        let const_evaluated = const_evaluated_call(cx, expr);
        let (caller_item_id, span_id) = self.record_call(cx, expr, def_id, dispatch, const_evaluated, None);
        self.collect_constructor(cx, expr, &caller_item_id, def_id, &span_id);
    }

    // An overloaded operator or deref calls a trait method. Its effect is named by
    // the Self type of the implementation it resolves to, such as
    // `std::sync::LazyLock::deref`, because the trait method names no effect.
    fn collect_overloaded_call<'tcx>(
        &mut self,
        cx: &LateContext<'tcx>,
        expr: &hir::Expr<'_>,
        def_id: DefId,
        args: ty::GenericArgsRef<'tcx>,
    ) {
        let dispatch = dispatch_kind(cx, def_id, Some(args));
        let primitive_definition = overloaded_primitive_definition(cx, def_id, args);
        let const_evaluated = const_evaluated_context(cx, expr.hir_id);
        self.record_call(cx, expr, Some(def_id), dispatch, const_evaluated, primitive_definition.as_deref());
    }

    // Records the call, its effect, and its unknowns; returns the caller item and
    // span identities. The effect is classified by `primitive_definition` when
    // given, by the resolved definition otherwise, and never for a call evaluated
    // at compile time.
    fn record_call(
        &mut self,
        cx: &LateContext<'_>,
        expr: &hir::Expr<'_>,
        def_id: Option<DefId>,
        dispatch: DispatchKind,
        const_evaluated: bool,
        primitive_definition: Option<&str>,
    ) -> (String, String) {
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
        let local_callee_item_id = def_id
            .filter(|current| current.is_local())
            .map(|current| self.local_item_identity(cx, current.expect_local()));
        let mut fact = CompilerArchitectureFact::Call {
            fact_id: String::new(),
            caller_item_id: caller_item_id.clone(),
            callee_item_id: local_callee_item_id,
            resolved_definition: resolved_definition.clone(),
            dispatch,
            span_id: span_id.clone(),
        };
        assign_fact_identity(&mut fact);
        self.push_fact(fact);

        if !const_evaluated
            && let Some(definition) = primitive_definition.or(resolved_definition.as_deref())
            && let Some(family) = def_id
                .and_then(|current| resolved_randomness_family(cx, current))
                .or_else(|| effect_family(definition))
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
        if expr.span.from_expansion() {
            self.collect_expansion(cx, expr.span, &span_id);
        }
        (caller_item_id, span_id)
    }

    // Struct literals and unit constructors build values without a call expression.
    fn collect_construction(&mut self, cx: &LateContext<'_>, expr: &hir::Expr<'_>, def_id: DefId) {
        let caller_item_id = self.caller_item_id(cx, expr.hir_id);
        let span_id = self.collect_span(cx, expr.span);
        self.collect_constructor(cx, expr, &caller_item_id, Some(def_id), &span_id);
        if expr.span.from_expansion() {
            self.collect_expansion(cx, expr.span, &span_id);
        }
    }

    // Value flow takes a parameter's origins from the direct calls to its
    // function, so it must know every other use: a reference that may be called
    // through a pointer, or a call from a body it does not analyze.
    fn record_function_use(&mut self, cx: &LateContext<'_>, hir_id: hir::HirId, def_id: DefId, direct_call: bool) {
        let Some(def_id) = def_id.as_local() else {
            return;
        };
        let analyzed_caller = matches!(
            cx.tcx.def_kind(cx.tcx.hir_enclosing_body_owner(hir_id)),
            DefKind::Fn | DefKind::AssocFn | DefKind::Closure
        );
        if !direct_call || !analyzed_caller {
            self.indirect_functions.insert(def_id);
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
        // `expr?`, `.await`, and `for` lower to calls such as `Try::branch`, `Future::poll`, and
        // `Iterator::next` that only forward what the written expression already produced.
        if [
            DesugaringKind::QuestionMark,
            DesugaringKind::Await,
            DesugaringKind::ForLoop,
        ]
        .into_iter()
        .any(|kind| expr.span.is_desugaring(kind))
        {
            return;
        }
        let constructed = constructed_types(cx, expr, def_id);
        let constructor_definition = def_id.map(|current| qualified_definition(cx, current));
        for current in constructed {
            let constructed_type_id = self.collect_type(cx, current);
            let mut fact = CompilerArchitectureFact::Constructor {
                fact_id: String::new(),
                caller_item_id: caller_item_id.to_owned(),
                constructed_type_id,
                constructor_definition: constructor_definition.clone(),
                span_id: span_id.to_owned(),
            };
            assign_fact_identity(&mut fact);
            self.push_fact(fact);
        }
    }

    fn local_item_identity(&self, cx: &LateContext<'_>, def_id: LocalDefId) -> String {
        let qualified_path = qualified_definition(cx, def_id.to_def_id());
        let kind = def_kind_to_item_kind(cx.tcx.def_kind(def_id));
        let signature_id = matches!(kind, ItemKind::Function).then(|| signature_shape_identity(cx, def_id));
        self.item_identity(&qualified_path, kind, signature_id.as_deref())
    }

    fn caller_item_id(&mut self, cx: &LateContext<'_>, hir_id: hir::HirId) -> String {
        let owner = cx.tcx.hir_enclosing_body_owner(hir_id);
        if let Some(item_id) = self.item_ids.get(&owner) {
            return item_id.clone();
        }
        if cx.tcx.def_kind(owner) == DefKind::Closure {
            let span = cx.tcx.def_span(owner);
            let span_id = self.collect_span(cx, span);
            let qualified_path =
                closure_qualified_path(cx, span, &self.selected.unit.package_name, &self.context.workspace_root);
            let item_id = self.item_identity(&qualified_path, ItemKind::Other, None);
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
        // r[impl octet.compiler_architecture_ir.body_owner_kinds]
        // A call can belong to a constant initializer, not only a function.
        // Keep classification identical to local_item_identity; never invent a
        // function signature or replace the real owner with an enclosing item.
        let kind = def_kind_to_item_kind(cx.tcx.def_kind(owner));
        self.collect_item(cx, owner, kind, cx.tcx.def_span(owner), kind == ItemKind::Function)
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

    // A stored fact costs nothing again; only a new fact of a full kind exceeds its limit.
    fn push_fact(&mut self, fact: CompilerArchitectureFact) {
        if self.facts.contains_key(fact.fact_id()) {
            return;
        }
        let (slot, limit) = self.fact_kind_limit(&fact);
        if self.fact_counts[slot] >= limit {
            self.incomplete = true;
            return;
        }
        self.fact_counts[slot] += 1;
        self.facts.insert(fact.fact_id().to_owned(), fact);
    }

    // The counter slot and collection limit of a fact's kind.
    fn fact_kind_limit(&self, fact: &CompilerArchitectureFact) -> (usize, usize) {
        let limits = &self.context.limits;
        match fact {
            CompilerArchitectureFact::Module { .. } => (0, limits.max_modules),
            CompilerArchitectureFact::Item { .. } => (1, limits.max_items),
            CompilerArchitectureFact::Type { .. } => (2, limits.max_types),
            CompilerArchitectureFact::Trait { .. } => (3, limits.max_traits),
            CompilerArchitectureFact::Implementation { .. } => (4, limits.max_implementations),
            CompilerArchitectureFact::Signature { .. } => (5, limits.max_signatures),
            CompilerArchitectureFact::Call { .. } => (6, limits.max_calls),
            CompilerArchitectureFact::Constructor { .. } => (7, limits.max_constructors),
            CompilerArchitectureFact::Effect { .. } => (8, limits.max_effects),
            CompilerArchitectureFact::Expansion { .. } => (9, limits.max_expansions),
            CompilerArchitectureFact::Configuration { .. } => (10, limits.max_configurations),
            CompilerArchitectureFact::TypeShape { .. } => (11, limits.max_type_shapes),
            CompilerArchitectureFact::FunctionSafety { .. } => (12, limits.max_function_safety_facts),
            CompilerArchitectureFact::CallArgumentProvenance { .. } => (13, limits.max_value_provenance_facts),
        }
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
                .map_or_else(
                    || {
                        self.incomplete = true;
                        self.selected.unit.role
                    },
                    |scope| scope.role,
                ),
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
            for impl_def_ids in cx.tcx.all_local_trait_impls(()).values() {
                for impl_def_id in impl_def_ids {
                    let trait_ref = cx.tcx.impl_trait_ref(*impl_def_id).instantiate_identity();
                    let ty::Adt(self_adt, _) = trait_ref.self_ty().kind() else {
                        continue;
                    };
                    if self_adt.did() == def_id.to_def_id() {
                        implemented_traits.insert(implemented_trait_path(cx, trait_ref));
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

    fn value_flow_body_def_ids(&self, cx: &LateContext<'_>) -> Vec<LocalDefId> {
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
        body_def_ids
    }

    fn collect_selected_value_provenance(&mut self, cx: &LateContext<'_>) {
        if self.context.value_flow_selections.is_empty() {
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
        // A body that never holds or passes a tracked value yields no
        // observation, argument update, or usable return origin, so it costs
        // no transfer steps and does not count toward the body bound.
        let mut tracked_definitions = HashMap::new();
        let body_def_ids = self
            .value_flow_body_def_ids(cx)
            .into_iter()
            .filter(|def_id| {
                let body = cx.tcx.optimized_mir(def_id.to_def_id());
                body_carries_tracked_values(cx, body, &tracked_types, &mut tracked_definitions)
            })
            .collect::<Vec<_>>();
        if body_def_ids.len() > self.context.limits.max_value_flow_bodies {
            self.exceed_value_flow_limit(
                UnknownFactKind::UnsupportedCompilerShape,
                "selected MIR body count exceeded the configured value-flow bound",
            );
            return;
        }
        let caller_seeded_bodies = body_def_ids
            .iter()
            .copied()
            .filter(|def_id| callers_supply_arguments(cx, *def_id, &self.indirect_functions))
            .collect::<HashSet<_>>();
        let mut argument_origins = HashMap::<(LocalDefId, usize), ValueOrigins>::new();
        let mut return_origins = HashMap::<LocalDefId, ValueOrigins>::new();
        let mut final_observations =
            BTreeMap::<(String, ProvenanceObservationKey), (LocalDefId, ProvenanceObservation<'_>)>::new();
        let mut steps = 0usize;
        let mut converged = false;
        for _ in 0..self.context.limits.max_value_flow_rounds {
            let mut next_argument_origins = HashMap::<(LocalDefId, usize), ValueOrigins>::new();
            let mut next_return_origins = HashMap::<LocalDefId, ValueOrigins>::new();
            let mut round_observations =
                BTreeMap::<(String, ProvenanceObservationKey), (LocalDefId, ProvenanceObservation<'_>)>::new();
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
                    caller_seeded_bodies.contains(def_id).then_some(&argument_origins),
                    &return_origins,
                    self.context.limits.max_value_flow_steps,
                    &mut steps,
                );
                let Ok(result) = result else {
                    self.exceed_value_flow_limit(
                        UnknownFactKind::UnsupportedCompilerShape,
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
            self.exceed_value_flow_limit(
                UnknownFactKind::AmbiguousValueProvenance,
                "selected MIR provenance did not converge within the configured round bound",
            );
            return;
        }
        for (_, (caller_def_id, observation)) in final_observations {
            self.push_value_provenance_fact(cx, caller_def_id, &observation);
        }
    }

    // r[impl octet.compiler_architecture_ir.boundary]
    // Value flow stops at its bound, so the shard cannot claim complete facts.
    fn exceed_value_flow_limit(&mut self, kind: UnknownFactKind, detail: &str) {
        self.incomplete = true;
        self.push_unknown(kind, None, Some(VALUE_FLOW_LIMIT_TARGET), None, detail);
    }

    fn push_value_provenance_fact(
        &mut self,
        cx: &LateContext<'_>,
        caller_def_id: LocalDefId,
        observation: &ProvenanceObservation<'_>,
    ) {
        // Value-flow bodies are drawn from recorded items. Reuse that identity:
        // closure items are keyed by source position, which `local_item_identity`
        // does not reproduce.
        let Some(caller_item_id) = self.item_ids.get(&caller_def_id).cloned() else {
            self.incomplete = true;
            return;
        };
        let argument_type_id = self.collect_argument_type(cx, observation);
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

    // Selected local types reuse the identity their TypeShape fact references.
    // Any other tracked type, such as a dependency type reached through a
    // reference, records the observed compiler type so the reference resolves.
    fn collect_argument_type(&mut self, cx: &LateContext<'_>, observation: &ProvenanceObservation<'_>) -> String {
        if let Some((_, type_id, _)) = self.selected_type_items.get(&observation.key.argument_type) {
            return type_id.clone();
        }
        self.collect_type(cx, observation.argument_ty.peel_refs())
    }

    fn collect_active_configuration(&mut self, cx: &LateContext<'_>) {
        // Built-in attributes are parsed in this compiler cohort; name matching
        // only sees unparsed attributes. Query this session's active crate attrs.
        if find_attr!(cx.tcx, crate, AttributeKind::NoStd(_)) {
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
        // Cargo can compile the same proc-macro source concurrently for metadata and linking.
        // Equivalent shards share their content path, but never a mutable staging file.
        let temporary_path = output_path.with_extension(format!("{}.{TEMP_FILE_SUFFIX}", std::process::id()));
        publish_shard(&temporary_path, &output_path, &bytes)
            .map_err(|error| format!("publish compiler architecture shard {}: {error}", output_path.display()))
    }
}

fn publish_shard(temporary: &Path, output: &Path, bytes: &[u8]) -> std::io::Result<()> {
    {
        let mut staged = fs::OpenOptions::new().write(true).create_new(true).open(temporary)?;
        std::io::Write::write_all(&mut staged, bytes)?;
    }
    fs::rename(temporary, output)
}

#[cfg(test)]
mod publication_tests;

fn bounded_source_line(line: usize, max_span_bytes: usize, incomplete: &mut bool) -> u32 {
    let one_based_line = line.max(1);
    bounded_source_coordinate(one_based_line, max_span_bytes, incomplete)
}

fn bounded_source_column(column: usize, max_span_bytes: usize, incomplete: &mut bool) -> u32 {
    let one_based_column = if let Some(value) = column.checked_add(1) {
        value
    } else {
        *incomplete = true;
        1
    };
    bounded_source_coordinate(one_based_column, max_span_bytes, incomplete)
}

fn bounded_source_coordinate(coordinate: usize, max_span_bytes: usize, incomplete: &mut bool) -> u32 {
    let coordinate_limit = max_span_bytes.max(1);
    let bounded = coordinate.min(coordinate_limit);
    if bounded != coordinate {
        *incomplete = true;
    }
    if let Ok(value) = u32::try_from(bounded) {
        value
    } else {
        *incomplete = true;
        1
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
        let item_kind = def_kind_to_item_kind(cx.tcx.def_kind(item.owner_id.def_id));
        self.collect_item(cx, item.owner_id.def_id, item_kind, item.span, signature);
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::TraitItem<'tcx>) {
        let signature = matches!(item.kind, hir::TraitItemKind::Fn(..));
        let item_kind = def_kind_to_item_kind(cx.tcx.def_kind(item.owner_id.def_id));
        let item_id = self.collect_item(cx, item.owner_id.def_id, item_kind, item.span, signature);
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
                let dispatch = def_id.map_or(DispatchKind::Unknown, |current| {
                    dispatch_kind(cx, current, cx.typeck_results().node_args_opt(callee.hir_id))
                });
                if let Some(def_id) = def_id {
                    self.record_function_use(cx, expr.hir_id, def_id, true);
                }
                self.collect_call(cx, expr, def_id, dispatch);
            }
            hir::ExprKind::MethodCall(..) => {
                let def_id = cx.typeck_results().type_dependent_def_id(expr.hir_id);
                let dispatch = def_id.map_or(DispatchKind::Unknown, |current| {
                    dispatch_kind(cx, current, cx.typeck_results().node_args_opt(expr.hir_id))
                });
                if let Some(def_id) = def_id {
                    self.record_function_use(cx, expr.hir_id, def_id, true);
                }
                self.collect_call(cx, expr, def_id, dispatch);
            }
            hir::ExprKind::Struct(qpath, ..) => {
                if let Some(def_id) = struct_literal_definition(cx, expr, qpath) {
                    self.collect_construction(cx, expr, def_id);
                }
            }
            hir::ExprKind::Path(qpath) => {
                if let Some(def_id) = unit_constructor_definition(cx, expr, qpath) {
                    self.collect_construction(cx, expr, def_id);
                } else if let Some(def_id) = function_reference_definition(cx, expr, qpath) {
                    self.record_function_use(cx, expr.hir_id, def_id, false);
                    let dispatch = dispatch_kind(cx, def_id, cx.typeck_results().node_args_opt(expr.hir_id));
                    self.collect_call(cx, expr, Some(def_id), dispatch);
                }
            }
            // Writeback keeps a method only for operators that are not built in.
            hir::ExprKind::Binary(..)
            | hir::ExprKind::AssignOp(..)
            | hir::ExprKind::Unary(..)
            | hir::ExprKind::Index(..) => {
                if let Some(def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id) {
                    self.collect_overloaded_call(cx, expr, def_id, cx.typeck_results().node_args(expr.hir_id));
                }
            }
            _ => {}
        }
        // Autoderef through `Deref` impls, as in `CONFIG.port` on a `LazyLock`,
        // calls `deref` or `deref_mut` on the type before each overloaded step.
        let typeck_results = cx.typeck_results();
        let mut source = typeck_results.expr_ty(expr);
        for adjustment in typeck_results.expr_adjustments(expr) {
            if let Adjust::Deref(DerefAdjustKind::Overloaded(deref)) = adjustment.kind {
                let args = cx.tcx.mk_args(&[source.into()]);
                self.collect_overloaded_call(cx, expr, deref.method_call(cx.tcx), args);
            }
            source = adjustment.target;
        }
    }

    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        if let Err(error) = self.emit_shard(cx) {
            cx.tcx.dcx().err(error);
        }
    }
}

// A struct literal names the struct, union, or variant it builds; `Self { .. }`
// and type aliases resolve through the constructed ADT.
fn struct_literal_definition(cx: &LateContext<'_>, expr: &hir::Expr<'_>, qpath: &hir::QPath<'_>) -> Option<DefId> {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).kind() else {
        return None;
    };
    match cx.qpath_res(qpath, expr.hir_id) {
        Res::Def(DefKind::Struct | DefKind::Union | DefKind::Variant, def_id) => Some(def_id),
        _ => Some(adt.did()),
    }
}

// Unit structs and unit variants are constructed by naming their constant
// constructor; tuple constructors are calls and reach `collect_call` instead.
fn unit_constructor_definition(cx: &LateContext<'_>, expr: &hir::Expr<'_>, qpath: &hir::QPath<'_>) -> Option<DefId> {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).kind() else {
        return None;
    };
    match cx.qpath_res(qpath, expr.hir_id) {
        Res::Def(DefKind::Ctor(_, CtorKind::Const), def_id) => Some(def_id),
        Res::SelfCtor(_) => adt.non_enum_variant().ctor_def_id(),
        _ => None,
    }
}

// A function item named outside a call's callee position, such as
// `unwrap_or_else(Instant::now)`, is a reference the receiver may call. Record
// it as a call so effects and dispatch stay visible; callee paths reach
// `collect_call` through their enclosing call expression.
fn function_reference_definition(cx: &LateContext<'_>, expr: &hir::Expr<'_>, qpath: &hir::QPath<'_>) -> Option<DefId> {
    let Res::Def(DefKind::Fn | DefKind::AssocFn, def_id) = cx.qpath_res(qpath, expr.hir_id) else {
        return None;
    };
    let is_callee = matches!(
        cx.tcx.parent_hir_node(expr.hir_id),
        hir::Node::Expr(hir::Expr { kind: hir::ExprKind::Call(callee, _), .. }) if callee.hir_id == expr.hir_id
    );
    (!is_callee).then_some(def_id)
}

// The ADTs a written call or literal constructs. A call constructs what its definition declares
// it returns: `Adapter::connect(url)?` constructs `Adapter` inside `Result<Self, _>`. A type
// parameter the callee received, through an input or a caller-supplied closure's return, is
// passed through, so `Arc::clone`, `Vec::pop`, `Some(x)`, and `Option::map` construct nothing
// they carry. A literal's type arguments come from its field expressions. Only `Result`,
// `Option`, `Box`, `Arc`, `Rc`, tuples, and arrays own their payload; a guard, iterator, `Pin`,
// or collection does not make its element constructed, and allocator or hasher parameters are
// never payload.
fn constructed_types<'tcx>(cx: &LateContext<'tcx>, expr: &hir::Expr<'_>, def_id: Option<DefId>) -> Vec<Ty<'tcx>> {
    let actual = cx.typeck_results().expr_ty(expr);
    let mut found = Vec::new();
    match expr.kind {
        hir::ExprKind::Call(..) | hir::ExprKind::MethodCall(..) => {
            let callee = def_id.filter(|current| {
                matches!(cx.tcx.def_kind(*current), DefKind::Fn | DefKind::AssocFn | DefKind::Ctor(_, CtorKind::Fn))
            });
            match callee {
                Some(callee) => {
                    let declared = cx.tcx.fn_sig(callee).instantiate_identity().skip_binder();
                    let received = received_parameters(cx, callee, declared.inputs());
                    declared_constructions(cx, declared.output(), actual, &received, &mut found);
                }
                // A closure or function-pointer value has no generic signature to consult.
                None => owned_constructions(cx, actual, &mut found),
            }
        }
        _ => {
            if let ty::Adt(adt, _) = actual.kind() {
                let declared = cx.tcx.type_of(adt.did()).instantiate_identity();
                let received = declared.walk().filter_map(parameter_index).collect();
                declared_constructions(cx, declared, actual, &received, &mut found);
            }
        }
    }
    found
}

fn received_parameters<'tcx>(cx: &LateContext<'tcx>, callee: DefId, inputs: &[Ty<'tcx>]) -> BTreeSet<u32> {
    let mut received =
        inputs.iter().flat_map(|input| input.walk()).filter_map(parameter_index).collect::<BTreeSet<_>>();
    if let Some(closure_output) = cx.tcx.lang_items().fn_once_output() {
        for clause in cx.tcx.predicates_of(callee).instantiate_identity(cx.tcx).predicates {
            if let Some(projection) = clause.as_projection_clause().map(ty::Binder::skip_binder)
                && projection.projection_term.def_id == closure_output
                && let Some(output) = projection.term.as_type()
            {
                received.extend(output.walk().filter_map(parameter_index));
            }
        }
    }
    received
}

fn parameter_index(argument: ty::GenericArg<'_>) -> Option<u32> {
    match argument.as_type()?.kind() {
        ty::Param(parameter) => Some(parameter.index),
        _ => None,
    }
}

fn declared_constructions<'tcx>(
    cx: &LateContext<'tcx>,
    declared: Ty<'tcx>,
    actual: Ty<'tcx>,
    received: &BTreeSet<u32>,
    found: &mut Vec<Ty<'tcx>>,
) {
    match (declared.kind(), actual.kind()) {
        // A received type parameter falls through to the final arm: it is passed through.
        (ty::Param(parameter), _) if !received.contains(&parameter.index) => owned_constructions(cx, actual, found),
        (ty::Alias(ty::Opaque, declared_alias), ty::Alias(ty::Opaque, actual_alias))
            if declared_alias.def_id == actual_alias.def_id =>
        {
            if let (Some(declared), Some(actual)) =
                (future_output(cx, *declared_alias), future_output(cx, *actual_alias))
            {
                declared_constructions(cx, declared, actual, received, found);
            }
        }
        // An associated type of a received value, such as `Self::Item`, is passed through.
        (ty::Alias(_, alias), _) => {
            let from_input = alias
                .args
                .types()
                .flat_map(Ty::walk)
                .filter_map(parameter_index)
                .any(|index| received.contains(&index));
            if !from_input {
                owned_constructions(cx, actual, found);
            }
        }
        (ty::Adt(declared_adt, declared_args), ty::Adt(actual_adt, actual_args))
            if declared_adt.did() == actual_adt.did() =>
        {
            record_construction(actual, found);
            let payload = owned_payload_arguments(cx, *actual_adt);
            for (declared, actual) in declared_args.types().zip(actual_args.types()).take(payload) {
                declared_constructions(cx, declared, actual, received, found);
            }
        }
        (ty::Tuple(declared_items), ty::Tuple(actual_items)) => {
            for (declared, actual) in declared_items.iter().zip(actual_items.iter()) {
                declared_constructions(cx, declared, actual, received, found);
            }
        }
        (ty::Array(declared, _), ty::Array(actual, _)) | (ty::Slice(declared), ty::Slice(actual)) => {
            declared_constructions(cx, *declared, *actual, received, found);
        }
        _ => {}
    }
}

// Everything the callee produced: an unreceived type parameter or a call without a signature.
fn owned_constructions<'tcx>(cx: &LateContext<'tcx>, actual: Ty<'tcx>, found: &mut Vec<Ty<'tcx>>) {
    match actual.kind() {
        ty::Adt(adt, args) => {
            record_construction(actual, found);
            for current in args.types().take(owned_payload_arguments(cx, *adt)) {
                owned_constructions(cx, current, found);
            }
        }
        ty::Tuple(items) => {
            for current in *items {
                owned_constructions(cx, current, found);
            }
        }
        ty::Array(item, _) | ty::Slice(item) => owned_constructions(cx, *item, found),
        _ => {}
    }
}

fn record_construction<'tcx>(actual: Ty<'tcx>, found: &mut Vec<Ty<'tcx>>) {
    if !found.contains(&actual) {
        found.push(actual);
    }
}

// Leading type arguments a wrapper owns by value; `Box`, `Arc`, and `Rc` exclude the allocator.
fn owned_payload_arguments(cx: &LateContext<'_>, adt: ty::AdtDef<'_>) -> usize {
    let single = adt.is_box()
        || [sym::Option, sym::Arc, sym::Rc]
            .into_iter()
            .any(|wrapper| cx.tcx.is_diagnostic_item(wrapper, adt.did()));
    if cx.tcx.is_diagnostic_item(sym::Result, adt.did()) {
        2
    } else {
        usize::from(single)
    }
}

// The `Future::Output` an async call's opaque future resolves to, instantiated for this call.
fn future_output<'tcx>(cx: &LateContext<'tcx>, alias: ty::AliasTy<'tcx>) -> Option<Ty<'tcx>> {
    let output = cx.tcx.lang_items().future_output()?;
    cx.tcx
        .explicit_item_bounds(alias.def_id)
        .iter_instantiated_copied(cx.tcx, alias.args)
        .find_map(|(clause, _)| {
            let projection = clause.as_projection_clause()?.skip_binder();
            if projection.projection_term.def_id == output {
                projection.term.as_type()
            } else {
                None
            }
        })
}

// rustc prints the local crate root as an empty path; name it by its crate.
fn qualified_definition(cx: &LateContext<'_>, def_id: DefId) -> String {
    let path = cx.tcx.def_path_str(def_id);
    if def_id.is_local() {
        let crate_name = cx.tcx.crate_name(def_id.krate).to_string();
        if path.is_empty() {
            crate_name
        } else if path == crate_name || path.starts_with(&format!("{crate_name}::")) {
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
    argument_origins: Option<&HashMap<(LocalDefId, usize), ValueOrigins>>,
    return_origins: &HashMap<LocalDefId, ValueOrigins>,
    max_steps: usize,
    steps: &mut usize,
) -> Result<MirValueFlowResult<'tcx>, ()> {
    let mut result = MirValueFlowResult::default();
    let mut entry_states = BTreeMap::<mir::BasicBlock, LocalValueOrigins>::new();
    let mut start_state = LocalValueOrigins::new();
    let definition = qualified_definition(cx, def_id.to_def_id());
    // Unless its in-crate direct calls are all its callers, a parameter's origin
    // is the argument each caller passes, which this body cannot see.
    for (position, local) in body.args_iter().enumerate() {
        if tracked_type_definition(cx, body.local_decls[local].ty, tracked_types).is_none() {
            continue;
        }
        let origins = argument_origins
            .and_then(|argument_origins| argument_origins.get(&(def_id, position)))
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
            mark_mutable_alias(&mut state, rvalue);
            let destination_ty = destination.ty(&body.local_decls, cx.tcx).ty;
            if tracked_type_definition(cx, destination_ty, tracked_types).is_none() {
                continue;
            }
            let origins = rvalue_origins(cx, body, rvalue, &state, tracked_types);
            assign_place_origins(&mut state, *destination, origins);
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
                    body,
                    func,
                    args,
                    Some(*destination),
                    *fn_span,
                    tracked_callees,
                    tracked_types,
                    root_definitions,
                    propagation_definitions,
                    return_origins,
                    &mut state,
                    &mut result,
                );
            }
            mir::TerminatorKind::TailCall { func, args, fn_span } => {
                analyze_call_terminator(
                    cx,
                    body,
                    func,
                    args,
                    None,
                    *fn_span,
                    tracked_callees,
                    tracked_types,
                    root_definitions,
                    propagation_definitions,
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
        // Visit every reachable block at least once, even before any tracked
        // value exists; afterwards revisit a block only when its entry grows.
        for successor in block_data.terminator().successors() {
            match entry_states.entry(successor) {
                Entry::Vacant(vacant) => {
                    vacant.insert(state.clone());
                    queue.push_back(successor);
                }
                Entry::Occupied(mut existing) => {
                    if union_local_origins(existing.get_mut(), &state) {
                        queue.push_back(successor);
                    }
                }
            }
        }
    }
    result.return_origins.extend(state_origins(&entry_states, mir::RETURN_PLACE));
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn analyze_call_terminator<'tcx>(
    cx: &LateContext<'tcx>,
    body: &mir::Body<'tcx>,
    func: &mir::Operand<'tcx>,
    args: &[Spanned<mir::Operand<'tcx>>],
    destination: Option<mir::Place<'tcx>>,
    span: Span,
    tracked_callees: &BTreeSet<String>,
    tracked_types: &BTreeSet<String>,
    root_definitions: &BTreeSet<String>,
    propagation_definitions: &BTreeSet<String>,
    return_origins: &HashMap<LocalDefId, ValueOrigins>,
    state: &mut LocalValueOrigins,
    result: &mut MirValueFlowResult<'tcx>,
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
                argument_ty,
                BTreeSet::from([unknown_origin("unresolved-call-target")]),
                span,
            ));
        }
        if let Some(destination) = destination
            && tracked_type_definition(cx, destination.ty(&body.local_decls, cx.tcx).ty, tracked_types).is_some()
        {
            assign_place_origins(state, destination, BTreeSet::from([unknown_origin("unresolved-call-target")]));
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
                argument_ty,
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
    assign_place_origins(state, destination, origins);
}

fn provenance_observation(
    callee_definition: String,
    argument_position: usize,
    argument_type: String,
    argument_ty: Ty<'_>,
    origins: ValueOrigins,
    span: Span,
) -> ProvenanceObservation<'_> {
    ProvenanceObservation {
        key: ProvenanceObservationKey {
            callee_definition,
            argument_position,
            argument_type,
            span_lo: span.lo().0,
            span_hi: span.hi().0,
        },
        argument_ty,
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
        | mir::Rvalue::Repeat(operand, _)
        | mir::Rvalue::WrapUnsafeBinder(operand, _) => operand_origins(operand, state),
        mir::Rvalue::Ref(_, _, place)
        | mir::Rvalue::RawPtr(_, place)
        | mir::Rvalue::CopyForDeref(place)
        | mir::Rvalue::Discriminant(place) => place_origins(place, state),
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
        mir::Rvalue::BinaryOp(_, operands) => {
            let mut origins = operand_origins(&operands.0, state);
            origins.extend(operand_origins(&operands.1, state));
            origins
        }
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

// A parameter takes its origins from the function's in-crate direct calls only
// when those are all of its calls: the function is a free function or inherent
// method no other crate can reach, by path or symbol, and no other use exists.
fn callers_supply_arguments(
    cx: &LateContext<'_>,
    def_id: LocalDefId,
    indirect_functions: &HashSet<LocalDefId>,
) -> bool {
    let free_or_inherent = match cx.tcx.def_kind(def_id) {
        DefKind::Fn => true,
        DefKind::AssocFn => cx.tcx.inherent_impl_of_assoc(def_id.to_def_id()).is_some(),
        _ => false,
    };
    free_or_inherent
        && !indirect_functions.contains(&def_id)
        && !cx.effective_visibilities.is_reachable(def_id)
        && !cx.tcx.codegen_fn_attrs(def_id).contains_extern_indicator()
}

// A mutable borrow or raw pointer can overwrite the borrowed local later,
// through any copy, so the local stays unknown from the borrow on.
fn mark_mutable_alias(state: &mut LocalValueOrigins, rvalue: &mir::Rvalue<'_>) {
    let (mir::Rvalue::Ref(_, mir::BorrowKind::Mut { .. }, place)
    | mir::Rvalue::RawPtr(mir::RawPtrKind::Mut | mir::RawPtrKind::Const, place)) = rvalue
    else {
        return;
    };
    state.entry(place.local).or_default().insert(unknown_origin(MUTABLE_ALIAS_REASON));
}

// A whole-local write replaces its origins, except the mark of a live mutable
// alias. A write to a field, or through a pointer, leaves the rest of the value
// in place, so it adds origins; when the local has none yet, its other parts
// came from writes this analysis did not track.
fn assign_place_origins(state: &mut LocalValueOrigins, place: mir::Place<'_>, mut origins: ValueOrigins) {
    match state.entry(place.local) {
        Entry::Occupied(mut existing) if place.projection.is_empty() => {
            if existing
                .get()
                .iter()
                .any(|origin| origin.strip_prefix(UNKNOWN_ORIGIN_PREFIX) == Some(MUTABLE_ALIAS_REASON))
            {
                origins.insert(unknown_origin(MUTABLE_ALIAS_REASON));
            }
            existing.insert(origins);
        }
        Entry::Occupied(mut existing) => existing.get_mut().extend(origins),
        Entry::Vacant(vacant) => {
            if !place.projection.is_empty() {
                origins.insert(unknown_origin("partial-write"));
            }
            vacant.insert(origins);
        }
    }
}

fn tracked_type_definition<'tcx>(
    cx: &LateContext<'tcx>,
    ty: Ty<'tcx>,
    tracked_types: &BTreeSet<String>,
) -> Option<String> {
    let definition = type_definition_path(cx, ty)?;
    tracked_types.contains(&definition).then_some(definition)
}

// `From<domain::Decision>` and `From<T>` differ only in their type arguments, so implemented
// traits keep them, qualified like definitions, after the implementing `Self` type.
fn implemented_trait_path<'tcx>(cx: &LateContext<'tcx>, trait_ref: ty::TraitRef<'tcx>) -> String {
    with_type_arguments(cx, qualified_definition(cx, trait_ref.def_id), trait_ref.args.types().skip(1))
}

fn qualified_type<'tcx>(cx: &LateContext<'tcx>, resolved: Ty<'tcx>) -> String {
    match resolved.kind() {
        ty::Adt(definition, args) => with_type_arguments(cx, qualified_definition(cx, definition.did()), args.types()),
        ty::Ref(_, referent, mutability) => format!("&{}{}", mutability.prefix_str(), qualified_type(cx, *referent)),
        _ => resolved.to_string(),
    }
}

fn with_type_arguments<'tcx>(
    cx: &LateContext<'tcx>,
    path: String,
    arguments: impl Iterator<Item = Ty<'tcx>>,
) -> String {
    let arguments = arguments.map(|current| qualified_type(cx, current)).collect::<Vec<_>>();
    if arguments.is_empty() {
        path
    } else {
        format!("{path}<{}>", arguments.join(", "))
    }
}

fn type_definition_path<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<String> {
    match ty.peel_refs().kind() {
        ty::Adt(definition, _) => Some(qualified_definition(cx, definition.did())),
        ty::Alias(_, alias) => Some(qualified_definition(cx, alias.def_id)),
        _ => None,
    }
}

// Whether a body holds a tracked value in a local or passes one to a call. A
// call argument can be a constant operand of a tracked type, which no local
// holds. `known` caches the answer per type definition, because naming a
// definition is costly.
fn body_carries_tracked_values<'tcx>(
    cx: &LateContext<'tcx>,
    body: &mir::Body<'tcx>,
    tracked_types: &BTreeSet<String>,
    known: &mut HashMap<DefId, bool>,
) -> bool {
    let mut tracked = |ty: Ty<'tcx>| {
        let definition = match ty.peel_refs().kind() {
            ty::Adt(definition, _) => definition.did(),
            ty::Alias(_, alias) => alias.def_id,
            _ => return false,
        };
        *known
            .entry(definition)
            .or_insert_with(|| tracked_types.contains(&qualified_definition(cx, definition)))
    };
    body.local_decls.iter().any(|decl| tracked(decl.ty))
        || body.basic_blocks.iter().any(|block| match &block.terminator().kind {
            mir::TerminatorKind::Call { args, .. } | mir::TerminatorKind::TailCall { args, .. } => {
                args.iter().any(|argument| tracked(argument.node.ty(&body.local_decls, cx.tcx)))
            }
            _ => false,
        })
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

// An empty update records nothing, so it neither creates an entry nor counts
// as a change: a body without return origins, or an argument without origins,
// would otherwise force another round. Readers treat a missing return entry as
// empty and a missing argument entry as the unknown parameter origin.
fn union_origin_map_entry<K: Eq + Hash>(target: &mut HashMap<K, ValueOrigins>, key: K, origins: &ValueOrigins) -> bool {
    if origins.is_empty() {
        return false;
    }
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

const LIBRARY_TARGET_KINDS: &[&str] = &["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"];

// r[impl octet.compiler_architecture_ir.identity]
// A package has at most one library target, so its identities stay
// package-scoped. Binaries, examples, tests, benches, and build scripts
// compile as separate crates that can reuse the library's crate name, so their
// identities name the Cargo target. A test configuration keeps its target's
// identities, as it does for the production obligations it retains.
fn cargo_target_qualifier(target_kind: &str, target_name: &str, test_configuration: bool) -> Option<String> {
    let kind = if test_configuration {
        target_kind.strip_suffix("+test").unwrap_or(target_kind)
    } else {
        target_kind
    };
    let library = kind.split('+').all(|current| LIBRARY_TARGET_KINDS.contains(&current));
    (!library).then(|| format!("{kind}:{target_name}"))
}

fn item_identity(
    package_name: &str,
    target_qualifier: Option<&str>,
    qualified_path: &str,
    item_kind: ItemKind,
    signature_id: Option<&str>,
    source_blake3: &str,
) -> String {
    match target_qualifier {
        None => octet_architecture_ir::canonical_identity(
            ITEM_ID_DOMAIN,
            &(package_name, qualified_path, item_kind, signature_id, source_blake3),
        ),
        Some(target) => octet_architecture_ir::canonical_identity(
            ITEM_ID_DOMAIN,
            &(package_name, target, qualified_path, item_kind, signature_id, source_blake3),
        ),
    }
    .expect("compiler item identity must serialize")
}

// Whether a type names a definition from the crate being compiled.
fn mentions_local_definition(resolved: Ty<'_>) -> bool {
    resolved.walk().filter_map(ty::GenericArg::as_type).any(|current| match current.kind() {
        ty::Adt(definition, _) => definition.did().is_local(),
        ty::Foreign(def_id)
        | ty::FnDef(def_id, _)
        | ty::Closure(def_id, _)
        | ty::CoroutineClosure(def_id, _)
        | ty::Coroutine(def_id, _)
        | ty::CoroutineWitness(def_id, _) => def_id.is_local(),
        ty::Alias(_, alias) => alias.def_id.is_local(),
        ty::Dynamic(predicates, _) => predicates.principal_def_id().is_some_and(DefId::is_local),
        _ => false,
    })
}

fn signature_shape_identity(cx: &LateContext<'_>, def_id: LocalDefId) -> String {
    let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
    let inputs = signature.inputs().iter().map(ToString::to_string).collect::<Vec<_>>();
    let output = signature.output().to_string();
    octet_architecture_ir::canonical_identity(SIGNATURE_ID_DOMAIN, &(inputs, output))
        .expect("compiler signature shape identity must serialize")
}

// r[impl octet.compiler_architecture_ir.body_owner_kinds]
fn definition_has_item_visibility(kind: DefKind) -> bool {
    !matches!(kind, DefKind::AnonConst | DefKind::InlineConst | DefKind::Closure | DefKind::SyntheticCoroutineBody)
}

fn def_kind_to_item_kind(kind: DefKind) -> ItemKind {
    match kind {
        DefKind::Mod => ItemKind::Module,
        DefKind::Struct => ItemKind::Struct,
        DefKind::Union => ItemKind::Union,
        DefKind::Enum => ItemKind::Enum,
        DefKind::Trait => ItemKind::Trait,
        DefKind::TyAlias | DefKind::AssocTy => ItemKind::TypeAlias,
        DefKind::Fn | DefKind::AssocFn => ItemKind::Function,
        DefKind::Const { .. } | DefKind::AssocConst { .. } | DefKind::AnonConst | DefKind::InlineConst => {
            ItemKind::Constant
        }
        DefKind::Static { .. } => ItemKind::Static,
        DefKind::ForeignMod | DefKind::ForeignTy => ItemKind::Foreign,
        _ => ItemKind::Other,
    }
}

fn type_metadata(
    cx: &LateContext<'_>,
    resolved: Ty<'_>,
    local_package: &str,
    selected_units: &[CollectorCompilationUnit],
) -> (Option<String>, String, TypeKind) {
    match resolved.kind() {
        ty::Adt(definition, _) => (
            Some(qualified_definition(cx, definition.did())),
            definition_origin(cx, definition.did(), local_package, selected_units),
            TypeKind::Adt,
        ),
        ty::Dynamic(..) => (None, "dynamic".to_owned(), TypeKind::TraitObject),
        ty::Tuple(..) => (None, "core".to_owned(), TypeKind::Tuple),
        ty::Ref(..) => (None, "core".to_owned(), TypeKind::Reference),
        ty::RawPtr(..) => (None, "core".to_owned(), TypeKind::Pointer),
        ty::FnDef(def_id, _) => (
            Some(qualified_definition(cx, *def_id)),
            definition_origin(cx, *def_id, local_package, selected_units),
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

// Workspace definitions name their Cargo package, as item facts do, because
// policy roles are keyed by package while rustc crate names replace `-` with `_`.
fn definition_origin(
    cx: &LateContext<'_>,
    def_id: DefId,
    local_package: &str,
    selected_units: &[CollectorCompilationUnit],
) -> String {
    if def_id.is_local() {
        return local_package.to_owned();
    }
    let crate_name = cx.tcx.crate_name(def_id.krate);
    workspace_library_package(selected_units, crate_name.as_str()).map_or_else(|| crate_name.to_string(), str::to_owned)
}

// A crate name maps to a package only through one selected library target.
fn workspace_library_package<'a>(selected_units: &'a [CollectorCompilationUnit], crate_name: &str) -> Option<&'a str> {
    let mut packages = selected_units
        .iter()
        .filter(|selected| {
            !selected.test_configuration
                && selected.rustc_crate_name == crate_name
                && selected
                    .unit
                    .crate_types
                    .iter()
                    .any(|crate_type| matches!(crate_type.as_str(), "lib" | "rlib" | "dylib" | "proc-macro"))
        })
        .map(|selected| selected.unit.package_name.as_str());
    let package = packages.next()?;
    packages.all(|other| other == package).then_some(package)
}

// Classify a trait item from its resolved `Self` argument, not the written
// receiver: method autoderef through `Box`, `Arc`, or `Rc`, and qualified
// paths such as `Trait::method(object)`, still select a trait-object `Self`.
fn dispatch_kind<'tcx>(cx: &LateContext<'tcx>, def_id: DefId, args: Option<ty::GenericArgsRef<'tcx>>) -> DispatchKind {
    if definition_is_foreign(cx, def_id) {
        return DispatchKind::Foreign;
    }
    let trait_and_self = cx
        .tcx
        .trait_of_assoc(def_id)
        .zip(args.and_then(|args| args.first()).and_then(|self_arg| self_arg.as_type()));
    if trait_and_self.is_some_and(|(trait_def_id, self_ty)| dispatches_through_trait_object(cx, trait_def_id, self_ty))
    {
        return DispatchKind::Dynamic;
    }
    if matches!(cx.tcx.def_kind(def_id), DefKind::AssocFn | DefKind::AssocConst { .. } | DefKind::AssocTy) {
        DispatchKind::Trait
    } else {
        DispatchKind::Static
    }
}

// Forwarding impls such as `impl<W: Write + ?Sized> Write for Box<W>` let
// method resolution select `Self = Box<dyn Write>`; the vtable call then
// happens inside the forwarding body. A reference, `Box`, `Rc`, `Arc`, or
// `Pin` around a trait object that itself implements the called trait (as its
// principal trait or a supertrait) dispatches dynamically. `Clone` on
// `Arc<dyn Trait>` does not, because the trait object does not implement it.
fn dispatches_through_trait_object<'tcx>(cx: &LateContext<'tcx>, trait_def_id: DefId, self_ty: Ty<'tcx>) -> bool {
    if let ty::Dynamic(..) = self_ty.kind() {
        return true;
    }
    // Each step strips one constructor from a finite type, so the walk ends.
    let mut current = self_ty;
    loop {
        current = match current.kind() {
            ty::Ref(_, pointee, _) => *pointee,
            ty::Adt(adt, args)
                if adt.is_box()
                    || cx.tcx.is_lang_item(adt.did(), LangItem::Pin)
                    || [sym::Rc, sym::Arc].into_iter().any(|pointer| cx.tcx.is_diagnostic_item(pointer, adt.did())) =>
            {
                args.type_at(0)
            }
            ty::Dynamic(predicates, _) => {
                return predicates.principal_def_id().is_some_and(|principal| {
                    ty::elaborate::supertrait_def_ids(cx.tcx, principal).any(|bound| bound == trait_def_id)
                });
            }
            _ => return false,
        };
    }
}

fn definition_is_foreign(cx: &LateContext<'_>, def_id: DefId) -> bool {
    if cx.tcx.def_kind(def_id) == DefKind::ForeignTy {
        return true;
    }
    cx.tcx.opt_parent(def_id).is_some_and(|parent| cx.tcx.def_kind(parent) == DefKind::ForeignMod)
}

// r[impl octet.compiler_architecture_ir.randomness_effects]
fn resolved_randomness_family(cx: &LateContext<'_>, def_id: DefId) -> Option<CapabilityFamily> {
    // Diagnostic paths can retain consumer extern-crate aliases. Classification
    // uses the defining crate, item, and root-function shape instead.
    if def_id.is_local() || cx.tcx.def_kind(def_id) != DefKind::Fn {
        return None;
    }
    let is_root_function = cx.tcx.def_path(def_id).data.len() == 1;
    randomness_family(cx.tcx.crate_name(def_id.krate).as_str(), cx.tcx.item_name(def_id).as_str(), is_root_function)
}

fn randomness_family(crate_name: &str, item_name: &str, is_root_function: bool) -> Option<CapabilityFamily> {
    (crate_name == "getrandom" && is_root_function && matches!(item_name, "fill" | "fill_uninit" | "u32" | "u64"))
        .then_some(CapabilityFamily::Randomness)
}

// A call in a `const` or `static` initializer, including inline and anonymous
// consts, runs during compilation and performs no runtime effect. A `const fn`
// body can also run at runtime, so its calls still count. Closure bodies run at
// runtime, and a function reference stored in a `static` is called through
// the pointer at runtime, so neither qualifies.
fn const_evaluated_call(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    matches!(expr.kind, hir::ExprKind::Call(..) | hir::ExprKind::MethodCall(..))
        && const_evaluated_context(cx, expr.hir_id)
}

// Overloaded operators and derefs in such an initializer are evaluated there too.
fn const_evaluated_context(cx: &LateContext<'_>, hir_id: hir::HirId) -> bool {
    matches!(
        cx.tcx.hir_body_const_context(cx.tcx.hir_enclosing_body_owner(hir_id)),
        Some(hir::ConstContext::Const { .. } | hir::ConstContext::Static(_))
    )
}

fn overloaded_primitive_definition<'tcx>(
    cx: &LateContext<'tcx>,
    def_id: DefId,
    args: ty::GenericArgsRef<'tcx>,
) -> Option<String> {
    let instance = ty::Instance::try_resolve(cx.tcx, cx.typing_env(), def_id, args).ok().flatten()?;
    let implementation = cx.tcx.impl_of_assoc(instance.def_id())?;
    let self_definition = type_definition_path(cx, cx.tcx.type_of(implementation).instantiate_identity())?;
    Some(format!("{self_definition}::{}", cx.tcx.item_name(def_id)))
}

fn effect_family(definition: &str) -> Option<CapabilityFamily> {
    let on_value_type = EFFECT_FREE_VALUE_TYPES.iter().any(|value_type| {
        definition.strip_prefix(value_type).is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
    });
    if on_value_type {
        return None;
    }
    if SYNCHRONIZATION_PATHS.iter().any(|prefix| definition.starts_with(prefix)) {
        let operation = definition.rsplit_once("::").map_or(definition, |(_, operation)| operation);
        return (!NON_SYNCHRONIZING_OPERATIONS.contains(&operation)).then_some(CapabilityFamily::Concurrency);
    }
    EFFECT_PATHS.iter().find_map(|(prefix, family)| definition.starts_with(prefix).then_some(*family))
}

fn path_in_scope(path: &str, scope: &str) -> bool {
    path == scope || path.strip_prefix(scope).is_some_and(|suffix| suffix.starts_with('/'))
}

fn closure_qualified_path(cx: &LateContext<'_>, span: Span, package_name: &str, workspace_root: &str) -> String {
    let source_map = cx.sess().source_map();
    let resolved_span = workspace_source_span(cx, span, workspace_root).unwrap_or(span);
    let filename = source_map.span_to_filename(resolved_span);
    let raw_path = filename.prefer_local_unconditionally().to_string();
    let source_path = normalized_workspace_source_path(&raw_path, workspace_root).unwrap_or_else(|| {
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

fn workspace_source_span(cx: &LateContext<'_>, span: Span, workspace_root: &str) -> Option<Span> {
    let mut candidate = span;
    for _ in 0..MAX_EXPANSION_CALLSITE_DEPTH {
        let filename = cx.sess().source_map().span_to_filename(candidate);
        if normalized_workspace_source_path(&filename.prefer_local_unconditionally().to_string(), workspace_root)
            .is_some()
        {
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

fn normalized_workspace_source_path(path: &str, workspace_root: &str) -> Option<String> {
    // rustc preserves `src/../tests/...` in #[path] module spans. Resolve physical
    // ownership before classifying a path; lexical rejection would call test code
    // generated, while lexical collapsing could hide a symlink escape.
    let workspace = fs::canonicalize(workspace_root).ok()?;
    let candidate = path
        .strip_prefix(REMAPPED_WORKSPACE_PREFIX)
        .map_or_else(|| PathBuf::from(path), |relative| workspace.join(relative));
    let physical = fs::canonicalize(candidate).ok()?;
    let relative = physical.strip_prefix(&workspace).ok()?;
    if inside_source_build_directory(&workspace, relative) {
        return None;
    }
    normalized_relative_path(relative)
}

// Apply the package source envelope rule: build and evidence directory names
// are excluded only beside a `Cargo.toml`. Build-script output under `target/`
// (`OUT_DIR`) is then generated source, while a module directory such as
// `src/target/` stays workspace source.
fn inside_source_build_directory(workspace: &Path, relative: &Path) -> bool {
    let mut directory = workspace.to_path_buf();
    for component in relative.parent().map(Path::components).into_iter().flatten() {
        let std::path::Component::Normal(name) = component else {
            return false;
        };
        if name.to_str().is_some_and(|name| octet_architecture_ir::SOURCE_BUILD_DIRECTORIES.contains(&name))
            && directory.join("Cargo.toml").is_file()
        {
            return true;
        }
        directory.push(name);
    }
    false
}

fn normalized_relative_path(path: &Path) -> Option<String> {
    let components = path
        .components()
        .map(|component| match component {
            std::path::Component::Normal(name) => name.to_str().map(str::to_owned),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    (!components.is_empty()).then(|| components.join("/"))
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

// These compiler passes synthesize code, not macro invocations. Keep their expansion
// facts and all generated calls/effects, but do not require a nonexistent macro DefId.
// Every other definition-less expansion stays unknown.
fn compiler_owned_expansion(kind: &ExpnKind) -> bool {
    matches!(kind, ExpnKind::AstPass(AstPass::StdImports | AstPass::TestHarness | AstPass::ProcMacroHarness))
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[verify octet.compiler_architecture_ir.body_owner_kinds]
    #[test]
    fn expression_body_owners_do_not_request_item_visibility() {
        for kind in [
            DefKind::AnonConst,
            DefKind::InlineConst,
            DefKind::Closure,
            DefKind::SyntheticCoroutineBody,
        ] {
            assert!(!definition_has_item_visibility(kind));
        }
        for kind in [
            DefKind::Fn,
            DefKind::AssocFn,
            DefKind::Const { is_type_const: false },
            DefKind::AssocConst { is_type_const: false },
        ] {
            assert!(definition_has_item_visibility(kind));
        }
    }

    // r[verify octet.compiler_architecture_ir.body_owner_kinds]
    #[test]
    fn body_owner_classification_does_not_invent_function_signatures() {
        for kind in [
            DefKind::Const { is_type_const: false },
            DefKind::AssocConst { is_type_const: false },
            DefKind::AnonConst,
            DefKind::InlineConst,
        ] {
            assert_eq!(def_kind_to_item_kind(kind), ItemKind::Constant);
        }
        for kind in [DefKind::Fn, DefKind::AssocFn] {
            assert_eq!(def_kind_to_item_kind(kind), ItemKind::Function);
        }
        for kind in [DefKind::Closure, DefKind::SyntheticCoroutineBody] {
            assert_eq!(def_kind_to_item_kind(kind), ItemKind::Other);
        }
        assert_eq!(
            def_kind_to_item_kind(DefKind::Static {
                safety: hir::Safety::Safe,
                mutability: rustc_ast::Mutability::Not,
                nested: false,
            }),
            ItemKind::Static
        );
    }

    #[test]
    fn compiler_passes_do_not_require_macro_definitions() {
        for pass in [AstPass::StdImports, AstPass::TestHarness, AstPass::ProcMacroHarness] {
            assert!(compiler_owned_expansion(&ExpnKind::AstPass(pass)));
        }
    }

    #[test]
    fn unresolved_macros_and_root_context_remain_unknown() {
        use rustc_span::hygiene::MacroKind;
        use rustc_span::sym;

        assert!(!compiler_owned_expansion(&ExpnKind::Root));
        assert!(!compiler_owned_expansion(&ExpnKind::Macro(MacroKind::Bang, sym::test)));
        assert!(!compiler_owned_expansion(&ExpnKind::Macro(MacroKind::Attr, sym::test)));
        assert!(!compiler_owned_expansion(&ExpnKind::Macro(MacroKind::Derive, sym::test)));
    }

    #[test]
    fn absent_context_does_not_activate_collection() {
        assert!(load_collector_context_at(None).expect("absent collector context").is_none());
    }

    // r[verify octet.compiler_architecture_ir.randomness_effects]
    #[test]
    fn getrandom_entry_points_are_randomness_effects() {
        for name in ["fill", "fill_uninit", "u32", "u64"] {
            assert_eq!(randomness_family("getrandom", name, true), Some(CapabilityFamily::Randomness), "{name}");
        }
    }

    // r[verify octet.compiler_architecture_ir.randomness_effects]
    #[test]
    fn getrandom_classification_rejects_similar_and_unrelated_paths() {
        for (owner, name, root) in [
            ("getrandom", "fill_helper", true),
            ("getrandom", "fill::other", true),
            ("getrandom", "raw_os_error", true),
            ("fixture", "fill", true),
            ("getrandom_other", "fill", true),
            ("rand", "unreviewed", true),
            ("getrandom", "fill", false),
        ] {
            assert_eq!(randomness_family(owner, name, root), None, "{owner}::{name}, root={root}");
        }
    }

    #[test]
    fn build_and_evidence_directories_beside_manifests_are_not_workspace_sources() {
        let root = std::env::temp_dir().join(format!("octet-collector-source-paths-{}", std::process::id()));
        for relative in [
            "Cargo.toml",
            "src/lib.rs",
            "src/target/mod.rs",
            "target/debug/build/app-0123/out/generated.rs",
            ".octet/compiler-architecture-fixtures/generated.rs",
            "fuzz/Cargo.toml",
            "fuzz/target/generated.rs",
        ] {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
            fs::write(&path, b"").expect("fixture file");
        }
        let workspace = root.display().to_string();
        let source =
            |relative: &str| normalized_workspace_source_path(&root.join(relative).display().to_string(), &workspace);
        assert_eq!(source("src/lib.rs").as_deref(), Some("src/lib.rs"));
        assert_eq!(source("src/target/mod.rs").as_deref(), Some("src/target/mod.rs"));
        assert_eq!(source("target/debug/build/app-0123/out/generated.rs"), None);
        assert_eq!(source(".octet/compiler-architecture-fixtures/generated.rs"), None);
        assert_eq!(source("fuzz/target/generated.rs"), None);
        assert_eq!(
            normalized_workspace_source_path(
                &format!("{REMAPPED_WORKSPACE_PREFIX}target/debug/build/app-0123/out/generated.rs"),
                &workspace
            ),
            None
        );
        fs::remove_dir_all(root).expect("remove owned test directory");
    }

    #[test]
    fn effect_classification_uses_resolved_definition_paths() {
        assert_eq!(effect_family("std::fs::read"), Some(CapabilityFamily::Filesystem));
        assert_eq!(effect_family("fixture::comment_named_std_fs_read"), None);
    }

    #[test]
    fn observability_and_synchronization_paths_are_effects() {
        for (definition, family) in [
            ("std::io::_print", CapabilityFamily::Observability),
            ("std::io::_eprint", CapabilityFamily::Observability),
            ("std::io::stdout", CapabilityFamily::Observability),
            ("std::io::stderr", CapabilityFamily::Observability),
            ("std::sync::Mutex::<T>::lock", CapabilityFamily::Concurrency),
            ("std::sync::RwLock::<T>::write", CapabilityFamily::Concurrency),
            ("std::sync::OnceLock::<T>::get_or_init", CapabilityFamily::Concurrency),
            ("std::sync::mpsc::channel", CapabilityFamily::Concurrency),
            ("std::sync::atomic::AtomicU32::fetch_add", CapabilityFamily::Concurrency),
            ("core::sync::atomic::AtomicU32::fetch_add", CapabilityFamily::Concurrency),
            ("std::sync::atomic::Atomic::<u32>::load", CapabilityFamily::Concurrency),
            ("std::sync::Mutex::<T>::try_lock", CapabilityFamily::Concurrency),
            ("std::thread::Builder::new", CapabilityFamily::Concurrency),
        ] {
            assert_eq!(effect_family(definition), Some(family), "{definition}");
        }
        for definition in [
            "std::sync::Arc::<T>::new",
            "std::sync::Mutex::<T>::new",
            "std::sync::Mutex::<T>::get_mut",
            "std::sync::Mutex::<T>::into_inner",
            "std::sync::OnceLock::<T>::new",
            "std::sync::atomic::Atomic::<u32>::new",
            "std::sync::atomic::Atomic::<u32>::get_mut",
            "std::io::Write::write_all",
            "alloc::fmt::format",
        ] {
            assert_eq!(effect_family(definition), None, "{definition}");
        }
    }

    #[test]
    fn value_types_under_effect_paths_are_not_effects() {
        for definition in [
            "std::net::IpAddr::eq",
            "std::net::Ipv4Addr::is_loopback",
            "std::net::SocketAddr::port",
            "std::sync::atomic::Ordering::eq",
            "core::sync::atomic::Ordering::eq",
            "std::sync::mpsc::SendError::<T>::eq",
            "std::thread::ThreadId::eq",
            "std::fs::Metadata::len",
            "std::process::ExitStatus::success",
        ] {
            assert_eq!(effect_family(definition), None, "{definition}");
        }
        for (definition, family) in [
            ("std::net::TcpStream::connect", CapabilityFamily::Network),
            ("std::net::UdpSocket::bind", CapabilityFamily::Network),
            ("std::fs::File::open", CapabilityFamily::Filesystem),
            ("std::process::Command::spawn", CapabilityFamily::Process),
            ("std::thread::Thread::unpark", CapabilityFamily::Concurrency),
            // A name that only starts like an excluded type is not that type.
            ("std::net::IpAddrResolver::lookup", CapabilityFamily::Network),
        ] {
            assert_eq!(effect_family(definition), Some(family), "{definition}");
        }
    }

    #[test]
    fn clock_reads_and_path_queries_are_effects() {
        for (definition, family) in [
            ("std::time::Instant::elapsed", CapabilityFamily::Time),
            ("std::time::SystemTime::elapsed", CapabilityFamily::Time),
            ("std::path::Path::canonicalize", CapabilityFamily::Filesystem),
            ("std::path::Path::exists", CapabilityFamily::Filesystem),
            ("std::path::Path::is_dir", CapabilityFamily::Filesystem),
            ("std::path::Path::is_file", CapabilityFamily::Filesystem),
            ("std::path::Path::is_symlink", CapabilityFamily::Filesystem),
            ("std::path::Path::metadata", CapabilityFamily::Filesystem),
            ("std::path::Path::read_dir", CapabilityFamily::Filesystem),
            ("std::path::Path::read_link", CapabilityFamily::Filesystem),
            ("std::path::Path::symlink_metadata", CapabilityFamily::Filesystem),
            ("std::path::Path::try_exists", CapabilityFamily::Filesystem),
        ] {
            assert_eq!(effect_family(definition), Some(family), "{definition}");
        }
        for definition in [
            "std::time::Instant::duration_since",
            "std::time::Instant::checked_duration_since",
            "std::time::SystemTime::duration_since",
            "std::time::Duration::as_secs",
            "std::path::Path::join",
            "std::path::Path::parent",
            "std::path::Path::file_name",
            "std::path::Path::extension",
            "std::path::Path::components",
            "std::path::Path::strip_prefix",
            "std::path::Path::starts_with",
            "std::path::Path::display",
            "std::path::PathBuf::push",
            "std::path::PathBuf::deref",
        ] {
            assert_eq!(effect_family(definition), None, "{definition}");
        }
    }

    // r[verify octet.compiler_architecture_ir.identity]
    #[test]
    fn only_non_library_targets_qualify_identities() {
        for (kind, test) in [
            ("lib", false),
            ("lib+test", true),
            ("rlib", false),
            ("cdylib+rlib", false),
            ("proc-macro", false),
            ("proc-macro+test", true),
        ] {
            assert_eq!(cargo_target_qualifier(kind, "fixture", test), None, "{kind}");
        }
        for (kind, test, qualifier) in [
            ("bin", false, "bin:fixture"),
            ("bin+test", true, "bin:fixture"),
            ("test+test", true, "test:fixture"),
            ("example", false, "example:fixture"),
            ("bench", false, "bench:fixture"),
            ("custom-build", false, "custom-build:fixture"),
        ] {
            assert_eq!(cargo_target_qualifier(kind, "fixture", test).as_deref(), Some(qualifier), "{kind}");
        }
        let identity = |target| item_identity("fixture", target, "fixture::run", ItemKind::Function, None, "source");
        assert_ne!(identity(None), identity(Some("bin:fixture")));
        assert_ne!(identity(Some("bin:fixture")), identity(Some("example:fixture")));
    }

    fn selected_unit(package: &str, crate_name: &str, crate_type: &str, test: bool) -> CollectorCompilationUnit {
        CollectorCompilationUnit {
            rustc_crate_name: crate_name.to_owned(),
            manifest_dir: format!("crates/{package}"),
            source_path: format!("crates/{package}/src/lib.rs"),
            test_configuration: test,
            source_roles: Vec::new(),
            unit: octet_architecture_ir::CompilationUnit {
                unit_id: String::new(),
                package_name: package.to_owned(),
                package_version: "0.1.0".to_owned(),
                target_name: crate_name.to_owned(),
                target_kind: if test { "lib+test" } else { "lib" }.to_owned(),
                source_path: format!("crates/{package}/src/lib.rs"),
                crate_types: vec![crate_type.to_owned()],
                target_triple: "x86_64-unknown-linux-gnu".to_owned(),
                features: Vec::new(),
                default_features: true,
                role: CompilerArchitectureRole::OutboundAdapter,
                toolchain_blake3: String::new(),
                collector_blake3: String::new(),
                policy_blake3: String::new(),
                source_blake3: String::new(),
                invocation_blake3: String::new(),
            },
        }
    }

    #[test]
    fn workspace_crate_names_resolve_to_hyphenated_packages() {
        let units = [
            selected_unit("postgres-adapter", "postgres_adapter", "rlib", false),
            selected_unit("postgres-adapter", "postgres_adapter", "bin", true),
        ];
        assert_eq!(workspace_library_package(&units, "postgres_adapter"), Some("postgres-adapter"));
        assert_eq!(workspace_library_package(&units, "serde"), None);
    }

    #[test]
    fn ambiguous_or_binary_crate_names_do_not_resolve_to_packages() {
        let ambiguous = [
            selected_unit("foo-bar", "foo_bar", "rlib", false),
            selected_unit("foo_bar", "foo_bar", "proc-macro", false),
        ];
        assert_eq!(workspace_library_package(&ambiguous, "foo_bar"), None);
        let binary = [selected_unit("tool-cli", "tool_cli", "bin", false)];
        assert_eq!(workspace_library_package(&binary, "tool_cli"), None);
    }

    fn collector_with_span_limit(max_spans: usize) -> ArchitectureCollector {
        const LIMIT: usize = 16;
        let limits = octet_architecture_ir::ArchitectureCollectionLimits {
            max_units: LIMIT,
            max_shards: LIMIT,
            max_modules: LIMIT,
            max_items: LIMIT,
            max_types: LIMIT,
            max_traits: LIMIT,
            max_implementations: LIMIT,
            max_signatures: LIMIT,
            max_calls: LIMIT,
            max_constructors: LIMIT,
            max_effects: LIMIT,
            max_expansions: LIMIT,
            max_configurations: LIMIT,
            max_type_shapes: LIMIT,
            max_function_safety_facts: LIMIT,
            max_value_provenance_facts: LIMIT,
            max_value_flow_bodies: LIMIT,
            max_value_flow_steps: LIMIT,
            max_value_flow_rounds: LIMIT,
            max_unknowns: LIMIT,
            max_spans,
            max_span_bytes: LIMIT,
            max_shard_bytes: LIMIT,
            max_artifact_bytes: LIMIT,
        };
        let context = CompilerArchitectureCollectorContext {
            schema_version: String::new(),
            workspace_root: String::new(),
            output_directory: String::new(),
            limits,
            selected_units: Vec::new(),
            value_flow_selections: Vec::new(),
            non_claims: Vec::new(),
        };
        ArchitectureCollector::new(context, selected_unit("fixture", "fixture", "rlib", false))
    }

    fn workspace_span(byte_start: u64) -> SourceSpan {
        let mut span = SourceSpan {
            span_id: String::new(),
            origin: SourceOrigin::Workspace {
                source_path: "src/lib.rs".to_owned(),
            },
            byte_start,
            byte_end: byte_start + 1,
            line_start: 1,
            column_start: 1,
            from_expansion: false,
        };
        span.span_id = octet_architecture_ir::source_span_identity(&span);
        span
    }

    #[test]
    fn recorded_spans_do_not_count_against_a_full_span_limit() {
        let mut collector = collector_with_span_limit(1);
        let recorded = workspace_span(0);
        collector.record_span(&recorded);
        collector.record_span(&recorded);
        assert!(!collector.incomplete, "re-reading a recorded span must not exceed the limit");
        collector.record_span(&workspace_span(1));
        assert!(collector.incomplete, "a new span beyond the limit must mark the shard incomplete");
        assert_eq!(collector.spans.keys().collect::<Vec<_>>(), [&recorded.span_id]);
    }

    fn constructor_fact(constructed_type_id: &str) -> CompilerArchitectureFact {
        let mut fact = CompilerArchitectureFact::Constructor {
            fact_id: String::new(),
            caller_item_id: "a".repeat(64),
            constructed_type_id: constructed_type_id.to_owned(),
            constructor_definition: None,
            span_id: "b".repeat(64),
        };
        assign_fact_identity(&mut fact);
        fact
    }

    #[test]
    fn stored_facts_count_once_per_kind_against_their_own_limit() {
        let mut collector = collector_with_span_limit(1);
        collector.context.limits.max_constructors = 1;
        let stored = constructor_fact(&"c".repeat(64));
        collector.push_fact(stored.clone());
        collector.push_fact(stored.clone());
        assert!(!collector.incomplete, "re-recording a stored fact must not exceed its limit");
        let mut call = CompilerArchitectureFact::Call {
            fact_id: String::new(),
            caller_item_id: "a".repeat(64),
            callee_item_id: None,
            resolved_definition: Some("fixture::run".to_owned()),
            dispatch: DispatchKind::Static,
            span_id: "b".repeat(64),
        };
        assign_fact_identity(&mut call);
        collector.push_fact(call.clone());
        assert!(!collector.incomplete, "a full constructor limit must not bound other fact kinds");
        collector.push_fact(constructor_fact(&"d".repeat(64)));
        assert!(collector.incomplete, "a new fact beyond its kind's limit must mark the shard incomplete");
        let mut stored_ids = vec![stored.fact_id(), call.fact_id()];
        stored_ids.sort_unstable();
        assert_eq!(collector.facts.keys().map(String::as_str).collect::<Vec<_>>(), stored_ids);
    }
}
