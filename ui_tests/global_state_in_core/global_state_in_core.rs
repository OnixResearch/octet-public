#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::cell::Cell;
use std::cell::RefCell;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;

static mut CORE_COUNTER: u64 = 0;
static CORE_CACHE: OnceLock<String> = OnceLock::new();
static CORE_LOCK: Mutex<u64> = Mutex::new(0);
static CORE_HITS: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static CORE_SCRATCH: Cell<u64> = const { Cell::new(0) };
    static CORE_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

// Nested declarations are reported once at the static, not again at the
// enclosing module or function.
mod nested_state {
    pub static mut NESTED_COUNTER: u64 = 0;
}

fn local_state() -> u64 {
    static mut LOCAL_COUNTER: u64 = 0;
    0
}

// Immutable statics, and `'static` borrows next to lock or atomic values, are
// not global state.
static GREETING: &str = "hello";

fn read_cache() -> Option<&'static String> {
    CORE_CACHE.get()
}

fn lock_label() -> &'static str {
    let _lock: Mutex<u8> = Mutex::new(0);
    "locked"
}

fn atomic_label() -> &'static str {
    let _hits: AtomicU64 = AtomicU64::new(0);
    "counted"
}

fn main() {
    let _cache = read_cache();
    let _local = local_state();
    let _labels = (GREETING, lock_label(), atomic_label());
}
