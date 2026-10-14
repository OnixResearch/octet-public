#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(tigerstyle::nondeterministic_output)]

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
