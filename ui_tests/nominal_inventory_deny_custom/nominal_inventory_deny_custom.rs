// Inventory mode caps nominal-domain lints at warn, so this suffix-only
// discovery candidate warns instead of failing the build.
type SessionId = String;

fn main() {
    let session: SessionId = String::new();
    let _ = session;
}
