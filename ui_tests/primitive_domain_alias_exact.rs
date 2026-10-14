#![warn(primitive_domain_alias)]

// Should warn: this exact core domain still aliases raw storage.
type FixtureUnitId = String;

fn main() {
    let value: FixtureUnitId = String::new();
    let _ = value;
}
