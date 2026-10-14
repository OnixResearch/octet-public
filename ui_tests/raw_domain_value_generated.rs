#![warn(raw_domain_value)]

mod generated {
    // Should not warn: generated representation scope is an explicit exception.
    pub struct RawDomainFixture {
        pub session: String,
    }
}

fn main() {
    let generated = generated::RawDomainFixture { session: String::new() };
    let _ = generated.session;
}
