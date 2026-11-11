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

// Should NOT warn (or crash): `Self::MAX` names an enum variant, not an integer extreme.
#[derive(Clone, Copy)]
enum Level {
    MIN,
    MAX,
}

impl Level {
    fn or_highest(maybe_level: Option<Level>) -> Level {
        maybe_level.unwrap_or(Self::MAX)
    }

    fn or_lowest(maybe_level: Option<Level>) -> Level {
        maybe_level.unwrap_or(Self::MIN)
    }
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

type Ticks = u64;

fn aliased_max(maybe_ticks: Option<Ticks>) -> Ticks {
    // Should warn: `Ticks::MAX` resolves to `u64::MAX`
    maybe_ticks.unwrap_or(Ticks::MAX)
}

fn qualified_min(raw_index: i128) -> i64 {
    // Should warn: `<i64>::MIN` resolves to `i64::MIN`
    i64::try_from(raw_index).unwrap_or(<i64>::MIN)
}

trait Ceiling {
    fn or_ceiling(maybe: Option<Self>) -> Self
    where
        Self: Sized;
}

impl Ceiling for u32 {
    fn or_ceiling(maybe: Option<u32>) -> u32 {
        // Should warn: `Self::MAX` resolves to `u32::MAX`
        maybe.unwrap_or(Self::MAX)
    }
}

struct Limits;

impl Limits {
    const MAX: u64 = 10;
}

fn user_const_named_max(maybe: Option<u64>) -> u64 {
    // Should NOT warn: `Limits::MAX` is a user constant, not an integer extreme
    maybe.unwrap_or(Limits::MAX)
}
