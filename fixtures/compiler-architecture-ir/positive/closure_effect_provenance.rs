pub struct ActorAuthority(u64);

pub struct PublicationAuthority(u64);

pub fn authorize_publication(authority: ActorAuthority) -> PublicationAuthority {
    PublicationAuthority(authority.0)
}

pub fn publish(_payload: &[u8], _authority: PublicationAuthority) {}

pub fn execute(authority: ActorAuthority, payload: &[u8]) {
    let run = |authority| publish(payload, authorize_publication(authority));
    run(authority);
}
