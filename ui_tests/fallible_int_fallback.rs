#![allow(dead_code)]

fn zero_fallback(count: u64) -> u32 {
    u32::try_from(count).unwrap_or(0)
}

fn max_fallback(count: u64) -> u8 {
    u8::try_from(count).unwrap_or(u8::MAX)
}

fn min_fallback(count: i64) -> i32 {
    i32::try_from(count).unwrap_or(i32::MIN)
}

fn default_fallback(count: u64) -> u16 {
    u16::try_from(count).unwrap_or_default()
}

fn ordinary_result_fallback(input: Result<u32, &'static str>) -> u32 {
    input.unwrap_or(0)
}

fn explicit_error_handling(count: u64) -> Result<u32, &'static str> {
    match u32::try_from(count) {
        Ok(value) => Ok(value),
        Err(_) => Err("count does not fit"),
    }
}

fn main() {}
