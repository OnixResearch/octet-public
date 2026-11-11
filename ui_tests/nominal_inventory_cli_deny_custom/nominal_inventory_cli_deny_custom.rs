// The command line passes `-D primitive_domain_alias` in inventory mode.

// Should not error: suffix-only discovery candidates never fail a build, so
// the candidate is skipped while the effective level is above warn.
type SessionId = String;

#[warn(primitive_domain_alias)]
mod reviewed {
    // Should warn: the same kind of candidate still reports at warn level.
    pub type RequestId = String;
}

fn main() {
    let session: SessionId = String::new();
    let request: reviewed::RequestId = String::new();
    let _ = (session, request);
}
