//! A violating workspace binary target that constructs the adapter directly.

fn main() {
    let database = infra_api::Database::connect();
    let _ = database.query();
}
