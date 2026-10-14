fn multiple_uses() {
    let active = true;
    let _a = active;
    let _b = active;
    let _c = active;
}

fn main() {
    multiple_uses();
}
