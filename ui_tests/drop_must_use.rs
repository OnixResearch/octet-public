#![allow(dead_code)]

use std::mem::drop;

#[must_use]
struct Token;

struct Plain;

#[must_use]
fn build_token() -> Token {
    Token
}

fn fallible() -> Result<String, String> {
    Ok("ready".to_owned())
}

fn dropped_result() {
    let result = fallible();
    drop(result);
}

fn dropped_must_use_return_value() {
    drop(build_token());
}

fn dropped_non_must_use_value() {
    let plain = Plain;
    drop(plain);
}

#[cfg(test)]
fn test_context_dropped_result() {
    let result = fallible();
    drop(result);
}

fn main() {}
