use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::FnKind;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::hir::nested_filter::All;
use rustc_middle::ty;
use rustc_middle::ty::Ty;
use rustc_middle::ty::TyCtxt;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;
use crate::markers::attrs_carry_marker;
use crate::performance::allocation_in_loop;
use crate::test_context;

const AMBIENT_ENV_LINT: &str = "ambient_env";
const AMBIENT_RANDOM_LINT: &str = "ambient_random";
const GLOBAL_STATE_IN_CORE_LINT: &str = "global_state_in_core";
const IMPURE_CALL_IN_CORE_LINT: &str = "impure_call_in_core";
const INTERIOR_MUTABILITY_IN_CORE_LINT: &str = "interior_mutability_in_core";
const LOGGING_IN_CORE_LINT: &str = "logging_in_core";
const MISSING_CONST_FN_LINT: &str = "missing_const_fn";
const MUTATING_INPUT_IN_PURE_LINT: &str = "mutating_input_in_pure";
const NONDETERMINISTIC_OUTPUT_LINT: &str = "nondeterministic_output";
const PURE_ATTR_CONTRACT_LINT: &str = "pure_attr_contract";
const ZERO_ALLOC_ATTR_CONTRACT_LINT: &str = "zero_alloc_attr_contract";
const RESOURCE_RETURN_FROM_CORE_LINT: &str = "resource_return_from_core";
const SPAWN_IN_CORE_LINT: &str = "spawn_in_core";

const ENV_CALLS: &[&str] = &[
    "args",
    "args_os",
    "current_dir",
    "remove_var",
    "set_current_dir",
    "set_var",
    "temp_dir",
    "var",
    "var_os",
    "vars",
    "vars_os",
];
/// Ambient randomness entry points. `fastrand` lists only the free functions
/// that draw from or expose the thread-local generator, plus the entropy-seeded
/// `Rng` constructors; `fastrand::Rng::with_seed(seed)` and `fastrand::seed(seed)`
/// take an explicit seed and stay clean.
const RANDOM_CALL_PATTERNS: &[&str] = &[
    "fastrand::Rng::default(",
    "fastrand::Rng::new(",
    "fastrand::alphabetic(",
    "fastrand::alphanumeric(",
    "fastrand::bool(",
    "fastrand::char(",
    "fastrand::choice(",
    "fastrand::choose_multiple(",
    "fastrand::digit(",
    "fastrand::f32(",
    "fastrand::f64(",
    "fastrand::fill(",
    "fastrand::get_seed(",
    "fastrand::i128(",
    "fastrand::i16(",
    "fastrand::i32(",
    "fastrand::i64(",
    "fastrand::i8(",
    "fastrand::isize(",
    "fastrand::lowercase(",
    "fastrand::shuffle(",
    "fastrand::u128(",
    "fastrand::u16(",
    "fastrand::u32(",
    "fastrand::u64(",
    "fastrand::u8(",
    "fastrand::uppercase(",
    "fastrand::usize(",
    "getrandom::fill(",
    "getrandom::getrandom(",
    "rand::random(",
    "rand::rng(",
    "rand::thread_rng(",
    "uuid::Uuid::new_v4(",
    "Uuid::new_v4(",
];
const FILESYSTEM_CALL_PATTERNS: &[&str] = &[
    "std::fs::create_dir(",
    "std::fs::create_dir_all(",
    "std::fs::read(",
    "std::fs::read_dir(",
    "std::fs::read_to_string(",
    "std::fs::remove_file(",
    "std::fs::write(",
    "std::fs::File::create(",
    "std::fs::File::open(",
    "fs::read(",
    "fs::read_to_string(",
    "fs::write(",
    "File::open(",
    "File::create(",
];
const IO_CALL_PATTERNS: &[&str] = &["std::io::stdin(", "std::io::stdout(", "std::io::stderr("];
const NET_CALL_PATTERNS: &[&str] = &[
    "std::net::TcpListener::bind(",
    "std::net::TcpStream::connect(",
    "std::net::UdpSocket::bind(",
    "TcpListener::bind(",
    "TcpStream::connect(",
    "UdpSocket::bind(",
];
const PROCESS_CALL_PATTERNS: &[&str] = &[
    "std::process::Command::new(",
    "std::process::exit(",
    "Command::new(",
    "process::Command::new(",
];
const TIME_CALL_PATTERNS: &[&str] = &[
    "Instant::now(",
    "Local::now(",
    "SystemTime::now(",
    "Utc::now(",
    "std::time::Instant::now(",
    "std::time::SystemTime::now(",
];
const RUNTIME_CALL_PATTERNS: &[&str] = &["tokio::runtime::", "Runtime::new(", "Builder::new_current_thread("];
const LOGGING_CALL_PATTERNS: &[&str] = &[
    "dbg!(",
    "eprint!(",
    "eprintln!(",
    "log::debug!(",
    "log::error!(",
    "log::info!(",
    "log::trace!(",
    "log::warn!(",
    "print!(",
    "println!(",
    "tracing::debug!(",
    "tracing::error!(",
    "tracing::event!(",
    "tracing::info!(",
    "tracing::trace!(",
    "tracing::warn!(",
];
const SPAWN_CALL_PATTERNS: &[&str] = &[
    "async_std::task::spawn(",
    "std::thread::spawn(",
    "thread::spawn(",
    "tokio::spawn(",
    "tokio::task::spawn(",
];
const INTERIOR_MUTABILITY_PATTERNS: &[&str] = &[
    "AtomicBool",
    "AtomicI",
    "AtomicPtr",
    "AtomicU",
    "Cell<",
    "Cell::new(",
    "LazyLock<",
    "LazyLock::new(",
    "Mutex<",
    "Mutex::new(",
    "OnceLock<",
    "OnceLock::new(",
    "RefCell<",
    "RefCell::new(",
    "RwLock<",
    "RwLock::new(",
    "UnsafeCell<",
    "UnsafeCell::new(",
];
const MUTATING_PARAM_PATTERNS: &[&str] = &[
    "&mut ",
    "*mut ",
    "AtomicBool",
    "AtomicI",
    "AtomicPtr",
    "AtomicU",
    "Cell<",
    "Mutex<",
    "RefCell<",
    "RwLock<",
    "UnsafeCell<",
];
/// Concrete resource handles by defining crate and type name: files, sockets,
/// processes and commands, task handles, channel endpoints, and external
/// client or connection handles. Matching the resolved ADT keeps domain types
/// such as `FileId` or a local `Command` enum, and std values such as
/// `SocketAddr`, out of scope.
const RESOURCE_HANDLE_TYPES: &[(&str, &[&str])] = &[
    ("std", &[
        "Child",
        "ChildStderr",
        "ChildStdin",
        "ChildStdout",
        "Command",
        "File",
        "JoinHandle",
        "Receiver",
        "ScopedJoinHandle",
        "Sender",
        "SyncSender",
        "TcpListener",
        "TcpStream",
        "UdpSocket",
        "UnixDatagram",
        "UnixListener",
        "UnixStream",
    ]),
    ("tokio", &[
        "Child",
        "ChildStderr",
        "ChildStdin",
        "ChildStdout",
        "Command",
        "File",
        "JoinHandle",
        "Receiver",
        "Sender",
        "TcpListener",
        "TcpStream",
        "UdpSocket",
        "UnboundedReceiver",
        "UnboundedSender",
        "UnixDatagram",
        "UnixListener",
        "UnixStream",
    ]),
    ("crossbeam_channel", &["Receiver", "Sender"]),
    ("flume", &["Receiver", "Sender"]),
    ("postgres", &["Client"]),
    ("redis", &["Client", "Connection"]),
    ("reqwest", &["Client"]),
    ("rusqlite", &["Connection"]),
    ("tokio_postgres", &["Client"]),
];
const CONST_INELIGIBLE_PATTERNS: &[&str] = &[
    " async fn ",
    " const fn ",
    " pub fn ",
    " unsafe fn ",
    " for ",
    " while ",
    " loop ",
    " await",
    " format!(",
    " vec!(",
    "Box::new(",
    "String::",
    "Vec::",
    "?",
];
const CONST_ELIGIBLE_HINTS: &[&str] = &[
    " + ", " - ", " * ", " / ", " % ", " == ", " != ", " < ", " > ", " match ",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EffectClass {
    Env,
    Filesystem,
    Io,
    Logging,
    Net,
    Process,
    Random,
    Runtime,
    Spawn,
    Time,
}

impl EffectClass {
    fn label(self) -> &'static str {
        match self {
            Self::Env => "environment",
            Self::Filesystem => "filesystem",
            Self::Io => "standard IO",
            Self::Logging => "logging",
            Self::Net => "network",
            Self::Process => "process",
            Self::Random => "randomness",
            Self::Runtime => "runtime construction",
            Self::Spawn => "task spawning",
            Self::Time => "time",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PureViolation {
    DirectEffect(EffectClass),
    GlobalState,
    InteriorMutability,
    MutatingInput,
    Panic,
    ResourceReturn,
}

impl PureViolation {
    fn label(self) -> &'static str {
        match self {
            Self::DirectEffect(effect) => effect.label(),
            Self::GlobalState => "global state",
            Self::InteriorMutability => "interior mutability",
            Self::MutatingInput => "mutating input",
            Self::Panic => "panic",
            Self::ResourceReturn => "resource return",
        }
    }
}

declare_tool_lint! {
    /// Flags direct process environment reads or mutations in production code.
    pub tigerstyle::AMBIENT_ENV,
    Warn,
    "direct ambient process environment access"
}

declare_tool_lint! {
    /// Flags direct ambient randomness and non-deterministic ID generation.
    pub tigerstyle::AMBIENT_RANDOM,
    Warn,
    "direct ambient randomness source"
}

declare_tool_lint! {
    /// Flags process-wide mutable or lazy state declarations in configured core scopes.
    pub tigerstyle::GLOBAL_STATE_IN_CORE,
    Warn,
    "global mutable state in configured core scope"
}

declare_tool_lint! {
    /// Flags direct platform-effect calls in configured core scopes.
    pub tigerstyle::IMPURE_CALL_IN_CORE,
    Warn,
    "platform effect call in configured core scope"
}

declare_tool_lint! {
    /// Flags interior mutability and synchronization primitives in configured core scopes.
    pub tigerstyle::INTERIOR_MUTABILITY_IN_CORE,
    Warn,
    "interior mutability in configured core scope"
}

declare_tool_lint! {
    /// Flags logging and stdout/stderr emission in configured core scopes.
    pub tigerstyle::LOGGING_IN_CORE,
    Warn,
    "logging or console output in configured core scope"
}

declare_tool_lint! {
    /// Advises when a private helper is conservatively eligible to become `const fn`.
    pub tigerstyle::MISSING_CONST_FN,
    Allow,
    "private helper can be declared const fn"
}

declare_tool_lint! {
    /// Flags mutating caller-owned state parameters in declared pure contexts.
    pub tigerstyle::MUTATING_INPUT_IN_PURE,
    Warn,
    "mutating input capability in pure context"
}

declare_tool_lint! {
    /// Advises when unordered hash iteration flows into ordered output without sorting.
    pub tigerstyle::NONDETERMINISTIC_OUTPUT,
    Allow,
    "unordered hash iteration collected into ordered output"
}

declare_tool_lint! {
    /// Enforces direct-effect purity checks for functions marked as pure.
    pub tigerstyle::PURE_ATTR_CONTRACT,
    Warn,
    "marked pure function contains a direct impurity"
}

declare_tool_lint! {
    /// Flags core functions that return concrete resource or capability handles.
    pub tigerstyle::RESOURCE_RETURN_FROM_CORE,
    Warn,
    "core function returns concrete resource handle"
}

declare_tool_lint! {
    /// Flags direct thread/task spawning in configured core scopes.
    pub tigerstyle::SPAWN_IN_CORE,
    Warn,
    "task or thread spawn in configured core scope"
}

declare_tool_lint! {
    /// Enforces direct-effect allocation checks for functions marked with the
    /// zero-allocation marker. Flags direct heap-allocation constructors in the
    /// annotated body and guides maintainers toward stack-local construction or
    /// pre-allocated buffers. This lint checks direct source shapes only, not
    /// transitive whole-program non-allocation proof.
    pub tigerstyle::ZERO_ALLOC_ATTR_CONTRACT,
    Warn,
    "marked zero-alloc function contains direct allocation"
}

pub struct AmbientEnv;
pub struct AmbientRandom;
pub struct GlobalStateInCore {
    core_source_scopes: Vec<String>,
}
pub struct ImpureCallInCore {
    core_source_scopes: Vec<String>,
}
pub struct InteriorMutabilityInCore {
    core_source_scopes: Vec<String>,
}
pub struct LoggingInCore {
    core_source_scopes: Vec<String>,
    reported_callsites: std::collections::HashSet<Span>,
}
pub struct MissingConstFn;
pub struct MutatingInputInPure {
    core_source_scopes: Vec<String>,
}
pub struct NondeterministicOutput;
pub struct PureAttrContract;
pub struct ResourceReturnFromCore {
    core_source_scopes: Vec<String>,
}
pub struct SpawnInCore {
    core_source_scopes: Vec<String>,
}
pub struct ZeroAllocAttrContract;

impl GlobalStateInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl ImpureCallInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl InteriorMutabilityInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl LoggingInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self {
            core_source_scopes,
            reported_callsites: std::collections::HashSet::new(),
        }
    }
}

impl MutatingInputInPure {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl ResourceReturnFromCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl SpawnInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl_lint_pass!(AmbientEnv => [AMBIENT_ENV]);
impl_lint_pass!(AmbientRandom => [AMBIENT_RANDOM]);
impl_lint_pass!(GlobalStateInCore => [GLOBAL_STATE_IN_CORE]);
impl_lint_pass!(ImpureCallInCore => [IMPURE_CALL_IN_CORE]);
impl_lint_pass!(InteriorMutabilityInCore => [INTERIOR_MUTABILITY_IN_CORE]);
impl_lint_pass!(LoggingInCore => [LOGGING_IN_CORE]);
impl_lint_pass!(MissingConstFn => [MISSING_CONST_FN]);
impl_lint_pass!(MutatingInputInPure => [MUTATING_INPUT_IN_PURE]);
impl_lint_pass!(NondeterministicOutput => [NONDETERMINISTIC_OUTPUT]);
impl_lint_pass!(PureAttrContract => [PURE_ATTR_CONTRACT]);
impl_lint_pass!(ResourceReturnFromCore => [RESOURCE_RETURN_FROM_CORE]);
impl_lint_pass!(SpawnInCore => [SPAWN_IN_CORE]);
impl_lint_pass!(ZeroAllocAttrContract => [ZERO_ALLOC_ATTR_CONTRACT]);

fn compact(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn contains_any(source: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| source.contains(pattern))
}

fn contains_env_call(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    ENV_CALLS.iter().copied().find(|call| {
        let std_pattern = format!("std::env::{call}(");
        let short_pattern = format!("env::{call}(");
        compact_source.contains(&std_pattern) || compact_source.contains(&short_pattern)
    })
}

fn classify_random_call(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    RANDOM_CALL_PATTERNS.iter().copied().find(|pattern| compact_source.contains(pattern))
}

fn classify_spawn_call(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    SPAWN_CALL_PATTERNS.iter().copied().find(|pattern| compact_source.contains(pattern))
}

fn classify_logging_call(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    LOGGING_CALL_PATTERNS.iter().copied().find(|pattern| compact_source.contains(pattern))
}

fn classify_effect(source: &str) -> Option<EffectClass> {
    let compact_source = compact(source);
    if contains_env_call(&compact_source).is_some() {
        return Some(EffectClass::Env);
    }
    if classify_random_call(&compact_source).is_some() {
        return Some(EffectClass::Random);
    }
    if classify_spawn_call(&compact_source).is_some() {
        return Some(EffectClass::Spawn);
    }
    if classify_logging_call(&compact_source).is_some() {
        return Some(EffectClass::Logging);
    }
    if contains_any(&compact_source, FILESYSTEM_CALL_PATTERNS) {
        return Some(EffectClass::Filesystem);
    }
    if contains_any(&compact_source, IO_CALL_PATTERNS) {
        return Some(EffectClass::Io);
    }
    if contains_any(&compact_source, NET_CALL_PATTERNS) {
        return Some(EffectClass::Net);
    }
    if contains_any(&compact_source, PROCESS_CALL_PATTERNS) {
        return Some(EffectClass::Process);
    }
    if contains_any(&compact_source, TIME_CALL_PATTERNS) {
        return Some(EffectClass::Time);
    }
    if contains_any(&compact_source, RUNTIME_CALL_PATTERNS) {
        return Some(EffectClass::Runtime);
    }
    None
}

/// Returns the span to report when `item` declares process-wide state: a
/// `static mut`, a `static` whose type has interior mutability (locks, atomics,
/// cells, lazy or once cells), or a `thread_local!` key. Statics generated
/// inside a `thread_local!` expansion belong to that key's declaration.
fn global_state_declaration_span(cx: &LateContext<'_>, item: &hir::Item<'_>) -> Option<Span> {
    match item.kind {
        hir::ItemKind::Static(mutability, ..) => {
            if span_is_from_thread_local_macro(cx, item.span) {
                return None;
            }
            let ty = cx.tcx.type_of(item.owner_id).instantiate_identity();
            let interior_mutable = !ty.is_freeze(cx.tcx, ty::TypingEnv::fully_monomorphized());
            (mutability.is_mut() || interior_mutable).then(|| item.span.source_callsite())
        }
        hir::ItemKind::Const(ident, ..) => {
            let ty::Adt(adt, _) = cx.tcx.type_of(item.owner_id).instantiate_identity().kind() else {
                return None;
            };
            let is_thread_local_key =
                cx.tcx.get_diagnostic_name(adt.did()).is_some_and(|name| name.as_str() == "LocalKey");
            is_thread_local_key.then_some(ident.span)
        }
        _ => None,
    }
}

fn span_is_from_thread_local_macro(cx: &LateContext<'_>, span: Span) -> bool {
    let mut current = span;
    while current.from_expansion() {
        let expn_data = current.ctxt().outer_expn_data();
        let is_thread_local = expn_data
            .macro_def_id
            .and_then(|macro_def_id| cx.tcx.get_diagnostic_name(macro_def_id))
            .is_some_and(|name| name.as_str() == "thread_local_macro");
        if is_thread_local {
            return true;
        }
        current = expn_data.call_site;
    }
    false
}

/// Finds global-state declarations among the items nested in a function body.
struct GlobalStateFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    found: bool,
}

impl<'tcx> Visitor<'tcx> for GlobalStateFinder<'_, 'tcx> {
    type NestedFilter = All;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.cx.tcx
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        if global_state_declaration_span(self.cx, item).is_some() {
            self.found = true;
            return;
        }
        rustc_hir::intravisit::walk_item(self, item);
    }
}

fn body_declares_global_state<'tcx>(cx: &LateContext<'tcx>, body: &'tcx hir::Body<'tcx>) -> bool {
    let mut finder = GlobalStateFinder { cx, found: false };
    finder.visit_body(body);
    finder.found
}

fn contains_interior_mutability(source: &str) -> bool {
    let compact_source = compact(source);
    contains_any(&compact_source, INTERIOR_MUTABILITY_PATTERNS)
}

fn mutating_param_kind(source: &str) -> Option<&'static str> {
    let compact_source = compact(source);
    MUTATING_PARAM_PATTERNS
        .iter()
        .copied()
        .find(|pattern| source.contains(pattern) || compact_source.contains(pattern))
}

fn resource_handle_name(crate_name: &str, type_name: &str) -> Option<&'static str> {
    RESOURCE_HANDLE_TYPES
        .iter()
        .filter(|(handle_crate, _)| *handle_crate == crate_name)
        .flat_map(|(_, names)| names.iter().copied())
        .find(|name| *name == type_name)
}

fn adt_resource_handle_name(cx: &LateContext<'_>, adt_def_id: DefId) -> Option<&'static str> {
    let def_path = cx.get_def_path(adt_def_id);
    resource_handle_name(def_path.first()?.as_str(), def_path.last()?.as_str())
}

/// Returns the first concrete resource handle in `ty`, looking through
/// wrappers, tuples, references, type aliases, and the bounds of `impl Trait`
/// types such as an `async fn`'s future output.
fn type_resource_handle_name<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<&'static str> {
    let mut pending = vec![ty];
    let mut visited_opaques = Vec::new();
    while let Some(ty) = pending.pop() {
        for inner in ty.walk().filter_map(ty::GenericArg::as_type) {
            match inner.kind() {
                ty::Adt(adt, _) => {
                    if let Some(name) = adt_resource_handle_name(cx, adt.did()) {
                        return Some(name);
                    }
                }
                ty::Alias(ty::AliasTyKind::Opaque, alias) if !visited_opaques.contains(&alias.def_id) => {
                    visited_opaques.push(alias.def_id);
                    pending.extend(
                        cx.tcx
                            .explicit_item_bounds(alias.def_id)
                            .iter_identity_copied()
                            .filter_map(|(clause, _)| clause.as_projection_clause())
                            .filter_map(|projection| projection.skip_binder().term.as_type()),
                    );
                }
                _ => {}
            }
        }
    }
    None
}

/// Returns the resolved return type of a function or closure that spells out
/// its return type.
fn declared_return_ty<'tcx>(cx: &LateContext<'tcx>, decl: &hir::FnDecl<'_>, def_id: LocalDefId) -> Option<Ty<'tcx>> {
    if matches!(decl.output, hir::FnRetTy::DefaultReturn(_)) {
        return None;
    }
    if cx.tcx.def_kind(def_id) == DefKind::Closure {
        let ty::Closure(_, args) = cx.tcx.type_of(def_id).instantiate_identity().kind() else {
            return None;
        };
        return Some(args.as_closure().sig().output().skip_binder());
    }
    Some(cx.tcx.fn_sig(def_id).instantiate_identity().output().skip_binder())
}

fn returned_resource_handle(cx: &LateContext<'_>, decl: &hir::FnDecl<'_>, def_id: LocalDefId) -> Option<&'static str> {
    type_resource_handle_name(cx, declared_return_ty(cx, decl, def_id)?)
}

fn source_has_panic(source: &str) -> bool {
    let compact_source = compact(source);
    ["panic!(", "unimplemented!(", "unreachable!(", "todo!("]
        .iter()
        .any(|pattern| compact_source.contains(pattern))
}

const PURE_MARKER: &str = "octet::pure";
const ZERO_ALLOC_MARKER: &str = "octet::zero_alloc";

/// Whether the function or closure `def_id` itself carries `marker`, under any
/// tool prefix (`octet::`, `tigerstyle::`). Markers on nested items or closures,
/// and marker text in strings, comments, or doc attributes, do not count.
fn fn_is_declared(cx: &LateContext<'_>, def_id: LocalDefId, marker: &str) -> bool {
    attrs_carry_marker(cx.tcx.hir_attrs(cx.tcx.local_def_id_to_hir_id(def_id)), marker)
}

fn source_is_in_core_scope(cx: &LateContext<'_>, span: Span, scopes: &[String]) -> bool {
    crate::source::span_in_scopes(cx.sess(), span, scopes)
}

fn late_expr_should_skip(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span)
}

fn expr_is_direct_call(expr: &hir::Expr<'_>) -> bool {
    matches!(expr.kind, hir::ExprKind::Call(..))
}

/// Returns the source text of a direct call's callee path followed by `(`, so
/// call patterns such as `std::env::var(` match.
///
/// Only the callee is inspected: arguments are separate expressions that report
/// on their own, so wrappers like `Some(std::env::var("X"))` or
/// `parse(std::env::var("X").ok())` must not match again. Callees produced by
/// macro expansion or desugaring (for example `format!` internals or the
/// `Try::branch` call behind `?`) have no user-written callee text and are
/// skipped.
fn direct_call_callee_snippet(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    let hir::ExprKind::Call(callee, _) = expr.kind else {
        return None;
    };
    if !matches!(callee.kind, hir::ExprKind::Path(..)) || callee.span.from_expansion() {
        return None;
    }
    let source = cx.sess().source_map().span_to_snippet(callee.span).ok()?;
    Some(format!("{source}("))
}

/// Maps a macro definition to its logging pattern when it is one of the
/// recognized std, `log`, or `tracing` logging macros.
fn logging_macro_pattern(cx: &LateContext<'_>, macro_def_id: rustc_hir::def_id::DefId) -> Option<&'static str> {
    if let Some(diagnostic_name) = cx.tcx.get_diagnostic_name(macro_def_id) {
        return match diagnostic_name.as_str() {
            "dbg_macro" => Some("dbg!("),
            "eprint_macro" => Some("eprint!("),
            "eprintln_macro" => Some("eprintln!("),
            "print_macro" => Some("print!("),
            "println_macro" => Some("println!("),
            _ => None,
        };
    }
    let crate_name = cx.tcx.crate_name(macro_def_id.krate);
    let macro_name = cx.tcx.opt_item_name(macro_def_id)?;
    match (crate_name.as_str(), macro_name.as_str()) {
        ("log", "debug") => Some("log::debug!("),
        ("log", "error") => Some("log::error!("),
        ("log", "info") => Some("log::info!("),
        ("log", "trace") => Some("log::trace!("),
        ("log", "warn") => Some("log::warn!("),
        ("tracing", "debug") => Some("tracing::debug!("),
        ("tracing", "error") => Some("tracing::error!("),
        ("tracing", "event") => Some("tracing::event!("),
        ("tracing", "info") => Some("tracing::info!("),
        ("tracing", "trace") => Some("tracing::trace!("),
        ("tracing", "warn") => Some("tracing::warn!("),
        _ => None,
    }
}

/// Returns the logging pattern and user-written callsite when `span` comes from
/// a direct invocation of a recognized logging macro.
///
/// The macro is identified by the outermost expansion's macro definition, so
/// nested expansions (for example the `eprintln!` inside `dbg!`) resolve to the
/// macro the user wrote. Invocations nested inside other macros are not
/// reported.
fn logging_macro_call(cx: &LateContext<'_>, span: Span) -> Option<(&'static str, Span)> {
    let mut outermost = None;
    let mut current = span;
    while current.from_expansion() {
        let expn_data = current.ctxt().outer_expn_data();
        current = expn_data.call_site;
        outermost = Some(expn_data);
    }

    let outermost = outermost?;
    let pattern = logging_macro_pattern(cx, outermost.macro_def_id?)?;
    Some((pattern, outermost.call_site))
}

fn span_snippet(cx: &LateContext<'_>, span: Span) -> Option<String> {
    cx.sess().source_map().span_to_snippet(span.source_callsite()).ok()
}

fn fn_snippet(cx: &LateContext<'_>, body: &hir::Body<'_>, span: Span) -> Option<String> {
    span_snippet(cx, span).or_else(|| span_snippet(cx, body.value.span))
}

fn params_contain_mutating_input(cx: &LateContext<'_>, body: &hir::Body<'_>) -> Option<(Span, &'static str)> {
    body.params.iter().find_map(|param| {
        let source = span_snippet(cx, param.pat.span)?;
        mutating_param_kind(&source).map(|kind| (param.pat.span, kind))
    })
}

fn decl_inputs_contain_mutating_type(cx: &LateContext<'_>, decl: &hir::FnDecl<'_>) -> Option<(Span, &'static str)> {
    decl.inputs.iter().find_map(|input| {
        let source = span_snippet(cx, input.span)?;
        mutating_param_kind(&source).map(|kind| (input.span, kind))
    })
}

fn function_has_nondeterministic_output(source: &str) -> bool {
    let compact_source = compact(source);
    if !contains_any(&compact_source, &["HashMap", "HashSet"]) {
        return false;
    }
    if contains_any(&compact_source, &["BTreeMap", "BTreeSet", ".sort", "sort_by", "sort_unstable"]) {
        return false;
    }
    let iterates_hash = contains_any(&compact_source, &[".iter(", ".keys(", ".values(", ".into_iter("]);
    let ordered_output = contains_any(&compact_source, &["->Vec<", "collect::<Vec", "Vec::from_iter"]);
    iterates_hash && ordered_output
}

fn function_is_const_candidate(source: &str) -> bool {
    let trimmed = source.trim_start();
    if !trimmed.contains("fn ") {
        return false;
    }
    if trimmed.starts_with("pub fn")
        || trimmed.starts_with("pub(crate) fn")
        || trimmed.starts_with("pub(super) fn")
        || trimmed.starts_with("const fn")
        || trimmed.starts_with("async fn")
        || trimmed.starts_with("unsafe fn")
        || contains_any(source, CONST_INELIGIBLE_PATTERNS)
    {
        return false;
    }
    if classify_effect(source).is_some() || contains_interior_mutability(source) || source_has_panic(source) {
        return false;
    }
    contains_any(source, CONST_ELIGIBLE_HINTS)
}

fn const_subset_type(ty: rustc_middle::ty::Ty<'_>) -> bool {
    use rustc_middle::ty::TyKind;

    matches!(
        ty.kind(),
        TyKind::Bool | TyKind::Char | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_) | TyKind::Never
    ) || ty.is_unit()
}

struct ConstSubsetVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    eligible: bool,
}

impl<'tcx> Visitor<'tcx> for ConstSubsetVisitor<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if !self.eligible {
            return;
        }
        // Absence of known effects is not evidence that a call is const. Admit
        // only primitive expressions with built-in (not trait-resolved) operators.
        let supported = matches!(
            expr.kind,
            hir::ExprKind::Lit(_)
                | hir::ExprKind::Path(_)
                | hir::ExprKind::Binary(..)
                | hir::ExprKind::Unary(..)
                | hir::ExprKind::Cast(..)
                | hir::ExprKind::If(..)
                | hir::ExprKind::Match(..)
                | hir::ExprKind::Block(..)
                | hir::ExprKind::Ret(_)
                | hir::ExprKind::Let(_)
                | hir::ExprKind::DropTemps(_)
        );
        let static_path = if let hir::ExprKind::Path(path) = &expr.kind {
            matches!(
                self.cx.typeck_results().qpath_res(path, expr.hir_id),
                hir::def::Res::Def(hir::def::DefKind::Static { .. }, _)
            )
        } else {
            false
        };
        self.eligible = supported
            && !static_path
            && const_subset_type(self.cx.typeck_results().expr_ty(expr))
            && self.cx.typeck_results().type_dependent_def_id(expr.hir_id).is_none();
        if self.eligible {
            hir::intravisit::walk_expr(self, expr);
        }
    }
}

/// Observed signature and body signals for one `#[tigerstyle::pure]` callable.
#[derive(Clone, Copy)]
struct PureContractSignals {
    global_state: bool,
    mutating_input: bool,
    resource_return: bool,
}

fn pure_contract_violation(function_source: &str, signals: PureContractSignals) -> Option<PureViolation> {
    if let Some(effect) = classify_effect(function_source) {
        return Some(PureViolation::DirectEffect(effect));
    }
    if signals.global_state {
        return Some(PureViolation::GlobalState);
    }
    if contains_interior_mutability(function_source) {
        return Some(PureViolation::InteriorMutability);
    }
    if signals.mutating_input {
        return Some(PureViolation::MutatingInput);
    }
    if source_has_panic(function_source) {
        return Some(PureViolation::Panic);
    }
    if signals.resource_return {
        return Some(PureViolation::ResourceReturn);
    }
    None
}

fn emit(cx: &LateContext<'_>, lint: &'static rustc_lint::Lint, span: Span, message: String, lint_name: &'static str) {
    emit_tiger_style_lint(cx, lint, span, message, lint_name);
}

impl<'tcx> LateLintPass<'tcx> for AmbientEnv {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr) || late_expr_should_skip(cx, expr) {
            return;
        }
        let Some(callee) = direct_call_callee_snippet(cx, expr) else {
            return;
        };
        let Some(call) = contains_env_call(&callee) else {
            return;
        };
        emit(
            cx,
            AMBIENT_ENV,
            expr.span.source_callsite(),
            format!("direct `std::env::{call}` access reads or mutates ambient process state"),
            AMBIENT_ENV_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for AmbientRandom {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr) || late_expr_should_skip(cx, expr) {
            return;
        }
        let Some(callee) = direct_call_callee_snippet(cx, expr) else {
            return;
        };
        let Some(pattern) = classify_random_call(&callee) else {
            return;
        };
        emit(
            cx,
            AMBIENT_RANDOM,
            expr.span.source_callsite(),
            format!("direct randomness source `{pattern}` makes core behavior non-deterministic"),
            AMBIENT_RANDOM_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for GlobalStateInCore {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let Some(report_span) = global_state_declaration_span(cx, item) else {
            return;
        };
        if !source_is_in_core_scope(cx, report_span, &self.core_source_scopes)
            || test_context::late_hir_id_is_in_test_context(cx, item.hir_id(), report_span)
        {
            return;
        }
        emit(
            cx,
            GLOBAL_STATE_IN_CORE,
            report_span,
            "core scope declares process-wide mutable or lazy state; thread explicit state through inputs instead"
                .to_owned(),
            GLOBAL_STATE_IN_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for ImpureCallInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr)
            || !source_is_in_core_scope(cx, expr.span, &self.core_source_scopes)
            || late_expr_should_skip(cx, expr)
        {
            return;
        }
        let Some(callee) = direct_call_callee_snippet(cx, expr) else {
            return;
        };
        let Some(effect) = classify_effect(&callee) else {
            return;
        };
        emit(
            cx,
            IMPURE_CALL_IN_CORE,
            expr.span.source_callsite(),
            format!(
                "core scope performs direct {} effect; inject a port or move the call to the shell",
                effect.label()
            ),
            IMPURE_CALL_IN_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for InteriorMutabilityInCore {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if !source_is_in_core_scope(cx, item.span, &self.core_source_scopes)
            || test_context::late_hir_id_is_in_test_context(cx, item.hir_id(), item.span)
        {
            return;
        }
        let Some(source) = span_snippet(cx, item.span) else {
            return;
        };
        if !contains_interior_mutability(&source) {
            return;
        }
        emit(
            cx,
            INTERIOR_MUTABILITY_IN_CORE,
            item.span,
            "core scope owns interior mutability; prefer explicit state transitions or shell-owned synchronization"
                .to_owned(),
            INTERIOR_MUTABILITY_IN_CORE_LINT,
        );
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr)
            || !source_is_in_core_scope(cx, expr.span, &self.core_source_scopes)
            || late_expr_should_skip(cx, expr)
        {
            return;
        }
        let Some(callee) = direct_call_callee_snippet(cx, expr) else {
            return;
        };
        if !contains_interior_mutability(&callee) {
            return;
        }
        emit(
            cx,
            INTERIOR_MUTABILITY_IN_CORE,
            expr.span.source_callsite(),
            "core scope constructs an interior-mutability primitive; keep synchronization in the shell".to_owned(),
            INTERIOR_MUTABILITY_IN_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for LoggingInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let Some((pattern, callsite)) = logging_macro_call(cx, expr.span) else {
            return;
        };
        if !source_is_in_core_scope(cx, callsite, &self.core_source_scopes) || late_expr_should_skip(cx, expr) {
            return;
        }
        // Every expression expanded from one invocation shares its callsite;
        // report the invocation once.
        if !self.reported_callsites.insert(callsite) {
            return;
        }
        emit(
            cx,
            LOGGING_IN_CORE,
            callsite,
            format!("core scope emits `{pattern}` directly; return events or diagnostics for the shell to log"),
            LOGGING_IN_CORE_LINT,
        );
    }
}

/// Trait methods and trait-impl methods cannot be declared `const fn` (E0379).
fn fn_is_trait_method(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    matches!(cx.tcx.def_kind(cx.tcx.local_parent(def_id)), DefKind::Trait | DefKind::Impl { of_trait: true })
}

/// Finds a call or method call whose callee is not a `const fn` (including
/// calls through function pointers, closures, and generic callables) in the
/// function's own body.
struct NonConstCallFinder<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    typeck_results: &'a ty::TypeckResults<'tcx>,
    found: bool,
}

impl<'tcx> Visitor<'tcx> for NonConstCallFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        let callee_is_const = match expr.kind {
            hir::ExprKind::Call(callee, _) => match self.typeck_results.expr_ty(callee).kind() {
                ty::FnDef(callee_def_id, _) => self.tcx.is_const_fn(*callee_def_id),
                _ => false,
            },
            hir::ExprKind::MethodCall(..) => self
                .typeck_results
                .type_dependent_def_id(expr.hir_id)
                .is_some_and(|callee_def_id| self.tcx.is_const_fn(callee_def_id)),
            _ => true,
        };
        if !callee_is_const {
            self.found = true;
            return;
        }
        rustc_hir::intravisit::walk_expr(self, expr);
    }
}

fn body_calls_non_const_fn<'tcx>(cx: &LateContext<'tcx>, body: &'tcx hir::Body<'tcx>) -> bool {
    let mut finder = NonConstCallFinder {
        tcx: cx.tcx,
        typeck_results: cx.tcx.typeck_body(body.id()),
        found: false,
    };
    finder.visit_expr(body.value);
    finder.found
}

impl<'tcx> LateLintPass<'tcx> for MissingConstFn {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if !matches!(kind, FnKind::ItemFn(..))
            || test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span)
        {
            return;
        }
        let Some(source) = fn_snippet(cx, body, span) else {
            return;
        };
        if !function_is_const_candidate(&source) || fn_is_trait_method(cx, def_id) || body_calls_non_const_fn(cx, body)
        {
            return;
        }
        let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
        if !signature.inputs().iter().copied().all(const_subset_type) || !const_subset_type(signature.output()) {
            return;
        }
        let mut subset = ConstSubsetVisitor { cx, eligible: true };
        subset.visit_expr(body.value);
        if !subset.eligible {
            return;
        }
        emit(
            cx,
            MISSING_CONST_FN,
            span,
            "private helper appears const-compatible; consider declaring it `const fn` if that does not freeze public API".to_owned(),
            MISSING_CONST_FN_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for MutatingInputInPure {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span) {
            return;
        }
        let declared_pure = fn_is_declared(cx, def_id, PURE_MARKER);
        let in_core_scope = source_is_in_core_scope(cx, span, &self.core_source_scopes);
        if !declared_pure && !in_core_scope {
            return;
        }
        let Some((report_span, kind)) =
            decl_inputs_contain_mutating_type(cx, decl).or_else(|| params_contain_mutating_input(cx, body))
        else {
            return;
        };
        emit(
            cx,
            MUTATING_INPUT_IN_PURE,
            report_span,
            format!(
                "pure context accepts mutating input capability `{kind}`; return an explicit state transition instead"
            ),
            MUTATING_INPUT_IN_PURE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for NondeterministicOutput {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        _def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span) {
            return;
        }
        let Some(source) = fn_snippet(cx, body, span) else {
            return;
        };
        if !function_has_nondeterministic_output(&source) {
            return;
        }
        emit(
            cx,
            NONDETERMINISTIC_OUTPUT,
            span,
            "hash-collection iteration flows into ordered output without a stable sort".to_owned(),
            NONDETERMINISTIC_OUTPUT_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for PureAttrContract {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span)
            || !fn_is_declared(cx, def_id, PURE_MARKER)
        {
            return;
        }
        let Some(source) = fn_snippet(cx, body, span) else {
            return;
        };
        let global_state = body_declares_global_state(cx, body);
        let mutating_input = decl_inputs_contain_mutating_type(cx, decl).is_some();
        let resource_return = returned_resource_handle(cx, decl, def_id).is_some();
        let Some(violation) = pure_contract_violation(&source, PureContractSignals {
            global_state,
            mutating_input,
            resource_return,
        }) else {
            return;
        };
        emit(
            cx,
            PURE_ATTR_CONTRACT,
            span,
            format!(
                "marked pure function contains direct {} impurity; this lint checks direct effects only, not transitive proof",
                violation.label()
            ),
            PURE_ATTR_CONTRACT_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for ZeroAllocAttrContract {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span)
            || !fn_is_declared(cx, def_id, ZERO_ALLOC_MARKER)
        {
            return;
        }
        check_marked_zero_alloc_body(cx, body, span);
    }
}

fn check_marked_zero_alloc_body<'tcx>(cx: &LateContext<'tcx>, body: &'tcx hir::Body<'tcx>, fn_span: Span) {
    // Walk the expressions of the marked body only; see `AllocExprFinder`.
    let mut finder = AllocExprFinder { cx, fn_span };
    finder.visit_expr(body.value);
}

/// Visits the marked function's own body with the default nested filter, so it
/// does not enter nested bodies: closures and nested functions are separate
/// functions, and nested const items, array lengths, and inline constants are
/// evaluated at compile time. Those bodies also have their own typeck results,
/// which the shared allocation classifier cannot read through `cx`.
#[allow(dead_code)]
struct AllocExprFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    fn_span: Span,
}

impl<'tcx> rustc_hir::intravisit::Visitor<'tcx> for AllocExprFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if let Some(type_name) = allocation_in_loop::expr_is_allocation(self.cx, expr) {
            let message = format!(
                "marked zero-alloc function constructs `{type_name}` which allocates on the heap; this lint checks direct allocation sites only, not transitive proof"
            );
            emit(self.cx, ZERO_ALLOC_ATTR_CONTRACT, expr.span, message, ZERO_ALLOC_ATTR_CONTRACT_LINT);
        }
        rustc_hir::intravisit::walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for ResourceReturnFromCore {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        if !source_is_in_core_scope(cx, span, &self.core_source_scopes)
            || test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span)
        {
            return;
        }
        let Some(resource) = returned_resource_handle(cx, decl, def_id) else {
            return;
        };
        emit(
            cx,
            RESOURCE_RETURN_FROM_CORE,
            span,
            format!(
                "core function returns concrete resource `{resource}`; return a command description or domain value instead"
            ),
            RESOURCE_RETURN_FROM_CORE_LINT,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for SpawnInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if !expr_is_direct_call(expr)
            || !source_is_in_core_scope(cx, expr.span, &self.core_source_scopes)
            || late_expr_should_skip(cx, expr)
        {
            return;
        }
        let Some(callee) = direct_call_callee_snippet(cx, expr) else {
            return;
        };
        let Some(pattern) = classify_spawn_call(&callee) else {
            return;
        };
        emit(
            cx,
            SPAWN_IN_CORE,
            expr.span.source_callsite(),
            format!("core scope starts work with `{pattern}`; return a command or use an injected executor port"),
            SPAWN_IN_CORE_LINT,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::EffectClass;
    use super::PureViolation;
    use super::classify_effect;
    use super::classify_random_call;
    use super::contains_env_call;
    use super::function_has_nondeterministic_output;
    use super::function_is_const_candidate;
    use super::pure_contract_violation;
    use super::resource_handle_name;

    #[test]
    fn env_classifier_accepts_direct_calls_and_rejects_explicit_config() {
        assert_eq!(contains_env_call("std::env::var(\"HOME\")"), Some("var"));
        assert_eq!(contains_env_call("config.home_dir.clone()"), None);
    }

    #[test]
    fn random_classifier_flags_ambient_fastrand_and_allows_seeded_rng() {
        assert_eq!(classify_random_call("fastrand::u64("), Some("fastrand::u64("));
        assert_eq!(classify_random_call("fastrand::Rng::new("), Some("fastrand::Rng::new("));
        assert_eq!(classify_random_call("fastrand::Rng::with_seed("), None);
        assert_eq!(classify_random_call("fastrand::seed("), None);
    }

    #[test]
    fn effect_classifier_keeps_shell_port_calls_out_of_scope() {
        assert_eq!(classify_effect("std::fs::read_to_string(path)"), Some(EffectClass::Filesystem));
        assert_eq!(classify_effect("ports.read_file(path)"), None);
    }

    #[test]
    fn effect_classifier_treats_print_and_eprint_as_logging() {
        assert_eq!(classify_effect("print!(\"{value}\")"), Some(EffectClass::Logging));
        assert_eq!(classify_effect("eprint!(\"{value}\")"), Some(EffectClass::Logging));
    }

    #[test]
    fn resource_return_classifier_matches_handle_types_not_name_fragments() {
        assert_eq!(resource_handle_name("std", "File"), Some("File"));
        assert_eq!(resource_handle_name("tokio", "JoinHandle"), Some("JoinHandle"));
        assert_eq!(resource_handle_name("std", "SocketAddr"), None);
        assert_eq!(resource_handle_name("my_core", "Command"), None);
        assert_eq!(resource_handle_name("my_core", "FileId"), None);
    }

    #[test]
    fn nondeterministic_output_requires_hash_iteration_without_sort() {
        assert!(function_has_nondeterministic_output(
            "fn names(map: HashMap<String, u8>) -> Vec<String> { map.keys().cloned().collect::<Vec<_>>() }"
        ));
        assert!(!function_has_nondeterministic_output(
            "fn names(map: HashMap<String, u8>) -> Vec<String> { let mut out = map.keys().cloned().collect::<Vec<_>>(); out.sort(); out }"
        ));
    }

    #[test]
    fn const_candidate_classifier_is_conservative() {
        assert!(function_is_const_candidate("fn double(value: u32) -> u32 { value * 2 }"));
        assert!(!function_is_const_candidate("pub fn double(value: u32) -> u32 { value * 2 }"));
        assert!(!function_is_const_candidate(
            "fn load(path: &Path) -> String { std::fs::read_to_string(path).unwrap() }"
        ));
    }

    #[test]
    fn pure_contract_reports_direct_violation_and_accepts_clean_logic() {
        assert_eq!(
            pure_contract_violation(
                "#[tigerstyle::pure] fn read() { std::fs::read(\"x\"); }",
                super::PureContractSignals {
                    global_state: false,
                    mutating_input: false,
                    resource_return: false,
                }
            ),
            Some(PureViolation::DirectEffect(EffectClass::Filesystem))
        );
        assert_eq!(
            pure_contract_violation(
                "#[tigerstyle::pure] fn count() { static mut N: u8 = 0; }",
                super::PureContractSignals {
                    global_state: true,
                    mutating_input: false,
                    resource_return: false,
                }
            ),
            Some(PureViolation::GlobalState)
        );
        assert_eq!(
            pure_contract_violation(
                "#[tigerstyle::pure] fn open() -> std::fs::File { loop {} }",
                super::PureContractSignals {
                    global_state: false,
                    mutating_input: false,
                    resource_return: true,
                }
            ),
            Some(PureViolation::ResourceReturn)
        );
        assert_eq!(
            pure_contract_violation(
                "#[tigerstyle::pure] fn label() -> &'static str { \"static mut\" }",
                super::PureContractSignals {
                    global_state: false,
                    mutating_input: false,
                    resource_return: false,
                }
            ),
            None
        );
    }
}
