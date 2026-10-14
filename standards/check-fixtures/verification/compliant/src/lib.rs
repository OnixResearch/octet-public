#![no_std]

pub enum Effect<'a> {
    ReadEnv { key: &'a str },
    WriteLog { message: &'a str },
}

pub struct ScriptedHandler<'a> {
    pub effects: &'a [Effect<'a>],
}

impl<'a> ScriptedHandler<'a> {
    pub const fn new(effects: &'a [Effect<'a>]) -> Self {
        Self { effects }
    }

    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }
}

pub fn proof_witness_deterministic_effect_handlers(handler: &ScriptedHandler<'_>) -> usize {
    handler.effect_count()
}
