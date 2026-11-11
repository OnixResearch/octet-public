//! Resource-watermark admission over explicit caller-supplied facts.
//!
//! The core does not observe resources, schedule work, or execute processes.

/// Number of bytes in a BLAKE3 identity.
pub const BLAKE3_IDENTITY_BYTES: usize = 32;

/// Number of required non-claims carried by each decision.
pub const WATERMARK_NON_CLAIM_COUNT: usize = 4;

const IMPLEMENTATION_IDENTITY_DOMAIN: &[u8] = b"bounded-exec/watermark/implementation/v1";
const POLICY_IDENTITY_DOMAIN: &[u8] = b"bounded-exec/watermark/policy/v1";

/// Boundaries retained by every admission decision.
pub const WATERMARK_NON_CLAIMS: [&str; WATERMARK_NON_CLAIM_COUNT] = [
    "the decision does not prove that a process started",
    "the decision does not prove sandboxing or hermeticity",
    "the decision does not enforce a resource limit",
    "the decision does not run or validate a scheduler",
];

/// Current hysteresis state for resource admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatermarkGate {
    /// New work can be admitted while the high watermark is not exceeded.
    Admitting,
    /// New work remains deferred until the low watermark is reached.
    Deferred,
}

/// Product-neutral admission disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatermarkDisposition {
    /// The supplied observation permits caller-owned admission.
    Admit,
    /// The supplied observation requires caller-owned deferral.
    Defer,
}

/// Explicit caller-supplied input to resource-watermark admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceWatermarkInput {
    /// Current bounded resource observation.
    pub observation: Option<u64>,
    /// Low watermark that permits deferred admission to resume.
    pub low_watermark: Option<u64>,
    /// High watermark above which admission becomes deferred.
    pub high_watermark: Option<u64>,
    /// Current hysteresis state.
    pub gate: Option<WatermarkGate>,
}

/// Invalid or incomplete resource-watermark input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceWatermarkError {
    /// The resource observation was absent.
    MissingObservation,
    /// The low watermark was absent.
    MissingLowWatermark,
    /// The high watermark was absent.
    MissingHighWatermark,
    /// The current hysteresis state was absent.
    MissingGate,
    /// The low watermark was not strictly less than the high watermark.
    WatermarksNotIncreasing,
}

/// Validated resource-watermark policy input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceWatermarkPolicy {
    observation: u64,
    low_watermark: u64,
    high_watermark: u64,
    gate: WatermarkGate,
}

impl TryFrom<ResourceWatermarkInput> for ResourceWatermarkPolicy {
    type Error = ResourceWatermarkError;

    fn try_from(input: ResourceWatermarkInput) -> Result<Self, Self::Error> {
        let observation = input
            .observation
            .ok_or(ResourceWatermarkError::MissingObservation)?;
        let low_watermark = input
            .low_watermark
            .ok_or(ResourceWatermarkError::MissingLowWatermark)?;
        let high_watermark = input
            .high_watermark
            .ok_or(ResourceWatermarkError::MissingHighWatermark)?;
        let gate = input.gate.ok_or(ResourceWatermarkError::MissingGate)?;

        if low_watermark >= high_watermark {
            return Err(ResourceWatermarkError::WatermarksNotIncreasing);
        }

        Ok(Self {
            observation,
            low_watermark,
            high_watermark,
            gate,
        })
    }
}

/// Deterministic admission decision with explicit inputs and boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceWatermarkDecision {
    /// Supplied resource observation.
    pub observation: u64,
    /// Supplied low watermark.
    pub low_watermark: u64,
    /// Supplied high watermark.
    pub high_watermark: u64,
    /// Hysteresis state before this decision.
    pub prior_gate: WatermarkGate,
    /// Hysteresis state after this decision.
    pub next_gate: WatermarkGate,
    /// Product-neutral admission disposition.
    pub disposition: WatermarkDisposition,
    /// BLAKE3 identity of the fixed implementation profile.
    pub implementation_identity: [u8; BLAKE3_IDENTITY_BYTES],
    /// BLAKE3 identity of the explicit watermark policy.
    pub policy_identity: [u8; BLAKE3_IDENTITY_BYTES],
    /// Required authority and result boundaries.
    pub non_claims: [&'static str; WATERMARK_NON_CLAIM_COUNT],
}

/// Computes one admission decision from validated caller-supplied facts.
///
/// This function does not read a clock, resource counter, scheduler, process,
/// filesystem, network, or other ambient state.
///
/// # Panics
///
/// Panics only if a value bypasses [`ResourceWatermarkPolicy::try_from`] or if
/// the fixed non-claim list does not match its declared length.
#[must_use]
// r[impl bounded_exec.watermark.policy]
// r[impl bounded_exec.watermark.decision]
pub fn decide_resource_watermark(policy: ResourceWatermarkPolicy) -> ResourceWatermarkDecision {
    assert!(
        policy.low_watermark < policy.high_watermark,
        "validated watermarks must be strictly increasing"
    );

    let (disposition, next_gate) = match policy.gate {
        WatermarkGate::Deferred => {
            if policy.observation > policy.low_watermark {
                (WatermarkDisposition::Defer, WatermarkGate::Deferred)
            } else {
                (WatermarkDisposition::Admit, WatermarkGate::Admitting)
            }
        }
        WatermarkGate::Admitting => {
            if policy.observation > policy.high_watermark {
                (WatermarkDisposition::Defer, WatermarkGate::Deferred)
            } else {
                (WatermarkDisposition::Admit, WatermarkGate::Admitting)
            }
        }
    };

    let decision = ResourceWatermarkDecision {
        observation: policy.observation,
        low_watermark: policy.low_watermark,
        high_watermark: policy.high_watermark,
        prior_gate: policy.gate,
        next_gate,
        disposition,
        implementation_identity: implementation_identity(),
        policy_identity: policy_identity(policy.low_watermark, policy.high_watermark),
        non_claims: WATERMARK_NON_CLAIMS,
    };

    assert_eq!(
        decision.non_claims.len(),
        WATERMARK_NON_CLAIM_COUNT,
        "every decision must retain all required non-claims"
    );
    decision
}

/// Claim that a consumer can make from a resource-watermark decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceWatermarkClaim {
    /// The pure core returned the recorded disposition from supplied facts.
    DecisionOnly,
    /// A process started successfully.
    ProcessStarted,
    /// The process ran in a sandbox.
    Sandboxed,
    /// Resource enforcement succeeded.
    ResourceEnforced,
    /// A scheduler executed the decision.
    SchedulerExecuted,
}

/// Rejected admission authority or effect claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceWatermarkClaimError {
    /// The claim promotes admission into caller-owned authority or effects.
    AuthorityPromotion,
}

/// Accepts only the bounded claim that the pure decision occurred.
///
/// # Errors
///
/// Returns [`ResourceWatermarkClaimError::AuthorityPromotion`] for process,
/// sandbox, scheduler, or resource-enforcement claims.
// r[impl bounded_exec.watermark.boundary]
pub const fn validate_resource_watermark_claim(
    claim: ResourceWatermarkClaim,
) -> Result<(), ResourceWatermarkClaimError> {
    match claim {
        ResourceWatermarkClaim::DecisionOnly => Ok(()),
        ResourceWatermarkClaim::ProcessStarted
        | ResourceWatermarkClaim::Sandboxed
        | ResourceWatermarkClaim::ResourceEnforced
        | ResourceWatermarkClaim::SchedulerExecuted => {
            Err(ResourceWatermarkClaimError::AuthorityPromotion)
        }
    }
}

fn implementation_identity() -> [u8; BLAKE3_IDENTITY_BYTES] {
    *blake3::hash(IMPLEMENTATION_IDENTITY_DOMAIN).as_bytes()
}

fn policy_identity(low_watermark: u64, high_watermark: u64) -> [u8; BLAKE3_IDENTITY_BYTES] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(POLICY_IDENTITY_DOMAIN);
    hasher.update(&low_watermark.to_le_bytes());
    hasher.update(&high_watermark.to_le_bytes());
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOW_WATERMARK: u64 = 10;
    const HIGH_WATERMARK: u64 = 20;
    const BELOW_HIGH_OBSERVATION: u64 = 15;
    const ABOVE_HIGH_OBSERVATION: u64 = 21;
    const ABOVE_LOW_OBSERVATION: u64 = 11;

    fn input(observation: u64, gate: WatermarkGate) -> ResourceWatermarkInput {
        ResourceWatermarkInput {
            observation: Some(observation),
            low_watermark: Some(LOW_WATERMARK),
            high_watermark: Some(HIGH_WATERMARK),
            gate: Some(gate),
        }
    }

    fn decision(observation: u64, gate: WatermarkGate) -> ResourceWatermarkDecision {
        let policy = ResourceWatermarkPolicy::try_from(input(observation, gate))
            .expect("complete increasing watermarks must validate");
        decide_resource_watermark(policy)
    }

    // r[verify bounded_exec.watermark.verification]
    #[test]
    fn admitting_gate_admits_through_high_and_defers_above_high() {
        let admitted = decision(BELOW_HIGH_OBSERVATION, WatermarkGate::Admitting);
        assert_eq!(admitted.disposition, WatermarkDisposition::Admit);
        assert_eq!(admitted.next_gate, WatermarkGate::Admitting);

        let deferred = decision(ABOVE_HIGH_OBSERVATION, WatermarkGate::Admitting);
        assert_eq!(deferred.disposition, WatermarkDisposition::Defer);
        assert_eq!(deferred.next_gate, WatermarkGate::Deferred);
    }

    // r[verify bounded_exec.watermark.verification]
    #[test]
    fn deferred_gate_remains_closed_until_low_is_reached() {
        let deferred = decision(ABOVE_LOW_OBSERVATION, WatermarkGate::Deferred);
        assert_eq!(deferred.disposition, WatermarkDisposition::Defer);
        assert_eq!(deferred.next_gate, WatermarkGate::Deferred);

        let admitted = decision(LOW_WATERMARK, WatermarkGate::Deferred);
        assert_eq!(admitted.disposition, WatermarkDisposition::Admit);
        assert_eq!(admitted.next_gate, WatermarkGate::Admitting);
    }

    // r[verify bounded_exec.watermark.verification]
    #[test]
    fn missing_or_non_increasing_values_fail_closed() {
        let missing_observation = ResourceWatermarkInput {
            observation: None,
            ..input(BELOW_HIGH_OBSERVATION, WatermarkGate::Admitting)
        };
        assert_eq!(
            ResourceWatermarkPolicy::try_from(missing_observation),
            Err(ResourceWatermarkError::MissingObservation)
        );

        let missing_low = ResourceWatermarkInput {
            low_watermark: None,
            ..input(BELOW_HIGH_OBSERVATION, WatermarkGate::Admitting)
        };
        assert_eq!(
            ResourceWatermarkPolicy::try_from(missing_low),
            Err(ResourceWatermarkError::MissingLowWatermark)
        );

        let missing_high = ResourceWatermarkInput {
            high_watermark: None,
            ..input(BELOW_HIGH_OBSERVATION, WatermarkGate::Admitting)
        };
        assert_eq!(
            ResourceWatermarkPolicy::try_from(missing_high),
            Err(ResourceWatermarkError::MissingHighWatermark)
        );

        let missing_gate = ResourceWatermarkInput {
            gate: None,
            ..input(BELOW_HIGH_OBSERVATION, WatermarkGate::Admitting)
        };
        assert_eq!(
            ResourceWatermarkPolicy::try_from(missing_gate),
            Err(ResourceWatermarkError::MissingGate)
        );

        let reversed = ResourceWatermarkInput {
            low_watermark: Some(HIGH_WATERMARK),
            high_watermark: Some(LOW_WATERMARK),
            ..input(BELOW_HIGH_OBSERVATION, WatermarkGate::Admitting)
        };
        assert_eq!(
            ResourceWatermarkPolicy::try_from(reversed),
            Err(ResourceWatermarkError::WatermarksNotIncreasing)
        );

        let equal = ResourceWatermarkInput {
            low_watermark: Some(HIGH_WATERMARK),
            high_watermark: Some(HIGH_WATERMARK),
            ..input(BELOW_HIGH_OBSERVATION, WatermarkGate::Admitting)
        };
        assert_eq!(
            ResourceWatermarkPolicy::try_from(equal),
            Err(ResourceWatermarkError::WatermarksNotIncreasing)
        );
    }

    // r[verify bounded_exec.watermark.boundary]
    #[test]
    fn claims_do_not_promote_admission_into_execution_authority() {
        assert_eq!(
            validate_resource_watermark_claim(ResourceWatermarkClaim::DecisionOnly),
            Ok(())
        );
        for claim in [
            ResourceWatermarkClaim::ProcessStarted,
            ResourceWatermarkClaim::Sandboxed,
            ResourceWatermarkClaim::ResourceEnforced,
            ResourceWatermarkClaim::SchedulerExecuted,
        ] {
            assert_eq!(
                validate_resource_watermark_claim(claim),
                Err(ResourceWatermarkClaimError::AuthorityPromotion)
            );
        }
    }

    #[test]
    fn identities_are_stable_and_policy_scoped() {
        let first = decision(BELOW_HIGH_OBSERVATION, WatermarkGate::Admitting);
        let second = decision(ABOVE_HIGH_OBSERVATION, WatermarkGate::Deferred);
        assert_eq!(
            first.implementation_identity,
            second.implementation_identity
        );
        assert_eq!(first.policy_identity, second.policy_identity);
        assert_ne!(first.implementation_identity, first.policy_identity);
        assert_eq!(first.non_claims, WATERMARK_NON_CLAIMS);
    }
}
