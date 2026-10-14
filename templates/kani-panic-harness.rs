// Copy into a crate-local `#[cfg(kani)]` module and replace the example core.
// Record the matching receipt with `cargo octet evidence collect --rail kani-model-check`.

#[cfg(kani)]
mod kani_panic_freedom {
    const MAX_ADMITTED_INDEX: usize = 16;

    fn admitted_index(index: usize) -> bool {
        index < MAX_ADMITTED_INDEX
    }

    fn core_lookup(index: usize) -> Option<usize> {
        if admitted_index(index) {
            Some(index)
        } else {
            None
        }
    }

    #[kani::proof]
    fn check_index_admission_no_panic() {
        let index: usize = kani::any();
        kani::assume(admitted_index(index));
        let result = core_lookup(index);
        assert!(result.is_some());
    }

    #[kani::proof]
    fn check_invalid_index_is_rejected() {
        let index: usize = kani::any();
        kani::assume(!admitted_index(index));
        let result = core_lookup(index);
        assert!(result.is_none());
    }
}
