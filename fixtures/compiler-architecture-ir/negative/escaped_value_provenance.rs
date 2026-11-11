pub struct Observation(bool);

pub struct SuccessReceipt(bool);

pub struct Observations {
    pub accepted: Observation,
    pub rejected: Observation,
}

pub fn success() -> Observation {
    Observation(true)
}

pub fn failure() -> Observation {
    Observation(false)
}

pub fn observations() -> Observations {
    Observations {
        accepted: success(),
        rejected: failure(),
    }
}

impl SuccessReceipt {
    pub fn new(observation: Observation) -> Self {
        Self(observation.0)
    }
}

// Every use is a direct call from an analyzed body, so its callers decide the origin.
fn finish_direct(observation: Observation) -> SuccessReceipt {
    SuccessReceipt::new(observation)
}

pub fn direct_only() -> SuccessReceipt {
    finish_direct(success())
}

fn finish_escaped(observation: Observation) -> SuccessReceipt {
    SuccessReceipt::new(observation)
}

pub fn direct_escaped() -> SuccessReceipt {
    finish_escaped(success())
}

pub fn through_pointer() -> SuccessReceipt {
    let finish: fn(Observation) -> SuccessReceipt = finish_escaped;
    finish(failure())
}

// Other crates may call a public function with any observation.
pub fn finish_public(observation: Observation) -> SuccessReceipt {
    SuccessReceipt::new(observation)
}

pub fn direct_public() -> SuccessReceipt {
    finish_public(success())
}

pub fn swapped() -> SuccessReceipt {
    let mut observation = success();
    let mut forged = failure();
    std::mem::swap(&mut observation, &mut forged);
    SuccessReceipt::new(observation)
}

pub fn written_through_reference() -> SuccessReceipt {
    let mut observation = success();
    let alias = &mut observation;
    *alias = failure();
    SuccessReceipt::new(observation)
}

pub fn field_write() -> SuccessReceipt {
    let mut observations = observations();
    observations.accepted = success();
    SuccessReceipt::new(observations.rejected)
}

pub fn replaced_through_pointer(make: fn() -> Observation, replace: bool) -> SuccessReceipt {
    let mut observation = success();
    if replace {
        observation = make();
    }
    SuccessReceipt::new(observation)
}

pub fn after_untracked_call(seed: u32) -> SuccessReceipt {
    std::hint::black_box(seed);
    SuccessReceipt::new(failure())
}
