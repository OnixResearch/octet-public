// `bounded-systems` raises nominal-domain mode to `deny`, so the inventory cap
// does not apply and no cap warning is printed. Deny mode skips this
// suffix-only discovery candidate.
type SessionId = String;

fn main() {
    let session: SessionId = String::new();
    let _ = session;
}
