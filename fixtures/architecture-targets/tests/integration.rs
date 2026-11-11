#[test]
fn integration_effect() {
    let _ = std::fs::read("never-executed");
}
