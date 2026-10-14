mod guest {
    pub fn spawn_guest_vm() {}
}

mod path_segment_repetition {
    // Should NOT warn: the repeated identity ties lint source, fixture, and metadata together.
    pub fn path_segment_repetition_classifier() {}
}

fn main() {
    guest::spawn_guest_vm();
    path_segment_repetition::path_segment_repetition_classifier();
}
