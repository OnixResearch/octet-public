use octet_architecture_ir::CompilerArchitectureCoverage;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::CompilerArchitectureReceipt;
use octet_standards::project_architecture_gate::ProjectArchitectureGateInput;
use octet_standards::project_architecture_gate::ProjectArchitectureReceipt;
use octet_standards::project_architecture_gate::evaluate_project_architecture;
use octet_standards::project_architecture_gate::verify_project_architecture_receipt;

use crate::architecture_ir::ArchitectureArtifacts;
use crate::architecture_ir::PreparedCollector;
use crate::config::SelectedProjectArchitecturePolicy;

pub(crate) const PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME: &str = "project-architecture-receipt.json";
pub(crate) const PROJECT_ARCHITECTURE_CARGO_GRAPH_ARTIFACT_NAME: &str = "project-architecture-cargo-graph.json";
pub(crate) const PROJECT_ARCHITECTURE_POLICY_SOURCE_ARTIFACT_NAME: &str = "project-architecture-policy-source.ncl";
pub(crate) const PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME: &str = "project-architecture-policy.json";
pub(crate) const PROJECT_ARCHITECTURE_PROVENANCE_SOURCE_ID: &str = "octet.project-architecture-gate.receipt";

#[derive(Clone)]
pub(crate) struct ProjectArchitectureArtifacts {
    pub receipt: ProjectArchitectureReceipt,
    pub receipt_json: String,
    pub cargo_graph_json: String,
    pub policy_source: String,
    pub policy_json: String,
}

pub(crate) fn evaluate(
    policy: &SelectedProjectArchitecturePolicy,
    prepared: &PreparedCollector,
    architecture: &ArchitectureArtifacts,
    command_line: &str,
    lint_phase_status: &str,
    effect_closure: Option<&octet_standards::project_architecture_gate::EffectClosureParent>,
) -> Result<ProjectArchitectureArtifacts, String> {
    let architecture_ir =
        parse_artifact::<CompilerArchitectureIr>(&architecture.architecture_ir, "compiler Architecture IR")?;
    let coverage =
        parse_artifact::<CompilerArchitectureCoverage>(&architecture.coverage, "compiler architecture coverage")?;
    let compiler_receipt =
        parse_artifact::<CompilerArchitectureReceipt>(&architecture.receipt, "compiler architecture receipt")?;
    let command_blake3 = crate::evidence::artifact_blake3_hex(format!("{command_line}\n").as_bytes());
    let receipt = evaluate_project_architecture(&ProjectArchitectureGateInput {
        policy: &policy.policy,
        policy_source_blake3: &policy.source_blake3,
        policy_blake3: &policy.policy_blake3,
        lint_phase_status,
        command_blake3: &command_blake3,
        cargo_graph: &prepared.cargo_graph,
        architecture_ir: &architecture_ir,
        coverage: &coverage,
        compiler_receipt: &compiler_receipt,
        effect_closure,
    });
    let diagnostics = verify_project_architecture_receipt(&receipt);
    if !diagnostics.is_empty() {
        return Err(format!("new project architecture receipt failed self-verification: {}", diagnostics.join("; ")));
    }
    Ok(ProjectArchitectureArtifacts {
        receipt_json: render_json(&receipt, "project architecture receipt")?,
        cargo_graph_json: render_json(&prepared.cargo_graph, "project architecture Cargo graph")?,
        policy_source: std::fs::read_to_string(&policy.source_path)
            .map_err(|error| format!("read architecture policy source {}: {error}", policy.source_path.display()))?,
        policy_json: std::fs::read_to_string(&policy.export_path)
            .map_err(|error| format!("read architecture policy export {}: {error}", policy.export_path.display()))?,
        receipt,
    })
}

fn parse_artifact<T: serde::de::DeserializeOwned>(contents: &str, label: &str) -> Result<T, String> {
    serde_json::from_str(contents).map_err(|error| format!("parse {label}: {error}"))
}

fn render_json<T: serde::Serialize>(value: &T, label: &str) -> Result<String, String> {
    let mut rendered = serde_json::to_string_pretty(value).map_err(|error| format!("serialize {label}: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}
