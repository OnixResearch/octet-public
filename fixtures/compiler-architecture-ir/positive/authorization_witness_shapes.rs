pub struct ActorAuthority(u64);

pub struct AuthorizedPublication(u64);

pub struct PublicationAuthority(u64);

pub fn authorize_publication(authority: &ActorAuthority) -> AuthorizedPublication {
    AuthorizedPublication(authority.0)
}

impl From<AuthorizedPublication> for PublicationAuthority {
    fn from(decision: AuthorizedPublication) -> Self {
        Self(decision.0)
    }
}

pub fn publish(_payload: &[u8], _authority: &PublicationAuthority) {}

pub fn publish_once(_payload: &[u8], _authority: PublicationAuthority) {}

pub fn publish_mut(_payload: &[u8], _authority: &mut PublicationAuthority) {}

pub fn mint(decision: &AuthorizedPublication) -> PublicationAuthority {
    PublicationAuthority(decision.0)
}

pub fn execute(authority: &ActorAuthority, payload: &[u8]) {
    let witness = PublicationAuthority::from(authorize_publication(authority));
    publish(payload, &witness);
    publish(payload, &witness);
}
