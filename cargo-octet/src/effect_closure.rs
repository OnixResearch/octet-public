use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;

use quote::ToTokens;
use serde::Deserialize;
use serde::Serialize;
use syn::visit::Visit;

pub const CATALOG_SCHEMA_VERSION: &str = "octet-effect-closure-catalog/v1";
pub const WORKSPACE_POLICY_SCHEMA_VERSION: &str = "octet-effect-closure-workspace-policy/v1";
pub const RECEIPT_SCHEMA_VERSION: &str = "octet-effect-closure-receipt/v2";
pub const DETERMINISTIC_CORE_PROFILE: &str = "deterministic-core";
pub const CLAIM_SCOPE: &str = "selected-source-token-effect-closure";
pub const WORKSPACE_POLICY_FILE: &str = "octet-effect-closure.json";
pub const RECEIPT_FILE: &str = "effect-closure.json";

const CATALOG_JSON: &str = include_str!("../../standards/effect-closure/generated/effect-closure-catalog.json");
const CALL_SITE_DOMAIN: &str = "octet-effect-closure-call-site/v2";
const RECEIPT_DOMAIN: &str = "octet-effect-closure-receipt/v2";
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
    /// The caller's module and what each path the call references resolves to outside the caller
    /// (for a method call, every declaration that may bring its trait into scope); part of the
    /// call-site ID.
    pub scope: String,
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
    module: Vec<String>,
    container: FunctionContainer,
    /// Whether an enclosing file, module, impl, or trait carries a non-inert attribute.
    opaque_context: bool,
    attrs: Vec<syn::Attribute>,
    /// Generic parameters of the enclosing `impl` or trait, which are in scope in the body.
    enclosing_generics: syn::Generics,
    signature: syn::Signature,
    block: syn::Block,
    outer_scope: OuterScope,
    source_identity: String,
}

/// The header of the `impl` or trait enclosing a selected function, whose generic parameters and
/// `Self` shadow module names in the body. Every function in the same `impl` shares it; the
/// enclosing module's declarations are in the [`ModuleIndex`].
#[derive(Clone, Default)]
struct OuterScope {
    header: Option<Rc<EnclosingHeader>>,
}

/// The declarations of one file or inline module that bind names for the functions in it, as
/// canonical text. Names do not inherit from parent modules, so only the immediate module counts.
#[derive(Default)]
struct ModuleScope {
    /// Each `use` binding and item, keyed by the name it binds.
    names: BTreeMap<String, BTreeSet<String>>,
    /// Declarations that may bind any name: glob `use`s, item-position macro invocations, and items
    /// carrying an attribute macro or `derive`.
    wildcards: BTreeSet<String>,
    /// Declarations that may bring a trait, and so its methods, into scope: every `use` binding
    /// (including `as _`) and every trait item.
    method_sources: BTreeSet<String>,
    /// The paths each `use` binding imports, keyed by the name it binds.
    imports: BTreeMap<String, Vec<ImportPath>>,
    /// The module path of each glob `use`.
    glob_imports: Vec<ImportPath>,
    /// Module declarations, keyed by module name.
    modules: BTreeMap<String, ModuleDeclaration>,
    /// Names of free `fn` items.
    functions: BTreeSet<String>,
    /// Names of structs, enums, and unions, which may own inherent methods.
    types: BTreeSet<String>,
    /// Names every item other than a `use` declares.
    item_names: BTreeSet<String>,
    /// The tokens of each `macro_rules!` definition, keyed by macro name.
    macro_definitions: BTreeMap<String, String>,
    /// Whether an item macro, an attributed item, or unparsed tokens may generate items of any
    /// name, which a glob import never does.
    generated_items: bool,
}

impl ModuleScope {
    /// Whether a `use` path starting with `head` starts from a declaration of this module rather
    /// than an extern crate. An import of `head` whose own path starts with `head`
    /// (`use log::log;`) does not count: its path names the extern crate.
    fn declares_path_head(&self, head: &str) -> bool {
        self.item_names.contains(head)
            || self.imports.get(head).is_some_and(|imports| {
                imports.iter().any(|import| import.segments.first().map(String::as_str) != Some(head))
            })
    }
}

/// The path a `use` binding or glob imports, as written.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ImportPath {
    leading_colon: bool,
    segments: Vec<String>,
}

/// How a module declaration places its module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleDeclaration {
    /// `mod name { ... }` in the same file.
    Inline,
    /// `mod name;`, whose file follows from the declaring module's crate position.
    File,
    /// A declaration carrying a non-inert attribute (`path`, `cfg`, or a macro), which a path
    /// through it cannot follow.
    Opaque,
}

/// A file or inline module: its selected source path and inline-module chain within the file.
type ModuleLocator = (String, Vec<String>);

/// A module's position in its crate: the crate root (a `src` directory, or a binary's own root)
/// and the module path from the crate root.
type ModuleKey = (String, Vec<String>);

/// The modules of the selected sources, and which of them have a proven crate position: reachable
/// from a selected crate root (`src/lib.rs`, `src/main.rs`, or a binary root) through module
/// declarations that conventional file placement resolves. A path through `crate::`, `super::`,
/// or an out-of-line module is followed only between proven modules; an inline module is always
/// followed, since its declaration and body are in the same file.
#[derive(Default)]
struct ModuleIndex {
    scopes: BTreeMap<ModuleLocator, Rc<ModuleScope>>,
    keys: BTreeMap<ModuleLocator, ModuleKey>,
    locators: BTreeMap<ModuleKey, ModuleLocator>,
    proven: BTreeSet<ModuleKey>,
}

/// The header of an enclosing `impl` or trait, whose generic parameters and `Self` shadow module
/// names in the body.
struct EnclosingHeader {
    names: BTreeSet<String>,
    text: String,
}

/// The item that holds a selected function. Local call resolution trusts only call paths that prove
/// one of the resolvable containers.
#[derive(Clone, Debug, Eq, PartialEq)]
enum FunctionContainer {
    /// A free `fn` item in a file or inline module.
    Module,
    /// A method in an inherent `impl Owner` block.
    InherentImpl(String),
    /// A trait default body, trait-impl method, or inherent method without a single path owner;
    /// v1 never resolves a call to one of these locally.
    Dispatched,
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
    /// Whether the call's path form can name a selected local function at all. A qualified-self
    /// (`<Type>::name`) or leading-`::` path never does, whatever its last segment, nor does a path
    /// whose first segment is a name bound inside the caller.
    locally_resolvable: bool,
    /// Paths, without generic arguments, through which the call resolves outside the caller: the
    /// call path, or for a qualified-self path the self type's path and the trait's path. Paths
    /// with a leading `::` name an extern crate and are left out.
    referenced_paths: Vec<ReferencedPath>,
    /// The `use` bindings of the blocks enclosing a method call, any of which may bring the
    /// method's trait into scope.
    body_method_sources: BodyLookup,
    /// Whether the call is inside a module item declared in the function body, whose name
    /// resolution the module index does not model.
    in_body_module: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReferencedPath {
    segments: Vec<String>,
    /// What the blocks lexically enclosing the call declare for the path's first segment.
    body: BodyLookup,
}

/// The body declarations a path's first segment may resolve through, from the innermost block
/// enclosing the call outward. Blocks of nested `fn` items enclose their calls too, since block
/// items stay visible in nested items.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct BodyLookup {
    /// The imports of the innermost block that imports the name.
    imports: Vec<ImportPath>,
    /// Whether the innermost declaring block declares the name as an item rather than an import.
    item: bool,
    /// Glob imports of the blocks inside the declaring block, or of every enclosing block.
    globs: Vec<ImportPath>,
}

/// The `use` bindings and items one block declares.
#[derive(Default)]
struct BlockImports {
    imports: BTreeMap<String, Vec<ImportPath>>,
    glob_imports: Vec<ImportPath>,
    items: BTreeSet<String>,
}

impl BlockImports {
    fn of_block(block: &syn::Block) -> Self {
        let mut declarations = Self::default();
        for statement in &block.stmts {
            match statement {
                syn::Stmt::Item(syn::Item::Use(node)) => collect_imports(
                    &node.tree,
                    node.leading_colon.is_some(),
                    &mut Vec::new(),
                    &mut declarations.imports,
                    &mut declarations.glob_imports,
                ),
                syn::Stmt::Item(item) => {
                    if let Some(ident) = item_ident(item) {
                        declarations.items.insert(ident.to_string());
                    }
                }
                _ => {}
            }
        }
        declarations
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RawCallKind {
    Function,
    /// A path used as a value rather than called, such as a function passed as an argument.
    Value,
    Method,
    Macro,
}

impl RawCallKind {
    fn shape_prefix(self) -> &'static str {
        match self {
            Self::Function => "call:",
            Self::Value => "value:",
            Self::Method => "method:",
            Self::Macro => "macro:",
        }
    }
}

#[derive(Default)]
struct FunctionCollector {
    functions: Vec<CollectedFunction>,
    module: Vec<String>,
    /// Whether an enclosing file, module, impl, or trait carries a non-inert attribute.
    opaque_context: bool,
    /// Declarations of the file or inline module being visited.
    module_scope: Rc<ModuleScope>,
    /// Every module of the file with its inline-module chain, the file itself first.
    modules: FileModules,
    /// Generic parameters and header of the enclosing trait while its items are visited.
    trait_context: Option<(syn::Generics, Rc<EnclosingHeader>)>,
}

#[derive(Clone)]
struct CollectedFunction {
    name: String,
    module: Vec<String>,
    container: FunctionContainer,
    opaque_context: bool,
    attrs: Vec<syn::Attribute>,
    enclosing_generics: syn::Generics,
    signature: syn::Signature,
    block: syn::Block,
    outer_scope: OuterScope,
}

/// Names bound inside a selected function that may shadow a module item of the same name:
/// parameters and other pattern bindings, generic parameters (including the enclosing `impl` or
/// trait), and `use` declarations and items inside the body. Collection is function-wide rather
/// than lexical, so a name bound anywhere in the function blocks local resolution everywhere in it.
#[derive(Default)]
struct CallerBindings {
    /// Every bound name: pattern bindings, generic parameters, and body `use` and item names.
    names: BTreeSet<String>,
    /// Whether the body holds a glob `use`, which may bring any name into scope.
    glob_import: bool,
}

/// One lexical scope of the body scanner: a function signature, block, match arm, `if let` or
/// `while let` condition and body, `for` body, or closure. A single-segment path naming one of
/// `locals` reads a local value. A path naming an item declared in the scope, or any name under a
/// glob `use` in it, may name a function, so it stays a value edge.
#[derive(Default)]
struct ValueScope {
    /// Pattern bindings and const generic parameters visible at the current scan position.
    locals: BTreeSet<String>,
    /// Items and `use` names declared directly in a block, which shadow outer locals.
    items: BTreeSet<String>,
    glob_import: bool,
}

/// Collects the identifiers a pattern binds, without entering the expressions or types it holds.
struct PatternBindings<'a>(&'a mut BTreeSet<String>);

struct BodyScanner<'a> {
    foreign_functions: &'a BTreeSet<String>,
    bindings: &'a CallerBindings,
    /// Lexical value scopes, innermost last; a nested item starts a fresh stack.
    scopes: Vec<ValueScope>,
    /// Depth of enclosing compile-time positions: patterns, types, array repeat lengths, const
    /// generic arguments and parameter defaults, enum discriminants, and `const` or `static`
    /// initializers. Their paths name constants, variants, or types rather than runtime values.
    compile_time_depth: usize,
    direct_effects: BTreeSet<Effect>,
    calls: Vec<RawCall>,
    /// Declarations of the blocks enclosing the scan position, innermost last. Unlike `scopes`, a
    /// nested `fn` item keeps them, since block items stay visible inside nested items.
    block_imports: Vec<BlockImports>,
    /// Depth of module items declared inside the body.
    body_module_depth: usize,
}

// The collector shares the function-address producer's item model: it records the outermost `fn`
// items and does not descend into function bodies. Items nested in a body belong to the enclosing
// function, whose body scan already covers their tokens.
impl<'ast> Visit<'ast> for FunctionCollector {
    fn visit_file(&mut self, node: &'ast syn::File) {
        self.opaque_context |= has_opaque_attribute(&node.attrs);
        self.module_scope = Rc::new(ModuleScope::of_items(&node.items));
        self.modules.push((Vec::new(), Rc::clone(&self.module_scope)));
        syn::visit::visit_file(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let enclosing_opaque = self.opaque_context;
        self.opaque_context |= has_opaque_attribute(&node.attrs);
        let items = node.content.as_ref().map_or(&[][..], |(_, items)| items.as_slice());
        let enclosing_scope = std::mem::replace(&mut self.module_scope, Rc::new(ModuleScope::of_items(items)));
        self.module.push(node.ident.to_string());
        if node.content.is_some() {
            self.modules.push((self.module.clone(), Rc::clone(&self.module_scope)));
        }
        syn::visit::visit_item_mod(self, node);
        self.module.pop();
        self.module_scope = enclosing_scope;
        self.opaque_context = enclosing_opaque;
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let enclosing_opaque = self.opaque_context;
        self.opaque_context |= has_opaque_attribute(&node.attrs);
        self.trait_context = Some((node.generics.clone(), Rc::new(EnclosingHeader::of_trait(node))));
        syn::visit::visit_item_trait(self, node);
        self.trait_context = None;
        self.opaque_context = enclosing_opaque;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.functions.push(CollectedFunction {
            name: node.sig.ident.to_string(),
            module: self.module.clone(),
            container: FunctionContainer::Module,
            opaque_context: self.opaque_context,
            attrs: node.attrs.clone(),
            enclosing_generics: syn::Generics::default(),
            signature: node.sig.clone(),
            block: (*node.block).clone(),
            outer_scope: OuterScope { header: None },
        });
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let container = match (&node.trait_, owner_from_type(&node.self_ty)) {
            (None, Some(owner)) => FunctionContainer::InherentImpl(owner),
            _ => FunctionContainer::Dispatched,
        };
        let opaque_context = self.opaque_context || has_opaque_attribute(&node.attrs);
        let outer_scope = OuterScope {
            header: Some(Rc::new(EnclosingHeader::of_impl(node))),
        };
        for item in &node.items {
            if let syn::ImplItem::Fn(function) = item {
                self.functions.push(CollectedFunction {
                    name: function.sig.ident.to_string(),
                    module: self.module.clone(),
                    container: container.clone(),
                    opaque_context,
                    attrs: function.attrs.clone(),
                    enclosing_generics: node.generics.clone(),
                    signature: function.sig.clone(),
                    block: function.block.clone(),
                    outer_scope: outer_scope.clone(),
                });
            }
        }
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            let (enclosing_generics, header) = self
                .trait_context
                .as_ref()
                .map(|(generics, header)| (generics.clone(), Some(Rc::clone(header))))
                .unwrap_or_default();
            self.functions.push(CollectedFunction {
                name: node.sig.ident.to_string(),
                module: self.module.clone(),
                container: FunctionContainer::Dispatched,
                opaque_context: self.opaque_context,
                attrs: node.attrs.clone(),
                enclosing_generics,
                signature: node.sig.clone(),
                block: block.clone(),
                outer_scope: OuterScope { header },
            });
        }
    }
}

fn has_opaque_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attribute| !is_inert_attribute(attribute))
}

/// Whether `attribute` is a built-in attribute that expands to no code and changes no call:
/// documentation, lint levels, codegen and layout hints, and crate-level settings. Built-in
/// attribute names cannot be shadowed, so a single-segment match is exact. Every other attribute
/// may change what the annotated item compiles to: `cfg` and `cfg_attr`, `test` and `bench`
/// (which compile only under the test harness), `derive` (whose generated items can shadow module
/// names inside a body), every procedural attribute macro, and tool attributes such as
/// `rustfmt::skip`, `clippy::msrv`, or `diagnostic::on_unimplemented`: a module, import, extern
/// crate, or renamed Cargo dependency of the tool's name shadows the tool, and the recorded
/// dependency graph does not carry dependency renames, so a tool path cannot be proven built-in.
fn is_inert_attribute(attribute: &syn::Attribute) -> bool {
    const INERT_ATTRIBUTES: &[&str] = &[
        "allow",
        "cold",
        "deny",
        "deprecated",
        "doc",
        "expect",
        "feature",
        "forbid",
        "inline",
        "must_use",
        "no_std",
        "non_exhaustive",
        "recursion_limit",
        "register_tool",
        "repr",
        "track_caller",
        "warn",
    ];
    attribute.path().get_ident().is_some_and(|name| INERT_ATTRIBUTES.iter().any(|inert| name == inert))
}

fn module_text(module: &[String]) -> String {
    let mut text = String::from("module=self");
    for segment in module {
        text.push_str("::");
        text.push_str(segment);
    }
    text
}

/// Places a selected source file in its crate by Cargo's conventional layout: the crate root is
/// the last `src` directory (`src/lib.rs` or `src/main.rs`), or a binary under `src/bin`, and the
/// module path follows the directories and file stem below it (`mod.rs` names its directory).
/// Returns the crate root, the module path, and whether the file is a crate root. A file outside
/// that layout has no crate position.
fn module_location(path: &str) -> Option<(String, Vec<String>, bool)> {
    let components = path.split('/').collect::<Vec<_>>();
    let src = components.iter().rposition(|component| *component == "src")?;
    let root = components[..=src].join("/");
    match &components[src + 1..] {
        ["lib.rs" | "main.rs"] => Some((root, Vec::new(), true)),
        ["bin", file] => file.strip_suffix(".rs").map(|_| (path.to_owned(), Vec::new(), true)),
        ["bin", binary, "main.rs"] => Some((format!("{root}/bin/{binary}"), Vec::new(), true)),
        ["bin", binary, rest @ ..] => Some((format!("{root}/bin/{binary}"), module_segments(rest)?, false)),
        rest => Some((root, module_segments(rest)?, false)),
    }
}

fn module_segments(components: &[&str]) -> Option<Vec<String>> {
    let (file, directories) = components.split_last()?;
    let stem = file.strip_suffix(".rs")?;
    let mut segments = directories.iter().map(|directory| (*directory).to_owned()).collect::<Vec<_>>();
    if stem != "mod" {
        segments.push(stem.to_owned());
    }
    let identifiers = segments.iter().all(|segment| syn::parse_str::<syn::Ident>(segment).is_ok());
    (!segments.is_empty() && identifiers).then_some(segments)
}

/// Every module of one file with its inline-module chain, the file itself first.
type FileModules = Vec<(Vec<String>, Rc<ModuleScope>)>;

impl ModuleIndex {
    /// Indexes every module of the selected files and proves crate positions from the selected
    /// crate roots. Two modules claiming one crate position (for example `src/lib.rs` and
    /// `src/main.rs`) leave that position unproven.
    fn build(files: Vec<(String, FileModules)>) -> Self {
        let mut index = Self::default();
        let mut roots = Vec::new();
        let mut ambiguous = BTreeSet::new();
        for (path, modules) in files {
            let location = module_location(&path);
            for (chain, scope) in modules {
                let locator = (path.clone(), chain);
                if let Some((root, file_module, is_root)) = &location {
                    let mut module = file_module.clone();
                    module.extend(locator.1.iter().cloned());
                    let key = (root.clone(), module);
                    if *is_root && locator.1.is_empty() {
                        roots.push(key.clone());
                    }
                    if index.locators.insert(key.clone(), locator.clone()).is_some() {
                        ambiguous.insert(key.clone());
                    }
                    index.keys.insert(locator.clone(), key);
                }
                index.scopes.insert(locator, scope);
            }
        }
        for key in &ambiguous {
            index.locators.remove(key);
        }
        while let Some(key) = roots.pop() {
            let Some(locator) = index.locators.get(&key) else {
                continue;
            };
            if !index.proven.insert(key.clone()) {
                continue;
            }
            for (name, declaration) in &index.scopes[locator].modules {
                let mut child = key.clone();
                child.1.push(name.clone());
                let Some(child_locator) = index.locators.get(&child) else {
                    continue;
                };
                let placed = match declaration {
                    ModuleDeclaration::Inline => {
                        child_locator.0 == locator.0
                            && child_locator.1.split_last().map(|(_, parent)| parent) == Some(&locator.1[..])
                    }
                    ModuleDeclaration::File => child_locator.0 != locator.0 && child_locator.1.is_empty(),
                    ModuleDeclaration::Opaque => false,
                };
                if placed {
                    roots.push(child);
                }
            }
        }
        index
    }

    fn proven_key(&self, locator: &ModuleLocator) -> Option<&ModuleKey> {
        self.keys.get(locator).filter(|key| self.proven.contains(*key))
    }

    fn proven_locator(&self, key: &ModuleKey) -> Option<ModuleLocator> {
        self.proven.contains(key).then(|| self.locators.get(key).cloned()).flatten()
    }

    /// The module `name` declares inside `locator`: an inline module always, an out-of-line one
    /// only from a proven module.
    fn child(&self, locator: &ModuleLocator, name: &str) -> Option<ModuleLocator> {
        match self.scopes.get(locator)?.modules.get(name)? {
            ModuleDeclaration::Inline => {
                let mut chain = locator.1.clone();
                chain.push(name.to_owned());
                let child = (locator.0.clone(), chain);
                self.scopes.contains_key(&child).then_some(child)
            }
            ModuleDeclaration::File => {
                let mut child = self.proven_key(locator)?.clone();
                child.1.push(name.to_owned());
                self.proven_locator(&child).filter(|child_locator| child_locator.1.is_empty())
            }
            ModuleDeclaration::Opaque => None,
        }
    }

    /// The parent of an inline module is in the same file; a file module's parent needs a proven
    /// crate position.
    fn parent(&self, locator: &ModuleLocator) -> Option<ModuleLocator> {
        if let Some((_, parent)) = locator.1.split_last() {
            return Some((locator.0.clone(), parent.to_vec()));
        }
        let key = self.proven_key(locator)?;
        let (_, parent) = key.1.split_last()?;
        self.proven_locator(&(key.0.clone(), parent.to_vec()))
    }

    fn crate_root(&self, locator: &ModuleLocator) -> Option<ModuleLocator> {
        let key = self.proven_key(&(locator.0.clone(), Vec::new()))?;
        self.proven_locator(&(key.0.clone(), Vec::new()))
    }
}

/// Bounds on following one referenced path: nesting of imports and module hops, and total lookup
/// steps. Past either bound the path counts as hidden, so resolution fails closed.
const MAX_RESOLUTION_DEPTH: usize = 32;
const MAX_RESOLUTION_STEPS: usize = 1024;

/// Everything one referenced path resolves through, across every candidate: imports,
/// re-exports, globs, and module hops. Each (module, path) lookup runs once, so a glob cycle adds
/// nothing new, and the scope grows with the distinct declarations reached rather than with the
/// tree of candidates.
#[derive(Default)]
struct Lookup<'a> {
    /// Each declaration reached, labelled with its module and the name it declares.
    reached: BTreeSet<String>,
    /// Selected functions reached as a final target.
    locals: Vec<&'a AddressedFunction>,
    /// Final targets that are not selected functions: dependencies, the prelude, local items.
    declarations: usize,
    /// Whether a candidate goes through a module the selected sources cannot prove.
    hidden: bool,
    visited: BTreeSet<(ModuleLocator, Vec<String>)>,
    steps: usize,
    /// Depth of glob imports being probed: a candidate a glob may or may not supply, as opposed
    /// to a path the caller or an import wrote out.
    glob_probes: usize,
}

impl<'a> Lookup<'a> {
    fn reach(&mut self, label: &str, name: &str, declaration: &str) {
        self.reached.insert(format!("{label} {name}: {declaration}"));
    }

    /// A final target other than a selected function.
    fn reach_declaration(&mut self, label: &str, name: &str, declaration: &str) {
        self.declarations += 1;
        self.reach(label, name, declaration);
    }

    fn hide(&mut self, label: &str, name: &str, reason: &str) {
        self.hidden = true;
        self.reach(label, name, reason);
    }

    /// Counts one lookup step; past the depth or step bound the path is hidden.
    fn step(&mut self, depth: usize) -> bool {
        self.steps += 1;
        if depth > MAX_RESOLUTION_DEPTH || self.steps > MAX_RESOLUTION_STEPS {
            self.hide("", "*", "resolution limit");
            return false;
        }
        true
    }

    /// The selected function the path proves: the only final target, reached without hiding.
    fn local_target(&self) -> Option<&'a AddressedFunction> {
        match self.locals.as_slice() {
            [function] if !self.hidden && self.declarations == 0 => Some(function),
            _ => None,
        }
    }

    fn text(&self) -> String {
        self.reached.iter().map(String::as_str).collect::<Vec<_>>().join(", ")
    }
}

/// The selected source path and inline-module chain of a module, as `path::a::b`.
fn module_label(locator: &ModuleLocator) -> String {
    let mut label = locator.0.clone();
    for segment in &locator.1 {
        label.push_str("::");
        label.push_str(segment);
    }
    label
}

/// How an undeclared name resolves where it is looked up.
#[derive(Clone, Copy)]
enum NameContext {
    /// A path in the caller's body, where the prelude and extern crates are in scope.
    Unqualified,
    /// A path after `crate::`, `super::`, `self::`, or a module, where only the module's items are.
    Qualified,
    /// The first segment of a `use` path, which may name an extern crate.
    Import,
}

/// The resolution scope of one call: the call-site scope text, the selected function the call
/// path proves, and whether the path goes through a module or import the selected sources cannot
/// prove.
struct CallScope<'a> {
    text: String,
    local: Option<&'a AddressedFunction>,
    hidden: bool,
}

/// Resolves call paths across the selected modules: through `crate::`, `super::`, and `self::`,
/// module declarations, and local `use` bindings and re-exports, to a selected function (a local
/// edge) or to the declaration the path finally names, whose text the call-site ID binds.
struct ScopeResolver<'a> {
    modules: &'a ModuleIndex,
    functions: BTreeMap<&'a str, Vec<&'a AddressedFunction>>,
    /// Each module's method sources, resolved once per run.
    method_cache: RefCell<BTreeMap<ModuleLocator, Rc<ModuleMethods>>>,
    /// Modules whose method sources are being resolved, so a glob cycle stops.
    methods_in_progress: RefCell<BTreeSet<ModuleLocator>>,
}

/// The declarations of one module that may bring a trait into scope, with what its imports
/// resolve to.
#[derive(Default)]
struct ModuleMethods {
    reached: BTreeSet<String>,
    hidden: bool,
}

impl ModuleMethods {
    fn absorb(&mut self, lookup: &Lookup<'_>) {
        self.reached.extend(lookup.reached.iter().cloned());
        self.hidden |= lookup.hidden;
    }
}

impl<'a> ScopeResolver<'a> {
    fn new(modules: &'a ModuleIndex, functions: &'a [AddressedFunction]) -> Self {
        let mut by_path = BTreeMap::<&str, Vec<&AddressedFunction>>::new();
        for function in functions {
            by_path.entry(function.draft.path.as_str()).or_default().push(function);
        }
        Self {
            modules,
            functions: by_path,
            method_cache: RefCell::default(),
            methods_in_progress: RefCell::default(),
        }
    }

    /// Renders `module=self::a; path=declarations` for every path the call references.
    fn call_scope(&self, caller: &AddressedFunction, raw_call: &RawCall) -> CallScope<'a> {
        let draft = &caller.draft;
        let locator = (draft.path.clone(), draft.module.clone());
        let mut scope = CallScope {
            text: module_text(&draft.module),
            local: None,
            hidden: raw_call.in_body_module,
        };
        if raw_call.in_body_module {
            scope.text.push_str("; in a module declared in the body");
        }
        for path in &raw_call.referenced_paths {
            let mut lookup = Lookup::default();
            self.resolve_from_caller(&mut lookup, &locator, &draft.outer_scope, path);
            scope.text.push_str("; ");
            scope.text.push_str(&path.segments.join("::"));
            scope.text.push('=');
            scope.text.push_str(&lookup.text());
            scope.hidden |= lookup.hidden;
            if raw_call.locally_resolvable && raw_call.referenced_paths.len() == 1 && !raw_call.in_body_module {
                scope.local = lookup.local_target();
            }
        }
        scope
    }

    /// Imports and items of the blocks enclosing the call come first, then the enclosing `impl`
    /// or trait header, then the caller's module; glob imports of enclosing blocks may also
    /// supply the name.
    fn resolve_from_caller(
        &self,
        lookup: &mut Lookup<'a>,
        locator: &ModuleLocator,
        outer: &OuterScope,
        path: &ReferencedPath,
    ) {
        let Some((first, rest)) = path.segments.split_first() else {
            lookup.hide("", "", "empty path");
            return;
        };
        if is_path_keyword(first) {
            self.resolve_qualified(lookup, locator, &path.segments, 0);
            return;
        }
        if path.body.item {
            lookup.reach_declaration("body", first, "item");
        } else if !path.body.imports.is_empty() {
            for import in &path.body.imports {
                lookup.reach("body", first, &import_text(import));
                self.follow_import(lookup, locator, import, rest, 0);
            }
        } else if let Some(header) = outer.header.as_deref().filter(|header| header.names.contains(first)) {
            lookup.reach_declaration("header", first, &header.text);
        } else {
            self.resolve_name(lookup, locator, &path.segments, NameContext::Unqualified, 0);
        }
        for glob in &path.body.globs {
            lookup.reach("body", "*", &format!("{}::*", import_text(glob)));
            if glob.leading_colon {
                lookup.declarations += 1;
            } else {
                let mut segments = glob.segments.clone();
                segments.extend(path.segments.iter().cloned());
                lookup.glob_probes += 1;
                self.resolve_import_path(lookup, locator, &segments, 1);
                lookup.glob_probes -= 1;
            }
        }
    }

    fn resolve_qualified(&self, lookup: &mut Lookup<'a>, locator: &ModuleLocator, path: &[String], depth: usize) {
        if !lookup.step(depth) {
            return;
        }
        let Some((first, rest)) = path.split_first() else {
            lookup.hide(&module_label(locator), "", "path names a module");
            return;
        };
        let next = match first.as_str() {
            "self" => Some(locator.clone()),
            "super" => self.modules.parent(locator),
            "crate" => self.modules.crate_root(locator),
            _ => {
                self.resolve_name(lookup, locator, path, NameContext::Qualified, depth);
                return;
            }
        };
        match next {
            Some(module) => self.resolve_qualified(lookup, &module, rest, depth + 1),
            None => lookup.hide(&module_label(locator), first, "not a proven selected module"),
        }
    }

    /// Resolves the first segment of `path` among the declarations of the module at `locator`,
    /// then the rest of the path through what it declares. A lookup already made for the same
    /// module and path adds nothing.
    fn resolve_name(
        &self,
        lookup: &mut Lookup<'a>,
        locator: &ModuleLocator,
        path: &[String],
        context: NameContext,
        depth: usize,
    ) {
        if !lookup.step(depth) {
            return;
        }
        let label = module_label(locator);
        let (Some(scope), Some((name, rest))) = (self.modules.scopes.get(locator), path.split_first()) else {
            lookup.hide(&label, "", "module not selected");
            return;
        };
        if !lookup.visited.insert((locator.clone(), path.to_vec())) {
            return;
        }
        let declarations = scope.names.get(name);
        for declaration in declarations.into_iter().flatten() {
            lookup.reach(&label, name, declaration);
        }
        if !rest.is_empty() && scope.modules.contains_key(name) {
            match self.modules.child(locator, name) {
                Some(child) => self.resolve_qualified(lookup, &child, rest, depth + 1),
                None => lookup.hide(&label, name, "not a proven selected module"),
            }
            return;
        }
        if declarations.is_some_and(|declarations| declarations.len() == 1) {
            if rest.is_empty() && scope.functions.contains(name) {
                self.local(lookup, locator, &FunctionContainer::Module, name);
                return;
            }
            if let [method] = rest
                && scope.types.contains(name)
            {
                self.local(lookup, locator, &FunctionContainer::InherentImpl(name.clone()), method);
                return;
            }
            if let Some(imports) = scope.imports.get(name) {
                for import in imports {
                    self.follow_import(lookup, locator, import, rest, depth);
                }
                return;
            }
        }
        if declarations.is_some() {
            lookup.declarations += 1;
            return;
        }
        self.resolve_undeclared(lookup, locator, scope, path, context, depth);
    }

    /// A name the module at `locator` does not declare: the prelude (or an extern crate, or
    /// nothing after a module path), any of the module's wildcards, or what its glob imports
    /// bring in. After a module path, a name the path goes through that only a macro or an
    /// attributed item can supply (a generated module or type) cannot be proven, so it is
    /// hidden.
    fn resolve_undeclared(
        &self,
        lookup: &mut Lookup<'a>,
        locator: &ModuleLocator,
        scope: &ModuleScope,
        path: &[String],
        context: NameContext,
        depth: usize,
    ) {
        let label = module_label(locator);
        let name = path.first().map_or("", String::as_str);
        // Only a module path written in the call or an import is unprovable here; a glob probe
        // binds the module's wildcard texts below instead.
        if matches!(context, NameContext::Qualified)
            && path.len() > 1
            && scope.generated_items
            && lookup.glob_probes == 0
        {
            lookup.hide(&label, name, "may be generated by a macro");
        }
        lookup.reach_declaration(&label, name, match context {
            NameContext::Unqualified => "prelude",
            NameContext::Qualified => "undeclared",
            NameContext::Import => "extern",
        });
        for wildcard in &scope.wildcards {
            lookup.reach(&label, "*", wildcard);
        }
        // Every glob is followed: its path may start from the module, an extern crate, or a
        // name another glob brings in, and `resolve_import_path` tells them apart.
        for glob in scope.glob_imports.iter().filter(|glob| !glob.leading_colon) {
            let mut segments = glob.segments.clone();
            segments.extend(path.iter().cloned());
            lookup.glob_probes += 1;
            self.resolve_import_path(lookup, locator, &segments, depth + 1);
            lookup.glob_probes -= 1;
        }
    }

    /// Follows one import of a name, appending the rest of the path.
    fn follow_import(
        &self,
        lookup: &mut Lookup<'a>,
        locator: &ModuleLocator,
        import: &ImportPath,
        rest: &[String],
        depth: usize,
    ) {
        if import.leading_colon {
            let head = import.segments.first().map_or("", String::as_str);
            lookup.reach_declaration(&module_label(locator), head, "extern");
            return;
        }
        let mut segments = import.segments.clone();
        segments.extend(rest.iter().cloned());
        self.resolve_import_path(lookup, locator, &segments, depth + 1);
    }

    /// Resolves a `use` path: from a keyword, from a name the module declares, or else from an
    /// extern crate or a name the module's local glob imports bring in.
    fn resolve_import_path(&self, lookup: &mut Lookup<'a>, locator: &ModuleLocator, segments: &[String], depth: usize) {
        let Some(first) = segments.first() else {
            lookup.hide(&module_label(locator), "", "empty import");
            return;
        };
        let Some(scope) = self.modules.scopes.get(locator) else {
            lookup.hide(&module_label(locator), first, "module not selected");
            return;
        };
        if is_path_keyword(first) {
            self.resolve_qualified(lookup, locator, segments, depth);
        } else if scope.declares_path_head(first) {
            self.resolve_name(lookup, locator, segments, NameContext::Import, depth);
        } else if lookup.step(depth) && lookup.visited.insert((locator.clone(), segments.to_vec())) {
            self.resolve_undeclared(lookup, locator, scope, segments, NameContext::Import, depth);
        }
    }

    /// Whether an import's path starts from a declaration of the module rather than an extern
    /// crate or a glob import.
    fn is_local_import(&self, locator: &ModuleLocator, import: &ImportPath) -> bool {
        !import.leading_colon
            && import.segments.first().is_some_and(|head| {
                is_path_keyword(head)
                    || self.modules.scopes.get(locator).is_some_and(|scope| scope.declares_path_head(head))
            })
    }

    /// Renders `module=self::a; methods=prelude, ...` for a method call: every declaration that
    /// may bring the method's trait into scope (the `use` bindings of the enclosing blocks, then
    /// each `use` binding, trait item, and wildcard of the caller's module), with local imports
    /// followed to the declaration they name and local glob imports to the method sources of
    /// their module, plus the enclosing `impl` or trait header, whose bounds may supply methods.
    fn method_scope(&self, caller: &AddressedFunction, raw_call: &RawCall) -> CallScope<'a> {
        let draft = &caller.draft;
        let locator = (draft.path.clone(), draft.module.clone());
        let mut lookup = Lookup::default();
        for import in &raw_call.body_method_sources.imports {
            lookup.reach("body", "trait", &import_text(import));
            if !import.leading_colon {
                self.resolve_import_path(&mut lookup, &locator, &import.segments, 1);
            }
        }
        for glob in &raw_call.body_method_sources.globs {
            lookup.reach("body", "*", &format!("{}::*", import_text(glob)));
            if !glob.leading_colon {
                self.glob_method_sources(&mut lookup, &locator, glob, 1);
            }
        }
        let (methods, _) = self.module_methods(&locator);
        lookup.reached.extend(methods.reached.iter().cloned());
        lookup.hidden |= methods.hidden;
        let mut text = module_text(&draft.module);
        text.push_str("; methods=prelude, ");
        text.push_str(&lookup.text());
        if let Some(header) = &draft.outer_scope.header {
            text.push_str("; header=");
            text.push_str(&header.text);
        }
        if raw_call.in_body_module {
            text.push_str("; in a module declared in the body");
        }
        CallScope {
            text,
            local: None,
            hidden: lookup.hidden || raw_call.in_body_module,
        }
    }

    /// The declarations of the module at `locator` that may bring a trait into scope, resolved
    /// once per run: each `use` binding is followed with its own bounded lookup, each glob import
    /// to the method sources of its module. Returns the modules whose in-progress lookup a glob
    /// cycle cut short; a result cut short by another module is not kept, since that module's
    /// sources are missing from it.
    fn module_methods(&self, locator: &ModuleLocator) -> (Rc<ModuleMethods>, BTreeSet<ModuleLocator>) {
        if let Some(cached) = self.method_cache.borrow().get(locator) {
            return (Rc::clone(cached), BTreeSet::new());
        }
        if !self.methods_in_progress.borrow_mut().insert(locator.clone()) {
            return (Rc::default(), BTreeSet::from([locator.clone()]));
        }
        let label = module_label(locator);
        let mut methods = ModuleMethods::default();
        let mut cut = BTreeSet::new();
        if let Some(scope) = self.modules.scopes.get(locator) {
            for source in scope.method_sources.iter().chain(&scope.wildcards) {
                methods.reached.insert(format!("{label} trait: {source}"));
            }
            for import in scope.imports.values().flatten().filter(|import| !import.leading_colon) {
                let mut lookup = Lookup::default();
                self.resolve_import_path(&mut lookup, locator, &import.segments, 1);
                methods.absorb(&lookup);
            }
            for glob in scope.glob_imports.iter().filter(|glob| !glob.leading_colon) {
                let mut lookup = Lookup::default();
                if let Some(module) = self.resolve_module(&mut lookup, locator, &glob.segments, 1) {
                    let (inner, inner_cut) = self.module_methods(&module);
                    methods.reached.extend(inner.reached.iter().cloned());
                    methods.hidden |= inner.hidden;
                    cut.extend(inner_cut);
                }
                methods.absorb(&lookup);
            }
        } else {
            methods.hidden = true;
            methods.reached.insert(format!("{label} *: module not selected"));
        }
        self.methods_in_progress.borrow_mut().remove(locator);
        cut.remove(locator);
        let methods = Rc::new(methods);
        if cut.is_empty() {
            self.method_cache.borrow_mut().insert(locator.clone(), Rc::clone(&methods));
        }
        (methods, cut)
    }

    /// Includes the method sources of the module a body glob import names.
    fn glob_method_sources(&self, lookup: &mut Lookup<'a>, locator: &ModuleLocator, glob: &ImportPath, depth: usize) {
        if let Some(module) = self.resolve_module(lookup, locator, &glob.segments, depth) {
            let (methods, _) = self.module_methods(&module);
            lookup.reached.extend(methods.reached.iter().cloned());
            lookup.hidden |= methods.hidden;
        }
    }

    /// The selected module a path names, through keywords, module declarations, and imports. A
    /// path ending at a local type (an enum's variants) or at an extern crate names no selected
    /// module; a name the module's local glob imports may supply is hidden.
    fn resolve_module(
        &self,
        lookup: &mut Lookup<'a>,
        locator: &ModuleLocator,
        segments: &[String],
        depth: usize,
    ) -> Option<ModuleLocator> {
        if !lookup.step(depth) {
            return None;
        }
        let label = module_label(locator);
        let Some((first, rest)) = segments.split_first() else {
            return Some(locator.clone());
        };
        let next = match first.as_str() {
            "self" => Some(locator.clone()),
            "super" => self.modules.parent(locator),
            "crate" => self.modules.crate_root(locator),
            name => {
                let Some(scope) = self.modules.scopes.get(locator) else {
                    lookup.hide(&label, name, "module not selected");
                    return None;
                };
                if scope.modules.contains_key(name) {
                    self.modules.child(locator, name)
                } else if let Some([import]) = scope.imports.get(name).map(Vec::as_slice) {
                    if import.leading_colon || import.segments.first().is_some_and(|head| head == name) {
                        lookup.reach_declaration(&label, name, &import_text(import));
                        return None;
                    }
                    lookup.reach(&label, name, &import_text(import));
                    let mut target = import.segments.clone();
                    target.extend(rest.iter().cloned());
                    return self.resolve_module(lookup, locator, &target, depth + 1);
                } else if scope.types.contains(name) || scope.names.contains_key(name) {
                    lookup.reach_declaration(&label, name, "not a module");
                    return None;
                } else if scope.glob_imports.iter().any(|glob| self.is_local_import(locator, glob)) {
                    None
                } else {
                    lookup.reach_declaration(&label, name, "extern");
                    return None;
                }
            }
        };
        if let Some(module) = next {
            self.resolve_module(lookup, &module, rest, depth + 1)
        } else {
            lookup.hide(&label, first, "not a proven selected module");
            None
        }
    }

    /// The selected function `name` in `container` of the module at `locator`. Without a unique
    /// one, the owner's declaration (with its attributes, already reached) and, for a method, the
    /// module's wildcards bind the path: a derive, an attribute macro on an `impl`, or an item
    /// macro may generate the method.
    fn local(&self, lookup: &mut Lookup<'a>, locator: &ModuleLocator, container: &FunctionContainer, name: &str) {
        let candidates = self.functions.get(locator.0.as_str()).map_or(&[][..], Vec::as_slice);
        let mut matching = candidates.iter().filter(|function| {
            function.draft.module == locator.1 && function.draft.container == *container && function.draft.name == name
        });
        if let (Some(function), None) = (matching.next(), matching.next())
            && function.function_address != MISSING_FUNCTION_ADDRESS
        {
            lookup.locals.push(function);
            return;
        }
        let label = module_label(locator);
        if let FunctionContainer::InherentImpl(owner) = container {
            lookup.reach_declaration(&label, &format!("{owner}::{name}"), "no selected method");
            if let Some(scope) = self.modules.scopes.get(locator) {
                for wildcard in &scope.wildcards {
                    lookup.reach(&label, "*", wildcard);
                }
            }
        } else {
            lookup.reach_declaration(&label, name, "no selected function");
        }
    }
}

fn import_text(import: &ImportPath) -> String {
    format!("use {}{}", if import.leading_colon { "::" } else { "" }, import.segments.join("::"))
}

fn is_path_keyword(segment: &str) -> bool {
    matches!(segment, "crate" | "self" | "super")
}

/// The resolution scope of a call from a file-root function in `source_path` whose call path
/// starts with a name the file does not declare, for tests that build summaries directly.
#[cfg(test)]
pub(crate) fn prelude_call_scope(source_path: &str, shape: &str) -> String {
    let path = shape.split_once(':').map_or(shape, |(_, path)| path);
    let head = path.split("::").next().unwrap_or(path);
    format!("module=self; {path}={source_path} {head}: prelude")
}

fn item_attributes(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(node) => &node.attrs,
        syn::Item::Enum(node) => &node.attrs,
        syn::Item::ExternCrate(node) => &node.attrs,
        syn::Item::Fn(node) => &node.attrs,
        syn::Item::ForeignMod(node) => &node.attrs,
        syn::Item::Impl(node) => &node.attrs,
        syn::Item::Macro(node) => &node.attrs,
        syn::Item::Mod(node) => &node.attrs,
        syn::Item::Static(node) => &node.attrs,
        syn::Item::Struct(node) => &node.attrs,
        syn::Item::Trait(node) => &node.attrs,
        syn::Item::TraitAlias(node) => &node.attrs,
        syn::Item::Type(node) => &node.attrs,
        syn::Item::Union(node) => &node.attrs,
        syn::Item::Use(node) => &node.attrs,
        _ => &[],
    }
}

/// Collects the path each binding of a `use` tree imports, keyed by the name it binds, and the
/// module path of each glob.
fn collect_imports(
    tree: &syn::UseTree,
    leading_colon: bool,
    prefix: &mut Vec<String>,
    imports: &mut BTreeMap<String, Vec<ImportPath>>,
    glob_imports: &mut Vec<ImportPath>,
) {
    let import = |segments: &[String]| ImportPath {
        leading_colon,
        segments: segments.to_vec(),
    };
    match tree {
        syn::UseTree::Path(node) => {
            prefix.push(node.ident.to_string());
            collect_imports(&node.tree, leading_colon, prefix, imports, glob_imports);
            prefix.pop();
        }
        // `use a::b::{self}` binds `b` to `a::b`.
        syn::UseTree::Name(node) if node.ident == "self" => {
            if let Some(name) = prefix.last() {
                imports.entry(name.clone()).or_default().push(import(prefix));
            }
        }
        syn::UseTree::Name(node) => {
            prefix.push(node.ident.to_string());
            imports.entry(node.ident.to_string()).or_default().push(import(prefix));
            prefix.pop();
        }
        syn::UseTree::Rename(node) if node.ident == "self" => {
            imports.entry(node.rename.to_string()).or_default().push(import(prefix));
        }
        syn::UseTree::Rename(node) => {
            prefix.push(node.ident.to_string());
            imports.entry(node.rename.to_string()).or_default().push(import(prefix));
            prefix.pop();
        }
        syn::UseTree::Glob(_) => glob_imports.push(import(prefix)),
        syn::UseTree::Group(node) => {
            for item in &node.items {
                collect_imports(item, leading_colon, prefix, imports, glob_imports);
            }
        }
    }
}

impl ModuleScope {
    fn of_items(items: &[syn::Item]) -> Self {
        let mut scope = Self::default();
        for item in items {
            if let syn::Item::Macro(node) = item
                && let Some(ident) = &node.ident
                && node.mac.path.is_ident("macro_rules")
            {
                scope.macro_definitions.insert(ident.to_string(), node.mac.to_token_stream().to_string());
            }
        }
        for item in items {
            scope.declare_item(item);
            scope.record_resolvable(item);
        }
        scope
    }

    /// A `macro_rules!` definition binds its name by its full tokens. An item-position macro
    /// invocation may expand to items of any name, so it is a wildcard bound by its tokens and,
    /// for a macro this module defines, by that definition's tokens too.
    fn declare_macro(&mut self, node: &syn::ItemMacro) {
        let tokens = node.mac.to_token_stream().to_string();
        match &node.ident {
            Some(ident) if node.mac.path.is_ident("macro_rules") => self.declare(ident, &node.attrs, tokens),
            _ => {
                self.generated_items = true;
                let definition =
                    node.mac.path.get_ident().and_then(|name| self.macro_definitions.get(&name.to_string()));
                let declaration = match definition {
                    Some(definition) => format!("{tokens} defined by {definition}"),
                    None => tokens,
                };
                self.declare_wildcard(&node.attrs, declaration);
            }
        }
    }

    /// Records the declarations a path can be followed through: imports, module declarations,
    /// free functions, and types that may own inherent methods.
    fn record_resolvable(&mut self, item: &syn::Item) {
        if let Some(ident) = item_ident(item) {
            self.item_names.insert(ident.to_string());
        }
        match item {
            syn::Item::Use(node) => collect_imports(
                &node.tree,
                node.leading_colon.is_some(),
                &mut Vec::new(),
                &mut self.imports,
                &mut self.glob_imports,
            ),
            syn::Item::Mod(node) => {
                let declaration = if has_opaque_attribute(&node.attrs) {
                    ModuleDeclaration::Opaque
                } else if node.content.is_some() {
                    ModuleDeclaration::Inline
                } else {
                    ModuleDeclaration::File
                };
                self.modules.insert(node.ident.to_string(), declaration);
            }
            syn::Item::Fn(node) => {
                self.functions.insert(node.sig.ident.to_string());
            }
            syn::Item::Struct(syn::ItemStruct { ident, .. })
            | syn::Item::Enum(syn::ItemEnum { ident, .. })
            | syn::Item::Union(syn::ItemUnion { ident, .. }) => {
                self.types.insert(ident.to_string());
            }
            _ => {}
        }
    }

    fn declare(&mut self, name: &syn::Ident, attrs: &[syn::Attribute], declaration: String) {
        self.names.entry(name.to_string()).or_default().insert(with_attributes(attrs, declaration));
    }

    fn declare_wildcard(&mut self, attrs: &[syn::Attribute], declaration: String) {
        self.wildcards.insert(with_attributes(attrs, declaration));
    }

    /// Records what `item` binds. Type-like items and functions bind by kind and name: the callee
    /// a path through them reaches is bound by its own function address. Aliases, constants,
    /// statics, and extern crates bind by their full tokens, since those decide what a path
    /// through them names. An item carrying an attribute macro or `derive` (any non-inert
    /// attribute other than `cfg`, which only keeps or drops the item) may also expand to any
    /// sibling item, so it is a wildcard bound by the hash of its full tokens.
    fn declare_item(&mut self, item: &syn::Item) {
        let attrs = item_attributes(item);
        if attrs.iter().any(|attribute| !is_inert_attribute(attribute) && !attribute.path().is_ident("cfg")) {
            self.generated_items = true;
            self.declare_wildcard(attrs, format!("item {}", full_hash(&item.to_token_stream().to_string())));
        }
        match item {
            syn::Item::Use(node) => {
                let leading_colon = node.leading_colon.is_some();
                self.declare_use_tree(&node.tree, &node.attrs, leading_colon, &mut Vec::new());
            }
            syn::Item::Fn(node) => self.declare(&node.sig.ident, &node.attrs, format!("fn {}", node.sig.ident)),
            syn::Item::Mod(node) => self.declare(&node.ident, &node.attrs, format!("mod {}", node.ident)),
            syn::Item::Struct(node) => self.declare(&node.ident, &node.attrs, format!("struct {}", node.ident)),
            syn::Item::Enum(node) => self.declare(&node.ident, &node.attrs, format!("enum {}", node.ident)),
            syn::Item::Union(node) => self.declare(&node.ident, &node.attrs, format!("union {}", node.ident)),
            syn::Item::Trait(node) => {
                let declaration = format!("trait {}", node.ident);
                self.method_sources.insert(with_attributes(&node.attrs, declaration.clone()));
                self.declare(&node.ident, &node.attrs, declaration);
            }
            syn::Item::TraitAlias(node) => {
                let syn::ItemTraitAlias {
                    trait_token,
                    ident,
                    generics,
                    eq_token,
                    bounds,
                    ..
                } = node;
                let where_clause = &generics.where_clause;
                let tokens = quote::quote!(#trait_token #ident #generics #eq_token #bounds #where_clause);
                self.declare(ident, &node.attrs, tokens.to_string());
            }
            syn::Item::Type(node) => {
                let syn::ItemType {
                    type_token,
                    ident,
                    generics,
                    eq_token,
                    ty,
                    ..
                } = node;
                let where_clause = &generics.where_clause;
                let tokens = quote::quote!(#type_token #ident #generics #where_clause #eq_token #ty);
                self.declare(ident, &node.attrs, tokens.to_string());
            }
            syn::Item::Const(node) => {
                let syn::ItemConst {
                    const_token,
                    ident,
                    generics,
                    colon_token,
                    ty,
                    eq_token,
                    expr,
                    ..
                } = node;
                let tokens = quote::quote!(#const_token #ident #generics #colon_token #ty #eq_token #expr);
                self.declare(ident, &node.attrs, tokens.to_string());
            }
            syn::Item::Static(node) => {
                let syn::ItemStatic {
                    static_token,
                    mutability,
                    ident,
                    colon_token,
                    ty,
                    eq_token,
                    expr,
                    ..
                } = node;
                let tokens = quote::quote!(#static_token #mutability #ident #colon_token #ty #eq_token #expr);
                self.declare(ident, &node.attrs, tokens.to_string());
            }
            syn::Item::ExternCrate(node) => {
                let declaration = match &node.rename {
                    Some((_, rename)) => format!("extern crate {} as {rename}", node.ident),
                    None => format!("extern crate {}", node.ident),
                };
                self.declare(node.rename.as_ref().map_or(&node.ident, |(_, rename)| rename), &node.attrs, declaration);
            }
            syn::Item::ForeignMod(node) => self.declare_foreign_items(&node.items),
            syn::Item::Macro(node) => self.declare_macro(node),
            syn::Item::Impl(_) => {}
            other => {
                self.generated_items = true;
                self.declare_wildcard(&[], other.to_token_stream().to_string());
            }
        }
    }

    fn declare_foreign_items(&mut self, items: &[syn::ForeignItem]) {
        for foreign in items {
            match foreign {
                syn::ForeignItem::Fn(item) => {
                    self.declare(&item.sig.ident, &item.attrs, format!("extern fn {}", item.sig.ident));
                }
                syn::ForeignItem::Static(item) => {
                    self.declare(&item.ident, &item.attrs, format!("extern static {}", item.ident));
                }
                syn::ForeignItem::Type(item) => {
                    self.declare(&item.ident, &item.attrs, format!("extern type {}", item.ident));
                }
                other => {
                    self.generated_items = true;
                    self.declare_wildcard(&[], other.to_token_stream().to_string());
                }
            }
        }
    }

    /// Declares a `use` binding, which may also bring a trait's methods into scope.
    fn declare_import(&mut self, name: &syn::Ident, attrs: &[syn::Attribute], declaration: String) {
        let binding = with_attributes(attrs, declaration);
        self.method_sources.insert(binding.clone());
        self.names.entry(name.to_string()).or_default().insert(binding);
    }

    /// Records the names a `use` tree binds, each as the full path it imports, so a grouped
    /// import binds each name exactly as a separate `use` would.
    fn declare_use_tree<'a>(
        &mut self,
        tree: &'a syn::UseTree,
        attrs: &[syn::Attribute],
        leading_colon: bool,
        segments: &mut Vec<&'a syn::Ident>,
    ) {
        match tree {
            syn::UseTree::Path(node) => {
                segments.push(&node.ident);
                self.declare_use_tree(&node.tree, attrs, leading_colon, segments);
                segments.pop();
            }
            // `use a::b::{self}` binds `b` to `use a::b`.
            syn::UseTree::Name(node) if node.ident == "self" => {
                if let Some(&name) = segments.last() {
                    self.declare_import(name, attrs, use_text(leading_colon, segments, None));
                }
            }
            syn::UseTree::Name(node) => {
                segments.push(&node.ident);
                self.declare_import(&node.ident, attrs, use_text(leading_colon, segments, None));
                segments.pop();
            }
            syn::UseTree::Rename(node) if node.ident == "self" => {
                self.declare_import(&node.rename, attrs, use_text(leading_colon, segments, Some(&node.rename)));
            }
            syn::UseTree::Rename(node) => {
                segments.push(&node.ident);
                self.declare_import(&node.rename, attrs, use_text(leading_colon, segments, Some(&node.rename)));
                segments.pop();
            }
            syn::UseTree::Glob(_) => {
                let mut declaration = use_text(leading_colon, segments, None);
                declaration.push_str(if segments.is_empty() { "*" } else { "::*" });
                self.declare_wildcard(attrs, declaration);
            }
            syn::UseTree::Group(node) => {
                for item in &node.items {
                    self.declare_use_tree(item, attrs, leading_colon, segments);
                }
            }
        }
    }
}

fn use_text(leading_colon: bool, segments: &[&syn::Ident], rename: Option<&syn::Ident>) -> String {
    let mut text = String::from(if leading_colon { "use ::" } else { "use " });
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            text.push_str("::");
        }
        text.push_str(&segment.to_string());
    }
    if let Some(rename) = rename {
        text.push_str(" as ");
        text.push_str(&rename.to_string());
    }
    text
}

/// Prefixes a declaration with its attributes other than documentation, so that `cfg` on an
/// import, for example, stays part of what the name binds.
fn with_attributes(attrs: &[syn::Attribute], declaration: String) -> String {
    let mut binding = attrs.iter().filter(|attribute| !attribute.path().is_ident("doc")).fold(
        String::new(),
        |mut text, attribute| {
            text.push_str(&attribute.to_token_stream().to_string());
            text.push(' ');
            text
        },
    );
    if binding.is_empty() {
        return declaration;
    }
    binding.push_str(&declaration);
    binding
}

impl EnclosingHeader {
    fn of_impl(node: &syn::ItemImpl) -> Self {
        let syn::ItemImpl {
            defaultness,
            unsafety,
            impl_token,
            generics,
            trait_,
            self_ty,
            ..
        } = node;
        let trait_tokens = trait_.as_ref().map(|(bang, path, for_token)| quote::quote!(#bang #path #for_token));
        let where_clause = &generics.where_clause;
        Self {
            names: header_names(generics),
            text: quote::quote!(#defaultness #unsafety #impl_token #generics #trait_tokens #self_ty #where_clause)
                .to_string(),
        }
    }

    fn of_trait(node: &syn::ItemTrait) -> Self {
        let syn::ItemTrait {
            unsafety,
            auto_token,
            trait_token,
            ident,
            generics,
            colon_token,
            supertraits,
            ..
        } = node;
        let where_clause = &generics.where_clause;
        Self {
            names: header_names(generics),
            text: quote::quote!(#unsafety #auto_token #trait_token #ident #generics #colon_token #supertraits #where_clause)
                .to_string(),
        }
    }
}

/// `Self` and the type and const parameters an `impl` or trait header brings into scope.
fn header_names(generics: &syn::Generics) -> BTreeSet<String> {
    let parameters = generics.params.iter().filter_map(|parameter| match parameter {
        syn::GenericParam::Type(parameter) => Some(parameter.ident.to_string()),
        syn::GenericParam::Const(parameter) => Some(parameter.ident.to_string()),
        syn::GenericParam::Lifetime(_) => None,
    });
    parameters.chain(std::iter::once("Self".to_owned())).collect()
}

/// The identifiers of the first `count` segments of `path`, or `None` for an empty prefix or a
/// leading-`::` path, which names an extern crate whose identity the provider summary binds.
fn path_segments(path: &syn::Path, count: usize) -> Option<Vec<String>> {
    if path.leading_colon.is_some() || count == 0 {
        return None;
    }
    Some(path.segments.iter().take(count).map(|segment| segment.ident.to_string()).collect())
}

impl CallerBindings {
    fn collect(function: &FunctionDraft) -> Self {
        let mut bindings = Self::default();
        bindings.visit_generics(&function.enclosing_generics);
        bindings.visit_signature(&function.signature);
        bindings.visit_block(&function.block);
        bindings
    }

    fn bind_name(&mut self, ident: &syn::Ident) {
        self.names.insert(ident.to_string());
    }

    /// Whether `path` may name a selected module item: it has no leading `::`, the caller holds no
    /// glob `use`, and its first segment is not a name bound inside the caller.
    fn admits_local_path(&self, path: &syn::Path) -> bool {
        path.leading_colon.is_none()
            && !self.glob_import
            && path.segments.first().is_none_or(|segment| !self.names.contains(&segment.ident.to_string()))
    }
}

impl<'ast> Visit<'ast> for CallerBindings {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.bind_name(&node.ident);
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_type_param(&mut self, node: &'ast syn::TypeParam) {
        self.bind_name(&node.ident);
        syn::visit::visit_type_param(self, node);
    }

    fn visit_const_param(&mut self, node: &'ast syn::ConstParam) {
        self.bind_name(&node.ident);
        syn::visit::visit_const_param(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        bind_use_tree(&node.tree, None, &mut self.names, &mut self.glob_import);
    }

    fn visit_item(&mut self, node: &'ast syn::Item) {
        if let Some(ident) = item_ident(node) {
            self.bind_name(ident);
        }
        syn::visit::visit_item(self, node);
    }

    fn visit_foreign_item(&mut self, node: &'ast syn::ForeignItem) {
        if let Some(ident) = foreign_item_ident(node) {
            self.bind_name(ident);
        }
        syn::visit::visit_foreign_item(self, node);
    }
}

/// Adds the names a `use` tree brings into scope to `names`, and records whether it holds a glob.
fn bind_use_tree(
    tree: &syn::UseTree,
    parent: Option<&syn::Ident>,
    names: &mut BTreeSet<String>,
    glob_import: &mut bool,
) {
    match tree {
        syn::UseTree::Path(path) => bind_use_tree(&path.tree, Some(&path.ident), names, glob_import),
        syn::UseTree::Name(name) if name.ident == "self" => {
            if let Some(parent) = parent {
                names.insert(parent.to_string());
            }
        }
        syn::UseTree::Name(name) => {
            names.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            names.insert(rename.rename.to_string());
        }
        syn::UseTree::Glob(_) => *glob_import = true,
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                bind_use_tree(tree, parent, names, glob_import);
            }
        }
    }
}

fn item_ident(item: &syn::Item) -> Option<&syn::Ident> {
    match item {
        syn::Item::Const(item) => Some(&item.ident),
        syn::Item::Enum(item) => Some(&item.ident),
        syn::Item::ExternCrate(item) => Some(item.rename.as_ref().map_or(&item.ident, |(_as, rename)| rename)),
        syn::Item::Fn(item) => Some(&item.sig.ident),
        syn::Item::Macro(item) => item.ident.as_ref(),
        syn::Item::Mod(item) => Some(&item.ident),
        syn::Item::Static(item) => Some(&item.ident),
        syn::Item::Struct(item) => Some(&item.ident),
        syn::Item::Trait(item) => Some(&item.ident),
        syn::Item::TraitAlias(item) => Some(&item.ident),
        syn::Item::Type(item) => Some(&item.ident),
        syn::Item::Union(item) => Some(&item.ident),
        _ => None,
    }
}

fn foreign_item_ident(item: &syn::ForeignItem) -> Option<&syn::Ident> {
    match item {
        syn::ForeignItem::Fn(item) => Some(&item.sig.ident),
        syn::ForeignItem::Static(item) => Some(&item.ident),
        syn::ForeignItem::Type(item) => Some(&item.ident),
        _ => None,
    }
}

impl ValueScope {
    /// Parameters and const generic parameters, visible in the whole function body.
    fn of_signature(enclosing_generics: &syn::Generics, signature: &syn::Signature) -> Self {
        let mut scope = Self::default();
        for param in enclosing_generics.const_params().chain(signature.generics.const_params()) {
            scope.locals.insert(param.ident.to_string());
        }
        for input in &signature.inputs {
            if let syn::FnArg::Typed(typed) = input {
                scope.bind_pattern(&typed.pat);
            }
        }
        scope
    }

    /// Const generic parameters of an `impl` or trait, visible in all of its items.
    fn of_generics(generics: &syn::Generics) -> Self {
        let mut scope = Self::default();
        scope.locals.extend(generics.const_params().map(|param| param.ident.to_string()));
        scope
    }

    /// Items and `use` names declared directly in `block`, visible throughout it.
    fn of_block(block: &syn::Block) -> Self {
        let mut scope = Self::default();
        for statement in &block.stmts {
            let syn::Stmt::Item(item) = statement else {
                continue;
            };
            match item {
                syn::Item::Use(item) => bind_use_tree(&item.tree, None, &mut scope.items, &mut scope.glob_import),
                syn::Item::ForeignMod(module) => {
                    scope.items.extend(module.items.iter().filter_map(foreign_item_ident).map(ToString::to_string));
                }
                item => scope.items.extend(item_ident(item).map(ToString::to_string)),
            }
        }
        scope
    }

    fn bind_pattern(&mut self, pattern: &syn::Pat) {
        PatternBindings(&mut self.locals).visit_pat(pattern);
    }
}

impl<'ast> Visit<'ast> for PatternBindings<'_> {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.0.insert(node.ident.to_string());
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_expr(&mut self, _node: &'ast syn::Expr) {}

    fn visit_expr_const(&mut self, _node: &'ast syn::ExprConst) {}

    fn visit_type(&mut self, _node: &'ast syn::Type) {}
}

impl<'a> BodyScanner<'a> {
    fn new(foreign_functions: &'a BTreeSet<String>, bindings: &'a CallerBindings, function_scope: ValueScope) -> Self {
        Self {
            foreign_functions,
            bindings,
            scopes: vec![function_scope],
            compile_time_depth: 0,
            direct_effects: BTreeSet::new(),
            calls: Vec::new(),
            block_imports: Vec::new(),
            body_module_depth: 0,
        }
    }

    fn record_call(
        &mut self,
        shape: String,
        kind: RawCallKind,
        effects: BTreeSet<Effect>,
        locally_resolvable: bool,
        referenced_paths: Vec<Vec<String>>,
    ) {
        self.direct_effects.extend(effects.iter().copied());
        let referenced_paths = referenced_paths
            .into_iter()
            .map(|segments| ReferencedPath {
                body: segments.first().map(|head| self.body_lookup(head)).unwrap_or_default(),
                segments,
            })
            .collect();
        let mut body_method_sources = BodyLookup::default();
        if kind == RawCallKind::Method {
            for block in &self.block_imports {
                body_method_sources.imports.extend(block.imports.values().flatten().cloned());
                body_method_sources.globs.extend(block.glob_imports.iter().cloned());
            }
        }
        self.calls.push(RawCall {
            shape,
            kind,
            effects,
            locally_resolvable,
            referenced_paths,
            body_method_sources,
            in_body_module: self.body_module_depth > 0,
        });
    }

    /// What the blocks enclosing the scan position declare for `head`, innermost first: an
    /// explicit import or item in a block shadows the blocks around it and the module, while a
    /// glob import only may.
    fn body_lookup(&self, head: &str) -> BodyLookup {
        let mut lookup = BodyLookup::default();
        for block in self.block_imports.iter().rev() {
            if let Some(imports) = block.imports.get(head) {
                lookup.imports.clone_from(imports);
                return lookup;
            }
            if block.items.contains(head) {
                lookup.item = true;
                return lookup;
            }
            lookup.globs.extend(block.glob_imports.iter().cloned());
        }
        lookup
    }

    fn record_path(&mut self, node: &syn::ExprPath, kind: RawCallKind) {
        let (shape_path, effect_path) = expr_path_text(node);
        let effects = classify_path_call(&effect_path, self.foreign_functions).unwrap_or_default();
        let locally_resolvable = node.qself.is_none() && self.bindings.admits_local_path(&node.path);
        let mut referenced_paths = Vec::new();
        if let Some(qself) = &node.qself {
            if let syn::Type::Path(self_type) = qself.ty.as_ref()
                && self_type.qself.is_none()
            {
                referenced_paths.extend(path_segments(&self_type.path, self_type.path.segments.len()));
            }
            referenced_paths.extend(path_segments(&node.path, qself.position));
        } else {
            referenced_paths.extend(path_segments(&node.path, node.path.segments.len()));
        }
        let shape = format!("{}{shape_path}", kind.shape_prefix());
        self.record_call(shape, kind, effects, locally_resolvable, referenced_paths);
    }

    /// Whether `path` reads `self` or a local value in lexical scope at the current scan position.
    /// The innermost scope that declares the name decides, so a block item or glob `use` shadows an
    /// outer local.
    fn is_local_value(&self, path: &syn::ExprPath) -> bool {
        let Some(ident) = path.path.get_ident().filter(|_| path.qself.is_none()) else {
            return false;
        };
        if ident == "self" {
            return true;
        }
        let name = ident.to_string();
        for scope in self.scopes.iter().rev() {
            if scope.locals.contains(&name) {
                return true;
            }
            if scope.glob_import || scope.items.contains(&name) {
                return false;
            }
        }
        false
    }

    /// Makes the pattern's bindings visible for the rest of the innermost scope.
    fn bind_pattern(&mut self, pattern: &syn::Pat) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.bind_pattern(pattern);
        }
    }

    fn in_scope(&mut self, scope: ValueScope, visit: impl FnOnce(&mut Self)) {
        self.scopes.push(scope);
        visit(self);
        self.scopes.pop();
    }

    /// Visits a position that is evaluated at compile time; calls inside it are still recorded.
    fn at_compile_time(&mut self, visit: impl FnOnce(&mut Self)) {
        self.compile_time_depth += 1;
        visit(self);
        self.compile_time_depth -= 1;
    }
}

impl<'ast> Visit<'ast> for BodyScanner<'_> {
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if !is_inert_attribute(node) {
            self.direct_effects.insert(Effect::Unknown);
        }
        syn::visit::visit_attribute(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        match node.func.as_ref() {
            syn::Expr::Path(callee) => {
                self.record_path(callee, RawCallKind::Function);
                // Descend into the callee's type arguments without recording it again as a value.
                syn::visit::visit_expr_path(self, callee);
            }
            callee => {
                self.record_call(
                    format!("{}<opaque-expression>", RawCallKind::Function.shape_prefix()),
                    RawCallKind::Function,
                    BTreeSet::from([Effect::Unknown]),
                    false,
                    Vec::new(),
                );
                self.visit_expr(callee);
            }
        }
        for argument in &node.args {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if self.compile_time_depth == 0 && !self.is_local_value(node) {
            self.record_path(node, RawCallKind::Value);
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        self.block_imports.push(BlockImports::of_block(node));
        self.in_scope(ValueScope::of_block(node), |scanner| syn::visit::visit_block(scanner, node));
        self.block_imports.pop();
    }

    // Keeps syn's order (pattern with its type annotation, then initializer and `else`) so call
    // occurrences stay stable. Pattern visits record no value paths, so binding after the
    // initializer and `else` still keeps a `let` binding out of them until the next statement.
    fn visit_local(&mut self, node: &'ast syn::Local) {
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_pat(&node.pat);
        if let Some(init) = &node.init {
            self.visit_local_init(init);
        }
        self.bind_pattern(&node.pat);
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        let mut scope = ValueScope::default();
        scope.bind_pattern(&node.pat);
        self.in_scope(scope, |scanner| syn::visit::visit_arm(scanner, node));
    }

    // `if let` bindings are visible in later conditions of a chain and the `then` block, not `else`.
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        self.in_scope(ValueScope::default(), |scanner| {
            scanner.visit_expr(&node.cond);
            scanner.visit_block(&node.then_branch);
        });
        if let Some((_else, branch)) = &node.else_branch {
            self.visit_expr(branch);
        }
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        self.in_scope(ValueScope::default(), |scanner| {
            scanner.visit_expr(&node.cond);
            scanner.visit_block(&node.body);
        });
    }

    fn visit_expr_let(&mut self, node: &'ast syn::ExprLet) {
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_pat(&node.pat);
        self.visit_expr(&node.expr);
        self.bind_pattern(&node.pat);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_pat(&node.pat);
        self.visit_expr(&node.expr);
        let mut scope = ValueScope::default();
        scope.bind_pattern(&node.pat);
        self.in_scope(scope, |scanner| scanner.visit_block(&node.body));
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        let mut scope = ValueScope::default();
        for input in &node.inputs {
            scope.bind_pattern(input);
        }
        self.in_scope(scope, |scanner| syn::visit::visit_expr_closure(scanner, node));
    }

    // A nested item does not see the enclosing function's locals.
    fn visit_item(&mut self, node: &'ast syn::Item) {
        let enclosing = std::mem::take(&mut self.scopes);
        syn::visit::visit_item(self, node);
        self.scopes = enclosing;
    }

    // A module item in the body starts a new scope that neither the enclosing blocks nor the
    // caller's module provides; calls in it are marked so no summary stands in for them.
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let enclosing = std::mem::take(&mut self.block_imports);
        self.body_module_depth += 1;
        syn::visit::visit_item_mod(self, node);
        self.body_module_depth -= 1;
        self.block_imports = enclosing;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let scope = ValueScope::of_signature(&syn::Generics::default(), &node.sig);
        self.in_scope(scope, |scanner| syn::visit::visit_item_fn(scanner, node));
    }

    // A nested `impl` or trait's const generic parameters are in scope in all of its items.
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let scope = ValueScope::of_generics(&node.generics);
        self.in_scope(scope, |scanner| syn::visit::visit_item_impl(scanner, node));
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let scope = ValueScope::of_generics(&node.generics);
        self.in_scope(scope, |scanner| syn::visit::visit_item_trait(scanner, node));
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let scope = ValueScope::of_signature(&syn::Generics::default(), &node.sig);
        self.in_scope(scope, |scanner| syn::visit::visit_impl_item_fn(scanner, node));
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let scope = ValueScope::of_signature(&syn::Generics::default(), &node.sig);
        self.in_scope(scope, |scanner| syn::visit::visit_trait_item_fn(scanner, node));
    }

    fn visit_pat(&mut self, node: &'ast syn::Pat) {
        self.at_compile_time(|scanner| syn::visit::visit_pat(scanner, node));
    }

    fn visit_type(&mut self, node: &'ast syn::Type) {
        self.at_compile_time(|scanner| syn::visit::visit_type(scanner, node));
    }

    fn visit_expr_repeat(&mut self, node: &'ast syn::ExprRepeat) {
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_expr(&node.expr);
        self.at_compile_time(|scanner| scanner.visit_expr(&node.len));
    }

    fn visit_generic_argument(&mut self, node: &'ast syn::GenericArgument) {
        if matches!(node, syn::GenericArgument::Const(_) | syn::GenericArgument::AssocConst(_)) {
            self.at_compile_time(|scanner| syn::visit::visit_generic_argument(scanner, node));
        } else {
            syn::visit::visit_generic_argument(self, node);
        }
    }

    fn visit_const_param(&mut self, node: &'ast syn::ConstParam) {
        self.at_compile_time(|scanner| syn::visit::visit_const_param(scanner, node));
    }

    // A variant's fields are types and its discriminant is a constant expression.
    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        self.at_compile_time(|scanner| syn::visit::visit_variant(scanner, node));
    }

    // `const` and `static` initializers, including associated consts, are evaluated at compile time.
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.at_compile_time(|scanner| syn::visit::visit_item_const(scanner, node));
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        self.at_compile_time(|scanner| syn::visit::visit_item_static(scanner, node));
    }

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        self.at_compile_time(|scanner| syn::visit::visit_impl_item_const(scanner, node));
    }

    fn visit_trait_item_const(&mut self, node: &'ast syn::TraitItemConst) {
        self.at_compile_time(|scanner| syn::visit::visit_trait_item_const(scanner, node));
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        let receiver = match node.receiver.as_ref() {
            syn::Expr::Path(path) if path.path.is_ident("self") => "self",
            _ => "value",
        };
        let effects = classify_method_call(&method);
        self.record_call(
            format!("{}{receiver}:{method}", RawCallKind::Method.shape_prefix()),
            RawCallKind::Method,
            effects,
            true,
            Vec::new(),
        );
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let path = path_text(&node.path);
        let mut effects = classify_macro(&path).unwrap_or_default();
        effects.insert(Effect::Unknown);
        let shape = format!("{}{path}", RawCallKind::Macro.shape_prefix());
        self.record_call(shape, RawCallKind::Macro, effects, false, Vec::new());
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
                | syn::BinOp::AddAssign(_)
                | syn::BinOp::SubAssign(_)
                | syn::BinOp::MulAssign(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
                | syn::BinOp::ShlAssign(_)
                | syn::BinOp::ShrAssign(_)
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
            diagnostics.push(format!("{DIAGNOSTIC_CATALOG}: `{DETERMINISTIC_CORE_PROFILE}` must allow only `alloc`"));
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
    let (drafts, modules) = parse_function_drafts(&input.sources, &mut diagnostics);
    let addressed = address_functions(drafts, &input.function_addresses, &roles, &mut diagnostics);
    let mut nodes = build_nodes(&addressed, &modules, &input, &mut diagnostics);
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

/// Reads only the `schema_version` of stored receipt bytes and reports it when it is not
/// [`RECEIPT_SCHEMA_VERSION`], so a receipt written under another schema (whose body this version
/// cannot decode) is refused as unsupported rather than as a missing or unknown field. Bytes that
/// are not a JSON object return `None` and are left to the full parse to report as malformed.
pub fn unsupported_receipt_schema(bytes: &[u8]) -> Option<String> {
    #[derive(Deserialize)]
    struct SchemaVersion {
        schema_version: Option<serde_json::Value>,
    }
    match serde_json::from_slice::<SchemaVersion>(bytes).ok()?.schema_version {
        Some(serde_json::Value::String(version)) if version == RECEIPT_SCHEMA_VERSION => None,
        Some(serde_json::Value::String(version)) => Some(unsupported_receipt_schema_diagnostic(&version)),
        Some(other) => Some(unsupported_receipt_schema_diagnostic(&other.to_string())),
        None => Some(unsupported_receipt_schema_diagnostic("<missing>")),
    }
}

fn unsupported_receipt_schema_diagnostic(version: &str) -> String {
    format!("unsupported effect-closure receipt schema `{version}`; expected `{RECEIPT_SCHEMA_VERSION}`")
}

pub fn verify_receipt_identity(receipt: &EffectClosureReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
        diagnostics.push(unsupported_receipt_schema_diagnostic(&receipt.schema_version));
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
    if receipt.status == AdmissionStatus::Admitted {
        diagnostics.extend(validate_admitted_closure(receipt));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

// Receipt structure validation is a flat checklist of independent field checks that reads best in
// one place.
#[allow(clippy::too_many_lines)]
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
                || call.scope.trim().is_empty()
                || call.call_site_id
                    != call_site_identity(&node.function_address, &node.path, &call.scope, &call.shape, call.occurrence)
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

/// Re-derives the admission facts an `admitted` status asserts from the receipt's own nodes, so a
/// denied receipt cannot be relabelled admitted by dropping its diagnostics and recomputing the
/// unkeyed identity.
fn validate_admitted_closure(receipt: &EffectClosureReceipt) -> Vec<String> {
    // Closure v1 only admits `deterministic-core`, which `validate_catalog` pins to `alloc` alone.
    let allowed_effects = &EFFECT_ORDER[..DETERMINISTIC_ALLOWED_EFFECT_COUNT];
    let closures = receipt
        .nodes
        .iter()
        .map(|node| (node.function_address.as_str(), node.closure_effects.as_slice()))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    for node in &receipt.nodes {
        for effect in node.closure_effects.iter().filter(|effect| !allowed_effects.contains(effect)) {
            diagnostics.push(format!(
                "admitted function `{}::{}` closure contains denied effect `{}`",
                node.path,
                node.name,
                effect.as_str()
            ));
        }
        for call in &node.calls {
            if call.resolution == CallResolution::Unknown {
                diagnostics.push(format!("admitted receipt has unknown call `{}`", call.call_site_id));
            }
            if call.effects.iter().any(|effect| !node.direct_effects.contains(effect)) {
                diagnostics.push(format!(
                    "effect-closure node `{}::{}` direct effects do not contain every call effect of `{}`",
                    node.path, node.name, call.call_site_id
                ));
            }
            let callee_closure = match (&call.resolution, call.callee_function_address.as_deref()) {
                (CallResolution::Local, Some(address)) => closures.get(address).copied().unwrap_or_default(),
                _ => &[],
            };
            if callee_closure.iter().any(|effect| !node.closure_effects.contains(effect)) {
                diagnostics.push(format!(
                    "effect-closure node `{}::{}` closure does not contain local callee closure of `{}`",
                    node.path, node.name, call.call_site_id
                ));
            }
        }
    }
    diagnostics
}

/// Checks that an `admitted` receipt matches the bound function-address records and workspace
/// policy, so a denied receipt cannot be relabelled admitted by dropping the address, role, or
/// summary diagnostics that denied it. Reviewed summaries count only with evidence bound to the
/// receipt's recorded Cargo.lock or provider identity, as at admission. Denied receipts make no
/// admission claim and are not checked here.
pub fn validate_admitted_bindings(
    receipt: &EffectClosureReceipt,
    records: &[FunctionAddressRecord],
    policy: &WorkspacePolicy,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.status != AdmissionStatus::Admitted {
        return diagnostics;
    }
    let node_keys = receipt
        .nodes
        .iter()
        .map(|node| (node.path.as_str(), node.name.as_str(), node.function_address.as_str()))
        .collect::<BTreeSet<_>>();
    let record_keys = records
        .iter()
        .map(|record| (record.path.as_str(), record.name.as_str(), record.function_address.as_str()))
        .collect::<BTreeSet<_>>();
    if node_keys != record_keys || record_keys.len() != records.len() {
        diagnostics.push("admitted receipt nodes do not exactly match the bound function-address records".to_owned());
    }
    let source_identities = receipt
        .sources
        .iter()
        .map(|source| (source.path.as_str(), source.source_identity.as_str()))
        .collect::<BTreeMap<_, _>>();
    for record in records {
        if source_identities.get(record.path.as_str()) != Some(&record.source_identity.as_str()) {
            diagnostics.push(format!(
                "function-address record `{}::{}` source identity does not match the admitted receipt source",
                record.path, record.name
            ));
        }
    }
    validate_admitted_roles(receipt, policy, &mut diagnostics);

    let summaries = policy
        .call_summaries
        .iter()
        .map(|summary| (summary.call_site_id.as_str(), summary))
        .collect::<BTreeMap<_, _>>();
    let mut used_summaries = BTreeSet::new();
    let reviewed_calls = receipt
        .nodes
        .iter()
        .flat_map(|node| &node.calls)
        .filter(|call| call.resolution == CallResolution::ReviewedSummary);
    for call in reviewed_calls {
        used_summaries.insert(call.call_site_id.as_str());
        let summary = summaries.get(call.call_site_id.as_str());
        let matches_summary = summary.is_some_and(|summary| {
            call.provider.as_ref() == Some(&summary.provider)
                && call.effects == summary.effects
                && call.review_label.as_deref() == Some(summary.review_label.as_str())
                && call.evidence.as_ref() == Some(&summary.evidence)
        });
        if !matches_summary {
            diagnostics.push(format!(
                "admitted reviewed call `{}` does not match its workspace-policy summary",
                call.call_site_id
            ));
        } else if summary.is_some_and(|summary| !summary_evidence_matches(summary, &receipt.inputs.cargo_lock_blake3)) {
            diagnostics.push(format!(
                "admitted reviewed call `{}` summary evidence is not bound to the recorded Cargo.lock or provider identity",
                call.call_site_id
            ));
        }
    }
    for summary in &policy.call_summaries {
        if !used_summaries.contains(summary.call_site_id.as_str()) {
            diagnostics.push(format!(
                "workspace-policy call summary `{}` is not used by the admitted receipt",
                summary.call_site_id
            ));
        }
    }
    diagnostics
}

/// Re-derives the role facts an `admitted` receipt asserts: closure v1 admits only selected paths
/// with exactly one explicit `core` assignment, so every node must be `core`, and every local call
/// must follow the embedded catalog's role-edge matrix.
fn validate_admitted_roles(receipt: &EffectClosureReceipt, policy: &WorkspacePolicy, diagnostics: &mut Vec<String>) {
    let source_paths = receipt.sources.iter().map(|source| source.path.as_str()).collect::<BTreeSet<_>>();
    let mut assigned = BTreeMap::<&str, Vec<ModuleRole>>::new();
    for assignment in &policy.module_roles {
        if !source_paths.contains(assignment.path.as_str()) {
            diagnostics.push(format!(
                "workspace-policy module role path `{}` is outside the admitted receipt sources",
                assignment.path
            ));
        }
        assigned.entry(assignment.path.as_str()).or_default().push(assignment.role);
    }
    for path in source_paths {
        if assigned.get(path).map(Vec::as_slice) != Some(&[ModuleRole::Core]) {
            diagnostics.push(format!(
                "admitted receipt source `{path}` does not have exactly one workspace-policy role `core`"
            ));
        }
    }
    for node in receipt.nodes.iter().filter(|node| node.role != ModuleRole::Core) {
        diagnostics.push(format!(
            "admitted function `{}::{}` has role `{}`, not `core`",
            node.path,
            node.name,
            node.role.as_str()
        ));
    }
    let allowed_edges = match embedded_catalog() {
        Ok(catalog) => catalog.role_edges.into_iter().collect::<BTreeSet<_>>(),
        Err(error) => {
            diagnostics.push(error);
            return;
        }
    };
    let roles = receipt
        .nodes
        .iter()
        .map(|node| (node.function_address.as_str(), node.role))
        .collect::<BTreeMap<_, _>>();
    for node in &receipt.nodes {
        for call in node.calls.iter().filter(|call| call.resolution == CallResolution::Local) {
            let callee = call.callee_function_address.as_deref().and_then(|address| roles.get(address)).copied();
            let allowed = callee.is_some_and(|callee| {
                allowed_edges.contains(&RoleEdge {
                    caller: node.role,
                    callee,
                })
            });
            if !allowed {
                diagnostics.push(format!(
                    "admitted local call `{}` follows a role edge the effect-closure catalog denies",
                    call.call_site_id
                ));
            }
        }
    }
}

/// The stable ID of one call site. It binds the caller's function address and source path, the
/// call's resolution scope (the caller's module and what each leading name of the call path
/// resolves to outside the caller), its shape, and its occurrence in the caller, so a reviewed
/// summary neither survives a change of what the callee path resolves to nor transfers to an
/// identical function in another file or module.
pub fn call_site_identity(function_address: &str, path: &str, scope: &str, shape: &str, occurrence: usize) -> String {
    let payload = serde_json::json!({
        "domain": CALL_SITE_DOMAIN,
        "caller": function_address,
        "path": path,
        "scope": scope,
        "shape": shape,
        "occurrence": occurrence,
    });
    full_hash(&payload.to_string())
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

fn parse_function_drafts(
    sources: &[SourceEnvelope],
    diagnostics: &mut Vec<ClosureDiagnostic>,
) -> (Vec<FunctionDraft>, ModuleIndex) {
    let mut drafts = Vec::new();
    let mut modules = Vec::new();
    for source in sources {
        match syn::parse_file(&source.source) {
            Ok(file) => {
                let mut collector = FunctionCollector::default();
                collector.visit_file(&file);
                modules.push((source.path.clone(), collector.modules));
                drafts.extend(collector.functions.into_iter().map(|function| FunctionDraft {
                    path: source.path.clone(),
                    name: function.name,
                    module: function.module,
                    container: function.container,
                    opaque_context: function.opaque_context,
                    attrs: function.attrs,
                    enclosing_generics: function.enclosing_generics,
                    signature: function.signature,
                    block: function.block,
                    outer_scope: function.outer_scope,
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
    (drafts, ModuleIndex::build(modules))
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

// Node construction resolves summaries, providers, and role edges against lookup tables built at
// the top of the function.
#[allow(clippy::too_many_lines)]
fn build_nodes(
    functions: &[AddressedFunction],
    modules: &ModuleIndex,
    input: &ClosureInput,
    diagnostics: &mut Vec<ClosureDiagnostic>,
) -> Vec<FunctionClosureNode> {
    let foreign_functions = foreign_function_names(&input.sources);
    let local_names = local_function_index(functions);
    let resolver = ScopeResolver::new(modules, functions);
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
        let bindings = CallerBindings::collect(&function.draft);
        let function_scope = ValueScope::of_signature(&function.draft.enclosing_generics, &function.draft.signature);
        let mut scanner = BodyScanner::new(&foreign_functions, &bindings, function_scope);
        if function.draft.opaque_context || function.draft.signature.asyncness.is_some() {
            scanner.direct_effects.insert(Effect::Unknown);
        }
        if function.draft.signature.unsafety.is_some() {
            scanner.direct_effects.insert(Effect::Unsafe);
        }
        if function.draft.signature.abi.is_some() {
            scanner.direct_effects.insert(Effect::Ffi);
        }
        let parameter_attrs = function.draft.signature.inputs.iter().flat_map(|input| match input {
            syn::FnArg::Receiver(receiver) => &receiver.attrs,
            syn::FnArg::Typed(typed) => &typed.attrs,
        });
        for attribute in function.draft.attrs.iter().chain(parameter_attrs) {
            scanner.visit_attribute(attribute);
        }
        scanner.visit_block(&function.draft.block);

        let mut calls = Vec::new();
        for (occurrence, raw_call) in scanner.calls.into_iter().enumerate() {
            let call_scope = if raw_call.kind == RawCallKind::Method {
                resolver.method_scope(function, &raw_call)
            } else {
                resolver.call_scope(function, &raw_call)
            };
            let scope = call_scope.text;
            let call_site_id = call_site_identity(
                &function.function_address,
                &function.draft.path,
                &scope,
                &raw_call.shape,
                occurrence,
            );
            let local = resolve_local_call(&raw_call, function, &local_names).or(call_scope.local);
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
                    scope,
                    resolution: CallResolution::Local,
                    callee_function_address: Some(callee.function_address.clone()),
                    provider: None,
                    effects: raw_call.effects.into_iter().collect(),
                    review_label: None,
                    evidence: None,
                }
            } else if call_scope.hidden && raw_call.kind != RawCallKind::Macro {
                scanner.direct_effects.insert(Effect::Unknown);
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_CALL_UNKNOWN,
                    format!(
                        "call `{}` resolves through a module or import the selected sources cannot prove, so no summary applies",
                        raw_call.shape
                    ),
                    Some(function.draft.path.clone()),
                    Some(function.function_address.clone()),
                    Some(call_site_id.clone()),
                ));
                unknown_edge(call_site_id, occurrence, raw_call.shape, scope)
            } else if raw_call.kind != RawCallKind::Macro {
                match summaries.get(&call_site_id).copied() {
                    Some(summary)
                        if providers.contains(&summary.provider)
                            && summary_evidence_matches(summary, &input.cargo_lock_blake3) =>
                    {
                        used_summaries.insert(call_site_id.clone());
                        scanner.direct_effects.extend(summary.effects.iter().copied());
                        ClosureCallEdge {
                            call_site_id,
                            occurrence,
                            shape: raw_call.shape,
                            scope,
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
                        unknown_edge(call_site_id, occurrence, raw_call.shape, scope)
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
                        unknown_edge(call_site_id, occurrence, raw_call.shape, scope)
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
                unknown_edge(call_site_id, occurrence, raw_call.shape, scope)
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

/// Resolves a call, or a path used as a value, to a selected function only when its source path
/// proves the target: a single-segment path names a free function in the caller's file and module,
/// `Owner::name` names an inherent method of `impl Owner` in that scope, and `self.name()` from an
/// inherent method names a method of the same inherent owner in that scope. Paths through
/// `crate::`, `super::`, `self::`, a module, or a local `use` binding resolve in
/// [`ScopeResolver`] instead, only through proven selected modules. Every other shape, including
/// qualified-self or leading-`::` paths, paths whose first segment is bound inside the caller,
/// and trait dispatch, stays unresolved so it requires an exact summary or remains unknown.
fn resolve_local_call<'a>(
    raw_call: &RawCall,
    caller: &AddressedFunction,
    local_names: &'a BTreeMap<String, Vec<&'a AddressedFunction>>,
) -> Option<&'a AddressedFunction> {
    if !raw_call.locally_resolvable {
        return None;
    }
    let expected_container = match raw_call.kind {
        RawCallKind::Macro => return None,
        RawCallKind::Method => {
            raw_call.shape.strip_prefix("method:self:")?;
            match &caller.draft.container {
                FunctionContainer::InherentImpl(owner) => FunctionContainer::InherentImpl(owner.clone()),
                FunctionContainer::Module | FunctionContainer::Dispatched => return None,
            }
        }
        RawCallKind::Function | RawCallKind::Value => {
            let path = raw_call.shape.strip_prefix(raw_call.kind.shape_prefix())?;
            match path.split("::").collect::<Vec<_>>().as_slice() {
                [_name] => FunctionContainer::Module,
                [owner, _name] => FunctionContainer::InherentImpl((*owner).to_owned()),
                _ => return None,
            }
        }
    };
    let name = raw_call.shape.rsplit(':').next()?;
    let mut proven = local_names.get(name)?.iter().copied().filter(|candidate| {
        candidate.draft.container == expected_container
            && candidate.draft.path == caller.draft.path
            && candidate.draft.module == caller.draft.module
    });
    let candidate = proven.next()?;
    proven.next().is_none().then_some(candidate)
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

/// The owner name of an inherent `impl` self type, only when that type is a single in-scope path
/// segment, so `Owner::name` in the same module provably names the same type.
fn owner_from_type(node: &syn::Type) -> Option<String> {
    match node {
        syn::Type::Path(path) if path.qself.is_none() && path.path.leading_colon.is_none() => {
            match path.path.segments.iter().collect::<Vec<_>>().as_slice() {
                [segment] => Some(segment.ident.to_string()),
                _ => None,
            }
        }
        syn::Type::Reference(reference) => owner_from_type(&reference.elem),
        _ => None,
    }
}

fn path_text(path: &syn::Path) -> String {
    segments_text(path.segments.iter())
}

fn segments_text<'a>(segments: impl Iterator<Item = &'a syn::PathSegment>) -> String {
    segments.map(|segment| segment.ident.to_string()).collect::<Vec<_>>().join("::")
}

/// Renders a path expression as `(shape path, effect path)`. A qualified-self path keeps its
/// `<Type>` or `<Type as Trait>` prefix in the shape so it never aliases a local function of the
/// same final name; its effect path joins the self type and remaining segments so
/// `<std::fs::File>::create` still classifies as `fs`.
fn expr_path_text(node: &syn::ExprPath) -> (String, String) {
    let Some(qself) = &node.qself else {
        let text = path_text(&node.path);
        return (text.clone(), text);
    };
    let self_type = match qself.ty.as_ref() {
        syn::Type::Path(path) if path.qself.is_none() => path_text(&path.path),
        _ => "_".to_owned(),
    };
    let trait_path = segments_text(node.path.segments.iter().take(qself.position));
    let rest = segments_text(node.path.segments.iter().skip(qself.position));
    let shape = if trait_path.is_empty() {
        format!("<{self_type}>::{rest}")
    } else {
        format!("<{self_type} as {trait_path}>::{rest}")
    };
    (shape, format!("{self_type}::{rest}"))
}

fn unknown_edge(call_site_id: String, occurrence: usize, shape: String, scope: String) -> ClosureCallEdge {
    ClosureCallEdge {
        call_site_id,
        occurrence,
        shape,
        scope,
        resolution: CallResolution::Unknown,
        callee_function_address: None,
        provider: None,
        effects: vec![Effect::Unknown],
        review_label: None,
        evidence: None,
    }
}

fn summary_evidence_matches(summary: &CallSummary, cargo_lock_blake3: &str) -> bool {
    match summary.provider.kind {
        ProviderKind::CargoPackage | ProviderKind::Workspace => {
            summary.evidence.blake3 == canonical_blake3_identity(cargo_lock_blake3)
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

pub(crate) fn finalize_receipt(mut receipt: EffectClosureReceipt) -> EffectClosureReceipt {
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

// r[verify octet.effect_closure.verification]
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
            call_site_id: call_site_identity(
                function_address,
                TEST_PATH,
                &prelude_call_scope(TEST_PATH, shape),
                shape,
                occurrence,
            ),
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
        assert!(verify_receipt_identity(&receipt).is_empty());
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
        let shape = "call:mystery";
        let scope = prelude_call_scope(TEST_PATH, shape);
        let call_site_id = call_site_identity(&function_address.function_address, TEST_PATH, &scope, shape, 0);
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
        // A single-segment call cannot prove a target in another file, so it is never a local
        // cross-role edge; it stays unresolved instead.
        let root = role_denied.nodes.iter().find(|node| node.name == "root").expect("root node");
        assert_eq!(root.calls[0].resolution, CallResolution::Unknown);
        assert!(role_denied.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_CALL_UNKNOWN));

        let address_denied = compute_closure(input("fn root() {}\n", Vec::new(), policy(Vec::new())));
        assert!(address_denied.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_ADDRESS_MISSING));
    }

    // r[verify octet.effect_closure.roles.invalid]
    #[test]
    fn admitted_bindings_rederive_node_roles_and_local_role_edges() {
        let closure_input = input(
            "fn root() { helper(); }\nfn helper() {}\n",
            vec![address("helper", 'a'), address("root", 'b')],
            policy(Vec::new()),
        );
        let admitted = compute_closure(closure_input.clone());
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{:?}", admitted.diagnostics);
        let root = admitted.nodes.iter().find(|node| node.name == "root").expect("root node");
        assert_eq!(root.calls[0].resolution, CallResolution::Local);
        let bindings = |receipt: &EffectClosureReceipt| {
            validate_admitted_bindings(receipt, &closure_input.function_addresses, &closure_input.policy)
        };
        assert_eq!(bindings(&admitted), Vec::<String>::new());

        let mut forged = admitted;
        let helper = forged.nodes.iter_mut().find(|node| node.name == "helper").expect("helper node");
        helper.role = ModuleRole::Shell;
        let diagnostics = bindings(&finalize_receipt(forged));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("has role `shell`, not `core`")));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("role edge the effect-closure catalog denies"))
        );
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

    // r[verify octet.effect_closure.inference.direct]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn compound_assignment_arithmetic_is_panic_capable() {
        for op in ["+=", "-=", "*=", "/=", "%=", "<<=", ">>="] {
            let source = format!("fn root(mut value: u32, divisor: u32) -> u32 {{ value {op} divisor; value }}\n");
            let receipt = compute_closure(input(&source, vec![address("root", 'c')], policy(Vec::new())));

            assert_eq!(receipt.status, AdmissionStatus::Denied, "`{op}` must be denied");
            assert!(receipt.nodes[0].direct_effects.contains(&Effect::Panic), "`{op}` must be panic-capable");
        }

        for op in ["=", "&=", "|=", "^="] {
            let source = format!("fn root(mut value: u32, divisor: u32) -> u32 {{ value {op} divisor; value }}\n");
            let receipt = compute_closure(input(&source, vec![address("root", 'c')], policy(Vec::new())));

            assert_eq!(receipt.status, AdmissionStatus::Admitted, "`{op}` cannot panic");
            assert!(receipt.nodes[0].direct_effects.is_empty(), "`{op}` has no direct effect");
        }
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

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_status_requires_rederivable_admission_facts() {
        fn forge_admitted(mut receipt: EffectClosureReceipt) -> EffectClosureReceipt {
            receipt.diagnostics.clear();
            receipt.status = AdmissionStatus::Admitted;
            finalize_receipt(receipt)
        }
        fn assert_rejected(receipt: &EffectClosureReceipt, expected: &str) {
            let diagnostics = verify_receipt_identity(receipt);
            assert!(
                diagnostics.iter().any(|message| message.contains(expected)),
                "expected `{expected}` in {diagnostics:?}"
            );
            assert!(!diagnostics.iter().any(|message| message.contains(DIAGNOSTIC_RECEIPT_IDENTITY)));
        }

        let panicking = compute_closure(input(
            "fn root(value: u32) -> u32 { value + 1 }\n",
            vec![address("root", 'a')],
            policy(Vec::new()),
        ));
        assert_eq!(panicking.status, AdmissionStatus::Denied);
        assert!(verify_receipt_identity(&panicking).is_empty());
        assert_rejected(&forge_admitted(panicking), "closure contains denied effect `panic`");

        let unknown =
            compute_closure(input("fn root() { mystery(); }\n", vec![address("root", 'b')], policy(Vec::new())));
        assert_eq!(unknown.status, AdmissionStatus::Denied);
        assert!(verify_receipt_identity(&unknown).is_empty());
        assert_rejected(&forge_admitted(unknown), "admitted receipt has unknown call");

        let source = "fn leaf() { let _values: Vec<u8> = Vec::new(); }\nfn root() { leaf(); }\n";
        let leaf_address = address("leaf", '4');
        let (summary, provider) =
            toolchain_summary(&leaf_address.function_address, "call:Vec::new", 0, "alloc", vec![Effect::Alloc]);
        let mut closure_input = input(source, vec![leaf_address, address("root", '5')], policy(vec![summary]));
        closure_input.resolved_providers.push(provider);
        let admitted = compute_closure(closure_input);
        assert_eq!(admitted.status, AdmissionStatus::Admitted);
        assert!(verify_receipt_identity(&admitted).is_empty());

        let mut dropped_callee_effects = admitted.clone();
        let root = dropped_callee_effects.nodes.iter_mut().find(|node| node.name == "root").expect("root node");
        root.closure_effects.clear();
        assert_rejected(&finalize_receipt(dropped_callee_effects), "closure does not contain local callee");

        let mut dropped_call_effects = admitted;
        let leaf = dropped_call_effects.nodes.iter_mut().find(|node| node.name == "leaf").expect("leaf node");
        leaf.direct_effects.clear();
        assert_rejected(&finalize_receipt(dropped_call_effects), "direct effects do not contain every call effect");
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn local_resolution_requires_a_path_that_proves_the_target() {
        let qualified =
            "struct Tick;\nimpl Tick {\n    fn now() -> u8 { 0 }\n}\nfn root() -> u8 { crate::shell::now() }\n";
        let receipt =
            compute_closure(input(qualified, vec![address("now", '1'), address("root", '2')], policy(Vec::new())));
        let root = receipt.nodes.iter().find(|node| node.name == "root").expect("root node");
        assert_eq!(root.calls[0].shape, "call:crate::shell::now");
        assert_eq!(root.calls[0].resolution, CallResolution::Unknown);
        assert_eq!(receipt.status, AdmissionStatus::Denied);

        let trait_default =
            "trait Header {\n    fn header(&self) -> u8 { 0 }\n    fn render(&self) -> u8 { self.header() }\n}\n";
        let receipt = compute_closure(input(
            trait_default,
            vec![address("header", '3'), address("render", '4')],
            policy(Vec::new()),
        ));
        let render = receipt.nodes.iter().find(|node| node.name == "render").expect("render node");
        assert_eq!(render.calls[0].shape, "method:self:header");
        assert_eq!(render.calls[0].resolution, CallResolution::Unknown);
        assert_eq!(receipt.status, AdmissionStatus::Denied);

        let other_module = "mod inner {\n    pub fn helper() {}\n}\nuse other::helper;\nfn root() { helper(); }\n";
        let receipt = compute_closure(input(
            other_module,
            vec![address("helper", '5'), address("root", '6')],
            policy(Vec::new()),
        ));
        let root = receipt.nodes.iter().find(|node| node.name == "root").expect("root node");
        assert_eq!(root.calls[0].resolution, CallResolution::Unknown);
    }

    // r[verify octet.effect_closure.roles.valid]
    #[test]
    fn proven_local_paths_resolve() {
        let source = "struct Tick;\nimpl Tick {\n    fn zero() -> u8 { 0 }\n    fn read(&self) -> u8 { self.offset() }\n    fn offset(&self) -> u8 { Tick::zero() }\n}\nfn helper() -> u8 { Tick::zero() }\nfn root() -> u8 { helper() }\n";
        let receipt = compute_closure(input(
            source,
            vec![
                address("helper", '1'),
                address("offset", '2'),
                address("read", '3'),
                address("root", '4'),
                address("zero", '5'),
            ],
            policy(Vec::new()),
        ));

        assert_eq!(receipt.status, AdmissionStatus::Admitted, "{:?}", receipt.diagnostics);
        assert!(
            receipt
                .nodes
                .iter()
                .flat_map(|node| &node.calls)
                .all(|call| call.resolution == CallResolution::Local)
        );
        assert!(verify_receipt_identity(&receipt).is_empty());
    }

    fn call_edge<'a>(receipt: &'a EffectClosureReceipt, caller: &str, shape: &str) -> &'a ClosureCallEdge {
        receipt
            .nodes
            .iter()
            .filter(|node| node.name == caller)
            .flat_map(|node| &node.calls)
            .find(|call| call.shape == shape)
            .unwrap_or_else(|| panic!("`{caller}` has no `{shape}` call in {:?}", receipt.nodes))
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn qualified_paths_never_resolve_to_same_named_local_functions() {
        for (source, shape, names) in [
            (
                "fn create() {}\nfn root() { let _file = <std::fs::File>::create(\"x\"); }\n",
                "call:<std::fs::File>::create",
                ["create", "root"],
            ),
            ("fn new() {}\nfn root() { let _values = <Vec<u8>>::new(); }\n", "call:<Vec>::new", ["new", "root"]),
            (
                "fn write() {}\nfn root(file: &mut File) { let _ = <File as Write>::write(file, b\"x\"); }\n",
                "call:<File as Write>::write",
                ["root", "write"],
            ),
            ("fn helper() {}\nfn root() { ::helper(); }\n", "call:helper", ["helper", "root"]),
        ] {
            let addresses = names.iter().zip(['1', '2']).map(|(name, fill)| address(name, fill)).collect();
            let receipt = compute_closure(input(source, addresses, policy(Vec::new())));

            assert_eq!(call_edge(&receipt, "root", shape).resolution, CallResolution::Unknown, "{source}");
            assert_eq!(receipt.status, AdmissionStatus::Denied, "{source}");
            assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_CALL_UNKNOWN));
        }
        let receipt = compute_closure(input(
            "fn create() {}\nfn root() { let _file = <std::fs::File>::create(\"x\"); }\n",
            vec![address("create", '1'), address("root", '2')],
            policy(Vec::new()),
        ));
        let root = receipt.nodes.iter().find(|node| node.name == "root").expect("root node");
        assert!(root.direct_effects.contains(&Effect::Fs));
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn names_bound_inside_the_caller_never_resolve_to_module_functions() {
        let owner = "struct Owner;\nimpl Owner {\n    fn name() {}\n}\ntrait Tick {\n    fn name();\n}\n";
        let cases = [
            ("fn helper() {}\nfn root(helper: fn()) { helper(); }\n".to_owned(), "call:helper"),
            ("fn helper() {}\nfn root() { let helper = || {}; helper(); }\n".to_owned(), "call:helper"),
            (
                "fn helper() {}\nfn root() { let run = |helper: fn()| helper(); run(root); }\n".to_owned(),
                "call:helper",
            ),
            (
                "fn helper() {}\nfn root() { use crate::shell::now as helper; helper(); }\n".to_owned(),
                "call:helper",
            ),
            (
                "fn helper() {}\nfn root() { { use crate::shell::{now as helper}; helper(); } }\n".to_owned(),
                "call:helper",
            ),
            ("fn helper() {}\nfn root() { use crate::shell::*; helper(); }\n".to_owned(), "call:helper"),
            (
                "fn helper() {}\nfn root() { struct helper(u8); let _value = helper(1); }\n".to_owned(),
                "call:helper",
            ),
            (
                "fn helper() {}\nfn root() { extern \"C\" { fn helper(); } unsafe { helper() } }\n".to_owned(),
                "call:helper",
            ),
            ("fn helper() {}\nfn root() { for helper in [root] { helper(); } }\n".to_owned(), "call:helper"),
            (format!("{owner}fn root<Owner: Tick>() {{ Owner::name() }}\n"), "call:Owner::name"),
            (
                format!(
                    "{owner}struct Wrapper<T>(T);\nimpl<Owner: Tick> Wrapper<Owner> {{\n    fn root() {{ Owner::name() }}\n}}\n"
                ),
                "call:Owner::name",
            ),
        ];
        for (source, shape) in &cases {
            let addresses = ["helper", "name", "root"]
                .into_iter()
                .zip(['1', '2', '3'])
                .filter(|(name, _fill)| {
                    source.contains(&format!("fn {name}(")) || source.contains(&format!("fn {name}<"))
                })
                .map(|(name, fill)| address(name, fill))
                .collect();
            let receipt = compute_closure(input(source, addresses, policy(Vec::new())));

            assert_eq!(call_edge(&receipt, "root", shape).resolution, CallResolution::Unknown, "{source}");
            assert_eq!(receipt.status, AdmissionStatus::Denied, "{source}");
        }
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn function_paths_used_as_values_are_call_edges() {
        let external = compute_closure(input(
            "fn run(_hook: fn()) {}\nfn root() { run(std::process::abort); }\n",
            vec![address("root", '1'), address("run", '2')],
            policy(Vec::new()),
        ));
        assert_eq!(call_edge(&external, "root", "call:run").resolution, CallResolution::Local);
        assert_eq!(call_edge(&external, "root", "value:std::process::abort").resolution, CallResolution::Unknown);
        assert_eq!(external.status, AdmissionStatus::Denied);

        let local = compute_closure(input(
            "fn helper() {}\nfn run(_hook: fn()) {}\nfn use_it(_value: u8) {}\nfn root(count: u8) -> u8 {\n    run(helper);\n    let f = 1;\n    use_it(f);\n    match count {\n        inner => use_it(inner),\n    }\n    for item in [count] {\n        use_it(item);\n    }\n    let copy = count;\n    copy\n}\n",
            vec![
                address("helper", '1'),
                address("root", '2'),
                address("run", '3'),
                address("use_it", '4'),
            ],
            policy(Vec::new()),
        ));
        assert_eq!(local.status, AdmissionStatus::Admitted, "{:?}", local.diagnostics);
        assert_eq!(call_edge(&local, "root", "value:helper").resolution, CallResolution::Local);
        let root = local.nodes.iter().find(|node| node.name == "root").expect("root node");
        assert_eq!(root.calls.len(), 5, "in-scope local bindings are not call edges: {:?}", root.calls);
        assert!(verify_receipt_identity(&local).is_empty());
    }

    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn out_of_scope_local_bindings_do_not_hide_value_edges() {
        let import = "use std::fs::remove_dir_all as cleanup;\n";
        for body in [
            "fn root(x: Option<&str>) { if false { let cleanup = 1; let _ = cleanup; } let _ = x.map(cleanup); }",
            "fn root(x: Option<&str>) { let _ = x.map(cleanup); let cleanup = 1; let _ = cleanup; }",
            "fn root(x: Option<u8>) { match x { Some(cleanup) => { let _ = cleanup; } None => { let _ = x.map(cleanup); } } }",
            "fn root(x: Option<&str>) { match 1 { cleanup => { let _ = cleanup; } } let _ = x.map(cleanup); }",
            "fn root(x: Option<u8>) { if let Some(cleanup) = x { let _ = cleanup; } else { let _ = x.map(cleanup); } }",
            "fn root(x: Option<&str>) { while let Some(cleanup) = None::<u8> { let _ = cleanup; } let _ = x.map(cleanup); }",
            "fn root(x: Option<&str>) { for cleanup in [1] { let _ = cleanup; } let _ = x.map(cleanup); }",
            "fn root(x: Option<&str>) { let cleanup = x.map(cleanup); let _ = cleanup; }",
            "fn root(x: Option<&str>) { if let Some(cleanup) = x.map(cleanup) { let _ = cleanup; } }",
            "fn root(x: Option<&str>) { while let Some(cleanup) = x.map(cleanup) { let _ = cleanup; } }",
            "fn root(x: Option<&str>) { for cleanup in x.map(cleanup) { let _ = cleanup; } }",
            "fn root(x: Option<&str>) { let apply = |cleanup: u8| cleanup; let _ = apply; let _ = x.map(cleanup); }",
            "fn root(cleanup: u8) { fn inner(x: Option<&str>) { let _ = x.map(cleanup); } let _ = cleanup; }",
            "fn root(x: Option<&str>) { let cleanup = 1; { use std::fs::remove_dir_all as cleanup; let _ = x.map(cleanup); } let _ = cleanup; }",
        ] {
            let source = format!("{import}{body}\n");
            let receipt = compute_closure(input(&source, vec![address("root", '1')], policy(Vec::new())));
            let root = receipt.nodes.iter().find(|node| node.name == "root").expect("root node");
            let edges = root.calls.iter().filter(|call| call.shape == "value:cleanup").collect::<Vec<_>>();

            assert_eq!(edges.len(), 1, "{body}: {:?}", root.calls);
            assert_eq!(edges[0].resolution, CallResolution::Unknown, "{body}");
            assert_eq!(receipt.status, AdmissionStatus::Denied, "{body}");
        }
    }

    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn compile_time_positions_record_calls_but_not_value_edges() {
        let items =
            "const LEN: usize = 4;\nconst fn width() -> usize { 4 }\nfn make<const N: usize>(_count: usize) {}\n";
        for (body, value_edges) in [
            ("fn root() { let b: [u8; LEN + width()] = [0; 4]; let _ = b; }", 0),
            ("fn root() { let b = [0u8; LEN + width()]; let _ = b; }", 0),
            ("fn root() { let b = [LEN; LEN + width()]; let _ = b; }", 1),
            ("fn root() { make::<{ LEN + width() }>(0); }", 0),
            ("fn root() { make::<{ LEN + width() }>(LEN); }", 1),
            ("fn root() { enum E { A = (LEN + width()) as isize } }", 0),
            ("fn root() { enum E { A = (LEN + width()) as isize } let _ = LEN; }", 1),
            ("fn root() { const X: usize = LEN + width(); }", 0),
            ("fn root() { const X: usize = LEN + width(); let _ = LEN; }", 1),
            ("fn root() { static X: usize = LEN + width(); }", 0),
            ("fn root() { static X: usize = LEN + width(); let _ = LEN; }", 1),
            ("fn root() { struct B; impl B { const X: usize = LEN + width(); } let _ = LEN; }", 1),
            ("fn root() { trait T { const X: usize = LEN + width(); } let _ = LEN; }", 1),
            ("fn root() { struct B<const N: usize = { LEN + width() }>; }", 0),
            ("fn root() { struct B<const N: usize = { LEN + width() }>; let _ = LEN; }", 1),
            ("fn root() { make::<N = { LEN + width() }>(0); }", 0),
            ("fn root() { make::<N = { LEN + width() }>(LEN); }", 1),
        ] {
            let source = format!("{items}{body}\n");
            let receipt = compute_closure(input(&source, vec![address("root", '1')], policy(Vec::new())));
            let root = receipt.nodes.iter().find(|node| node.name == "root").expect("root node");
            let edges = root.calls.iter().filter(|call| call.shape == "value:LEN").count();

            assert_eq!(edges, value_edges, "{body}: {:?}", root.calls);
            assert!(root.calls.iter().any(|call| call.shape == "call:width"), "{body}: {:?}", root.calls);
        }
    }

    // r[verify octet.effect_closure.dependency_summaries]
    #[test]
    fn binding_patterns_are_numbered_before_their_initializers() {
        for body in [
            "fn root(v: u64) { let b: [u8; core::mem::size_of::<u64>()] = v.to_le_bytes(); let _ = b; }",
            "fn root(v: u64) { if let [const { core::mem::size_of::<u64>() as u8 }, ..] = v.to_le_bytes() {} }",
            "fn root(v: u64) { while let [const { core::mem::size_of::<u64>() as u8 }, ..] = v.to_le_bytes() {} }",
            "fn root(v: u64) { for [const { core::mem::size_of::<u64>() as u8 }, ..] in [v.to_le_bytes()] {} }",
        ] {
            let receipt = compute_closure(input(&format!("{body}\n"), vec![address("root", '1')], policy(Vec::new())));
            let root = receipt.nodes.iter().find(|node| node.name == "root").expect("root node");
            let mut calls = root.calls.iter().map(|call| (call.occurrence, call.shape.as_str())).collect::<Vec<_>>();
            calls.sort_unstable();

            assert_eq!(calls, [(0, "call:core::mem::size_of"), (1, "method:value:to_le_bytes")], "{body}");
        }
    }

    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn nested_impl_and_trait_const_generics_are_not_value_edges() {
        for (body, value_edges) in [
            ("fn root() { struct B<const N: usize>; impl<const N: usize> B<N> { fn len() -> usize { N } } }", 0),
            ("fn root() { trait T<const N: usize> { fn len() -> usize { N } } }", 0),
            ("fn root() { struct B<const N: usize>; impl<const N: usize> B<N> { const LEN: usize = N; } }", 0),
            ("fn root() { struct B<const N: usize>; impl<const N: usize> B<N> {} let _ = N; }", 1),
            ("fn root() { struct B<const M: usize>; impl<const M: usize> B<M> { fn len() -> usize { N } } }", 1),
            ("fn root() { trait T<const M: usize> { fn len() -> usize { N } } }", 1),
        ] {
            let source = format!("const N: usize = 4;\n{body}\n");
            let receipt = compute_closure(input(&source, vec![address("root", '1')], policy(Vec::new())));
            let root = receipt.nodes.iter().find(|node| node.name == "root").expect("root node");
            let edges = root.calls.iter().filter(|call| call.shape == "value:N").count();

            assert_eq!(edges, value_edges, "{body}: {:?}", root.calls);
        }
    }

    // r[verify octet.effect_closure.inference.direct]
    #[test]
    fn nested_function_items_belong_to_their_enclosing_function() {
        let pure = compute_closure(input(
            "fn outer() -> u8 {\n    fn inner() -> u8 { 0 }\n    0\n}\n",
            vec![address("outer", '1')],
            policy(Vec::new()),
        ));
        assert_eq!(pure.status, AdmissionStatus::Admitted, "{:?}", pure.diagnostics);
        assert_eq!(pure.nodes.len(), 1);

        let panicking = compute_closure(input(
            "fn outer(value: u32) -> u32 {\n    fn inner(value: u32) -> u32 { value + 1 }\n    value\n}\n",
            vec![address("outer", '2')],
            policy(Vec::new()),
        ));
        assert!(!panicking.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_ADDRESS_MISSING));
        assert_eq!(panicking.nodes.len(), 1);
        assert!(panicking.nodes[0].direct_effects.contains(&Effect::Panic));
    }

    // r[verify octet.effect_closure.inference.unknown]
    #[test]
    fn enclosing_conditional_compilation_makes_functions_unknown() {
        for source in [
            "#![cfg(feature = \"special\")]\nfn root() {}\n",
            "#[cfg(feature = \"special\")]\nmod gated {\n    fn root() {}\n}\n",
            "mod gated {\n    #![cfg(test)]\n    fn root() {}\n}\n",
            "struct Tick;\n#[cfg(feature = \"special\")]\nimpl Tick {\n    fn root() {}\n}\n",
            "struct Tick;\n#[cfg_attr(test, allow(dead_code))]\nimpl Tick {\n    fn root() {}\n}\n",
            "#[cfg(test)]\ntrait Gate {\n    fn root(&self) {}\n}\n",
        ] {
            let receipt = compute_closure(input(source, vec![address("root", '1')], policy(Vec::new())));
            assert_eq!(receipt.status, AdmissionStatus::Denied, "{source}");
            assert!(receipt.nodes[0].direct_effects.contains(&Effect::Unknown), "{source}");
        }
    }

    // r[verify octet.effect_closure.inference.unknown]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn attribute_macros_make_functions_unknown() {
        for source in [
            "#[dep::persist]\nfn root() {}\n",
            "#[persist]\nfn root() {}\n",
            "#[test]\nfn root() {}\n",
            "fn root(#[cfg(test)] _value: u8) {}\n",
            "struct Tick;\n#[dep::persist]\nimpl Tick {\n    fn root() {}\n}\n",
            "#[dep::persist]\ntrait Gate {\n    fn root(&self) {}\n}\n",
            "#[dep::persist]\nmod generated {\n    fn root() {}\n}\n",
            "mod generated {\n    #![dep::persist]\n    fn root() {}\n}\n",
            "#![dep::persist]\nfn root() {}\n",
            "fn root() {\n    #[dep::persist]\n    fn inner() {}\n}\n",
            "fn root() {\n    #[derive(Clone)]\n    struct Copied;\n}\n",
        ] {
            let receipt = compute_closure(input(source, vec![address("root", '1')], policy(Vec::new())));
            assert_eq!(receipt.status, AdmissionStatus::Denied, "{source}");
            assert!(receipt.nodes[0].direct_effects.contains(&Effect::Unknown), "{source}");
        }
    }

    // r[verify octet.effect_closure.inference.direct]
    #[test]
    fn inert_built_in_attributes_keep_functions_admissible() {
        for source in [
            "/// Documented.\n#[inline]\n#[must_use]\n#[allow(clippy::pedantic)]\nfn root() -> u8 { 0 }\n",
            "#![forbid(unsafe_code)]\n#![doc = \"crate docs\"]\n#[cold]\n#[track_caller]\n#[deny(warnings)]\nfn root() -> u8 { 0 }\n",
            "#[deprecated(note = \"replaced\")]\n#[expect(dead_code)]\n#[warn(unused)]\nfn root(#[allow(unused)] _value: u8) -> u8 { 0 }\n",
            "struct Tick;\n/// Inherent.\n#[allow(dead_code)]\nimpl Tick {\n    #[inline(always)]\n    fn root() -> u8 { 0 }\n}\n",
            "#[doc(hidden)]\nmod quiet {\n    #![allow(dead_code)]\n    #[must_use = \"use it\"]\n    fn root() -> u8 { #[allow(unused)] let value = 0; value }\n}\n",
        ] {
            let receipt = compute_closure(input(source, vec![address("root", '1')], policy(Vec::new())));
            assert_eq!(receipt.status, AdmissionStatus::Admitted, "{source}: {:?}", receipt.diagnostics);
            assert!(receipt.nodes[0].direct_effects.is_empty(), "{source}");
        }
    }

    /// Reviews every call of a denied run the way an operator would: one exact summary per call
    /// site, keyed by the call-site ID the receipt reports.
    fn reviewed_summaries(receipt: &EffectClosureReceipt, provider: &ProviderIdentity) -> Vec<CallSummary> {
        let mut summaries = receipt
            .nodes
            .iter()
            .flat_map(|node| &node.calls)
            .map(|call| CallSummary {
                call_site_id: call.call_site_id.clone(),
                review_label: format!("reviewed `{}`", call.shape),
                callee: call.shape.clone(),
                provider: provider.clone(),
                effects: Vec::new(),
                evidence: EvidenceReference {
                    kind: "reviewed-source-summary-v1".to_owned(),
                    locator: "Cargo.lock#dep".to_owned(),
                    blake3: TEST_LOCK_HASH.to_owned(),
                },
            })
            .collect::<Vec<_>>();
        summaries.sort();
        summaries
    }

    fn dependency_provider() -> ProviderIdentity {
        ProviderIdentity {
            kind: ProviderKind::CargoPackage,
            name: "dep".to_owned(),
            version: "1.0.0".to_owned(),
            source: "registry+test".to_owned(),
            target: TEST_TARGET.to_owned(),
            features: Vec::new(),
        }
    }

    fn reviewed_input(source: &str, summaries: Vec<CallSummary>) -> ClosureInput {
        let mut closure_input = input(source, vec![address("root", '5')], policy(summaries));
        closure_input.resolved_providers.push(dependency_provider());
        closure_input
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn call_summaries_are_bound_to_the_callee_resolution_in_scope() {
        let reviewed = "use dep::tidy;\nuse dep::Store;\nfn root() { tidy(); Store::open(); }\n";
        let denied = compute_closure(input(reviewed, vec![address("root", '5')], policy(Vec::new())));
        assert_eq!(denied.status, AdmissionStatus::Denied);
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let admitted = compute_closure(reviewed_input(reviewed, summaries.clone()));
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{:?}", admitted.diagnostics);
        assert!(verify_receipt_identity(&admitted).is_empty());

        // Edits that leave each callee's resolution alone keep every reviewed summary.
        for unchanged in [
            "use dep::tidy;\nuse other::unrelated;\nuse dep::Store;\nconst LIMIT: u8 = 1;\nfn root() { tidy(); Store::open(); }\n",
            "use dep::{Store, tidy};\nfn root() { tidy(); Store::open(); }\n",
        ] {
            let receipt = compute_closure(reviewed_input(unchanged, summaries.clone()));
            assert_eq!(receipt.status, AdmissionStatus::Admitted, "{unchanged}: {:?}", receipt.diagnostics);
        }

        // The same caller address under a different resolution of a callee's leading segment
        // leaves that summary stale and the call unknown.
        for (changed, shape) in [
            ("use other::tidy;\nuse dep::Store;\nfn root() { tidy(); Store::open(); }\n", "call:tidy"),
            ("use dep::other as tidy;\nuse dep::Store;\nfn root() { tidy(); Store::open(); }\n", "call:tidy"),
            ("use dep::*;\nuse dep::Store;\nfn root() { tidy(); Store::open(); }\n", "call:tidy"),
            (
                "use dep::tidy;\ntype Store = other::Store;\nfn root() { tidy(); Store::open(); }\n",
                "call:Store::open",
            ),
            (
                "use dep::tidy;\nuse dep::Store;\nmod inner {\n    fn root() { tidy(); Store::open(); }\n}\n",
                "call:tidy",
            ),
        ] {
            let receipt = compute_closure(reviewed_input(changed, summaries.clone()));
            assert_eq!(receipt.status, AdmissionStatus::Denied, "{changed}");
            assert!(
                receipt.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_SUMMARY_STALE),
                "{changed}"
            );
            assert_eq!(call_edge(&receipt, "root", shape).resolution, CallResolution::Unknown, "{changed}");
        }

        // A generic parameter or `Self` resolves through the enclosing `impl` header.
        let generic = "struct Store<T>(T);\nimpl<T: dep::Persist> Store<T> {\n    fn root() { T::make(); }\n}\n";
        let denied = compute_closure(input(generic, vec![address("root", '5')], policy(Vec::new())));
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let admitted = compute_closure(reviewed_input(generic, summaries.clone()));
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{:?}", admitted.diagnostics);
        let rebound = generic.replace("dep::Persist", "other::Persist");
        let receipt = compute_closure(reviewed_input(&rebound, summaries));
        assert_eq!(call_edge(&receipt, "root", "call:T::make").resolution, CallResolution::Unknown);
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn call_summaries_do_not_transfer_to_identical_functions_in_other_files() {
        const OTHER_PATH: &str = "crates/domain/src/other.rs";
        let reviewed = "use dep::tidy;\nfn root() { tidy(); }\n";
        let denied = compute_closure(input(reviewed, vec![address("root", '5')], policy(Vec::new())));
        let summaries = reviewed_summaries(&denied, &dependency_provider());

        let mut closure_input = reviewed_input(reviewed, summaries);
        closure_input.sources.push(SourceEnvelope {
            path: OTHER_PATH.to_owned(),
            source: "use other::tidy;\nfn root() { tidy(); }\n".to_owned(),
            source_identity: TEST_SOURCE_ID.to_owned(),
        });
        closure_input.function_addresses.push(address_at(OTHER_PATH, "root", '5'));
        closure_input.policy.module_roles.push(ModuleRoleAssignment {
            path: OTHER_PATH.to_owned(),
            role: ModuleRole::Core,
        });
        let receipt = compute_closure(closure_input);

        assert_eq!(receipt.status, AdmissionStatus::Denied);
        let other = receipt.nodes.iter().find(|node| node.path == OTHER_PATH).expect("other node");
        assert_eq!(other.calls[0].resolution, CallResolution::Unknown);
        let reviewed_node = receipt.nodes.iter().find(|node| node.path == TEST_PATH).expect("reviewed node");
        assert_eq!(reviewed_node.calls[0].resolution, CallResolution::ReviewedSummary);
    }

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn stored_receipt_schema_is_checked_before_decoding() {
        let receipt =
            compute_closure(input("fn root() { mystery(); }\n", vec![address("root", '1')], policy(Vec::new())));
        let current = serde_json::to_vec(&receipt).expect("receipt JSON");
        assert_eq!(unsupported_receipt_schema(&current), None);
        assert!(serde_json::from_slice::<EffectClosureReceipt>(&current).is_ok());

        for stored in [
            r#"{"schema_version":"octet-effect-closure-receipt/v1","nodes":[{"calls":[{"shape":"call:x"}]}]}"#,
            r#"{"schema_version":2}"#,
            r#"{"status":"admitted"}"#,
        ] {
            let diagnostic = unsupported_receipt_schema(stored.as_bytes());
            assert!(
                diagnostic
                    .as_deref()
                    .is_some_and(|message| message.contains("unsupported effect-closure receipt schema")),
                "{stored}: {diagnostic:?}"
            );
        }
        // Bytes that are not a JSON object are left to the full parse to report as malformed.
        assert_eq!(unsupported_receipt_schema(b"not json"), None);
        assert_eq!(unsupported_receipt_schema(b"[]"), None);
    }

    // r[verify octet.effect_closure.inference.unknown]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn tool_attributes_are_unknown_because_tool_names_can_be_shadowed() {
        for source in [
            "#[rustfmt::skip]\nfn root() {}\n",
            "mod rustfmt {}\n#[rustfmt::skip]\nfn root() {}\n",
            "use dep as rustfmt;\n#[rustfmt::skip]\nfn root() {}\n",
            "extern crate dep as clippy;\n#[clippy::msrv = \"1.70\"]\nfn root() {}\n",
            "#[diagnostic::on_unimplemented(message = \"gate\")]\ntrait Gate {\n    fn root(&self) {}\n}\n",
        ] {
            let receipt = compute_closure(input(source, vec![address("root", '1')], policy(Vec::new())));
            assert_eq!(receipt.status, AdmissionStatus::Denied, "{source}");
            assert!(receipt.nodes[0].direct_effects.contains(&Effect::Unknown), "{source}");
        }
    }

    /// Asserts that each source keeps or loses the reviewed summaries of `reviewed`.
    fn assert_summaries_follow_resolution(reviewed: &str, unchanged: &[&str], changed: &[(&str, &str)]) {
        let denied = compute_closure(input(reviewed, vec![address("root", '5')], policy(Vec::new())));
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let admitted = compute_closure(reviewed_input(reviewed, summaries.clone()));
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{reviewed}: {:?}", admitted.diagnostics);
        for source in unchanged {
            let receipt = compute_closure(reviewed_input(source, summaries.clone()));
            assert_eq!(receipt.status, AdmissionStatus::Admitted, "{source}: {:?}", receipt.diagnostics);
        }
        for (source, shape) in changed {
            let receipt = compute_closure(reviewed_input(source, summaries.clone()));
            assert!(
                receipt.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_SUMMARY_STALE),
                "{source}: {:?}",
                receipt.diagnostics
            );
            assert_eq!(call_edge(&receipt, "root", shape).resolution, CallResolution::Unknown, "{source}");
        }
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn attributed_items_may_bind_any_name_in_their_module() {
        assert_summaries_follow_resolution(
            "struct Plain;\nfn root() { tidy(); }\n",
            &[
                "#[allow(dead_code)]\nstruct Plain;\nfn root() { tidy(); }\n",
                "/// Documented.\nstruct Plain;\nfn root() { tidy(); }\n",
                "#[cfg(test)]\nstruct Plain;\nfn root() { tidy(); }\n",
            ],
            &[
                ("#[derive(Persist)]\nstruct Plain;\nfn root() { tidy(); }\n", "call:tidy"),
                ("struct Plain;\n#[dep::generate]\nimpl Plain {}\nfn root() { tidy(); }\n", "call:tidy"),
                ("#[dep::generate]\nconst LIMIT: u8 = 1;\nstruct Plain;\nfn root() { tidy(); }\n", "call:tidy"),
            ],
        );
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn method_call_summaries_are_bound_to_the_traits_in_scope() {
        assert_summaries_follow_resolution(
            "use fs_ext::PersistExt;\nfn root(value: u8) { value.persist(); }\n",
            &["use fs_ext::PersistExt;\nconst LIMIT: u8 = 1;\nfn root(value: u8) { value.persist(); }\n"],
            &[
                ("use other::PersistExt;\nfn root(value: u8) { value.persist(); }\n", "method:value:persist"),
                ("use fs_ext::*;\nfn root(value: u8) { value.persist(); }\n", "method:value:persist"),
                (
                    "use fs_ext::PersistExt;\ntrait Local {}\nfn root(value: u8) { value.persist(); }\n",
                    "method:value:persist",
                ),
            ],
        );
    }

    const LIB_PATH: &str = "crates/domain/src/lib.rs";
    const UTIL_PATH: &str = "crates/domain/src/util.rs";

    /// A closure input over several selected `core` files of one crate.
    fn workspace_input(
        files: &[(&str, &str)],
        addresses: Vec<FunctionAddressRecord>,
        summaries: Vec<CallSummary>,
    ) -> ClosureInput {
        let mut closure_input = input("", addresses, policy(summaries));
        closure_input.sources = files
            .iter()
            .map(|(path, source)| SourceEnvelope {
                path: (*path).to_owned(),
                source: (*source).to_owned(),
                source_identity: TEST_SOURCE_ID.to_owned(),
            })
            .collect();
        closure_input.policy.module_roles = files
            .iter()
            .map(|(path, _)| ModuleRoleAssignment {
                path: (*path).to_owned(),
                role: ModuleRole::Core,
            })
            .collect();
        closure_input.policy.module_roles.sort();
        closure_input.resolved_providers.push(dependency_provider());
        closure_input
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn qualified_local_paths_follow_the_target_module() {
        let lib = "mod core;\nmod util;\n";
        let caller_address = || vec![address("root", '5')];
        for (caller, shape) in [
            ("pub fn root() { crate::util::tidy(); }\n", "call:crate::util::tidy"),
            ("pub fn root() { super::util::tidy(); }\n", "call:super::util::tidy"),
            ("use crate::util::tidy;\npub fn root() { tidy(); }\n", "call:tidy"),
            ("use crate::util;\npub fn root() { util::tidy(); }\n", "call:util::tidy"),
        ] {
            let reviewed = [
                (TEST_PATH, caller),
                (LIB_PATH, lib),
                (UTIL_PATH, "pub use dep_a::tidy;\n"),
            ];
            let denied = compute_closure(workspace_input(&reviewed, caller_address(), Vec::new()));
            let summaries = reviewed_summaries(&denied, &dependency_provider());
            let admitted = compute_closure(workspace_input(&reviewed, caller_address(), summaries.clone()));
            assert_eq!(admitted.status, AdmissionStatus::Admitted, "{caller}: {:?}", admitted.diagnostics);

            // A re-export that now names another dependency leaves the summary stale.
            let rebound = [
                (TEST_PATH, caller),
                (LIB_PATH, lib),
                (UTIL_PATH, "pub use dep_b::tidy;\n"),
            ];
            let receipt = compute_closure(workspace_input(&rebound, caller_address(), summaries.clone()));
            assert!(
                receipt.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_SUMMARY_STALE),
                "{caller}: {:?}",
                receipt.diagnostics
            );
            assert_eq!(call_edge(&receipt, "root", shape).resolution, CallResolution::Unknown, "{caller}");

            // A local function in the target module is followed as a local edge.
            let local = [
                (TEST_PATH, caller),
                (LIB_PATH, lib),
                (UTIL_PATH, "pub fn tidy() { let _ = std::fs::remove_file(\"x\"); }\n"),
            ];
            let mut addresses = caller_address();
            addresses.push(address_at(UTIL_PATH, "tidy", '6'));
            let receipt = compute_closure(workspace_input(&local, addresses, summaries.clone()));
            assert_eq!(call_edge(&receipt, "root", shape).resolution, CallResolution::Local, "{caller}");
            let root = receipt.nodes.iter().find(|node| node.name == "root").expect("root node");
            assert!(root.closure_effects.contains(&Effect::Fs), "{caller}");

            // Without the crate root, the module the path names cannot be proven: no summary applies.
            let hidden = [(TEST_PATH, caller), (UTIL_PATH, "pub use dep_a::tidy;\n")];
            let denied = compute_closure(workspace_input(&hidden, caller_address(), Vec::new()));
            let summaries = reviewed_summaries(&denied, &dependency_provider());
            let receipt = compute_closure(workspace_input(&hidden, caller_address(), summaries));
            assert_eq!(receipt.status, AdmissionStatus::Denied, "{caller}");
            assert_eq!(call_edge(&receipt, "root", shape).resolution, CallResolution::Unknown, "{caller}");
        }

        // An inline module needs no crate root: its declaration is in the caller's file.
        let inline = |dependency: &str| {
            format!("mod inner {{\n    pub use {dependency}::tidy;\n}}\npub fn root() {{ self::inner::tidy(); }}\n")
        };
        assert_summaries_follow_resolution(&inline("dep_a"), &[], &[(
            inline("dep_b").as_str(),
            "call:self::inner::tidy",
        )]);

        // An import named after its own crate (`use tidy::tidy;`) still names the extern crate.
        assert_summaries_follow_resolution("use tidy::tidy;\nfn root() { tidy(); }\n", &[], &[(
            "use other::tidy;\nfn root() { tidy(); }\n",
            "call:tidy",
        )]);
    }

    // r[verify octet.effect_closure.dependency_summaries]
    #[test]
    fn glob_cycles_keep_prelude_calls_summarizable() {
        // A child re-importing its parent while the parent re-exports the child is a cycle that
        // never declares `Some`: the prelude call stays summarizable.
        let cycle = "mod inner {\n    use super::*;\n    pub fn helper() {}\n}\npub use inner::*;\npub fn root() -> Option<u8> { Some(1) }\n";
        let addresses = || vec![address("helper", '6'), address("root", '5')];
        let denied = compute_closure(input(cycle, addresses(), policy(Vec::new())));
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let mut closure_input = input(cycle, addresses(), policy(summaries));
        closure_input.resolved_providers.push(dependency_provider());
        let admitted = compute_closure(closure_input);
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{:?}", admitted.diagnostics);
    }

    // r[verify octet.effect_closure.dependency_summaries]
    #[test]
    fn glob_fan_out_stays_bounded() {
        // Eight modules that each glob-import their parent and all eight siblings resolve in
        // bounded work, with a scope bounded by the distinct declarations reached.
        let siblings = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let globs = siblings.iter().map(|sibling| format!("{sibling}::*")).collect::<Vec<_>>().join(", ");
        let modules = siblings
            .iter()
            .map(|module| {
                [
                    "mod ",
                    module,
                    " {\n    pub use super::*;\n    pub use super::{",
                    &globs,
                    "};\n}\n",
                ]
                .concat()
            })
            .collect::<String>();
        let source = format!(
            "pub use self::{{{globs}}};\n{modules}pub fn root() {{ let _ = (Some(1), Some(2), Some(3), Some(4), Some(5)); }}\n"
        );
        let receipt = compute_closure(input(&source, vec![address("root", '5')], policy(Vec::new())));
        let calls = &receipt.nodes[0].calls;
        assert_eq!(calls.len(), 5);
        for call in calls {
            assert!(call.scope.len() < 16 * 1024, "scope of {} bytes", call.scope.len());
        }
        assert!(!receipt.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("cannot prove")));
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn body_imports_bind_only_in_enclosing_blocks() {
        assert_summaries_follow_resolution(
            "use dep_a::tidy;\npub fn root() { { use dep_a::tidy; } tidy(); }\n",
            &[],
            &[("use dep_b::tidy;\npub fn root() { { use dep_a::tidy; } tidy(); }\n", "call:tidy")],
        );
        assert_summaries_follow_resolution(
            "use dep_a::tidy;\npub fn root() { fn helper() { use dep_a::tidy; } tidy(); }\n",
            &[],
            &[("use dep_b::tidy;\npub fn root() { fn helper() { use dep_a::tidy; } tidy(); }\n", "call:tidy")],
        );
        // An import in an enclosing block shadows the module's binding.
        assert_summaries_follow_resolution(
            "use dep_b::tidy;\npub fn root() { use dep_a::tidy; tidy(); }\n",
            &["use other::tidy;\npub fn root() { use dep_a::tidy; tidy(); }\n"],
            &[],
        );
        // A module item in the body resolves its own names: no summary stands in for its calls.
        let body_module =
            "use dep_a::tidy;\npub fn root() {\n    mod inner {\n        pub fn run() { tidy(); }\n    }\n}\n";
        let denied = compute_closure(input(body_module, vec![address("root", '5')], policy(Vec::new())));
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let receipt = compute_closure(reviewed_input(body_module, summaries));
        assert_eq!(receipt.status, AdmissionStatus::Denied);
        assert_eq!(call_edge(&receipt, "root", "call:tidy").resolution, CallResolution::Unknown);
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn owner_paths_bind_the_owner_declaration() {
        assert_summaries_follow_resolution("pub struct Foo;\npub fn root() { Foo::persist(); }\n", &[], &[
            (
                "#[derive(dep_b::Persist)]\npub struct Foo;\npub fn root() { Foo::persist(); }\n",
                "call:Foo::persist",
            ),
            ("#[dep_b::make_persist]\npub struct Foo;\npub fn root() { Foo::persist(); }\n", "call:Foo::persist"),
            (
                "pub struct Foo;\n#[dep_b::make_persist]\nimpl Foo {}\npub fn root() { Foo::persist(); }\n",
                "call:Foo::persist",
            ),
        ]);
    }

    /// Reviews every call of `reviewed` (several selected files, `root` in `TEST_PATH`), then
    /// asserts that `changed` leaves the summary for `shape` stale and the call unknown.
    fn assert_workspace_summary_goes_stale(reviewed: &[(&str, &str)], changed: &[(&str, &str)], shape: &str) {
        let caller_address = || vec![address("root", '5')];
        let denied = compute_closure(workspace_input(reviewed, caller_address(), Vec::new()));
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let admitted = compute_closure(workspace_input(reviewed, caller_address(), summaries.clone()));
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{reviewed:?}: {:?}", admitted.diagnostics);
        let receipt = compute_closure(workspace_input(changed, caller_address(), summaries));
        assert!(
            receipt.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_SUMMARY_STALE),
            "{changed:?}: {:?}",
            receipt.diagnostics
        );
        assert_eq!(call_edge(&receipt, "root", shape).resolution, CallResolution::Unknown, "{changed:?}");
    }

    const EXT_PATH: &str = "crates/domain/src/ext.rs";

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn method_scopes_follow_local_trait_imports() {
        let lib = "mod core;\nmod ext;\n";
        for caller in [
            "use crate::ext::PersistExt;\npub fn root(value: u8) { value.persist(); }\n",
            "use crate::ext::*;\npub fn root(value: u8) { value.persist(); }\n",
        ] {
            assert_workspace_summary_goes_stale(
                &[
                    (TEST_PATH, caller),
                    (EXT_PATH, "pub use fs_ext::PersistExt;\n"),
                    (LIB_PATH, lib),
                ],
                &[
                    (TEST_PATH, caller),
                    (EXT_PATH, "pub use other::PersistExt;\n"),
                    (LIB_PATH, lib),
                ],
                "method:value:persist",
            );
        }
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn import_heads_from_local_globs_are_followed() {
        let lib = "mod core;\nmod util;\n";
        let caller = "use crate::*;\nuse util::tidy;\npub fn root() { tidy(); }\n";
        assert_workspace_summary_goes_stale(
            &[
                (TEST_PATH, caller),
                (LIB_PATH, lib),
                (UTIL_PATH, "pub use dep_a::tidy;\n"),
            ],
            &[
                (TEST_PATH, caller),
                (LIB_PATH, lib),
                (UTIL_PATH, "pub use dep_b::tidy;\n"),
            ],
            "call:tidy",
        );
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.verification.adversarial]
    #[test]
    fn macro_generated_items_bind_the_macro_definition() {
        // A same-module `macro_rules!` body is part of what an undeclared name may resolve to.
        let generated = |body: &str| {
            format!("macro_rules! m {{\n    () => {{ {body} }};\n}}\nm!();\npub fn root() {{ tidy(); }}\n")
        };
        assert_summaries_follow_resolution(&generated("pub use dep_a::tidy;"), &[], &[(
            generated("pub use dep_b::tidy;").as_str(),
            "call:tidy",
        )]);

        // A path through a module only a macro can supply cannot be proven: no summary applies.
        let through = "macro_rules! m {\n    () => { mod gen { pub use dep_a::tidy; } };\n}\nm!();\npub fn root() { self::gen::tidy(); }\n";
        let denied = compute_closure(input(through, vec![address("root", '5')], policy(Vec::new())));
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let receipt = compute_closure(reviewed_input(through, summaries));
        assert_eq!(receipt.status, AdmissionStatus::Denied);
        assert_eq!(call_edge(&receipt, "root", "call:self::gen::tidy").resolution, CallResolution::Unknown);
    }

    /// Reviews every call of `files` (`root` in `TEST_PATH`) and asserts the reviewed summaries
    /// admit them: no call is left unknown or hidden.
    fn assert_workspace_summaries_admit(files: &[(&str, &str)]) {
        let caller_address = || vec![address("root", '5')];
        let denied = compute_closure(workspace_input(files, caller_address(), Vec::new()));
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let admitted = compute_closure(workspace_input(files, caller_address(), summaries));
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{files:?}: {:?}", admitted.diagnostics);
    }

    // r[verify octet.effect_closure.dependency_summaries]
    #[test]
    fn glob_probes_through_attributed_modules_stay_summarizable() {
        let derived = "#[derive(Debug, Clone, PartialEq)]\npub struct Item;\n";
        assert_workspace_summaries_admit(&[
            (TEST_PATH, "use crate::util::*;\npub fn root() { let _ = Vec::<u8>::new(); }\n"),
            (LIB_PATH, "mod core;\nmod util;\n"),
            (UTIL_PATH, derived),
        ]);
        assert_workspace_summaries_admit(&[
            (
                TEST_PATH,
                "use crate::util::*;\nuse std::collections::BTreeMap;\npub fn root(values: Vec<u8>) {\n    let _ = Vec::<u8>::new();\n    let _ = BTreeMap::<u8, u8>::new();\n    let _ = values.iter();\n    let _ = values.len();\n}\n",
            ),
            (LIB_PATH, "mod core;\nmod util;\npub use util::*;\n"),
            (UTIL_PATH, derived),
        ]);
        let inline = "#[derive(Clone, Debug)]\npub struct Item;\nmod inner {\n    use super::*;\n    pub fn root() { let _ = String::from(\"x\"); }\n}\n";
        let denied = compute_closure(input(inline, vec![address("root", '5')], policy(Vec::new())));
        let summaries = reviewed_summaries(&denied, &dependency_provider());
        let admitted = compute_closure(reviewed_input(inline, summaries));
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{:?}", admitted.diagnostics);
    }

    // r[verify octet.effect_closure.dependency_summaries]
    #[test]
    fn method_scopes_resolve_many_imports_within_bounds() {
        let names = (0..220).map(|index| format!("A{index}")).collect::<Vec<_>>();
        let caller =
            format!("use crate::util::{{{}}};\npub fn root(v: Vec<u8>) {{ let _ = v.len(); }}\n", names.join(", "));
        let util = names.iter().map(|name| ["pub struct ", name, ";\n"].concat()).collect::<String>();
        assert_workspace_summaries_admit(&[
            (TEST_PATH, &caller),
            (LIB_PATH, "mod core;\nmod util;\n"),
            (UTIL_PATH, &util),
        ]);
    }
}
