//! A violating macro expansion that hides an outbound-adapter construction.

macro_rules! construct_database {
    () => {
        infra_api::Database::connect()
    };
}

pub fn invoke() -> infra_api::Database {
    construct_database!()
}
