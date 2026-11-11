//! Pure bounded IR evaluator used as the prototype backend.

use std::collections::BTreeMap;

use crate::model::AbortKind;
use crate::model::ArithmeticModel;
use crate::model::Expr;
use crate::model::Function;
use crate::model::UnsupportedReason;
use crate::model::Value;

/// Logical-name function table after item resolution.
pub type ResolvedProgram = BTreeMap<String, Function>;

/// Stable backend identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendIdentity {
    /// Backend identifier.
    pub id: String,
    /// Backend version.
    pub version: String,
}

/// Evaluation context bound to one admitted request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvalContext {
    /// Arithmetic model.
    pub arithmetic: ArithmeticModel,
    /// Maximum expression steps in one evaluation.
    pub max_steps: usize,
}

/// Result of one backend evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalOutcome {
    /// A normal or abnormal observation.
    Value(Value),
    /// The per-evaluation step bound was exhausted.
    Exhausted,
    /// The evaluator reached an unsupported construct or malformed call.
    Unsupported {
        /// Unsupported reason.
        reason: UnsupportedReason,
        /// Detail for the receipt.
        detail: String,
    },
}

/// Backend failure, reported as `tool-error` rather than a semantic conflict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendError {
    /// Backend error message.
    pub message: String,
}

/// Injected evaluation backend.
pub trait Backend {
    /// Return the stable backend identity.
    fn identity(&self) -> BackendIdentity;

    /// Evaluate one logical item over concrete arguments.
    ///
    /// # Errors
    ///
    /// Returns [`BackendError`] when the backend itself fails.
    fn evaluate(
        &self,
        program: &ResolvedProgram,
        item: &str,
        args: &[Value],
        context: &EvalContext,
    ) -> Result<EvalOutcome, BackendError>;
}

/// Deterministic in-memory scalar evaluator.
#[derive(Clone, Copy, Debug, Default)]
pub struct IrBackend;

impl IrBackend {
    /// Stable backend identifier.
    pub const ID: &'static str = "typed-scalar-ir-evaluator";
    /// Stable backend version.
    pub const VERSION: &'static str = "1";
}

impl Backend for IrBackend {
    fn identity(&self) -> BackendIdentity {
        BackendIdentity {
            id: Self::ID.to_owned(),
            version: Self::VERSION.to_owned(),
        }
    }

    fn evaluate(
        &self,
        program: &ResolvedProgram,
        item: &str,
        args: &[Value],
        context: &EvalContext,
    ) -> Result<EvalOutcome, BackendError> {
        let mut steps = 0usize;
        Ok(eval_item(program, item, args, context, &mut steps))
    }
}

#[derive(Clone, Debug)]
enum Step {
    Value(Value),
    Exhausted,
    Unsupported { reason: UnsupportedReason, detail: String },
}

fn eval_item(
    program: &ResolvedProgram,
    item: &str,
    args: &[Value],
    context: &EvalContext,
    steps: &mut usize,
) -> EvalOutcome {
    let Some(function) = program.get(item) else {
        return EvalOutcome::Unsupported {
            reason: UnsupportedReason::UndefinedDependency,
            detail: format!("evaluator could not resolve logical item `{item}`"),
        };
    };
    if usize::from(function.params) != args.len() {
        return EvalOutcome::Unsupported {
            reason: UnsupportedReason::SignatureMismatch,
            detail: format!(
                "logical item `{item}` expects {} argument(s) but received {}",
                function.params,
                args.len()
            ),
        };
    }
    let args = args
        .iter()
        .map(|value| match value {
            Value::Int(value) => Ok(*value),
            Value::Aborted(kind) => Err(*kind),
        })
        .collect::<Result<Vec<_>, _>>();
    match args {
        Ok(args) => match eval_expr(&function.body, program, &args, context, steps) {
            Step::Value(value) => EvalOutcome::Value(value),
            Step::Exhausted => EvalOutcome::Exhausted,
            Step::Unsupported { reason, detail } => EvalOutcome::Unsupported { reason, detail },
        },
        Err(kind) => EvalOutcome::Value(Value::Aborted(kind)),
    }
}

fn eval_expr(expr: &Expr, program: &ResolvedProgram, args: &[i64], context: &EvalContext, steps: &mut usize) -> Step {
    *steps = steps.saturating_add(1);
    if *steps > context.max_steps {
        return Step::Exhausted;
    }
    match expr {
        Expr::Int(value) => Step::Value(Value::Int(*value)),
        Expr::Arg(index) => match args.get(usize::from(*index)) {
            Some(value) => Step::Value(Value::Int(*value)),
            None => Step::Unsupported {
                reason: UnsupportedReason::InvalidRequest,
                detail: format!("argument index {index} is outside the reviewed signature"),
            },
        },
        Expr::Add(left, right) => binary(left, right, program, args, context, steps, context.arithmetic, BinaryOp::Add),
        Expr::Sub(left, right) => binary(left, right, program, args, context, steps, context.arithmetic, BinaryOp::Sub),
        Expr::Mul(left, right) => binary(left, right, program, args, context, steps, context.arithmetic, BinaryOp::Mul),
        Expr::Div(left, right) => binary(left, right, program, args, context, steps, context.arithmetic, BinaryOp::Div),
        Expr::Rem(left, right) => binary(left, right, program, args, context, steps, context.arithmetic, BinaryOp::Rem),
        Expr::Neg(value) => match eval_expr(value, program, args, context, steps) {
            Step::Value(Value::Int(value)) => Step::Value(Value::Int(match context.arithmetic {
                ArithmeticModel::Wrapping => value.wrapping_neg(),
                ArithmeticModel::Checked => match value.checked_neg() {
                    Some(value) => value,
                    None => return Step::Value(Value::Aborted(AbortKind::Overflow)),
                },
            })),
            other => other,
        },
        Expr::If(test, then_value, else_value) => {
            let test = match eval_expr(test, program, args, context, steps) {
                Step::Value(Value::Int(value)) => value,
                Step::Value(Value::Aborted(kind)) => return Step::Value(Value::Aborted(kind)),
                other => return other,
            };
            if test == 0 {
                eval_expr(else_value, program, args, context, steps)
            } else {
                eval_expr(then_value, program, args, context, steps)
            }
        }
        Expr::Call { item, args: call_args } => {
            let mut values = Vec::with_capacity(call_args.len());
            for arg in call_args {
                match eval_expr(arg, program, args, context, steps) {
                    Step::Value(value) => values.push(value),
                    other => return other,
                }
            }
            match eval_item(program, item, &values, context, steps) {
                EvalOutcome::Value(value) => Step::Value(value),
                EvalOutcome::Exhausted => Step::Exhausted,
                EvalOutcome::Unsupported { reason, detail } => Step::Unsupported { reason, detail },
            }
        }
        Expr::Unsupported { construct } => Step::Unsupported {
            reason: UnsupportedReason::UnsupportedConstruct,
            detail: format!("unsupported construct `{}`", construct.as_str()),
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

fn binary(
    left: &Expr,
    right: &Expr,
    program: &ResolvedProgram,
    args: &[i64],
    context: &EvalContext,
    steps: &mut usize,
    arithmetic: ArithmeticModel,
    operation: BinaryOp,
) -> Step {
    let left = match eval_expr(left, program, args, context, steps) {
        Step::Value(Value::Int(value)) => value,
        Step::Value(Value::Aborted(kind)) => return Step::Value(Value::Aborted(kind)),
        other => return other,
    };
    let right = match eval_expr(right, program, args, context, steps) {
        Step::Value(Value::Int(value)) => value,
        Step::Value(Value::Aborted(kind)) => return Step::Value(Value::Aborted(kind)),
        other => return other,
    };
    let result = match (operation, arithmetic) {
        (BinaryOp::Add, ArithmeticModel::Wrapping) => Some(left.wrapping_add(right)),
        (BinaryOp::Sub, ArithmeticModel::Wrapping) => Some(left.wrapping_sub(right)),
        (BinaryOp::Mul, ArithmeticModel::Wrapping) => Some(left.wrapping_mul(right)),
        (BinaryOp::Div, ArithmeticModel::Wrapping) => {
            if right == 0 {
                return Step::Value(Value::Aborted(AbortKind::DivideByZero));
            }
            Some(left.wrapping_div(right))
        }
        (BinaryOp::Rem, ArithmeticModel::Wrapping) => {
            if right == 0 {
                return Step::Value(Value::Aborted(AbortKind::DivideByZero));
            }
            Some(left.wrapping_rem(right))
        }
        (BinaryOp::Add, ArithmeticModel::Checked) => left.checked_add(right),
        (BinaryOp::Sub, ArithmeticModel::Checked) => left.checked_sub(right),
        (BinaryOp::Mul, ArithmeticModel::Checked) => left.checked_mul(right),
        (BinaryOp::Div, ArithmeticModel::Checked) => {
            if right == 0 {
                return Step::Value(Value::Aborted(AbortKind::DivideByZero));
            }
            left.checked_div(right)
        }
        (BinaryOp::Rem, ArithmeticModel::Checked) => {
            if right == 0 {
                return Step::Value(Value::Aborted(AbortKind::DivideByZero));
            }
            left.checked_rem(right)
        }
    };
    match result {
        Some(value) => Step::Value(Value::Int(value)),
        None => Step::Value(Value::Aborted(AbortKind::Overflow)),
    }
}
