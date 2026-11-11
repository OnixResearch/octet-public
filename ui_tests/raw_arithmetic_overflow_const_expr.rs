#![warn(raw_arithmetic_overflow)]

const MAX_BYTES: u64 = 4_194_304;
const MAX_REGION_BYTES: u64 = 65_536;
const MEMORY_BYTES: u32 = 1_048_576;
const HEAP_START: u32 = 65_536;
const MAX_ALLOCATION: u32 = (MEMORY_BYTES - HEAP_START) as u32;

struct Limits;
impl Limits {
    const SIZE: u64 = MAX_BYTES;
}

fn compile_time_only() -> (u64, u64, u32) {
    (
        (MAX_BYTES + 1) * 2,
        Limits::SIZE + MAX_REGION_BYTES * 2,
        (!MEMORY_BYTES + 1) - HEAP_START,
    )
}

const fn runtime_in_const_fn(value: u64) -> u64 {
    value + 1
}

#[allow(non_snake_case)]
fn runtime_values(value: u64) -> (u64, u64, u64) {
    let bound = MAX_BYTES;
    let CONST_LOOKING = 7_u64;
    (value + 1, bound + MAX_REGION_BYTES, CONST_LOOKING + 1)
}

static GLOBAL: u64 = 7;

fn runtime_static() -> u64 {
    GLOBAL + 1
}

const RAW_PTR: *const u8 = std::ptr::null();

fn pointer_cast() -> usize {
    (RAW_PTR as usize) + 1
}

fn main() {
    let _ = MAX_ALLOCATION;
    let _ = compile_time_only();
    let _ = runtime_in_const_fn(1);
    let _ = runtime_values(1);
    let _ = runtime_static();
    let _ = pointer_cast();
    let _ = generic_limit::<Small>();
    let _ = generic_limit_const_fn::<Small>();
    let _ = generic_inline_const::<Small>();
    let _ = dynamic_limit(DYNAMIC_LIMIT);
}

trait Limit {
    const MAX: u8;
}

struct Small;
impl Limit for Small {
    const MAX: u8 = 2;
}

fn generic_limit<T: Limit>() -> u8 {
    T::MAX + 1
}

const fn generic_limit_const_fn<T: Limit>() -> u8 {
    T::MAX + 1
}

fn generic_inline_const<T: Limit>() -> u8 {
    (const { T::MAX }) + 1
}

trait RuntimeLimit {
    fn max(&self) -> u8;
}

struct DynamicLimit;
impl RuntimeLimit for DynamicLimit {
    fn max(&self) -> u8 {
        2
    }
}

const DYNAMIC_LIMIT: &dyn RuntimeLimit = &DynamicLimit;

fn dynamic_limit(value: &dyn RuntimeLimit) -> u8 {
    value.max() + 1
}
