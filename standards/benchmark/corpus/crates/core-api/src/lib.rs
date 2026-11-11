//! Domain-facing port owned by the core boundary.

pub trait Port {
    fn run(&self) -> u32;
}

#[doc(hidden)]
pub mod internal {
    pub fn leak() -> u32 {
        7
    }
}
