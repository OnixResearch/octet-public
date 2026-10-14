#![no_std]

pub enum Effect<'a> {
    ReadEnv { key: &'a str },
}

pub struct ScriptedHandler<'a> {
    pub effects: &'a [Effect<'a>],
}
