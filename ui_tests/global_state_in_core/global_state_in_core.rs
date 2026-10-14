use std::sync::OnceLock;

static mut CORE_COUNTER: u64 = 0;
static CORE_CACHE: OnceLock<String> = OnceLock::new();

fn read_cache() -> Option<&'static String> {
    CORE_CACHE.get()
}

fn main() {
    let _cache = read_cache();
}
