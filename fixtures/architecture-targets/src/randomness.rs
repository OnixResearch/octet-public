extern crate getrandom as entropy;

/// Real registry operations, compiled but never executed by the collector control.
pub fn random_effects() {
    let mut bytes = [0_u8; 4];
    let _ = entropy::fill(&mut bytes);
    let mut uninitialized = [core::mem::MaybeUninit::<u8>::uninit(); 4];
    let _ = entropy::fill_uninit(&mut uninitialized);
    let _ = entropy::u32();
    let _ = entropy::u64();
}

/// Similar local names are negative controls, not randomness implementations.
pub mod getrandom {
    pub fn fill(value: u32) -> u32 {
        value
    }
}

pub fn ordinary_local_call(value: u32) -> u32 {
    getrandom::fill(value)
}
