#![warn(raw_domain_value)]

// Should not warn: the same raw field is admitted in a declared wire scope.
struct RawDomainFixture {
    session: String,
}

fn main() {
    let wire = RawDomainFixture { session: String::new() };
    let _ = wire.session;
}
