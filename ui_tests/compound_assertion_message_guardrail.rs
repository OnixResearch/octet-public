fn check_bounds(a: u32, b: u32) {
    assert!(a > 0 && b > 0, "bounds broke");
}

fn main() {
    check_bounds(1, 2);
}
