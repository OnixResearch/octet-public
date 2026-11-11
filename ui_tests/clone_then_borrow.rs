#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::borrowed_argument_types)]

fn emit(value: &String) -> usize {
    value.len()
}

fn sum(values: &Vec<u8>) -> u32 {
    values.iter().map(|value| u32::from(*value)).sum()
}

fn takes_owned(value: String) -> usize {
    value.len()
}

struct Store {
    name: String,
}

fn borrowed_string_clone(name: &String) -> usize {
    emit(&name.clone())
}

fn borrowed_string_clone_parenthesized(name: &String) -> usize {
    emit(&(name.clone()))
}

fn borrowed_vec_clone(bytes: &Vec<u8>) -> u32 {
    sum(&bytes.clone())
}

fn owned_clone_transfer(name: &String) -> usize {
    takes_owned(name.clone())
}

fn stored_clone(name: &String) -> Store {
    Store { name: name.clone() }
}

fn mutate_independent_clone(name: &String) -> usize {
    let mut owned = name.clone();
    owned.push('!');
    emit(&owned)
}

fn snapshot_before_push(values: &mut Vec<u8>) -> u32 {
    // Should NOT warn: `values` is mutated while the borrowed snapshot is live,
    // so `&values` would not compile (E0502).
    let before = &values.clone();
    values.push(1);
    sum(before) + sum(values)
}

fn snapshot_before_field_mutation(mut store: Store) -> usize {
    // Should NOT warn: the stored snapshot outlives a mutation of `store.name`.
    let before = &store.name.clone();
    store.name.push('!');
    emit(before) + emit(&store.name)
}

fn snapshot_before_reassignment(mut name: String) -> usize {
    // Should NOT warn: `name` is reassigned while the snapshot is live.
    let before = &name.clone();
    name = String::from("lion");
    emit(before) + emit(&name)
}

fn snapshot_in_tuple_before_closure_mutation(mut values: Vec<u8>) -> u32 {
    // Should NOT warn: a closure mutates `values` while the tuple keeps the snapshot.
    let pair = (&values.clone(), 1);
    let mut extend = || values.push(2);
    extend();
    sum(pair.0) + pair.1
}

fn stored_borrow_without_mutation(values: &Vec<u8>) -> u32 {
    // Should warn: nothing mutates `values` after the borrow is stored.
    let view = &values.clone();
    sum(view)
}

fn mutation_before_stored_borrow(mut values: Vec<u8>) -> u32 {
    values.push(1);
    // Should warn: the mutation happens before the borrow is stored.
    let view = &values.clone();
    sum(view)
}

fn consume(values: Vec<u8>) -> u32 {
    sum(&values)
}

fn snapshot_before_move(values: Vec<u8>) -> u32 {
    // Should NOT warn: `values` is moved while the snapshot is live, so
    // `&values` would not compile (E0505).
    let before = &values.clone();
    let moved = consume(values);
    sum(before) + moved
}

fn snapshot_before_guard_drop(lock: &std::sync::Mutex<Vec<u8>>) -> u32 {
    // Should NOT warn: the snapshot detaches from the guard, which is dropped
    // while the snapshot is live.
    let guard = lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let snapshot = &guard.clone();
    drop(guard);
    sum(snapshot)
}

fn detach_from_temporary_guard(lock: &std::sync::Mutex<String>) -> usize {
    // Should NOT warn: the clone detaches from a temporary lock guard instead of
    // holding the lock while `emit` runs.
    emit(&lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone())
}

fn clone_through_held_guard(lock: &std::sync::Mutex<String>) -> usize {
    // Should warn: the guard is held anyway, so borrowing it avoids the clone.
    let guard = lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    emit(&guard.clone())
}

fn stored_borrow_then_shared_use(values: Vec<u8>) -> u32 {
    // Should warn: `values` is only borrowed after the snapshot is stored.
    let view = &values.clone();
    sum(view) + sum(&values)
}

macro_rules! borrowed_clone_macro {
    ($value:expr) => {
        emit(&$value.clone())
    };
}

fn macro_generated_clone(name: &String) -> usize {
    borrowed_clone_macro!(name)
}

fn main() {
    let name = String::from("tiger");
    let bytes = vec![1, 2, 3];
    let _ = borrowed_string_clone(&name);
    let _ = borrowed_string_clone_parenthesized(&name);
    let _ = borrowed_vec_clone(&bytes);
    let _ = owned_clone_transfer(&name);
    let store = stored_clone(&name);
    let _ = store.name.len();
    let _ = mutate_independent_clone(&name);
    let mut snapshot_values = vec![1, 2];
    let _ = snapshot_before_push(&mut snapshot_values);
    let _ = snapshot_before_field_mutation(Store { name: name.clone() });
    let _ = snapshot_before_reassignment(name.clone());
    let _ = snapshot_in_tuple_before_closure_mutation(bytes.clone());
    let _ = stored_borrow_without_mutation(&bytes);
    let _ = mutation_before_stored_borrow(bytes.clone());
    let _ = snapshot_before_move(bytes.clone());
    let byte_lock = std::sync::Mutex::new(bytes.clone());
    let _ = snapshot_before_guard_drop(&byte_lock);
    let name_lock = std::sync::Mutex::new(name.clone());
    let _ = detach_from_temporary_guard(&name_lock);
    let _ = clone_through_held_guard(&name_lock);
    let _ = stored_borrow_then_shared_use(bytes.clone());
    let _ = macro_generated_clone(&name);
}
