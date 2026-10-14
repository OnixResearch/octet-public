const SUPPORTED_CLIENT_PROFILE: &str = "contract-consumption-fixture/v1";
const REQUIRED_CLIENT_NON_CLAIM: &str =
    "generated client success proves contract consumption only, not API behavior correctness";
const REQUIRED_BOUNDARY_NON_CLAIM: &str =
    "contract rail evidence does not prove service correctness, evidence truth, or release eligibility";
const DIAGNOSTIC_MISSING_SOURCE: &str = "contract_source_missing";
const DIAGNOSTIC_HIDDEN_AUTHORITY: &str = "hidden_contract_authority";
const DIAGNOSTIC_REQUIRED_FIELD_REMOVED: &str = "required_field_removed";
const DIAGNOSTIC_REQUIREDNESS_TIGHTENED: &str = "requiredness_tightened";
const DIAGNOSTIC_UNSUPPORTED_GENERATOR: &str = "unsupported_generator_profile";
const DIAGNOSTIC_GENERATOR_FAILURE: &str = "generated_client_fixture_failed";
const DIAGNOSTIC_CLIENT_NON_CLAIM_MISSING: &str = "generated_client_non_claim_missing";
const DIAGNOSTIC_CLIENT_OVERCLAIM: &str = "generated_client_overclaim";
const EXPECTED_DECLARED_SOURCE_KINDS: usize = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContractSourceKind {
    RustDto,
    NickelPolicy,
    StandardsOutput,
    CommandJson,
    ExternalSchemaInput,
    CompatibilityFixture,
    HiddenDocument,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContractChangeKind {
    Additive,
    Deprecated,
    Breaking,
    Malformed,
    Unsupported,
    Unchanged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ContractField {
    name: String,
    required: bool,
    deprecated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ContractDocument {
    contract_id: String,
    source_kind: ContractSourceKind,
    supported_claim: String,
    fields: Vec<ContractField>,
    generator_profile: String,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompatibilityReport {
    valid: bool,
    change: ContractChangeKind,
    diagnostics: Vec<String>,
    generated_client_eligible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneratedClientFixture {
    generator_profile: String,
    compiles: bool,
    exercises_fixture_response: bool,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReferenceInput {
    name: &'static str,
    role: &'static str,
    license_boundary: &'static str,
    trust_boundary: &'static str,
}

fn reference_inventory() -> Vec<ReferenceInput> {
    vec![
        ReferenceInput {
            name: "Dropshot",
            role: "typed service surface and generated OpenAPI reference",
            license_boundary: "reference-only; no vendored source or runtime dependency",
            trust_boundary: "typed context pattern only; no hidden middleware authority imported",
        },
        ReferenceInput {
            name: "Progenitor",
            role: "generated client fixture reference",
            license_boundary: "reference-only; generated clients are optional fixtures",
            trust_boundary: "client compilation proves contract consumption only",
        },
        ReferenceInput {
            name: "Typify",
            role: "schema-to-Rust interop reference",
            license_boundary: "reference-only; Octet Rust DTOs remain source of truth",
            trust_boundary: "external schemas need declared source boundaries before generation",
        },
        ReferenceInput {
            name: "Drift",
            role: "compatibility diff classification reference",
            license_boundary: "reference-only; no managed API state imported",
            trust_boundary: "diff output is review evidence, not release authorization",
        },
        ReferenceInput {
            name: "openapi-lint",
            role: "contract linting reference",
            license_boundary: "reference-only; Octet checks are local deterministic fixtures",
            trust_boundary: "lint success does not prove service behavior",
        },
        ReferenceInput {
            name: "dropshot-api-manager",
            role: "managed contract promotion reference",
            license_boundary: "reference-only; Octet lifecycle remains Cairn-owned",
            trust_boundary: "promotion requires Octet/Cairn gates, not external manager state",
        },
    ]
}

fn classify_contract_change(
    old_contract: &ContractDocument,
    new_contract: &ContractDocument,
    client_fixture: &GeneratedClientFixture,
) -> CompatibilityReport {
    let mut diagnostics = boundary_diagnostics(new_contract);
    diagnostics.extend(field_compatibility_diagnostics(old_contract, new_contract));
    let client_diagnostics = generated_client_diagnostics(client_fixture);
    let generated_client_eligible = client_diagnostics.is_empty();
    diagnostics.extend(client_diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let change = change_kind(old_contract, new_contract, &diagnostics);
    CompatibilityReport {
        valid: diagnostics.is_empty(),
        change,
        diagnostics,
        generated_client_eligible,
    }
}

fn boundary_diagnostics(contract: &ContractDocument) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if contract.supported_claim.trim().is_empty()
        || contract.contract_id.trim().is_empty()
        || contract.generator_profile.trim().is_empty()
    {
        diagnostics.push(format!(
            "{DIAGNOSTIC_MISSING_SOURCE}: contract `{}` must name source and supported claim",
            contract.contract_id
        ));
    }
    if contract.source_kind == ContractSourceKind::HiddenDocument {
        diagnostics.push(format!(
            "{DIAGNOSTIC_HIDDEN_AUTHORITY}: contract `{}` has hidden or hand-maintained authority",
            contract.contract_id
        ));
    }
    if !contract.non_claims.iter().any(|non_claim| non_claim == REQUIRED_BOUNDARY_NON_CLAIM) {
        diagnostics.push(format!(
            "{DIAGNOSTIC_HIDDEN_AUTHORITY}: contract `{}` missing boundary non-claim",
            contract.contract_id
        ));
    }
    diagnostics
}

fn field_compatibility_diagnostics(old_contract: &ContractDocument, new_contract: &ContractDocument) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for old_field in &old_contract.fields {
        match new_contract.fields.iter().find(|new_field| new_field.name == old_field.name) {
            Some(new_field) => {
                if !old_field.required && new_field.required {
                    diagnostics.push(format!(
                        "{DIAGNOSTIC_REQUIREDNESS_TIGHTENED}: field `{}` became required",
                        old_field.name
                    ));
                }
            }
            None => {
                if old_field.required {
                    diagnostics.push(format!(
                        "{DIAGNOSTIC_REQUIRED_FIELD_REMOVED}: required field `{}` was removed",
                        old_field.name
                    ));
                }
            }
        }
    }
    diagnostics
}

fn generated_client_diagnostics(fixture: &GeneratedClientFixture) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if fixture.generator_profile != SUPPORTED_CLIENT_PROFILE {
        diagnostics.push(format!(
            "{DIAGNOSTIC_UNSUPPORTED_GENERATOR}: generator profile `{}` is not supported",
            fixture.generator_profile
        ));
    }
    if !fixture.compiles || !fixture.exercises_fixture_response {
        diagnostics.push(DIAGNOSTIC_GENERATOR_FAILURE.to_owned());
    }
    if !fixture.non_claims.iter().any(|non_claim| non_claim == REQUIRED_CLIENT_NON_CLAIM) {
        diagnostics.push(DIAGNOSTIC_CLIENT_NON_CLAIM_MISSING.to_owned());
    }
    let joined = fixture.non_claims.join(" ").to_lowercase();
    if joined.contains("proves api behavior") || joined.contains("proves service correctness") {
        diagnostics.push(DIAGNOSTIC_CLIENT_OVERCLAIM.to_owned());
    }
    diagnostics
}

fn change_kind(
    old_contract: &ContractDocument,
    new_contract: &ContractDocument,
    diagnostics: &[String],
) -> ContractChangeKind {
    if diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_HIDDEN_AUTHORITY)) {
        return ContractChangeKind::Malformed;
    }
    if diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_UNSUPPORTED_GENERATOR)) {
        return ContractChangeKind::Unsupported;
    }
    if diagnostics.iter().any(|diagnostic| {
        diagnostic.contains(DIAGNOSTIC_REQUIRED_FIELD_REMOVED) || diagnostic.contains(DIAGNOSTIC_REQUIREDNESS_TIGHTENED)
    }) {
        return ContractChangeKind::Breaking;
    }
    if new_contract.fields.iter().any(|field| field.deprecated) {
        return ContractChangeKind::Deprecated;
    }
    if new_contract.fields.len() > old_contract.fields.len() {
        return ContractChangeKind::Additive;
    }
    ContractChangeKind::Unchanged
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRACT_ID: &str = "octet.receipts.ArtifactVerificationV1";
    const SUPPORTED_CLAIM: &str = "contract records verified artifact receipt shape only";

    #[test]
    fn api_contract_reference_inventory_names_boundaries_for_each_oxide_input() {
        let inventory = reference_inventory();

        assert!(inventory.iter().all(|input| !input.role.is_empty()));
        assert!(
            inventory
                .iter()
                .any(|input| input.name == "Dropshot" && input.trust_boundary.contains("hidden middleware"))
        );
        assert!(
            inventory
                .iter()
                .any(|input| input.name == "Progenitor" && input.trust_boundary.contains("contract consumption only"))
        );
        assert!(
            inventory
                .iter()
                .any(|input| input.name == "Typify" && input.license_boundary.contains("reference-only"))
        );
        assert!(
            inventory
                .iter()
                .any(|input| input.name == "Drift" && input.trust_boundary.contains("review evidence"))
        );
        assert!(
            inventory
                .iter()
                .any(|input| input.name == "openapi-lint" && input.trust_boundary.contains("does not prove"))
        );
        assert!(
            inventory
                .iter()
                .any(|input| input.name == "dropshot-api-manager" && input.trust_boundary.contains("Cairn"))
        );
    }

    #[test]
    fn api_contract_source_kinds_cover_octet_contract_surfaces() {
        let declared_source_kinds = [
            ContractSourceKind::RustDto,
            ContractSourceKind::NickelPolicy,
            ContractSourceKind::StandardsOutput,
            ContractSourceKind::CommandJson,
            ContractSourceKind::ExternalSchemaInput,
            ContractSourceKind::CompatibilityFixture,
        ];

        assert_eq!(declared_source_kinds.len(), EXPECTED_DECLARED_SOURCE_KINDS);
    }

    #[test]
    fn additive_contract_change_with_generated_client_fixture_is_eligible() {
        let old_contract = base_contract();
        let mut new_contract = base_contract();
        new_contract.fields.push(optional_field("diagnostics"));
        let report = classify_contract_change(&old_contract, &new_contract, &valid_client_fixture());

        assert!(report.valid, "unexpected diagnostics: {:?}", report.diagnostics);
        assert_eq!(report.change, ContractChangeKind::Additive);
        assert!(report.generated_client_eligible);
    }

    #[test]
    fn deprecated_contract_change_stays_bounded_and_valid() {
        let old_contract = base_contract();
        let mut new_contract = base_contract();
        new_contract.fields.push(deprecated_field("legacy_status"));
        let report = classify_contract_change(&old_contract, &new_contract, &valid_client_fixture());

        assert!(report.valid, "unexpected diagnostics: {:?}", report.diagnostics);
        assert_eq!(report.change, ContractChangeKind::Deprecated);
    }

    #[test]
    fn breaking_malformed_unsupported_and_overclaim_cases_fail_closed() {
        let old_contract = base_contract();
        let mut removed_required = base_contract();
        removed_required.fields.retain(|field| field.name != "schema_version");
        let breaking = classify_contract_change(&old_contract, &removed_required, &valid_client_fixture());

        let mut hidden = base_contract();
        hidden.source_kind = ContractSourceKind::HiddenDocument;
        hidden.non_claims.clear();
        let malformed = classify_contract_change(&old_contract, &hidden, &valid_client_fixture());

        let mut unsupported_fixture = valid_client_fixture();
        unsupported_fixture.generator_profile = "ad-hoc-generator".to_owned();
        let unsupported = classify_contract_change(&old_contract, &base_contract(), &unsupported_fixture);

        let mut overclaim_fixture = valid_client_fixture();
        overclaim_fixture.non_claims.push("generated client proves API behavior".to_owned());
        let overclaim = classify_contract_change(&old_contract, &base_contract(), &overclaim_fixture);

        assert!(!breaking.valid);
        assert_eq!(breaking.change, ContractChangeKind::Breaking);
        assert!(breaking.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_REQUIRED_FIELD_REMOVED)));
        assert!(!malformed.valid);
        assert_eq!(malformed.change, ContractChangeKind::Malformed);
        assert!(!unsupported.valid);
        assert_eq!(unsupported.change, ContractChangeKind::Unsupported);
        assert!(!overclaim.valid);
        assert!(overclaim.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_CLIENT_OVERCLAIM)));
    }

    fn base_contract() -> ContractDocument {
        ContractDocument {
            contract_id: CONTRACT_ID.to_owned(),
            source_kind: ContractSourceKind::RustDto,
            supported_claim: SUPPORTED_CLAIM.to_owned(),
            fields: vec![
                required_field("schema_version"),
                required_field("receipt_digest"),
                optional_field("diagnostics"),
            ],
            generator_profile: SUPPORTED_CLIENT_PROFILE.to_owned(),
            non_claims: vec![REQUIRED_BOUNDARY_NON_CLAIM.to_owned()],
        }
    }

    fn valid_client_fixture() -> GeneratedClientFixture {
        GeneratedClientFixture {
            generator_profile: SUPPORTED_CLIENT_PROFILE.to_owned(),
            compiles: true,
            exercises_fixture_response: true,
            non_claims: vec![REQUIRED_CLIENT_NON_CLAIM.to_owned()],
        }
    }

    fn required_field(name: &str) -> ContractField {
        ContractField {
            name: name.to_owned(),
            required: true,
            deprecated: false,
        }
    }

    fn optional_field(name: &str) -> ContractField {
        ContractField {
            name: name.to_owned(),
            required: false,
            deprecated: false,
        }
    }

    fn deprecated_field(name: &str) -> ContractField {
        ContractField {
            name: name.to_owned(),
            required: false,
            deprecated: true,
        }
    }
}
