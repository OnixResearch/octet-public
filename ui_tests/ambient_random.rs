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

fn generate_ambient_values() -> (u64, uuid::Uuid) {
    let _rng = rand::thread_rng();
    let mut bytes = [0_u8; NONCE_BYTES];
    let _fill_succeeded = getrandom::fill(&mut bytes).is_ok();
    (rand::random(), uuid::Uuid::new_v4())
}

fn main() {
    let _values = generate_ambient_values();
}
