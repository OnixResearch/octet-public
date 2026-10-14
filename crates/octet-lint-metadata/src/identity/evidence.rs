const BLAKE3_HEX_LENGTH: usize = 64;
const RECEIPT_DIAGNOSTIC_CAPACITY: usize = 12;
const WRAPPER_VIOLATION_CAPACITY: usize = 4;

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn finding_sort_key(finding: &super::model::InventoryFinding) -> (&str, usize, &str, &str) {
    (&finding.source_path, finding.line, &finding.lint_name, &finding.symbol)
}

#[must_use]
pub fn validate_inventory_receipt(receipt: &super::model::InventoryReceipt) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(RECEIPT_DIAGNOSTIC_CAPACITY);
    if receipt.schema_version != super::model::NOMINAL_DOMAIN_INVENTORY_SCHEMA_VERSION {
        diagnostics.push(format!("unsupported inventory schema `{}`", receipt.schema_version));
    }
    if receipt.mode != super::model::EnforcementMode::Inventory {
        diagnostics.push("pilot receipt mode must be `inventory`".to_owned());
    }
    if receipt.pilot_id.trim().is_empty() {
        diagnostics.push("pilot_id must not be empty".to_owned());
    }
    if !is_blake3_hex(&receipt.source.blake3) {
        diagnostics.push("source.blake3 must be lowercase BLAKE3 hex".to_owned());
    }
    if !is_blake3_hex(&receipt.policy_blake3) {
        diagnostics.push("policy_blake3 must be lowercase BLAKE3 hex".to_owned());
    }
    if receipt.findings.windows(2).any(|pair| finding_sort_key(&pair[0]) > finding_sort_key(&pair[1])) {
        diagnostics.push("findings must use deterministic source, line, lint, and symbol order".to_owned());
    }
    for finding in &receipt.findings {
        if finding.line == 0 {
            diagnostics.push(format!("finding `{}` must have a one-based line", finding.symbol));
        }
        match finding.classification {
            super::model::InventoryClassification::ExactCore if finding.domain_id.is_none() => {
                diagnostics.push(format!("exact finding `{}` must name a domain_id", finding.symbol));
            }
            super::model::InventoryClassification::HeuristicCandidate if finding.domain_id.is_some() => {
                diagnostics.push(format!("heuristic finding `{}` must not claim a domain_id", finding.symbol));
            }
            super::model::InventoryClassification::ExactCore
            | super::model::InventoryClassification::HeuristicCandidate => {}
        }
    }
    let exact_findings = receipt
        .findings
        .iter()
        .filter(|finding| finding.classification == super::model::InventoryClassification::ExactCore)
        .count();
    let heuristic_candidates = receipt
        .findings
        .iter()
        .filter(|finding| finding.classification == super::model::InventoryClassification::HeuristicCandidate)
        .count();
    if receipt.summary.exact_findings != exact_findings {
        diagnostics.push("summary.exact_findings does not match findings".to_owned());
    }
    if receipt.summary.heuristic_candidates != heuristic_candidates {
        diagnostics.push("summary.heuristic_candidates does not match findings".to_owned());
    }
    if receipt.review.true_positives.saturating_add(receipt.review.false_positives) != receipt.findings.len() {
        diagnostics.push("review counts do not cover every finding".to_owned());
    }
    if receipt.non_claims.is_empty() || receipt.non_claims.iter().any(|claim| claim.trim().is_empty()) {
        diagnostics.push("non_claims must contain non-empty text".to_owned());
    }
    diagnostics
}

#[must_use]
pub fn wrapper_contract_violations(
    domain: &super::model::NominalDomainPolicy,
    facts: super::model::WrapperFacts,
) -> Vec<super::model::WrapperViolation> {
    let mut violations = Vec::with_capacity(WRAPPER_VIOLATION_CAPACITY);
    let is_refined = domain.wrapper_kind == super::model::WrapperKind::RefinedNewtype;
    if is_refined && facts.inner_field_public {
        violations.push(super::model::WrapperViolation::PublicInnerField);
    }
    if is_refined && facts.unchecked_from_raw {
        violations.push(super::model::WrapperViolation::UncheckedPrimitiveConversion);
    }
    if domain.constructor_policy == super::model::ConstructorPolicy::PrivateFallible && !facts.has_fallible_constructor
    {
        violations.push(super::model::WrapperViolation::MissingFallibleConstructor);
    }
    if facts.boundary_erasing_deref {
        violations.push(super::model::WrapperViolation::BoundaryErasingDeref);
    }
    violations.sort();
    violations.dedup();
    violations
}
