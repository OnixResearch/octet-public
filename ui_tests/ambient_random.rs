#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

const SAMPLE_RANDOM_VALUE: u64 = 4;
const NONCE_BYTES: usize = 16;

mod rand {
    pub fn random() -> u64 {
        crate::SAMPLE_RANDOM_VALUE
    }

    pub fn thread_rng() -> ThreadRng {
        ThreadRng
    }

    pub struct ThreadRng;
}

mod getrandom {
    pub fn fill(bytes: &mut [u8]) -> Result<(), ()> {
        bytes.fill(0);
        Ok(())
    }
}

mod uuid {
    pub struct Uuid;

    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
    }
}

mod fastrand {
    pub fn u64(_range: std::ops::RangeFull) -> u64 {
        crate::SAMPLE_RANDOM_VALUE
    }

    pub fn shuffle<T>(_items: &mut [T]) {}

    pub struct Rng(u64);

    impl Rng {
        pub fn new() -> Self {
            Self(crate::SAMPLE_RANDOM_VALUE)
        }

        pub fn with_seed(seed: u64) -> Self {
            Self(seed)
        }

        pub fn u64(&mut self, _range: std::ops::RangeFull) -> u64 {
            self.0
        }
    }
}

fn generate_ambient_values() -> (u64, uuid::Uuid) {
    let _rng = rand::thread_rng();
    let mut bytes = [0_u8; NONCE_BYTES];
    let _fill_succeeded = getrandom::fill(&mut bytes).is_ok();
    (rand::random(), uuid::Uuid::new_v4())
}

fn wrapped_ambient_values() -> (Option<u64>, Result<uuid::Uuid, ()>) {
    (Some(rand::random()), Ok(uuid::Uuid::new_v4()))
}

fn ambient_fastrand_values(items: &mut [u64]) -> (u64, fastrand::Rng) {
    fastrand::shuffle(items);
    (fastrand::u64(..), fastrand::Rng::new())
}

// A generator built from an explicit seed is injected randomness.
fn seeded_fastrand_value(seed: u64) -> u64 {
    let mut rng = fastrand::Rng::with_seed(seed);
    rng.u64(..)
}

fn main() {
    let _values = generate_ambient_values();
    let _wrapped = wrapped_ambient_values();
    let _fastrand = ambient_fastrand_values(&mut [SAMPLE_RANDOM_VALUE]);
    let _seeded = seeded_fastrand_value(SAMPLE_RANDOM_VALUE);
}
