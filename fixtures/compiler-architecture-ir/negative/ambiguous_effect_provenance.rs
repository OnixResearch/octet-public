pub struct ActorAuthority(u64);

#[derive(Clone)]
pub struct PublicationAuthority(u64);

pub fn authorize_publication(authority: ActorAuthority) -> PublicationAuthority {
    PublicationAuthority(authority.0)
}

pub fn forge_publication() -> PublicationAuthority {
    PublicationAuthority(0)
}

pub fn publish(_payload: &[u8], _authority: PublicationAuthority) {}

pub fn execute(authority: ActorAuthority, payload: &[u8], use_forged: bool) {
    let witness = if use_forged {
        forge_publication()
    } else {
        authorize_publication(authority)
    };
    publish(payload, witness);
}
