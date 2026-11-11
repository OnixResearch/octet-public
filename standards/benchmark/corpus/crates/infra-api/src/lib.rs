//! Outbound adapter surface that must stay behind an application port.

pub struct Database;

impl Database {
    pub fn connect() -> Self {
        Self
    }

    pub fn query(&self) -> u32 {
        1
    }
}

#[doc(hidden)]
pub mod internal {
    pub struct Connection;
}
