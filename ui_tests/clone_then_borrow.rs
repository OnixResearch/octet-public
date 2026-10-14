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
    let _ = macro_generated_clone(&name);
}
