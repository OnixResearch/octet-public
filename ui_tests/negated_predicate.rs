struct State;

fn is_not_ready(_state: &State) -> bool {
    false
}

fn has_no_capacity() -> bool {
    true
}

fn is_ready(_state: &State) -> bool {
    true
}

fn notify(_state: &State) -> bool {
    true
}

fn not(value: bool) -> bool {
    !value
}

fn locals() {
    let no_capacity = true;
    let no_progress: usize = 3;
    let _ = (no_capacity, no_progress);
}

fn main() {
    let state = State;
    let _ = is_not_ready(&state);
    let _ = has_no_capacity();
    let _ = is_ready(&state);
    let _ = notify(&state);
    let _ = not(true);
    locals();
}
