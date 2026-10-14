#[path = "identity/admission.rs"]
mod admission;
#[path = "identity/evidence.rs"]
mod evidence;
#[path = "identity/model.rs"]
mod model;

pub use admission::classify_descriptor;
pub use admission::load_nominal_domain_catalog;
pub use admission::nominal_domain_catalog_diagnostics;
pub use admission::parse_nominal_domain_catalog;
pub use evidence::validate_inventory_receipt;
pub use evidence::wrapper_contract_violations;
pub use model::ConstructorPolicy;
pub use model::DescriptorKind;
pub use model::DomainDescriptor;
pub use model::DomainDisposition;
pub use model::EnforcementMode;
pub use model::InventoryClassification;
pub use model::InventoryFinding;
pub use model::InventoryReceipt;
pub use model::InventoryReview;
pub use model::InventorySource;
pub use model::InventorySummary;
pub use model::NOMINAL_DOMAIN_INVENTORY_SCHEMA_VERSION;
pub use model::NOMINAL_DOMAIN_SCHEMA_VERSION;
pub use model::NominalDomainCatalog;
pub use model::NominalDomainPolicy;
pub use model::RawRepresentation;
pub use model::WrapperFacts;
pub use model::WrapperKind;
pub use model::WrapperViolation;

#[cfg(test)]
#[path = "identity/tests.rs"]
mod tests;
