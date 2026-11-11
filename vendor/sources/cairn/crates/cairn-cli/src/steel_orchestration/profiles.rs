use cairn_core::{SteelOrchestrationProfile, STEEL_ORCHESTRATION_RECEIPT_SCHEMA_V1};

pub(super) const DEFAULT_PROFILE_ID: &str = "disabled.lifecycle.default";

// r[impl cairn.lifecycle_orchestration.steel_lifecycle_orchestration.boundary_split.evaluation]
pub(super) fn select_profile<'a>(
    profiles: &'a [SteelOrchestrationProfile],
    requested_profile_id: Option<&str>,
) -> Option<&'a SteelOrchestrationProfile> {
    match requested_profile_id {
        Some(profile_id) => profiles.iter().find(|profile| profile.id == profile_id),
        None => profiles
            .iter()
            .find(|profile| profile.id == DEFAULT_PROFILE_ID)
            .or_else(|| profiles.first()),
    }
}

pub(super) fn fallback_profile() -> SteelOrchestrationProfile {
    SteelOrchestrationProfile {
        id: DEFAULT_PROFILE_ID.into(),
        mode: "disabled".into(),
        allowed_host_functions: Vec::new(),
        deterministic_budgets: Vec::new(),
        redaction_classes: vec!["safe_metadata".into()],
        fallback_modes: vec!["rust_nickel".into()],
        receipt_schema_version: STEEL_ORCHESTRATION_RECEIPT_SCHEMA_V1.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_policy_selects_default_profile() {
        let profile = SteelOrchestrationProfile {
            id: DEFAULT_PROFILE_ID.into(),
            mode: "disabled".into(),
            allowed_host_functions: Vec::new(),
            deterministic_budgets: Vec::new(),
            redaction_classes: vec!["safe_metadata".into()],
            fallback_modes: vec!["rust_nickel".into()],
            receipt_schema_version: STEEL_ORCHESTRATION_RECEIPT_SCHEMA_V1.into(),
        };

        assert_eq!(
            select_profile(&[profile], None).unwrap().id,
            DEFAULT_PROFILE_ID
        );
    }
}
