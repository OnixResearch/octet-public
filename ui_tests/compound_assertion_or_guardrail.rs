fn check_either(a: u32, b: u32) {
    assert!(a > 0 || b > 0);
}

fn main() {
    check_either(1, 2);
}
