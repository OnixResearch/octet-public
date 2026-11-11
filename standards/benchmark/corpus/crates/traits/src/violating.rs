//! A violating inherent-method reference on a concrete adapter type.

pub fn call_inherent() -> u32 {
    infra_api::Database::connect().query()
}
