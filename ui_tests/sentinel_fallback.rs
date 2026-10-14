#![warn(sentinel_fallback)]
#![allow(fallible_int_fallback)]

fn formatted_recent(command_index: i128) -> String {
    format!("recent_{}", u64::try_from(command_index).unwrap_or(u64::MAX))
}

fn optional_index(maybe_index: Option<u32>) -> u32 {
    maybe_index.unwrap_or(u32::MAX)
}

fn closure_min(raw_index: i128) -> i64 {
    i64::try_from(raw_index).unwrap_or_else(|err| {
        eprintln!("{err}");
        i64::MIN
    })
}

fn explicit_branch(command_index: i128) -> String {
    match u64::try_from(command_index) {
        Ok(index) => format!("recent_{index}"),
        Err(_) => "recent_missing_index".to_string(),
    }
}

fn non_extreme_default(maybe_index: Option<u32>) -> u32 {
    maybe_index.unwrap_or(0)
}

fn non_extreme_closure(raw_index: i128) -> i64 {
    i64::try_from(raw_index).unwrap_or_else(|_| 0)
}

#[cfg(test)]
fn test_only(raw_index: i128) -> u64 {
    u64::try_from(raw_index).unwrap_or(u64::MAX)
}

fn main() {
    let _ = formatted_recent(-1);
    let _ = optional_index(None);
    let _ = closure_min(i128::MAX);
    let _ = explicit_branch(-1);
    let _ = non_extreme_default(None);
    let _ = non_extreme_closure(i128::MAX);
    let _ = test_only(-1);
}
