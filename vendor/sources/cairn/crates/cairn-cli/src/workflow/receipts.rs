use std::{error::Error, path::Path};

use cairn_core::{
    canonical_json_hash, layout_name, AgentGuidanceResult, WorkflowInstructionEnvelope,
    WorkflowProfileEvaluation, WorkflowProfileSelection, WorkflowSchemaPolicy,
    WorkflowStatusReport,
};
use serde_json::{json, Value};

use crate::support::{path_string, with_portable_receipt_hash};

pub(super) struct WorkflowReceiptContext<'a> {
    pub root: &'a Path,
    pub layout: cairn_core::LifecycleLayout,
    pub change: &'a str,
    pub policy_name: &'a str,
    pub policy_hash: &'a str,
    pub input_hash: &'a str,
    pub schema: &'a WorkflowSchemaPolicy,
    pub selection: &'a WorkflowProfileSelection,
    pub evaluation: &'a WorkflowProfileEvaluation,
    pub profile_identity_hash: &'a str,
}

pub(super) fn status_receipt(
    context: &WorkflowReceiptContext<'_>,
    report: &WorkflowStatusReport,
) -> Result<Value, Box<dyn Error>> {
    let artifacts = serde_json::to_value(&report.artifacts)?;
    Ok(with_portable_receipt_hash(
        context.root,
        json!({
            "command": "workflow status",
            "schema_version": report.schema_version,
            "root": path_string(context.root),
            "layout": layout_name(context.layout),
            "change": context.change,
            "workflow_schema": context.schema.id,
            "workflow_profile": context.selection.profile_id,
            "workflow_profile_version": context.selection.profile_version,
            "workflow_profile_source": context.selection.source,
            "workflow_profile_compatibility_fallback": context.selection.compatibility_fallback,
            "profile_identity_hash": context.profile_identity_hash,
            "spec_effect": context.evaluation.spec_effect,
            "acceptance_ids": context.evaluation.acceptance_ids,
            "review_receipt_hashes": context.evaluation.review_receipt_hashes,
            "workflow_description": report.schema_description,
            "artifacts": artifacts,
            "next_actions": report.next_actions,
            "issues": report.issues,
            "valid": report.valid,
            "policy": context.policy_name,
            "policy_hash": context.policy_hash,
            "input_hash": context.input_hash,
            "read_only": true,
            "mutated": false,
            "non_claims": report.non_claims,
        }),
    ))
}

pub(super) fn instructions_receipt(
    context: &WorkflowReceiptContext<'_>,
    instructions: &WorkflowInstructionEnvelope,
    guidance: Option<&AgentGuidanceResult>,
) -> Result<Value, Box<dyn Error>> {
    let instruction_value = serde_json::to_value(instructions)?;
    let mut receipt = json!({
        "command": "workflow instructions",
        "schema_version": instructions.schema_version,
        "root": path_string(context.root),
        "layout": layout_name(context.layout),
        "change": context.change,
        "workflow_schema": context.schema.id,
        "workflow_profile": context.selection.profile_id,
        "workflow_profile_version": context.selection.profile_version,
        "workflow_profile_source": context.selection.source,
        "workflow_profile_compatibility_fallback": context.selection.compatibility_fallback,
        "profile_identity_hash": context.profile_identity_hash,
        "spec_effect": context.evaluation.spec_effect,
        "acceptance_ids": context.evaluation.acceptance_ids,
        "review_receipt_hashes": context.evaluation.review_receipt_hashes,
        "artifact": instruction_value,
        "valid": guidance.is_none_or(|result| result.valid),
        "policy": context.policy_name,
        "policy_hash": context.policy_hash,
        "input_hash": context.input_hash,
        "read_only": true,
        "mutated": false,
        "non_claims": instructions.non_claims,
    });
    if let Some(guidance) = guidance {
        receipt["guidance"] = crate::agent_guidance_shell::agent_guidance_json(guidance);
    }
    Ok(with_portable_receipt_hash(context.root, receipt))
}

pub(super) fn workflow_input_hash(
    change: &str,
    schema: &WorkflowSchemaPolicy,
    observations: &[cairn_core::WorkflowArtifactObservation],
) -> Result<String, Box<dyn Error>> {
    Ok(canonical_json_hash(&json!({
        "change": change,
        "schema": schema,
        "observations": observations,
    })))
}
