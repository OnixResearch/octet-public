pub struct Authority(u64);

pub struct Marker;

pub fn authorize() -> Authority {
    Authority(1)
}

pub fn publish(_authority: Authority) {}

pub fn record(_marker: Marker) {}

pub fn execute() {
    publish(authorize());
}

// Builds the tracked value inline instead of taking it from `authorize`.
pub fn forged() {
    record(Marker);
}

macro_rules! twice {
    ($($statement:tt)*) => {
        $($statement)*
        $($statement)*
    };
}

// Thousands of MIR statements, none of them on a tracked value.
pub fn checksum(seed: u64) -> u64 {
    let mut total = seed;
    twice! { twice! { twice! { twice! { twice! { twice! { twice! { twice! { twice! { twice! { twice! {
        total ^= total >> 3;
    } } } } } } } } } } }
    total
}
