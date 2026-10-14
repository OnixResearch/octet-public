#![no_std]

pub trait Clock {
    fn now_ticks(&self) -> u64;
}

pub fn schedule(clock: &dyn Clock) -> u64 {
    clock.now_ticks()
}
