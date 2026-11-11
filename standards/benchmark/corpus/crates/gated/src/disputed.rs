//! Feature-selection ambiguity: the direct adapter call is active in every
//! configuration because both cfg branches compile the same body.

#[cfg(any(feature = "strict", not(feature = "strict")))]
pub fn evaluate() -> u32 {
    infra_api::Database::connect().query()
}
