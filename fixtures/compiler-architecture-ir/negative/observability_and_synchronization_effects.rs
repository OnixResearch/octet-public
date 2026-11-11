use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

pub fn print_value(value: u32) {
    println!("{value}");
}

pub fn print_error(value: u32) {
    eprintln!("{value}");
}

pub fn write_stdout(bytes: &[u8]) -> std::io::Result<()> {
    std::io::stdout().write_all(bytes)
}

pub fn locked(counter: &Mutex<u32>) -> u32 {
    *counter.lock().expect("unpoisoned counter")
}

pub fn atomic(counter: &AtomicU32) -> u32 {
    counter.fetch_add(1, Ordering::SeqCst)
}

pub fn channel() -> u32 {
    let (sender, receiver) = std::sync::mpsc::channel();
    sender.send(1).expect("open channel");
    receiver.recv().expect("sent value")
}

pub fn shared(value: u32) -> Arc<u32> {
    Arc::new(value)
}

pub fn formatted(value: u32) -> String {
    format!("{value}")
}

const INIT: AtomicU32 = AtomicU32::new(0);
pub static SLOTS: [AtomicU32; 4] = [INIT; 4];
pub static COUNTER: Mutex<u32> = Mutex::new(0);
pub const SLOT_POINTER_IS_NULL: bool = AtomicU32::new(0).as_ptr().is_null();

pub fn locked_static() -> u32 {
    *COUNTER.lock().expect("unpoisoned counter")
}

pub fn exclusive(counter: &mut AtomicU32, guarded: Mutex<u32>) -> u32 {
    *counter.get_mut() + guarded.into_inner().expect("unpoisoned counter")
}

pub fn constructed() -> Mutex<u32> {
    Mutex::new(0)
}

pub struct Config {
    pub port: u16,
}

fn load() -> Config {
    Config { port: 8080 }
}

pub static CONFIG: std::sync::LazyLock<Config> = std::sync::LazyLock::new(load);

pub fn lazy_port() -> u16 {
    CONFIG.port
}

pub fn guarded_increment(guard: &std::sync::MutexGuard<'_, u32>) -> u32 {
    **guard + 1
}

pub struct Ticks(pub u32);

impl std::ops::Add for Ticks {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

pub fn add_ticks(left: Ticks, right: Ticks) -> Ticks {
    left + right
}
