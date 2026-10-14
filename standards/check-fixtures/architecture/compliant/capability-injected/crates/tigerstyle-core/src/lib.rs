#![no_std]

pub trait Clock {
    fn now_ms(&self) -> u64;
}

pub fn compute_deadline_ms(clock: &impl Clock, timeout_ms: u32) -> u64 {
    let now_ms = clock.now_ms();
    now_ms.saturating_add(u64::from(timeout_ms))
}
