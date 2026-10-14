#![warn(primitive_domain_alias)]

// Should not warn without explicit inventory mode: suffixes never become hard policy.
type SessionId = String;

// Should not warn: this alias has no registered domain suffix.
type DisplayName = String;

// Should not warn: a nominal wrapper does not erase its type identity.
struct SafeId(String);

fn main() {
    let session: SessionId = String::new();
    let display: DisplayName = String::new();
    let safe = SafeId(String::new());
    let _ = (session, display, safe.0);
}
