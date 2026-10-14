#![no_std]

use core::marker::PhantomData;

pub struct Disconnected;
pub struct Connected;
pub struct Authenticated;

pub struct Session<State> {
    sequence: u32,
    state: PhantomData<State>,
}

impl Session<Disconnected> {
    #[must_use]
    pub const fn new() -> Self { Self { sequence: 0, state: PhantomData } }
    #[must_use]
    pub const fn connect(self) -> Session<Connected> {
        Session { sequence: self.sequence + 1, state: PhantomData }
    }
}
impl Session<Connected> {
    #[must_use]
    pub const fn authenticate(self) -> Session<Authenticated> {
        Session { sequence: self.sequence + 1, state: PhantomData }
    }
}
impl Session<Authenticated> {
    #[must_use]
    pub const fn send(self, bytes_len: u16) -> (Self, u32) {
        assert!(bytes_len > 0);
        assert!(bytes_len <= 4096);
        (Self { sequence: self.sequence + 1, state: PhantomData }, u32::from(bytes_len))
    }
}
