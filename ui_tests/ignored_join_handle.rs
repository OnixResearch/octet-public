const STATEMENT_RESULT: u8 = 1;
const WILDCARD_RESULT: u8 = 2;
const JOINED_RESULT: u8 = 3;

fn discard_thread_spawn_statement() {
    // Should warn: spawned work is detached without a lifecycle decision.
    std::thread::spawn(|| STATEMENT_RESULT);
}

fn discard_thread_spawn_wildcard() {
    // Should warn: wildcard binding still discards the join handle.
    let _ = std::thread::spawn(|| WILDCARD_RESULT);
}

fn join_thread_handle() {
    // Should NOT warn: the handle is named and joined explicitly.
    let handle = std::thread::spawn(|| JOINED_RESULT);
    match handle.join() {
        Ok(value) => {
            std::hint::black_box(value);
        },
        Err(error) => std::panic::resume_unwind(error),
    }
}

fn main() {
    discard_thread_spawn_statement();
    discard_thread_spawn_wildcard();
    join_thread_handle();
}
