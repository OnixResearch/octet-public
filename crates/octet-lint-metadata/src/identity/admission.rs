const HEADER_DIAGNOSTIC_CAPACITY: usize = 4;
const DOMAIN_DIAGNOSTICS_PER_ENTRY: usize = 8;
const SCOPE_DIAGNOSTIC_CAPACITY: usize = 4;
const NOMINAL_DOMAIN_CATALOG_JSON: &str = include_str!("../../../../standards/nominal-domains/generated/catalog.json");

/// Loads the checked-in nominal-domain catalog.
///
/// # Errors
///
/// Returns an error when the JSON projection is malformed or violates the Rust contract.
pub fn load_nominal_domain_catalog() -> Result<super::model::NominalDomainCatalog, String> {
    parse_nominal_domain_catalog(NOMINAL_DOMAIN_CATALOG_JSON)
}

/// Parses and checks a nominal-domain catalog without file or compiler access.
///
/// # Errors
///
/// Returns deterministic diagnostics for malformed or invalid policy data.
pub fn parse_nominal_domain_catalog(text: &str) -> Result<super::model::NominalDomainCatalog, String> {
    let catalog = serde_json::from_str::<super::model::NominalDomainCatalog>(text)
        .map_err(|error| format!("parse nominal-domain catalog: {error}"))?;
    let diagnostics = nominal_domain_catalog_diagnostics(&catalog);
    if diagnostics.is_empty() {
        return Ok(catalog);
    }

    Err(format!("invalid nominal-domain catalog: {}", diagnostics.join("; ")))
}

#[must_use]
pub fn nominal_domain_catalog_diagnostics(catalog: &super::model::NominalDomainCatalog) -> Vec<String> {
    let mut diagnostics = catalog_header_diagnostics(catalog);
    diagnostics.extend(domain_rows_diagnostics(catalog));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn catalog_header_diagnostics(catalog: &super::model::NominalDomainCatalog) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(HEADER_DIAGNOSTIC_CAPACITY);
    if catalog.schema_version != super::model::NOMINAL_DOMAIN_SCHEMA_VERSION {
        diagnostics.push(format!(
            "schema_version must be `{}`, got `{}`",
            super::model::NOMINAL_DOMAIN_SCHEMA_VERSION,
            catalog.schema_version
        ));
    }
    if catalog.domains.is_empty() {
        diagnostics.push("domains must not be empty".to_owned());
    }
    diagnostics.extend(non_empty_unique_value_diagnostics("heuristic_suffixes", &catalog.heuristic_suffixes));
    diagnostics.extend(non_empty_unique_value_diagnostics("non_claims", &catalog.non_claims));
    diagnostics
}

fn domain_rows_diagnostics(catalog: &super::model::NominalDomainCatalog) -> Vec<String> {
    let capacity = catalog.domains.len().saturating_mul(DOMAIN_DIAGNOSTICS_PER_ENTRY);
    let mut diagnostics = Vec::with_capacity(capacity);
    let mut domain_ids = std::collections::BTreeSet::new();
    let mut symbols = std::collections::BTreeSet::new();
    for domain in &catalog.domains {
        diagnostics.extend(domain_key_diagnostics(domain, &mut domain_ids, &mut symbols));
        diagnostics.extend(domain_scope_diagnostics(domain));
        diagnostics.extend(constructor_policy_diagnostics(domain));
    }
    diagnostics
}

fn domain_key_diagnostics(
    domain: &super::model::NominalDomainPolicy,
    domain_ids: &mut std::collections::BTreeSet<String>,
    symbols: &mut std::collections::BTreeSet<String>,
) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(domain.symbols.len().saturating_add(1));
    if domain.domain_id.trim().is_empty() {
        diagnostics.push("domain_id must not be empty".to_owned());
    } else if !domain_ids.insert(domain.domain_id.clone()) {
        diagnostics.push(format!("duplicate domain_id `{}`", domain.domain_id));
    }
    if domain.symbols.is_empty() {
        diagnostics.push(format!("domain `{}` symbols must not be empty", domain.domain_id));
    }
    for symbol in &domain.symbols {
        if symbol.trim().is_empty() {
            diagnostics.push(format!("domain `{}` contains an empty symbol", domain.domain_id));
        } else if !symbols.insert(symbol.clone()) {
            diagnostics.push(format!("duplicate nominal-domain symbol `{symbol}`"));
        }
    }
    diagnostics
}

fn domain_scope_diagnostics(domain: &super::model::NominalDomainPolicy) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(SCOPE_DIAGNOSTIC_CAPACITY);
    if domain.core_scopes.is_empty() {
        diagnostics.push(format!("domain `{}` core_scopes must not be empty", domain.domain_id));
    }
    diagnostics.extend(non_empty_unique_value_diagnostics(
        &format!("domain `{}` core_scopes", domain.domain_id),
        &domain.core_scopes,
    ));
    diagnostics.extend(non_empty_unique_value_diagnostics(
        &format!("domain `{}` compatibility_scopes", domain.domain_id),
        &domain.compatibility_scopes,
    ));
    if domain.compatibility_required && domain.compatibility_scopes.is_empty() {
        diagnostics.push(format!("domain `{}` requires at least one compatibility scope", domain.domain_id));
    }
    for core_scope in &domain.core_scopes {
        if domain
            .compatibility_scopes
            .iter()
            .any(|compatibility_scope| normalized_path(core_scope) == normalized_path(compatibility_scope))
        {
            diagnostics
                .push(format!("domain `{}` repeats scope `{core_scope}` as core and compatibility", domain.domain_id));
        }
    }
    diagnostics
}

fn constructor_policy_diagnostics(domain: &super::model::NominalDomainPolicy) -> Vec<String> {
    let is_valid = match domain.wrapper_kind {
        super::model::WrapperKind::RefinedNewtype => {
            domain.constructor_policy == super::model::ConstructorPolicy::PrivateFallible
        }
        super::model::WrapperKind::MarkerGeneric => {
            domain.constructor_policy == super::model::ConstructorPolicy::MarkerOnly
        }
        super::model::WrapperKind::OpaqueNewtype | super::model::WrapperKind::UnitNewtype => {
            domain.constructor_policy != super::model::ConstructorPolicy::MarkerOnly
        }
    };
    if is_valid {
        Vec::new()
    } else {
        vec![format!(
            "domain `{}` constructor policy does not match wrapper kind",
            domain.domain_id
        )]
    }
}

fn non_empty_unique_value_diagnostics(label: &str, values: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(values.len());
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            diagnostics.push(format!("{label} must not contain empty text"));
        } else if !seen.insert(value.as_str()) {
            diagnostics.push(format!("{label} repeats `{value}`"));
        }
    }
    diagnostics
}

#[must_use]
pub fn classify_descriptor(
    catalog: &super::model::NominalDomainCatalog,
    descriptor: super::model::DomainDescriptor<'_>,
) -> super::model::DomainDisposition {
    if let Some(domain) = exact_match(catalog, descriptor) {
        return classify_exact_scope(domain, descriptor.source_path);
    }
    if let Some(suffix) = heuristic_suffix(catalog, descriptor.symbol) {
        return super::model::DomainDisposition::HeuristicCandidate {
            suffix: suffix.to_owned(),
        };
    }
    super::model::DomainDisposition::Unclassified
}

fn exact_match<'a>(
    catalog: &'a super::model::NominalDomainCatalog,
    descriptor: super::model::DomainDescriptor<'_>,
) -> Option<&'a super::model::NominalDomainPolicy> {
    catalog.domains.iter().find(|domain| {
        domain.raw_representation == descriptor.raw_representation
            && domain.symbols.iter().any(|symbol| symbol == descriptor.symbol)
    })
}

fn classify_exact_scope(
    domain: &super::model::NominalDomainPolicy,
    source_path: &str,
) -> super::model::DomainDisposition {
    if source_matches_any_scope(source_path, &domain.compatibility_scopes) {
        return super::model::DomainDisposition::Compatibility {
            domain_id: domain.domain_id.clone(),
        };
    }
    if source_matches_any_scope(source_path, &domain.core_scopes) {
        return super::model::DomainDisposition::ExactCore {
            domain_id: domain.domain_id.clone(),
        };
    }
    super::model::DomainDisposition::ExactOutsideScope {
        domain_id: domain.domain_id.clone(),
    }
}

fn heuristic_suffix<'a>(catalog: &'a super::model::NominalDomainCatalog, symbol: &str) -> Option<&'a str> {
    let terminal = symbol.rsplit("::").next().unwrap_or(symbol);
    catalog
        .heuristic_suffixes
        .iter()
        .find(|suffix| terminal.ends_with(suffix.as_str()))
        .map(String::as_str)
}

fn source_matches_any_scope(source_path: &str, scopes: &[String]) -> bool {
    scopes.iter().any(|scope| source_matches_scope(source_path, scope))
}

fn source_matches_scope(source_path: &str, scope: &str) -> bool {
    let source_path = normalized_path(source_path);
    let scope = normalized_path(scope);
    if source_path == scope {
        return true;
    }
    source_path.ends_with(&format!("/{scope}"))
}

fn normalized_path(path: &str) -> String {
    path.trim_matches('"').replace('\\', "/").trim_start_matches("./").to_owned()
}

#[cfg(test)]
pub(super) fn catalog_json() -> &'static str {
    NOMINAL_DOMAIN_CATALOG_JSON
}
