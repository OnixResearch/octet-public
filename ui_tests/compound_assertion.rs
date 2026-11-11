fn check_bounds(a: u32, b: u32) {
    // Should warn: compound assertion with &&
    assert!(a > 0 && b > 0);
}

fn check_either(a: u32, b: u32) {
    // Should warn: compound assertion with ||
    assert!(a > 0 || b > 0);
}

fn check_ok(a: u32, b: u32) {
    // Should NOT warn: separate assertions
    assert!(a > 0);
    assert!(b > 0);
}

fn check_closure(v: Vec<u32>) {
    // Should NOT warn: && is inside a closure, not top-level
    assert!(v.iter().all(|x| *x > 0 && *x < 100));
}

fn check_assert_eq(a: u32, b: u32) {
    // Should NOT warn: assert_eq! is fine
    assert_eq!(a, b);
}

fn check_message_literal(ready: bool) {
    // Should NOT warn: `&&` only appears inside the panic message literal
    assert!(ready, "left && right must hold");
}

fn check_message_args(ready: bool, a: bool, b: bool) {
    // Should NOT warn: `||` only appears in the panic message arguments
    assert!(ready, "state {}", a || b);
}

fn main() {
    check_bounds(1, 2);
    check_either(1, 2);
    check_ok(1, 2);
    check_closure(vec![1, 2, 3]);
    check_assert_eq(1, 1);
    check_message_literal(true);
    check_message_args(true, true, false);
}

fn check_double_reference(items: &[&u32], expected: u32) {
    // Should NOT warn: `&&expected` is a double reference, not a logical AND
    for item in items {
        assert!(item == &&expected);
    }
}

fn check_or_closure(ready: bool, fallback: fn(fn() -> bool) -> bool) {
    // Should warn: top-level `||` even though the right side is a call
    assert!(ready || fallback(|| true));
}

fn check_commented_clauses(ready: bool, done: bool) {
    // Should warn without a split: a comment inside a clause would swallow the `)`
    assert!(
        ready // set by init
            && done
    );
    assert!(ready /* set by init */ && done);
}
