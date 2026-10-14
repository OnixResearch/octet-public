pub struct ActorAuthority(u64);

pub struct PublicationPlanId(u64);

pub struct AuthorizedPublication(u64);

pub struct PublicationAuthority(PublicationPlanId);

pub struct PublicationObservation(bool);

pub struct PublicationReceipt(bool);

pub fn authorize_publication(authority: ActorAuthority) -> AuthorizedPublication {
    AuthorizedPublication(authority.0)
}

impl PublicationAuthority {
    pub(crate) fn from_decision(decision: AuthorizedPublication) -> Self {
        Self(PublicationPlanId(decision.0))
    }
}

pub fn publish(_payload: &[u8], _authority: PublicationAuthority) {}

fn publish_wrapper(payload: &[u8], authority: PublicationAuthority) {
    publish(payload, authority);
}

impl PublicationObservation {
    pub fn success() -> Self {
        Self(true)
    }
}

impl PublicationReceipt {
    pub fn from_observation(observation: PublicationObservation) -> Self {
        Self(observation.0)
    }
}

pub fn execute(authority: ActorAuthority, payload: &[u8]) -> PublicationReceipt {
    let decision = authorize_publication(authority);
    let witness = PublicationAuthority::from_decision(decision);
    publish_wrapper(payload, witness);
    let observation = PublicationObservation::success();
    PublicationReceipt::from_observation(observation)
}
