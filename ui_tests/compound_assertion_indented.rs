fn nested_split(a: u32, b: u32) {
    if a > 0 {
        assert!(a > 1 && b > 1);
    }
}

fn main() {
    nested_split(2, 3);
}
