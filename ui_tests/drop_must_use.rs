#![allow(dead_code)]
#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

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

fn dropped_option_value() {
    // `Option` is not `#[must_use]`; dropping it discards nothing required.
    let maybe = Some(Plain);
    drop(maybe);
}

#[allow(dropping_references)]
fn dropped_result_reference() -> bool {
    // Dropping a reference leaves the `Result` itself unconsumed.
    let result = fallible();
    drop(&result);
    result.is_ok()
}

#[cfg(test)]
fn test_context_dropped_result() {
    let result = fallible();
    drop(result);
}

fn main() {}

fn dropped_mutex_guard(lock: &std::sync::Mutex<u32>) -> u32 {
    // Should NOT warn: releasing an RAII guard early is the purpose of `drop`.
    let Ok(guard) = lock.lock() else {
        return 0;
    };
    let value = *guard;
    drop(guard);
    value
}

fn dropped_rwlock_guards(lock: &std::sync::RwLock<u32>) {
    // Should NOT warn: `RwLock` read and write guards are RAII guards.
    if let Ok(reader) = lock.read() {
        drop(reader);
    }
    if let Ok(writer) = lock.write() {
        drop(writer);
    }
}

fn dropped_must_use_token_binding() {
    // Should warn: `Token` is `#[must_use]` and has no destructor to run.
    let token = Token;
    drop(token);
}
