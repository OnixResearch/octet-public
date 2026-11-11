mod guest {
    pub fn spawn_vm() {}
}

// Should NOT warn: a binary crate name is the target name, not a path segment
// anyone writes, so `repeating` here does not repeat a path word.
fn repeating_report() {}

fn main() {
    guest::spawn_vm();
}
