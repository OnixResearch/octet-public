//! A violating facade that reexports outbound-adapter internals and constructs them.

pub use infra_api::internal::Connection as LeakedConnection;

pub fn construct() -> infra_api::Database {
    infra_api::Database::connect()
}
