use std::{error::Error, io::ErrorKind, path::Path};

use cairn_core::{
    admit_agent_guidance_declaration, blake3_hex, build_agent_guidance_result, canonical_json_hash,
    select_agent_guidance, AgentArtifactGuidance, AgentContextReference, AgentGuidanceDeclaration,
    AgentGuidanceObservationStatus, AgentGuidancePolicy, AgentGuidanceRequest, AgentGuidanceResult,
    AgentGuidanceSourceObservation, AgentOperationGuidance, Policy,
    AGENT_GUIDANCE_SOURCE_LIFECYCLE, AGENT_GUIDANCE_SOURCE_PROJECT,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::support::{CapabilityRoot, CapabilityRootKind};

const PROJECT_SOURCE_PATH: &str = ".cairn/project.ncl";
const PROJECT_PROJECTION_PATH: &str = ".cairn/project.json";
const PROJECT_PROJECTION_SCHEMA_V1: &str = "cairn.lifecycle-store-project.v1";
const PROJECT_CONFIG_MAX_BYTES: u64 = 1_048_576;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectGuidanceProjection {
    schema: String,
    lifecycle_store: Option<Value>,
    #[serde(default)]
    references: Vec<Value>,
    #[serde(default)]
    context_refs: Vec<AgentContextReference>,
    #[serde(default)]
    artifact_guidance: Vec<AgentArtifactGuidance>,
    #[serde(default)]
    operation_guidance: Vec<AgentOperationGuidance>,
    source_blake3: String,
    #[serde(default)]
    non_claims: Vec<String>,
}

// r[impl cairn.agent_guidance_context.sources]
// r[impl cairn.agent_guidance_context.functional_boundary.shell]
pub(crate) fn load_agent_guidance(
    project_root: &Path,
    lifecycle_root: &Path,
    request: AgentGuidanceRequest,
    policy_constraints: &[String],
    policy: &Policy,
) -> Result<Option<AgentGuidanceResult>, Box<dyn Error>> {
    let project_capability = CapabilityRoot::open(CapabilityRootKind::Repository, project_root)?;
    let source_path = Path::new(PROJECT_SOURCE_PATH);
    let projection_path = Path::new(PROJECT_PROJECTION_PATH);
    let source = read_optional_bounded(&project_capability, source_path, PROJECT_CONFIG_MAX_BYTES)?;
    let projection_bytes = read_optional_bounded(
        &project_capability,
        projection_path,
        PROJECT_CONFIG_MAX_BYTES,
    )?;
    let (source, projection_bytes) = match (source, projection_bytes) {
        (Some(source), Some(projection)) => (source, projection),
        (None, None) => return Ok(None),
        _ => {
            return Err(
                "checked project guidance requires both .cairn/project.ncl and .cairn/project.json"
                    .into(),
            );
        }
    };
    let projection_value: Value = serde_json::from_slice(&projection_bytes)?;
    let projection: ProjectGuidanceProjection = serde_json::from_value(projection_value.clone())?;
    if projection.schema != PROJECT_PROJECTION_SCHEMA_V1 {
        return Err(format!(
            "unsupported project guidance projection schema: {}",
            projection.schema
        )
        .into());
    }
    let _project_pointer = projection.lifecycle_store;
    let _direct_references = projection.references;
    let declaration = AgentGuidanceDeclaration {
        context_refs: projection.context_refs,
        artifact_guidance: projection.artifact_guidance,
        operation_guidance: projection.operation_guidance,
    };
    if declaration.context_refs.is_empty()
        && declaration.artifact_guidance.is_empty()
        && declaration.operation_guidance.is_empty()
    {
        return Ok(None);
    }
    if blake3_hex(&source) != projection.source_blake3 {
        return Err(
            "agent_guidance.project_projection_stale: project guidance projection is stale".into(),
        );
    }
    for required in &policy
        .lifecycle_store_policy
        .project_config
        .required_non_claims
    {
        if !projection.non_claims.contains(required) {
            return Err(format!(
                "agent_guidance.project_projection_invalid: missing required non-claim: {required}"
            )
            .into());
        }
    }
    let admitted = admit_agent_guidance_declaration(&declaration, &policy.agent_guidance_policy)
        .map_err(guidance_issues_error)?;
    let plan = select_agent_guidance(&admitted, &request, &policy.agent_guidance_policy)
        .map_err(guidance_issues_error)?;

    let lifecycle_capability = if plan
        .context_refs
        .iter()
        .any(|reference| reference.source_root == AGENT_GUIDANCE_SOURCE_LIFECYCLE)
    {
        Some(CapabilityRoot::open(
            CapabilityRootKind::Repository,
            lifecycle_root,
        )?)
    } else {
        None
    };
    let mut observations = Vec::with_capacity(plan.context_refs.len());
    for reference in &plan.context_refs {
        let capability = match reference.source_root.as_str() {
            AGENT_GUIDANCE_SOURCE_PROJECT => &project_capability,
            AGENT_GUIDANCE_SOURCE_LIFECYCLE => lifecycle_capability
                .as_ref()
                .ok_or("lifecycle capability root is unavailable")?,
            _ => return Err("agent guidance source root escaped admission".into()),
        };
        observations.push(observe_context_source(
            capability,
            reference,
            &policy.agent_guidance_policy,
        ));
    }
    let result = build_agent_guidance_result(
        &plan,
        &observations,
        policy_constraints,
        Some(canonical_json_hash(&projection_value)),
        canonical_json_hash(&policy.raw),
        &policy.agent_guidance_policy,
    );
    Ok(Some(result))
}

// r[impl cairn.agent_guidance_context.privacy]
// r[impl cairn.agent_guidance_context.receipts.portable]
pub(crate) fn agent_guidance_json(result: &AgentGuidanceResult) -> Value {
    json!({
        "valid": result.valid,
        "request": result.request,
        "bounds": result.bounds,
        "entries": result.entries,
        "portable_entries": result.portable_entries,
        "portable_evidence": cairn_core::portable_agent_guidance_result(result),
        "issues": result.issues,
        "local_result_hash": result.local_result_hash,
        "portable_identity": result.portable_identity,
        "project_identity": result.project_identity,
        "policy_hash": result.policy_hash,
        "non_claims": result.non_claims,
        "portable_omits_raw_text": true,
        "read_only": true,
        "mutated": false,
    })
}

fn observe_context_source(
    capability: &CapabilityRoot,
    reference: &AgentContextReference,
    policy: &AgentGuidancePolicy,
) -> AgentGuidanceSourceObservation {
    let relative = Path::new(&reference.path);
    let metadata = match capability.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return AgentGuidanceSourceObservation {
                id: reference.id.clone(),
                status: if reference.required {
                    AgentGuidanceObservationStatus::MissingRequired
                } else {
                    AgentGuidanceObservationStatus::MissingOptional
                },
                text: None,
                byte_length: None,
                content_blake3: None,
                diagnostic: None,
            };
        }
        Err(error) => return invalid_observation(reference, error.to_string()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return invalid_observation(
            reference,
            "context source must be a non-symlink regular file".to_string(),
        );
    }
    let max_bytes = match u64::try_from(policy.bounds.max_source_text_bytes) {
        Ok(value) => value,
        Err(error) => return invalid_observation(reference, error.to_string()),
    };
    let bytes = match capability.read_bounded(relative, max_bytes) {
        Ok(bytes) => bytes,
        Err(error) => return invalid_observation(reference, error.to_string()),
    };
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => return invalid_observation(reference, error.to_string()),
    };
    AgentGuidanceSourceObservation {
        id: reference.id.clone(),
        status: AgentGuidanceObservationStatus::Present,
        byte_length: Some(text.len()),
        content_blake3: Some(blake3_hex(text.as_bytes())),
        text: Some(text),
        diagnostic: None,
    }
}

fn invalid_observation(
    reference: &AgentContextReference,
    diagnostic: String,
) -> AgentGuidanceSourceObservation {
    AgentGuidanceSourceObservation {
        id: reference.id.clone(),
        status: AgentGuidanceObservationStatus::Invalid,
        text: None,
        byte_length: None,
        content_blake3: None,
        diagnostic: Some(diagnostic),
    }
}

fn read_optional_bounded(
    capability: &CapabilityRoot,
    path: &Path,
    max_bytes: u64,
) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    match capability.read_bounded(path, max_bytes) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn guidance_issues_error(issues: Vec<cairn_core::AgentGuidanceIssue>) -> Box<dyn Error> {
    let diagnostics = issues
        .iter()
        .map(|issue| format!("{}:{}: {}", issue.code, issue.field_path, issue.message))
        .collect::<Vec<_>>()
        .join("; ");
    format!("agent guidance admission failed: {diagnostics}").into()
}
