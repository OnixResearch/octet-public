#![warn(compound_condition)]

fn complex_condition(a: bool, b: bool, c: bool, d: bool) -> bool {
    // Should warn: 3+ boolean operators
    if a && b && c && d {
        return true;
    }
    false
}

fn simple_condition(a: bool, b: bool) -> bool {
    // Should NOT warn: only 1 operator
    if a && b {
        return true;
    }
    false
}

fn simple_leaves(a: bool, count: u32, limit: u32, b: bool) -> bool {
    // Should warn with suggestion: later leaves have no side effects
    if a && count > limit && !b && count != limit {
        return true;
    }
    false
}

fn parenthesized_leaf(a: bool, b: bool, c: bool, d: bool, e: bool) -> bool {
    // Should warn with suggestion: the parenthesized `||` stays one predicate
    if a && (b || c) && d && e {
        return true;
    }
    false
}

fn mixed_operators(a: bool, b: bool, c: bool, d: bool) -> bool {
    // Should warn without suggestion: `&&` and `||` cannot be rejoined with one operator
    if a && b || c && d {
        return true;
    }
    false
}

fn short_circuit_guard(values: &[u32], a: bool, b: bool) -> bool {
    // Should warn without suggestion: hoisting `values[0]` would panic on empty input
    if !values.is_empty() && values[0] == 1 && a && b {
        return true;
    }
    false
}

fn else_if_branch(a: bool, b: bool, c: bool, d: bool) -> bool {
    // Should warn without suggestion: `let` bindings cannot follow `else`
    if !a {
        return false;
    } else if a && b && c && d {
        return true;
    }
    false
}

fn expression_position(a: bool, b: bool, c: bool, d: bool) -> u8 {
    // Should warn without suggestion: the `if` is a `let` initializer
    let value = if a && b && c && d { 1 } else { 0 };
    value
}

fn let_chain(value: Option<u8>, a: bool, b: bool, c: bool) -> bool {
    // Should warn without suggestion: `let` leaves cannot be hoisted
    if let Some(_) = value && a && b && c {
        return true;
    }
    false
}

fn main() {
    complex_condition(true, true, true, true);
    simple_condition(true, true);
    simple_leaves(true, 2, 1, false);
    parenthesized_leaf(true, false, true, true, true);
    mixed_operators(true, true, false, false);
    short_circuit_guard(&[], true, true);
    else_if_branch(true, true, true, true);
    expression_position(true, true, true, true);
    let_chain(Some(1), true, true, true);
}
