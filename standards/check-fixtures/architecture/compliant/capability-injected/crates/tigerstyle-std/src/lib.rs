use std::time::{SystemTime, UNIX_EPOCH};

pub struct SystemClock;

impl SystemClock {
    pub fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis() as u64)
    }
}
