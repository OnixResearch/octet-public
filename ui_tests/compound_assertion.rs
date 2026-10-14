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

fn main() {
    check_bounds(1, 2);
    check_either(1, 2);
    check_ok(1, 2);
    check_closure(vec![1, 2, 3]);
    check_assert_eq(1, 1);
}
