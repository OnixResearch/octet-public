#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(dead_code)]
#![allow(tigerstyle::ambiguous_params)]
#![allow(tigerstyle::borrowed_argument_types)]
#![allow(tigerstyle::function_length)]
#![allow(tigerstyle::no_unwrap)]
#![allow(tigerstyle::usize_in_public_api)]

use std::ops::Deref;

pub struct Engine {
    value: u32,
}

impl Engine {
    pub fn value(&self) -> u32 {
        self.value
    }
}

pub struct EngineHandle {
    inner: Engine,
}

impl Deref for EngineHandle {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct QualifiedHandle {
    inner: Engine,
}

impl std::ops::Deref for QualifiedHandle {
    type Target = crate::Engine;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

struct PrivateHandle {
    inner: Engine,
}

impl Deref for PrivateHandle {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct Name(String);

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

pub struct Bytes(Vec<u8>);

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

pub struct ExplicitHandle {
    inner: Engine,
}

impl ExplicitHandle {
    pub fn as_engine(&self) -> &Engine {
        &self.inner
    }
}

fn main() {
    let handle = EngineHandle {
        inner: Engine { value: 1 },
    };
    let _ = handle.value();
    let qualified = QualifiedHandle {
        inner: Engine { value: 2 },
    };
    let _ = qualified.value();
    let private = PrivateHandle {
        inner: Engine { value: 3 },
    };
    let _ = private.value();
    let name = Name(String::from("tiger"));
    let _ = name.len();
    let bytes = Bytes(vec![1, 2, 3]);
    let _ = bytes.len();
    let explicit = ExplicitHandle {
        inner: Engine { value: 4 },
    };
    let _ = explicit.as_engine().value();
}
