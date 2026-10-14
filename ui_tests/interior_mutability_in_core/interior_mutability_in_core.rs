struct CoreState {
    value: std::cell::RefCell<u64>,
    permits: std::sync::atomic::AtomicUsize,
}

fn build_state() -> CoreState {
    CoreState {
        value: std::cell::RefCell::new(0),
        permits: std::sync::atomic::AtomicUsize::new(0),
    }
}

fn main() {
    let _state = build_state();
}
