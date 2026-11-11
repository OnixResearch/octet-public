//! Bounded Rust source-to-IR frontend for the semantic merge rail.
//!
//! The frontend accepts the pure scalar subset documented in
//! `docs/semantic-merge-verification-rail.md`: `i64` parameters and return
//! values, integer literals, parameter references, checked or wrapping
//! arithmetic, conditionals, and direct calls. It maps one Rust source file to
//! the typed scalar IR and records an explicit syntactic correspondence
//! obligation.
//!
//! The frontend does not prove Rust semantic equivalence, compiler
//! correctness, whole-program behavior, or the absence of effects outside the
//! parsed items. Unsupported constructs fail closed with a typed error rather
//! than being interpreted or silently skipped.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::checker::SOURCE_DOMAIN;
use crate::checker::SUBJECT_DOMAIN;
use crate::identity::canonical_identity;
use crate::identity::framed_blake3;
use crate::model::Expr;
use crate::model::Function;
use crate::model::Program;
use crate::model::Role;
use crate::model::RoleBinding;
use crate::model::UnsupportedConstruct;
use crate::model::Variant;

/// Stable frontend profile identity.
pub const FRONTEND_PROFILE_ID: &str = "octet-merge-rail-rust-syn/v1";
/// Stable frontend profile version.
pub const FRONTEND_PROFILE_VERSION: &str = "1";
/// Explicit syntactic correspondence obligation.
pub const CORRESPONDENCE_OBLIGATION: &str = "Every admitted source function is mapped exactly once; unsupported constructs fail closed; source and subject identities bind the mapping. This is a syntactic mapping over the documented scalar subset, not a semantic-equivalence, compiler-correctness, or whole-program proof.";
/// Required frontend non-claims.
pub const FRONTEND_NON_CLAIMS: [&str; 4] = [
    "the frontend maps a documented syntactic subset; it does not prove Rust semantics, compiler correctness, or whole-program behavior",
    "unsupported constructs are rejected, not interpreted or silently skipped",
    "source and subject identities bind the exact source text and emitted IR; binding agreement is not semantic equivalence",
    "the frontend does not prove the absence of effects outside the parsed items",
];

/// Typed frontend failure. No partial program is returned.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FrontendError {
    /// Stable error code.
    pub code: String,
    /// Reviewed unsupported class, when the source is outside the subset.
    pub construct: Option<UnsupportedConstruct>,
    /// Human-readable detail.
    pub detail: String,
}

impl FrontendError {
    fn new(code: &str, construct: Option<UnsupportedConstruct>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            construct,
            detail: detail.into(),
        }
    }

    fn unsupported(construct: UnsupportedConstruct, detail: impl Into<String>) -> Self {
        Self::new("unsupported-construct", Some(construct), detail)
    }
}

/// One source function and its emitted function shape.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FunctionMapping {
    /// Source-level function name.
    pub name: String,
    /// Number of positional parameters.
    pub params: u16,
}

/// Explicit record of the frontend's syntactic correspondence obligation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CorrespondenceReport {
    /// Frontend profile identity.
    pub profile_id: String,
    /// Frontend profile version.
    pub profile_version: String,
    /// BLAKE3 identity of the exact source text.
    pub source_identity: String,
    /// BLAKE3 identity of the emitted role-local program.
    pub subject_identity: String,
    /// Emitted source-level functions.
    pub functions: Vec<FunctionMapping>,
    /// Correspondence obligation.
    pub obligation: String,
    /// Frontend non-claims.
    pub non_claims: Vec<String>,
}

/// Deterministic frontend output for one source file.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FrontendOutput {
    /// Frontend profile identity.
    pub profile_id: String,
    /// Frontend profile version.
    pub profile_version: String,
    /// Exact reviewed source text.
    pub source_text: String,
    /// Role-local typed program keyed by source-level function names.
    pub program: Program,
    /// Emitted source functions in canonical name order.
    pub functions: Vec<FunctionMapping>,
    /// BLAKE3 identity of the source text.
    pub source_identity: String,
    /// BLAKE3 identity of the emitted program.
    pub subject_identity: String,
    /// Frontend non-claims.
    pub non_claims: Vec<String>,
}

impl FrontendOutput {
    /// Build the role-tagged variant for this frontend output.
    #[must_use]
    pub fn into_variant(self, role: Role) -> Variant {
        Variant {
            role,
            source_text: self.source_text,
            program: self.program,
        }
    }

    /// Build the source/subject binding for this frontend output.
    #[must_use]
    pub fn binding(&self, role: Role) -> RoleBinding {
        RoleBinding {
            role,
            source_identity: self.source_identity.clone(),
            subject_identity: self.subject_identity.clone(),
        }
    }

    /// Return the explicit correspondence report.
    #[must_use]
    pub fn correspondence(&self) -> CorrespondenceReport {
        CorrespondenceReport {
            profile_id: self.profile_id.clone(),
            profile_version: self.profile_version.clone(),
            source_identity: self.source_identity.clone(),
            subject_identity: self.subject_identity.clone(),
            functions: self.functions.clone(),
            obligation: CORRESPONDENCE_OBLIGATION.to_owned(),
            non_claims: self.non_claims.clone(),
        }
    }
}

/// Parse one Rust source file into the bounded scalar IR.
///
/// # Errors
///
/// Returns [`FrontendError`] for parse failures, unsupported signatures,
/// unsupported statements or expressions, duplicate functions, or an empty
/// program. No partial program is returned.
pub fn parse_source(source: &str) -> Result<FrontendOutput, FrontendError> {
    let file = syn::parse_file(source).map_err(|error| FrontendError::new("parse", None, error.to_string()))?;
    let mut program = Program::new();
    let mut functions = Vec::new();
    for item in &file.items {
        let syn::Item::Fn(function) = item else {
            return Err(FrontendError::unsupported(
                UnsupportedConstruct::Other,
                "source item is outside the admitted function-only subset",
            ));
        };
        let name = function.sig.ident.to_string();
        if program.contains_key(&name) {
            return Err(FrontendError::new(
                "duplicate-function",
                None,
                format!("function `{name}` appears more than once"),
            ));
        }
        let (params, body) = parse_function(function)?;
        program.insert(name.clone(), Function::new(params, body));
        functions.push(FunctionMapping { name, params });
    }
    if program.is_empty() {
        return Err(FrontendError::new("empty-program", None, "source file contains no admitted functions"));
    }
    functions.sort_by(|left, right| left.name.cmp(&right.name));
    let source_identity = framed_blake3(SOURCE_DOMAIN, &[source.as_bytes()]);
    let subject_identity =
        canonical_identity(SUBJECT_DOMAIN, &program).map_err(|error| FrontendError::new("identity", None, error))?;
    Ok(FrontendOutput {
        profile_id: FRONTEND_PROFILE_ID.to_owned(),
        profile_version: FRONTEND_PROFILE_VERSION.to_owned(),
        source_text: source.to_owned(),
        program,
        functions,
        source_identity,
        subject_identity,
        non_claims: FRONTEND_NON_CLAIMS.iter().map(|claim| (*claim).to_owned()).collect(),
    })
}

fn parse_function(function: &syn::ItemFn) -> Result<(u16, Expr), FrontendError> {
    let signature = &function.sig;
    if signature.constness.is_some()
        || signature.asyncness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
    {
        return Err(FrontendError::unsupported(
            UnsupportedConstruct::Other,
            "function qualifiers outside `fn` are not admitted",
        ));
    }
    if !signature.generics.params.is_empty() || signature.generics.where_clause.is_some() {
        return Err(FrontendError::unsupported(
            UnsupportedConstruct::Other,
            "generic functions are outside the admitted scalar subset",
        ));
    }
    let mut params = BTreeMap::new();
    for (index, argument) in signature.inputs.iter().enumerate() {
        let syn::FnArg::Typed(typed) = argument else {
            return Err(FrontendError::unsupported(
                UnsupportedConstruct::Other,
                "method receivers are outside the admitted scalar subset",
            ));
        };
        let syn::Pat::Ident(binding) = &*typed.pat else {
            return Err(FrontendError::unsupported(
                UnsupportedConstruct::Other,
                "parameter patterns must be plain identifiers",
            ));
        };
        if binding.mutability.is_some() || binding.subpat.is_some() {
            return Err(FrontendError::unsupported(
                UnsupportedConstruct::Other,
                "mutable or destructuring parameters are outside the admitted scalar subset",
            ));
        }
        if !is_i64(&typed.ty) {
            return Err(FrontendError::unsupported(UnsupportedConstruct::Other, "parameters must have type `i64`"));
        }
        let index = u16::try_from(index)
            .map_err(|_| FrontendError::new("parameter-count", None, "parameter count exceeds u16"))?;
        if params.insert(binding.ident.to_string(), index).is_some() {
            return Err(FrontendError::new(
                "duplicate-parameter",
                None,
                format!("parameter `{}` appears more than once", binding.ident),
            ));
        }
    }
    let syn::ReturnType::Type(_, return_type) = &signature.output else {
        return Err(FrontendError::unsupported(UnsupportedConstruct::Other, "functions must return `i64`"));
    };
    if !is_i64(return_type) {
        return Err(FrontendError::unsupported(UnsupportedConstruct::Other, "return type must be `i64`"));
    }
    let body = parse_block(&function.block, &params)?;
    let params = u16::try_from(params.len())
        .map_err(|_| FrontendError::new("parameter-count", None, "parameter count exceeds u16"))?;
    Ok((params, body))
}

fn parse_block(block: &syn::Block, params: &BTreeMap<String, u16>) -> Result<Expr, FrontendError> {
    if block.stmts.len() != 1 {
        return Err(FrontendError::unsupported(
            UnsupportedConstruct::Other,
            "blocks must contain exactly one tail expression",
        ));
    }
    let syn::Stmt::Expr(expression, None) = &block.stmts[0] else {
        return Err(FrontendError::unsupported(
            UnsupportedConstruct::Other,
            "statements, semicolons, and local bindings are outside the admitted subset",
        ));
    };
    parse_expression(expression, params)
}

fn parse_expression(expression: &syn::Expr, params: &BTreeMap<String, u16>) -> Result<Expr, FrontendError> {
    match expression {
        syn::Expr::Lit(literal) => parse_literal(&literal.lit),
        syn::Expr::Path(path) => parse_path(path, params),
        syn::Expr::Paren(parenthesized) => parse_expression(&parenthesized.expr, params),
        syn::Expr::Group(grouped) => parse_expression(&grouped.expr, params),
        syn::Expr::Block(block) => parse_block(&block.block, params),
        syn::Expr::Binary(binary) => {
            let left = parse_expression(&binary.left, params)?;
            let right = parse_expression(&binary.right, params)?;
            let expression = match binary.op {
                syn::BinOp::Add(_) => Expr::plus(left, right),
                syn::BinOp::Sub(_) => Expr::minus(left, right),
                syn::BinOp::Mul(_) => Expr::times(left, right),
                syn::BinOp::Div(_) => Expr::divide(left, right),
                syn::BinOp::Rem(_) => Expr::remainder(left, right),
                _ => {
                    return Err(FrontendError::unsupported(
                        UnsupportedConstruct::Other,
                        "binary operator is outside the admitted scalar subset",
                    ));
                }
            };
            Ok(expression)
        }
        syn::Expr::Unary(unary) => match unary.op {
            syn::UnOp::Neg(_) => Ok(Expr::negate(parse_expression(&unary.expr, params)?)),
            _ => Err(FrontendError::unsupported(
                UnsupportedConstruct::Other,
                "unary operator is outside the admitted scalar subset",
            )),
        },
        syn::Expr::If(conditional) => {
            let test = parse_expression(&conditional.cond, params)?;
            let then_value = parse_block(&conditional.then_branch, params)?;
            let Some((_, else_expression)) = &conditional.else_branch else {
                return Err(FrontendError::unsupported(
                    UnsupportedConstruct::Other,
                    "conditionals must have an explicit else branch",
                ));
            };
            let else_value = parse_expression(else_expression, params)?;
            Ok(Expr::condition(test, then_value, else_value))
        }
        syn::Expr::Call(call) => {
            let item = parse_call_target(&call.func)?;
            let mut args = Vec::with_capacity(call.args.len());
            for argument in &call.args {
                args.push(parse_expression(argument, params)?);
            }
            Ok(Expr::call(item, args))
        }
        syn::Expr::Unsafe(_) => {
            Err(FrontendError::unsupported(UnsupportedConstruct::Unsafe, "unsafe blocks are excluded"))
        }
        syn::Expr::Macro(_) => {
            Err(FrontendError::unsupported(UnsupportedConstruct::MacroExpansion, "macros are excluded"))
        }
        syn::Expr::Reference(_) | syn::Expr::RawAddr(_) => Err(FrontendError::unsupported(
            UnsupportedConstruct::RawPointer,
            "references and raw pointers are excluded",
        )),
        syn::Expr::MethodCall(_) | syn::Expr::Field(_) | syn::Expr::Index(_) => Err(FrontendError::unsupported(
            UnsupportedConstruct::Dispatch,
            "method, field, and index operations are outside the admitted scalar subset",
        )),
        syn::Expr::Closure(_) | syn::Expr::Async(_) | syn::Expr::Await(_) | syn::Expr::Try(_) => {
            Err(FrontendError::unsupported(
                UnsupportedConstruct::Effect,
                "closures, async blocks, awaits, and try expressions are excluded",
            ))
        }
        _ => Err(FrontendError::unsupported(
            UnsupportedConstruct::Other,
            "expression is outside the admitted scalar subset",
        )),
    }
}

fn parse_literal(literal: &syn::Lit) -> Result<Expr, FrontendError> {
    let syn::Lit::Int(integer) = literal else {
        return Err(FrontendError::unsupported(UnsupportedConstruct::Other, "only integer literals are admitted"));
    };
    if !integer.suffix().is_empty() && integer.suffix() != "i64" {
        return Err(FrontendError::unsupported(
            UnsupportedConstruct::Other,
            "integer literal suffix must be omitted or `i64`",
        ));
    }
    let value = integer
        .base10_parse::<i64>()
        .map_err(|error| FrontendError::new("integer-literal", None, error.to_string()))?;
    Ok(Expr::Int(value))
}

fn parse_path(path: &syn::ExprPath, params: &BTreeMap<String, u16>) -> Result<Expr, FrontendError> {
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return Err(FrontendError::unsupported(
            UnsupportedConstruct::Dispatch,
            "qualified paths and multi-segment paths are excluded",
        ));
    }
    let segment = &path.path.segments[0];
    if !matches!(segment.arguments, syn::PathArguments::None) {
        return Err(FrontendError::unsupported(UnsupportedConstruct::Other, "generic path arguments are excluded"));
    }
    let name = segment.ident.to_string();
    let Some(index) = params.get(&name).copied() else {
        return Err(FrontendError::new(
            "unknown-local",
            None,
            format!("identifier `{name}` is not an admitted parameter; local bindings are excluded"),
        ));
    };
    Ok(Expr::Arg(index))
}

fn parse_call_target(function: &syn::Expr) -> Result<String, FrontendError> {
    let syn::Expr::Path(path) = function else {
        return Err(FrontendError::unsupported(
            UnsupportedConstruct::Dispatch,
            "call targets must be direct function names",
        ));
    };
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return Err(FrontendError::unsupported(
            UnsupportedConstruct::Dispatch,
            "call targets must be direct function names",
        ));
    }
    let segment = &path.path.segments[0];
    if !matches!(segment.arguments, syn::PathArguments::None) {
        return Err(FrontendError::unsupported(UnsupportedConstruct::Other, "generic call targets are excluded"));
    }
    Ok(segment.ident.to_string())
}

fn is_i64(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    path.qself.is_none() && path.path.is_ident("i64")
}
