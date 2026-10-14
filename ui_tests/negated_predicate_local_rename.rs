fn multiple_uses() {
    let is_not_ready = false;
    let _a = is_not_ready;
    let _b = is_not_ready;
    let _c = !is_not_ready;
}

fn main() {
    multiple_uses();
}
