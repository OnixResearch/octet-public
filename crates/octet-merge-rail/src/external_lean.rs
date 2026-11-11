//! Lean-checked external backend for the bounded scalar merge rail.
//!
//! The backend first evaluates the request with the deterministic in-memory
//! evaluator, then asks the pinned Lean kernel to check a concrete theorem that
//! the encoded program returns that exact value or abort. It is a bounded,
//! per-input external verifier: it does not prove a universal relation, does not
//! prove the Rust compiler, and does not make a semantic-equivalence claim.
//!
//! Trusted base: the Lean kernel, the fixed generated encoding in this module,
//! and the pinned Lean executable. A Lean rejection, timeout, spawn failure, or
//! nonzero exit is a tool error, never a semantic conflict.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::evaluate::Backend;
use crate::evaluate::BackendError;
use crate::evaluate::BackendIdentity;
use crate::evaluate::EvalContext;
use crate::evaluate::EvalOutcome;
use crate::evaluate::IrBackend;
use crate::evaluate::ResolvedProgram;
use crate::model::AbortKind;
use crate::model::ArithmeticModel;
use crate::model::Expr;
use crate::model::UnsupportedReason;
use crate::model::Value;

/// Stable Lean backend identifier.
pub const LEAN_BACKEND_ID: &str = "lean-checked-scalar-verifier";
/// Stable Lean backend version.
pub const LEAN_BACKEND_VERSION: &str = "1";
/// Environment variable naming the pinned Lean executable.
pub const LEAN_ENVIRONMENT_VARIABLE: &str = "OCTET_MERGE_RAIL_LEAN";
/// Default per-evaluation Lean timeout in milliseconds.
pub const DEFAULT_LEAN_TIMEOUT_MS: u64 = 10_000;
/// Minimum positive Lean timeout.
const MIN_LEAN_TIMEOUT_MS: u64 = 1;

static NEXT_WORK_ID: AtomicU64 = AtomicU64::new(0);

/// Lean-checked external backend.
#[derive(Debug)]
pub struct LeanBackend {
    lean: PathBuf,
    work_root: PathBuf,
    timeout_ms: u64,
}

impl LeanBackend {
    /// Build a Lean backend using the supplied executable and work root.
    #[must_use]
    pub fn new(lean: impl Into<PathBuf>, work_root: impl Into<PathBuf>) -> Self {
        Self {
            lean: lean.into(),
            work_root: work_root.into(),
            timeout_ms: DEFAULT_LEAN_TIMEOUT_MS,
        }
    }

    /// Build a Lean backend from [`LEAN_ENVIRONMENT_VARIABLE`].
    ///
    /// # Errors
    ///
    /// Returns [`BackendError`] when the variable is unset or empty.
    pub fn from_environment(work_root: impl Into<PathBuf>) -> Result<Self, BackendError> {
        let lean = std::env::var(LEAN_ENVIRONMENT_VARIABLE).map_err(|error| BackendError {
            message: format!("read `{LEAN_ENVIRONMENT_VARIABLE}`: {error}"),
        })?;
        if lean.trim().is_empty() {
            return Err(BackendError {
                message: format!("`{LEAN_ENVIRONMENT_VARIABLE}` is empty"),
            });
        }
        Ok(Self::new(lean, work_root))
    }

    /// Set the per-evaluation Lean timeout.
    #[must_use]
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms.max(MIN_LEAN_TIMEOUT_MS);
        self
    }

    /// Return the configured Lean executable.
    #[must_use]
    pub fn lean(&self) -> &Path {
        &self.lean
    }

    /// Return the configured work root.
    #[must_use]
    pub fn work_root(&self) -> &Path {
        &self.work_root
    }
}

impl Backend for LeanBackend {
    fn identity(&self) -> BackendIdentity {
        BackendIdentity {
            id: LEAN_BACKEND_ID.to_owned(),
            version: LEAN_BACKEND_VERSION.to_owned(),
        }
    }

    fn evaluate(
        &self,
        program: &ResolvedProgram,
        item: &str,
        args: &[Value],
        context: &EvalContext,
    ) -> Result<EvalOutcome, BackendError> {
        if context.arithmetic != ArithmeticModel::Checked {
            return Ok(EvalOutcome::Unsupported {
                reason: UnsupportedReason::UnsupportedConstruct,
                detail: "Lean backend supports checked arithmetic only; wrapping is outside this backend profile"
                    .to_owned(),
            });
        }
        if let Err(error) = topological_order(program, item) {
            return Ok(match error {
                EncodeError::Cycle => EvalOutcome::Unsupported {
                    reason: UnsupportedReason::Recursion,
                    detail: "Lean backend cannot encode a cyclic call graph".to_owned(),
                },
                EncodeError::Undefined(name) => EvalOutcome::Unsupported {
                    reason: UnsupportedReason::UndefinedDependency,
                    detail: format!("Lean backend could not resolve `{name}`"),
                },
            });
        }
        let expected = IrBackend.evaluate(program, item, args, context)?;
        let EvalOutcome::Value(value) = expected else {
            return Ok(expected);
        };
        if args.iter().any(|argument| matches!(argument, Value::Aborted(_))) {
            return Ok(EvalOutcome::Value(value));
        }
        let source = match lean_source(program, item, args, value) {
            Ok(source) => source,
            Err(EncodeError::Cycle) => {
                return Ok(EvalOutcome::Unsupported {
                    reason: UnsupportedReason::Recursion,
                    detail: "Lean backend cannot encode a cyclic call graph".to_owned(),
                });
            }
            Err(EncodeError::Undefined(name)) => {
                return Ok(EvalOutcome::Unsupported {
                    reason: UnsupportedReason::UndefinedDependency,
                    detail: format!("Lean backend could not resolve `{name}`"),
                });
            }
        };
        self.run_lean(&source)?;
        Ok(EvalOutcome::Value(value))
    }
}

impl LeanBackend {
    fn run_lean(&self, source: &str) -> Result<(), BackendError> {
        let id = NEXT_WORK_ID.fetch_add(1, Ordering::Relaxed);
        let directory = self.work_root.join(format!("octet-merge-rail-lean-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&directory).map_err(|error| BackendError {
            message: format!("create Lean work directory `{}`: {error}", directory.display()),
        })?;
        let file = directory.join("Check.lean");
        let result = self.execute_lean(&file, source);
        let _ = std::fs::remove_dir_all(&directory);
        result
    }

    fn execute_lean(&self, file: &Path, source: &str) -> Result<(), BackendError> {
        std::fs::write(file, source).map_err(|error| BackendError {
            message: format!("write Lean source `{}`: {error}", file.display()),
        })?;
        let mut child = Command::new(&self.lean)
            .arg(file)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| BackendError {
                message: format!("spawn Lean executable `{}`: {error}", self.lean.display()),
            })?;
        let deadline = Instant::now() + Duration::from_millis(self.timeout_ms);
        loop {
            if let Some(status) = child.try_wait().map_err(|error| BackendError {
                message: format!("wait for Lean: {error}"),
            })? {
                let diagnostic = read_bounded_output(&mut child);
                if status.success() {
                    return Ok(());
                }
                return Err(BackendError {
                    message: format!("Lean rejected or failed to check the concrete theorem: {diagnostic}"),
                });
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(BackendError {
                    message: format!("Lean exceeded the {} ms bounded timeout", self.timeout_ms),
                });
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

fn read_bounded_output(child: &mut std::process::Child) -> String {
    let mut bytes = Vec::new();
    if let Some(stdout) = child.stdout.as_mut() {
        let _ = stdout.take(4_096).read_to_end(&mut bytes);
    }
    if let Some(stderr) = child.stderr.as_mut() {
        let _ = stderr.take(4_096).read_to_end(&mut bytes);
    }
    String::from_utf8_lossy(&bytes).trim().to_owned()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum EncodeError {
    Cycle,
    Undefined(String),
}

fn lean_source(program: &ResolvedProgram, item: &str, args: &[Value], value: Value) -> Result<String, EncodeError> {
    let order = topological_order(program, item)?;
    let mut names = BTreeMap::new();
    for (index, name) in order.iter().enumerate() {
        names.insert(name.clone(), format!("fn_{index}"));
    }
    let mut source = String::from(
        r"inductive MrAbort where
  | overflow
  | divideByZero
deriving DecidableEq, Repr

inductive MrResult where
  | ok (value : Int)
  | error (abort : MrAbort)
deriving DecidableEq, Repr

def mrCheckedAdd (a b : Int) : MrResult :=
  let r := a + b
  if r < -9223372036854775808 then .error .overflow
  else if r > 9223372036854775807 then .error .overflow
  else .ok r

def mrCheckedSub (a b : Int) : MrResult :=
  let r := a - b
  if r < -9223372036854775808 then .error .overflow
  else if r > 9223372036854775807 then .error .overflow
  else .ok r

def mrCheckedMul (a b : Int) : MrResult :=
  let r := a * b
  if r < -9223372036854775808 then .error .overflow
  else if r > 9223372036854775807 then .error .overflow
  else .ok r

def mrCheckedDiv (a b : Int) : MrResult :=
  if b = 0 then .error .divideByZero
  else if a = -9223372036854775808 && b = -1 then .error .overflow
  else .ok (Int.tdiv a b)

def mrCheckedRem (a b : Int) : MrResult :=
  if b = 0 then .error .divideByZero
  else if a = -9223372036854775808 && b = -1 then .error .overflow
  else .ok (Int.tmod a b)

def mrCheckedNeg (a : Int) : MrResult :=
  if a = -9223372036854775808 then .error .overflow
  else .ok (-a)

",
    );
    for name in &order {
        let function = program.get(name.as_str()).ok_or_else(|| EncodeError::Undefined(name.clone()))?;
        let lean_name = names.get(name.as_str()).ok_or_else(|| EncodeError::Undefined(name.clone()))?;
        let parameters = (0..function.params).map(|index| format!("a{index}")).collect::<Vec<_>>();
        let signature = if parameters.is_empty() {
            String::new()
        } else {
            format!(" ({} : Int)", parameters.join(" "))
        };
        let body = encode_expr(&function.body, &names);
        writeln!(&mut source, "def {lean_name}{signature} : MrResult := {body}")
            .expect("writing to a String cannot fail");
    }
    let lean_item = names.get(item).ok_or_else(|| EncodeError::Undefined(item.to_owned()))?;
    let arguments = args
        .iter()
        .map(|argument| match argument {
            Value::Int(value) => format!("({value} : Int)"),
            Value::Aborted(_) => "0".to_owned(),
        })
        .collect::<Vec<_>>();
    let call = if arguments.is_empty() {
        lean_item.clone()
    } else {
        format!("{lean_item} {}", arguments.join(" "))
    };
    let expected = match value {
        Value::Int(value) => format!("MrResult.ok ({value} : Int)"),
        Value::Aborted(AbortKind::Overflow) => "MrResult.error MrAbort.overflow".to_owned(),
        Value::Aborted(AbortKind::DivideByZero) => "MrResult.error MrAbort.divideByZero".to_owned(),
    };
    writeln!(&mut source, "example : {call} = {expected} := by decide").expect("writing to a String cannot fail");
    Ok(source)
}

fn encode_expr(expression: &Expr, names: &BTreeMap<String, String>) -> String {
    match expression {
        Expr::Int(value) => format!("MrResult.ok ({value} : Int)"),
        Expr::Arg(index) => format!("MrResult.ok a{index}"),
        Expr::Add(left, right) => encode_binary("mrCheckedAdd", left, right, names),
        Expr::Sub(left, right) => encode_binary("mrCheckedSub", left, right, names),
        Expr::Mul(left, right) => encode_binary("mrCheckedMul", left, right, names),
        Expr::Div(left, right) => encode_binary("mrCheckedDiv", left, right, names),
        Expr::Rem(left, right) => encode_binary("mrCheckedRem", left, right, names),
        Expr::Neg(value) => {
            let value = encode_expr(value, names);
            format!("match {value} with | MrResult.error e => MrResult.error e | MrResult.ok a => mrCheckedNeg a")
        }
        Expr::If(test, then_value, else_value) => {
            let test = encode_expr(test, names);
            let then_value = encode_expr(then_value, names);
            let else_value = encode_expr(else_value, names);
            format!("match {test} with | .error e => .error e | .ok c => if c = 0 then {else_value} else {then_value}")
        }
        Expr::Call { item, args } => {
            let callee = names.get(item).map_or_else(|| "fn_undefined".to_owned(), Clone::clone);
            encode_call(&callee, args, names)
        }
        Expr::Unsupported { .. } => "MrResult.error MrAbort.overflow".to_owned(),
    }
}

fn encode_binary(name: &str, left: &Expr, right: &Expr, names: &BTreeMap<String, String>) -> String {
    let left = encode_expr(left, names);
    let right = encode_expr(right, names);
    format!(
        "match {left} with | MrResult.error e => MrResult.error e | MrResult.ok x => match {right} with | MrResult.error e => MrResult.error e | MrResult.ok y => {name} x y"
    )
}

fn encode_call(callee: &str, args: &[Expr], names: &BTreeMap<String, String>) -> String {
    let variables = (0..args.len()).map(|index| format!("x{index}")).collect::<Vec<_>>();
    let call = if variables.is_empty() {
        callee.to_owned()
    } else {
        format!("{callee} {}", variables.join(" "))
    };
    encode_call_at(args, names, 0, &call)
}

fn encode_call_at(args: &[Expr], names: &BTreeMap<String, String>, index: usize, call: &str) -> String {
    let Some(argument) = args.first() else {
        return call.to_owned();
    };
    let value = encode_expr(argument, names);
    let rest = encode_call_at(&args[1..], names, index + 1, call);
    format!("match {value} with | MrResult.error e => MrResult.error e | MrResult.ok x{index} => {rest}")
}

fn topological_order(program: &ResolvedProgram, item: &str) -> Result<Vec<String>, EncodeError> {
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut order = Vec::new();
    visit(program, item, &mut visiting, &mut visited, &mut order)?;
    Ok(order)
}

fn visit(
    program: &ResolvedProgram,
    item: &str,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) -> Result<(), EncodeError> {
    if visited.contains(item) {
        return Ok(());
    }
    if !visiting.insert(item.to_owned()) {
        return Err(EncodeError::Cycle);
    }
    let function = program.get(item).ok_or_else(|| EncodeError::Undefined(item.to_owned()))?;
    let mut calls = Vec::new();
    collect_calls(&function.body, &mut calls);
    for call in calls {
        visit(program, &call, visiting, visited, order)?;
    }
    visiting.remove(item);
    visited.insert(item.to_owned());
    order.push(item.to_owned());
    Ok(())
}

fn collect_calls(expression: &Expr, calls: &mut Vec<String>) {
    match expression {
        Expr::Int(_) | Expr::Arg(_) | Expr::Unsupported { .. } => {}
        Expr::Add(left, right)
        | Expr::Sub(left, right)
        | Expr::Mul(left, right)
        | Expr::Div(left, right)
        | Expr::Rem(left, right) => {
            collect_calls(left, calls);
            collect_calls(right, calls);
        }
        Expr::Neg(value) => collect_calls(value, calls),
        Expr::If(test, then_value, else_value) => {
            collect_calls(test, calls);
            collect_calls(then_value, calls);
            collect_calls(else_value, calls);
        }
        Expr::Call { item, args } => {
            calls.push(item.clone());
            for argument in args {
                collect_calls(argument, calls);
            }
        }
    }
}
