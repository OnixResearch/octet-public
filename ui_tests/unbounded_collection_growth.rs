#![allow(explicit_defaults, no_unwrap, unbounded_loop)]

use std::collections::BTreeMap;
use std::collections::HashMap;

fn keep_going() -> bool {
    true
}

fn next_byte() -> u8 {
    1
}

fn push_without_bound() {
    let mut items = Vec::new();
    loop {
        items.push(next_byte());
    }
}

fn insert_without_bound() {
    let mut items: HashMap<u8, u8> = Default::default();
    while keep_going() {
        items.insert(next_byte(), next_byte());
    }
}

fn extend_without_bound() {
    let mut items = Vec::new();
    while keep_going() {
        items.extend([next_byte()]);
    }
}

fn push_in_for_without_bound() {
    let mut items = Vec::new();
    for value in 0..10u8 {
        items.push(value);
    }
}

fn with_capacity_is_allowed() {
    let mut items = Vec::with_capacity(16);
    loop {
        items.push(next_byte());
    }
}

fn reserve_exact_is_allowed() {
    let mut items = Vec::new();
    items.reserve_exact(16);
    while keep_going() {
        items.push(next_byte());
    }
}

fn try_reserve_is_allowed() {
    let mut items = Vec::new();
    items.try_reserve(16).expect("reserve works");
    while keep_going() {
        items.push(next_byte());
    }
}

fn guarded_by_len_is_allowed(limit: usize) {
    let mut items: HashMap<u8, u8> = Default::default();
    while items.len() < limit {
        items.insert(next_byte(), next_byte());
    }
}

fn guarded_by_break_is_allowed(limit: usize) {
    let mut items = Vec::new();
    loop {
        if items.len() >= limit {
            break;
        }
        items.push(next_byte());
    }
}

fn outer_binding_is_ignored() {
    let mut items = BTreeMap::new();
    {
        while keep_going() {
            items.insert(next_byte(), next_byte());
        }
    }
}

fn main() {}
