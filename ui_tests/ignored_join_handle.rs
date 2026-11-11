const STATEMENT_RESULT: u8 = 1;
const WILDCARD_RESULT: u8 = 2;
const JOINED_RESULT: u8 = 3;
const BUILDER_RESULT: u8 = 4;
const OPTIONAL_RESULT: u8 = 5;
const STORED_KEY: u8 = 6;
const STORED_RESULT: u8 = 7;

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

fn discard_thread_builder_spawn() {
    // Should warn: `Builder::spawn` wraps the handle in `io::Result`.
    let _ = std::thread::Builder::new().spawn(|| BUILDER_RESULT);
}

fn discard_optional_handle(spawn: bool) {
    // Should warn: an optional handle is still discarded.
    let _ = spawn.then(|| std::thread::spawn(|| OPTIONAL_RESULT));
}

fn join_builder_handle() {
    // Should NOT warn: the wrapped handle is named and joined explicitly.
    let spawned = std::thread::Builder::new().spawn(|| BUILDER_RESULT);
    if let Ok(handle) = spawned {
        match handle.join() {
            Ok(value) => {
                std::hint::black_box(value);
            },
            Err(error) => std::panic::resume_unwind(error),
        }
    }
}

// Should NOT warn: a `_` async fn parameter is not a discarded handle, even
// though rustc lowers it into `let _ = ...` inside the async body.
async fn on_spawned(_: std::thread::JoinHandle<u8>) {}

fn store_handle_in_map(workers: &mut std::collections::HashMap<u8, std::thread::JoinHandle<u8>>) {
    // Should NOT warn: the new handle is stored; `insert` returns the previous entry.
    workers.insert(STORED_KEY, std::thread::spawn(|| STORED_RESULT));
}

fn discard_borrowed_handle(handles: &[std::thread::JoinHandle<u8>]) {
    // Should NOT warn: dropping a borrowed handle detaches nothing.
    let _ = handles.first();
}

fn main() {
    discard_thread_spawn_statement();
    discard_thread_spawn_wildcard();
    join_thread_handle();
    discard_thread_builder_spawn();
    discard_optional_handle(true);
    join_builder_handle();
    let mut workers = std::collections::HashMap::new();
    store_handle_in_map(&mut workers);
    discard_borrowed_handle(&[]);
    for (_, handle) in workers {
        if let Err(error) = handle.join() {
            std::panic::resume_unwind(error);
        }
    }
}
