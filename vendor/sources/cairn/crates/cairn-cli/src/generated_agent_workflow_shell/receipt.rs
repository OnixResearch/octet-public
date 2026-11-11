use cairn_core::{AdmittedGeneratedWorkflowCatalog, GeneratedWorkflowInstallPlan};
use serde_json::{json, Value};

use crate::support::with_receipt_hash;

pub(super) fn catalog_receipt(
    catalog: &AdmittedGeneratedWorkflowCatalog,
    policy_hash: &str,
) -> Value {
    let workflows = catalog
        .policy
        .descriptors
        .iter()
        .map(|descriptor| {
            json!({
                "id": descriptor.id,
                "version": descriptor.version,
                "goal": descriptor.goal,
                "mutation_class": descriptor.mutation_class,
                "command_classes": descriptor.command_classes,
                "success_evidence": descriptor.success_evidence,
            })
        })
        .collect::<Vec<_>>();
    with_receipt_hash(json!({
        "schema": "cairn.generated-agent-workflow-catalog.v1",
        "command": "agent workflow list",
        "valid": true,
        "generator_version": catalog.policy.generator_version,
        "catalog_identity": catalog.catalog_identity,
        "policy_hash": policy_hash,
        "workflows": workflows,
        "adapters": catalog.policy.adapters,
        "non_claims": catalog.policy.required_non_claims,
    }))
}

pub(super) fn descriptor_receipt(
    catalog: &AdmittedGeneratedWorkflowCatalog,
    policy_hash: &str,
    workflow_id: &str,
) -> Value {
    let descriptor = catalog
        .policy
        .descriptors
        .iter()
        .find(|descriptor| descriptor.id == workflow_id);
    with_receipt_hash(json!({
        "schema": "cairn.generated-agent-workflow-descriptor.v1",
        "command": "agent workflow show",
        "valid": descriptor.is_some(),
        "workflow": descriptor,
        "catalog_identity": catalog.catalog_identity,
        "policy_hash": policy_hash,
        "non_claims": catalog.policy.required_non_claims,
    }))
}

pub(super) fn install_plan_receipt(
    command: &str,
    plan: &GeneratedWorkflowInstallPlan,
    execute: bool,
    mutated: bool,
) -> Value {
    let files = plan
        .files
        .iter()
        .map(|file| {
            json!({
                "workflow_id": file.workflow_id,
                "relative_path": file.relative_path,
                "generated_blake3": file.generated_blake3,
                "descriptor_identity": file.descriptor_identity,
                "expected_prior_blake3": file.expected_prior_blake3,
                "observed_blake3": file.observed_blake3,
                "freshness": file.freshness,
                "action": if file.freshness == cairn_core::GeneratedWorkflowFreshness::Current {
                    "none"
                } else {
                    "write"
                },
            })
        })
        .collect::<Vec<_>>();
    with_receipt_hash(json!({
        "schema": "cairn.generated-agent-workflow-install-receipt.v1",
        "command": format!("agent workflow {command}"),
        "valid": true,
        "blocked": plan.blocked(),
        "current": plan.current(),
        "dry_run": !execute,
        "executed": execute,
        "mutated": mutated,
        "adapter": {"id": plan.adapter_id, "version": plan.adapter_version},
        "destination": plan.destination,
        "policy_hash": plan.policy_hash,
        "catalog_identity": plan.catalog_identity,
        "files": files,
        "manifest": {
            "relative_path": plan.manifest_path,
            "identity": plan.manifest.manifest_identity,
            "expected_prior_identity": plan.expected_prior_manifest_identity,
            "freshness": plan.manifest_freshness,
            "schema": plan.manifest.schema,
            "generator_version": plan.manifest.generator_version,
            "removal_candidates": plan.removal_candidates,
        },
        "blockers": plan.blockers,
        "plan_hash": plan.plan_hash,
        "non_claims": plan.non_claims,
    }))
}
