//! A violating application-shell call that bypasses the core-owned port.

pub fn bypass_internal() -> u32 {
    core_api::internal::leak()
}

pub fn construct_concrete() -> infra_api::Database {
    infra_api::Database::connect()
}
