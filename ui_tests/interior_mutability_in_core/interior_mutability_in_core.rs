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

fn build_wrapped_state() -> (Option<std::cell::Cell<u64>>, std::sync::Arc<std::sync::Mutex<u64>>) {
    (Some(std::cell::Cell::new(0)), std::sync::Arc::new(std::sync::Mutex::new(0)))
}

fn main() {
    let _state = build_state();
    let _wrapped = build_wrapped_state();
}
