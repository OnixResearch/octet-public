#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(tigerstyle::nondeterministic_output)]
#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::collections::HashMap;

const SAMPLE_SCORE: u32 = 1;

fn names_by_hash_iteration(scores: &HashMap<String, u32>) -> Vec<String> {
    scores.keys().cloned().collect::<Vec<_>>()
}

fn main() {
    let mut scores = HashMap::new();
    scores.insert("alice".to_owned(), SAMPLE_SCORE);
    let _names = names_by_hash_iteration(&scores);
}
