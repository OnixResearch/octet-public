#![no_std]

pub struct DomainCommand {
    pub value: u8,
}

pub fn accept_external_request(request: axum::Json<DomainCommand>) -> DomainCommand {
    request.0
}
