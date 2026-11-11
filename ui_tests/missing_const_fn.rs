#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(tigerstyle::missing_const_fn)]

const SCALE_FACTOR: u64 = 2;

fn scale_value(value: u64) -> u64 {
    value * SCALE_FACTOR
}

const fn const_offset(value: u64) -> u64 {
    value
}

fn offset_scaled_value(value: u64) -> u64 {
    const_offset(value) * SCALE_FACTOR
}

struct Doubler;

impl Doubler {
    fn twice(value: u64) -> u64 {
        value * SCALE_FACTOR
    }
}

// Trait methods cannot be `const` (E0379), and helpers that call non-const
// functions are not const-compatible.
trait Scale {
    fn scale(&self, value: u64) -> u64;

    fn halve(&self, value: u64) -> u64 {
        value / SCALE_FACTOR
    }
}

impl Scale for Doubler {
    fn scale(&self, value: u64) -> u64 {
        value * SCALE_FACTOR
    }
}

fn runtime_offset(value: u64) -> u64 {
    std::hint::black_box(value)
}

fn offset_then_scale(value: u64) -> u64 {
    runtime_offset(value) * SCALE_FACTOR
}

fn sum_then_scale(values: &[u64]) -> u64 {
    values.iter().sum::<u64>() * SCALE_FACTOR
}

// Non-const string parsing and iteration remain ineligible even when the body
// contains comparisons and arithmetic tokens.
fn identifier(value: &str) -> bool {
    value.len() < 256 && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'-')
}

fn versioned_interface(value: &str) -> bool {
    if value.len() > 256 {
        return false;
    }
    let Some((path, version)) = value.split_once('@') else {
        return false;
    };
    identifier(path) && version.parse::<u32>().is_ok()
}

// A generic overloaded operator is trait-resolved, not a built-in primitive
// operation.
fn overloaded_add<T: std::ops::Add<Output = T>>(left: T, right: T) -> T {
    left + right
}

static SHARED_SCALE: u64 = SCALE_FACTOR;

fn reads_static(value: u64) -> u64 {
    value * SHARED_SCALE
}

fn main() {
    let _scaled = scale_value(SCALE_FACTOR);
    let _offset = offset_scaled_value(SCALE_FACTOR);
    let _twice = Doubler::twice(SCALE_FACTOR);
    let _trait_scaled = Doubler.scale(SCALE_FACTOR);
    let _halved = Doubler.halve(SCALE_FACTOR);
    let _runtime = offset_then_scale(SCALE_FACTOR);
    let _sum = sum_then_scale(&[SCALE_FACTOR]);
    let _identifier = identifier("value");
    let _versioned = versioned_interface("path@1");
    let _overloaded = overloaded_add(SCALE_FACTOR, SCALE_FACTOR);
    let _static = reads_static(SCALE_FACTOR);
}
