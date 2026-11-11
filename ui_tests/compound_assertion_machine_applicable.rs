fn split_pair(a: u32, b: u32) {
    assert!(a > 0 && b > 0);
}

fn split_three(a: u32, b: u32, c: u32) {
    assert!(a > 0 && b > 0 && c > 0);
}

fn split_in_match_arm(v: Option<u32>) {
    match v {
        Some(x) => assert!(x > 0 && x < 9),
        None => {}
    }
}

fn split_in_closure(a: bool, b: u32) {
    let check = || assert!(a && b > 0);
    check();
}

fn split_double_reference(items: &[&u32], expected: u32, ready: bool) {
    for item in items {
        assert!(item == &&expected && ready);
    }
}

fn main() {
    split_pair(1, 2);
    split_three(1, 2, 3);
    split_in_match_arm(Some(1));
    split_in_closure(true, 1);
    split_double_reference(&[&1], 1, true);
}

