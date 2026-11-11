// A library crate: its name `path_segment_repetition` is a path segment users write.
#![crate_type = "lib"]

mod guest {
    pub fn spawn_guest_vm() {}
}

mod storage {
    // Should warn: identifiers in the item body are not documentation.
    pub fn storage_flush() -> u32 {
        let rapid = 1;
        rapid
    }
}

mod path_segment_repetition {
    // Should NOT warn: the repeated identity ties lint source, fixture, and metadata together.
    pub fn path_segment_repetition_classifier() {}
}

mod registry {
    // Should warn: a module named `registry` is ordinary source, not generated code.
    pub fn registry_lookup() {}
}

// Should NOT warn: the module file sits under a `generated/` source directory, which the
// default `generated_searchable_names` exception class keeps exempt.
#[path = "../policy_inputs/generated_searchable_names/path_segment_repetition/generated/items.rs"]
mod generated_items;

// Should warn: the crate name `path_segment_repetition` already supplies `repetition`.
fn repetition_report() {}

mod scheduler {
    // Should warn: crate-name words count for nested items too.
    pub fn next_segment() {}
}

// Should NOT warn: no word repeats the crate name or a module path.
fn flush_all() {}

fn main() {
    guest::spawn_guest_vm();
    let _ = storage::storage_flush();
    path_segment_repetition::path_segment_repetition_classifier();
    registry::registry_lookup();
}
