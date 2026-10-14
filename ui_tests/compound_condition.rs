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

fn main() {
    complex_condition(true, true, true, true);
    simple_condition(true, true);
}
