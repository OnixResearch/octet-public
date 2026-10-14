#![warn(primitive_domain_alias)]

// Should not warn: this raw alias is inside a declared compatibility scope.
type FixtureUnitId = String;

fn main() {
    let value: FixtureUnitId = String::new();
    let _ = value;
}
