use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use cairn_core::{
    admit_lifecycle_store_identity, admit_lifecycle_store_locator, archive_dir,
    build_lifecycle_store_context, canonical_json_hash, changes_dir,
    lifecycle_store_source_identity, normalize_direct_references, parse_policy_json,
    resolve_lifecycle_root, spec_title, specs_dir, AcceptedSpecMetadata, AcceptedSpecSnapshot,
    AgentArtifactGuidance, AgentContextReference, AgentGuidanceRequest, AgentGuidanceResult,
    AgentOperationGuidance, ContextObservations, ContextResult, DirectStoreReference,
    DoctorFindingStatus, ExplicitLifecycleStoreSelector, ExplicitRootSelector, FallbackFacts,
    LifecycleAuthorityEffect, LifecycleLayout, LifecycleStoreDoctorFinding, LifecycleStoreIdentity,
    LifecycleStoreIdentityProjection, LifecycleStoreLocatorProjection, LocatorSnapshot,
    LocatorStoreObservation, NativeRootCandidate, ProjectPointerObservation,
    ReferenceContentBoundary, ReferenceTraversalBoundary, RegistryLookupBoundary,
    RootResolutionObservations, RootResolutionResult, RootResolutionSource,
    StoreIdentityObservation, VcsBoundaryFacts, AGENT_GUIDANCE_OPERATION_CONTEXT,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::support::{print_json, CapabilityRoot, CapabilityRootKind};

const GENERATED_POLICY_JSON: &str =
    include_str!("../../../cairn-policy/generated/cairn-policy.json");
const ROOT_FLAG: &str = "--root";
const LIFECYCLE_STORE_FLAG: &str = "--lifecycle-store";
const CAIRN_DATA_DIR_ENV: &str = "CAIRN_DATA_DIR";
const XDG_DATA_HOME_ENV: &str = "XDG_DATA_HOME";
const HOME_ENV: &str = "HOME";
const CONFIG_READ_MAX_BYTES: u64 = 1_048_576;
const LOCATOR_READ_MAX_BYTES: u64 = 4_194_304;
const SPEC_READ_MAX_BYTES: u64 = 1_048_576;
const MAX_VCS_ANCESTORS: usize = 256;
const OPTION_TOKEN_WIDTH: usize = 2;

#[derive(Debug)]
pub(crate) struct PreparedInvocation {
    pub(crate) args: Vec<String>,
    pub(crate) root_local: Option<RootLocalInvocation>,
}

#[derive(Debug)]
pub(crate) struct RootLocalInvocation {
    pub(crate) resolution: RootResolutionResult,
    project: Option<ObservedProject>,
    locator: LocatorSnapshot,
    policy: ShellPolicy,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectStoreReference {
    store_id: String,
    remote_hint: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectProjection {
    schema: String,
    lifecycle_store: Option<ProjectStoreReference>,
    #[serde(default)]
    references: Vec<ProjectStoreReference>,
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

#[derive(Debug)]
struct ObservedProject {
    root: PathBuf,
    identity: String,
    current: bool,
    projection: ProjectProjection,
}

#[derive(Debug, Deserialize)]
struct GeneratedPolicy {
    lifecycle_store_policy: GeneratedLifecycleStorePolicy,
}

#[derive(Debug, Deserialize)]
struct GeneratedLifecycleStorePolicy {
    bounds: GeneratedBounds,
    locator: GeneratedLocatorPolicy,
    project_config: GeneratedProjectionPolicy,
    root_resolution: GeneratedRootResolutionPolicy,
    store_identity: GeneratedProjectionPolicy,
}

#[derive(Debug, Deserialize)]
struct GeneratedBounds {
    max_context_specs_per_store: usize,
    max_purpose_summary_bytes: usize,
    max_spec_title_bytes: usize,
}

#[derive(Debug, Deserialize)]
struct GeneratedLocatorPolicy {
    relative_path: String,
}

#[derive(Debug, Deserialize)]
struct GeneratedProjectionPolicy {
    projection_path: String,
    projection_schema: String,
    required_non_claims: Vec<String>,
    source_path: String,
}

#[derive(Debug, Deserialize)]
struct GeneratedRootResolutionPolicy {
    consult_global_registry: bool,
    eligible_commands: Vec<String>,
    lifecycle_store_flag: String,
    machine_default_store: bool,
    root_flag: String,
    selectors_mutually_exclusive: bool,
}

#[derive(Debug)]
struct ShellPolicy {
    eligible_commands: Vec<String>,
    locator_relative_path: PathBuf,
    project_projection_path: PathBuf,
    project_projection_schema: String,
    project_required_non_claims: Vec<String>,
    project_source_path: PathBuf,
    store_projection_path: PathBuf,
    store_source_path: PathBuf,
    max_context_specs_per_store: usize,
    max_purpose_summary_bytes: usize,
    max_spec_title_bytes: usize,
}

// r[impl cairn.lifecycle_stores.root_cli]
pub(crate) fn prepare_invocation(
    command: &str,
    args: Vec<String>,
) -> Result<PreparedInvocation, Box<dyn Error>> {
    let policy = ShellPolicy::load()?;
    let command_identity = policy.command_identity(command, &args);
    let resolution_enabled = is_resolution_enabled_command(command_identity.as_deref());
    let explicit_root = read_unique_selector(&args, ROOT_FLAG)?;
    let explicit_store = read_unique_selector(&args, LIFECYCLE_STORE_FLAG)?;

    if explicit_root.is_some() && explicit_store.is_some() {
        return Err(format!(
            "{}: {ROOT_FLAG} and {LIFECYCLE_STORE_FLAG} cannot be used together",
            cairn_core::ROOT_SELECTOR_CONFLICT
        )
        .into());
    }

    if !resolution_enabled {
        if explicit_store.is_some() {
            let identity = command_identity.unwrap_or_else(|| command.to_string());
            return Err(format!(
                "lifecycle_store.root.ineligible_command: {LIFECYCLE_STORE_FLAG} is unavailable for {identity}: command does not support lifecycle-store root resolution"
            )
            .into());
        }
        return Ok(PreparedInvocation {
            args,
            root_local: None,
        });
    }

    let identity = command_identity.ok_or("root-local command identity is unavailable")?;
    if !policy.eligible_commands.contains(&identity) {
        return Err(format!(
            "lifecycle_store.root.ineligible_command: generated policy does not list {identity}"
        )
        .into());
    }

    let root_local = observe_root_local_invocation(&policy, explicit_root, explicit_store)?;
    let selected = root_local
        .resolution
        .selected
        .as_ref()
        .ok_or("root resolution did not select a root")?;
    let rewritten = rewrite_root_selectors(args, &selected.root);
    Ok(PreparedInvocation {
        args: rewritten,
        root_local: Some(root_local),
    })
}

pub(crate) fn attach_resolution(mut value: Value, invocation: &RootLocalInvocation) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "resolution".to_string(),
            resolution_json(&invocation.resolution),
        );
    }
    value
}

// r[impl cairn.agent_guidance_context.integration]
// r[impl cairn.agent_guidance_context.compatibility]
pub(crate) fn run_context(
    args: Vec<String>,
    invocation: &RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    let context = invocation.context_result()?;
    let request = context_guidance_request(&args)?;
    let guidance = invocation.agent_guidance(request)?;
    let valid = context.valid && guidance.as_ref().is_none_or(|result| result.valid);
    let mut value = attach_resolution(context_result_json(&context), invocation);
    if let Some(guidance) = guidance {
        value["guidance"] = crate::agent_guidance_shell::agent_guidance_json(&guidance);
    }
    print_json(value)?;
    if !valid {
        return Err("lifecycle-store context or agent guidance is invalid".into());
    }
    Ok(())
}

fn context_guidance_request(args: &[String]) -> Result<AgentGuidanceRequest, Box<dyn Error>> {
    let mut profile = None;
    let mut artifact = None;
    let mut operation = None;
    let mut index = 0;
    while index < args.len() {
        let target = match args[index].as_str() {
            "--profile" => &mut profile,
            "--artifact" => &mut artifact,
            "--operation" => &mut operation,
            ROOT_FLAG | LIFECYCLE_STORE_FLAG => {
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
                continue;
            }
            unknown => return Err(format!("unknown context argument: {unknown}").into()),
        };
        if target.is_some() {
            return Err(format!("context received {} more than once", args[index]).into());
        }
        *target = Some(
            args.get(index + 1)
                .ok_or_else(|| format!("context {} requires a value", args[index]))?
                .clone(),
        );
        index = index.saturating_add(OPTION_TOKEN_WIDTH);
    }
    Ok(AgentGuidanceRequest {
        profile,
        artifact,
        operation: operation.or_else(|| Some(AGENT_GUIDANCE_OPERATION_CONTEXT.to_string())),
    })
}

pub(crate) fn context_result_json(result: &ContextResult) -> Value {
    json!({
        "valid": result.valid,
        "healthy": result.healthy,
        "selected_root": result.selected_root.as_ref().map(resolved_root_json),
        "direct_references": result.direct_references.iter().map(|reference| json!({
            "store_id": reference.store_id,
            "remote_hint": reference.remote_hint,
        })).collect::<Vec<_>>(),
        "specs": result.specs.iter().map(|spec| json!({
            "store_id": spec.store_id,
            "store_identity": spec.store_identity,
            "spec_id": spec.spec_id,
            "title": spec.title,
            "purpose_summary": spec.purpose_summary,
            "fetch_command": spec.fetch_command,
        })).collect::<Vec<_>>(),
        "reference_health": result.doctor_findings.iter().map(doctor_finding_json).collect::<Vec<_>>(),
        "boundary": {
            "traversal": reference_traversal_name(result.boundary.traversal),
            "content": reference_content_name(result.boundary.content),
            "registry_lookup": registry_lookup_name(result.boundary.registry_lookup),
            "authority_effect": authority_effect_name(result.boundary.authority_effect),
        },
    })
}

pub(crate) fn resolution_json(result: &RootResolutionResult) -> Value {
    json!({
        "valid": result.valid,
        "selected": result.selected.as_ref().map(resolved_root_json),
        "diagnostics": result.findings.iter().map(doctor_finding_json).collect::<Vec<_>>(),
        "locator_consulted": result.locator_consulted,
        "registry_lookup": registry_lookup_name(result.registry_lookup),
    })
}

impl RootLocalInvocation {
    pub(crate) fn project_root(&self) -> Option<&Path> {
        self.project.as_ref().map(|project| project.root.as_path())
    }

    pub(crate) fn selected_root(&self) -> Result<&Path, Box<dyn Error>> {
        self.resolution
            .selected
            .as_ref()
            .map(|selected| Path::new(&selected.root))
            .ok_or_else(|| "lifecycle root is unavailable".into())
    }

    pub(crate) fn agent_guidance(
        &self,
        request: AgentGuidanceRequest,
    ) -> Result<Option<AgentGuidanceResult>, Box<dyn Error>> {
        let lifecycle_root = self.selected_root()?;
        let project_root = self.project_root().unwrap_or(lifecycle_root);
        let policy = parse_policy_json(GENERATED_POLICY_JSON)?;
        crate::agent_guidance_shell::load_agent_guidance(
            project_root,
            lifecycle_root,
            request,
            &[],
            &policy,
        )
    }

    pub(crate) fn context_result(&self) -> Result<ContextResult, Box<dyn Error>> {
        let references = self
            .project
            .as_ref()
            .filter(|project| project.current)
            .map(ObservedProject::direct_references)
            .unwrap_or_default();
        let subject_store_id = self
            .resolution
            .selected
            .as_ref()
            .and_then(|selected| selected.store_id.as_deref());
        let normalized = normalize_direct_references(subject_store_id, &references);
        let accepted_specs = if normalized.valid {
            collect_direct_accepted_spec_snapshots(
                &normalized.references,
                &self.locator,
                &self.policy,
            )?
        } else {
            Vec::new()
        };
        Ok(build_lifecycle_store_context(&ContextObservations {
            resolution: self.resolution.clone(),
            direct_references: references,
            locator: self.locator.clone(),
            accepted_specs,
        }))
    }
}

impl ObservedProject {
    fn direct_references(&self) -> Vec<DirectStoreReference> {
        self.projection
            .references
            .iter()
            .map(|reference| DirectStoreReference {
                store_id: reference.store_id.clone(),
                remote_hint: reference.remote_hint.clone(),
            })
            .collect()
    }
}

impl ShellPolicy {
    fn load() -> Result<Self, Box<dyn Error>> {
        let generated: GeneratedPolicy = serde_json::from_str(GENERATED_POLICY_JSON)?;
        let policy = generated.lifecycle_store_policy;
        if policy.root_resolution.root_flag != ROOT_FLAG
            || policy.root_resolution.lifecycle_store_flag != LIFECYCLE_STORE_FLAG
            || !policy.root_resolution.selectors_mutually_exclusive
            || policy.root_resolution.consult_global_registry
            || policy.root_resolution.machine_default_store
        {
            return Err("generated lifecycle-store root policy violates the shell boundary".into());
        }
        Ok(Self {
            eligible_commands: policy.root_resolution.eligible_commands,
            locator_relative_path: PathBuf::from(policy.locator.relative_path),
            project_projection_path: PathBuf::from(policy.project_config.projection_path),
            project_projection_schema: policy.project_config.projection_schema,
            project_required_non_claims: policy.project_config.required_non_claims,
            project_source_path: PathBuf::from(policy.project_config.source_path),
            store_projection_path: PathBuf::from(policy.store_identity.projection_path),
            store_source_path: PathBuf::from(policy.store_identity.source_path),
            max_context_specs_per_store: policy.bounds.max_context_specs_per_store,
            max_purpose_summary_bytes: policy.bounds.max_purpose_summary_bytes,
            max_spec_title_bytes: policy.bounds.max_spec_title_bytes,
        })
    }

    fn command_identity(&self, command: &str, args: &[String]) -> Option<String> {
        let mut tokens = Vec::with_capacity(args.len().saturating_add(1));
        tokens.push(command);
        tokens.extend(args.iter().map(String::as_str));
        self.eligible_commands
            .iter()
            .filter(|candidate| command_words_match(candidate, &tokens))
            .max_by_key(|candidate| candidate.split_whitespace().count())
            .cloned()
    }
}

fn observe_root_local_invocation(
    policy: &ShellPolicy,
    explicit_root: Option<String>,
    explicit_store: Option<String>,
) -> Result<RootLocalInvocation, Box<dyn Error>> {
    let current_directory = env::current_dir()?;
    let (vcs_boundary, ancestors) = observe_vcs_boundary(&current_directory)?;
    let mut native_roots = Vec::new();
    let mut projects = Vec::new();

    for (ancestor_distance, ancestor) in ancestors.iter().enumerate() {
        if native_lifecycle_root_is_present(ancestor)? {
            native_roots.push(NativeRootCandidate {
                root: path_text(ancestor),
                ancestor_distance,
                store: observe_current_store_identity(ancestor, policy)?.map(|identity| {
                    StoreIdentityObservation {
                        store_id: identity.store_id.as_str().to_string(),
                        store_identity: identity.store_identity.as_str().to_string(),
                    }
                }),
            });
        }
        if let Some(project) = observe_project(ancestor, policy)? {
            projects.push((ancestor_distance, project));
        }
    }

    let project_pointers =
        projects
            .iter()
            .filter_map(|(ancestor_distance, project)| {
                project.projection.lifecycle_store.as_ref().map(|pointer| {
                    ProjectPointerObservation {
                        project_root: path_text(&project.root),
                        project_identity: project.identity.clone(),
                        store_id: pointer.store_id.clone(),
                        ancestor_distance: *ancestor_distance,
                        projection_current: project.current,
                    }
                })
            })
            .collect::<Vec<_>>();

    let locator_needed = explicit_store.is_some()
        || (native_roots.is_empty()
            && project_pointers
                .iter()
                .any(|pointer| pointer.projection_current));
    let mut locator = if locator_needed {
        observe_local_locator(policy)?
    } else {
        LocatorSnapshot::default()
    };

    let observations = RootResolutionObservations {
        explicit_root: explicit_root.map(|root| ExplicitRootSelector { root }),
        explicit_lifecycle_store: explicit_store
            .map(|store_id| ExplicitLifecycleStoreSelector { store_id }),
        vcs_boundary,
        native_roots,
        project_pointers,
        locator: locator.clone(),
        fallback: FallbackFacts {
            current_directory: path_text(&current_directory),
            classic_root: path_text(&current_directory),
        },
    };
    let mut resolution = resolve_lifecycle_root(&observations);
    if !resolution.valid {
        return Err(root_resolution_error(&resolution).into());
    }

    let selected = resolution
        .selected
        .as_ref()
        .ok_or("valid root resolution omitted its selected root")?;
    let project = if selected.source == RootResolutionSource::ProjectPointer {
        projects
            .into_iter()
            .map(|(_, project)| project)
            .find(|project| selected.project_identity.as_deref() == Some(&project.identity))
    } else {
        observe_project(Path::new(&selected.root), policy)?
    };

    if let Some(project) = &project {
        if let Some(selected) = resolution.selected.as_mut() {
            if selected.project_identity.is_none() {
                selected.project_identity = Some(project.identity.clone());
            }
        }
        if !project.projection.references.is_empty() && !locator_needed {
            locator = observe_local_locator(policy)?;
        }
    }

    Ok(RootLocalInvocation {
        resolution,
        project,
        locator,
        policy: ShellPolicy {
            eligible_commands: policy.eligible_commands.clone(),
            locator_relative_path: policy.locator_relative_path.clone(),
            project_projection_path: policy.project_projection_path.clone(),
            project_projection_schema: policy.project_projection_schema.clone(),
            project_required_non_claims: policy.project_required_non_claims.clone(),
            project_source_path: policy.project_source_path.clone(),
            store_projection_path: policy.store_projection_path.clone(),
            store_source_path: policy.store_source_path.clone(),
            max_context_specs_per_store: policy.max_context_specs_per_store,
            max_purpose_summary_bytes: policy.max_purpose_summary_bytes,
            max_spec_title_bytes: policy.max_spec_title_bytes,
        },
    })
}

fn observe_vcs_boundary(
    current_directory: &Path,
) -> Result<(VcsBoundaryFacts, Vec<PathBuf>), Box<dyn Error>> {
    let mut ancestors = Vec::new();
    let mut current = Some(current_directory);
    for ancestor_distance in 0..MAX_VCS_ANCESTORS {
        let Some(ancestor) = current else {
            break;
        };
        ancestors.push(ancestor.to_path_buf());
        if vcs_marker_is_present(ancestor)? {
            return Ok((
                VcsBoundaryFacts::Known {
                    root: path_text(ancestor),
                    max_ancestor_distance: ancestor_distance,
                },
                ancestors,
            ));
        }
        current = ancestor.parent();
    }
    Ok((
        VcsBoundaryFacts::Unknown,
        vec![current_directory.to_path_buf()],
    ))
}

fn vcs_marker_is_present(root: &Path) -> io::Result<bool> {
    Ok(path_entry_exists(&root.join(".git"))? || path_entry_exists(&root.join(".jj"))?)
}

fn native_lifecycle_root_is_present(root: &Path) -> io::Result<bool> {
    // r[impl cairn.lifecycle_directory.hidden_default.runtime]
    // r[impl cairn.lifecycle_directory.hidden_default.project_pointer]
    let layout = LifecycleLayout::Native;
    let lifecycle_directories = [
        specs_dir(layout),
        changes_dir(layout),
        archive_dir(layout).to_string(),
    ];
    for directory in lifecycle_directories {
        if path_is_directory(&root.join(directory))? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn observe_project(
    root: &Path,
    policy: &ShellPolicy,
) -> Result<Option<ObservedProject>, Box<dyn Error>> {
    let source_path = root.join(&policy.project_source_path);
    let projection_path = root.join(&policy.project_projection_path);
    let source_present = path_is_regular_file(&source_path)?;
    let projection_present = path_is_regular_file(&projection_path)?;
    if !source_present && !projection_present {
        return Ok(None);
    }
    if !source_present || !projection_present {
        return Err(format!(
            "checked project configuration requires both {} and {}",
            source_path.display(),
            projection_path.display()
        )
        .into());
    }

    let source = read_bounded_file(&source_path, CONFIG_READ_MAX_BYTES)?;
    let projection_bytes = read_bounded_file(&projection_path, CONFIG_READ_MAX_BYTES)?;
    let projection_value: Value = serde_json::from_slice(&projection_bytes)?;
    let projection: ProjectProjection = serde_json::from_value(projection_value.clone())?;
    let _guidance_entry_count = projection
        .context_refs
        .len()
        .saturating_add(projection.artifact_guidance.len())
        .saturating_add(projection.operation_guidance.len());
    if projection.schema != policy.project_projection_schema {
        return Err(format!(
            "unsupported project projection schema: {}",
            projection.schema
        )
        .into());
    }
    for required in &policy.project_required_non_claims {
        if !projection.non_claims.contains(required) {
            return Err(
                format!("project projection is missing required non-claim: {required}").into(),
            );
        }
    }
    let current = lifecycle_store_source_identity(&source) == projection.source_blake3;
    Ok(Some(ObservedProject {
        root: root.to_path_buf(),
        identity: canonical_json_hash(&projection_value),
        current,
        projection,
    }))
}

fn observe_local_locator(policy: &ShellPolicy) -> Result<LocatorSnapshot, Box<dyn Error>> {
    let Some(data_root) = platform_data_root() else {
        return Ok(LocatorSnapshot::default());
    };
    let locator_path = data_root.join(&policy.locator_relative_path);
    if !path_entry_exists(&locator_path)? {
        return Ok(LocatorSnapshot::default());
    }
    let bytes = read_bounded_file(&locator_path, LOCATOR_READ_MAX_BYTES)?;
    let projection: LifecycleStoreLocatorProjection = serde_json::from_slice(&bytes)?;
    let admission = admit_lifecycle_store_locator(&projection);
    let state = admission.state.ok_or_else(|| {
        let diagnostics = admission
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>()
            .join("; ");
        format!("local lifecycle-store locator is invalid: {diagnostics}")
    })?;

    let mut stores = Vec::with_capacity(state.mappings.len());
    for mapping in state.mappings {
        let root = PathBuf::from(mapping.checkout_path.as_str());
        let observed = observe_current_store_identity(&root, policy)?;
        let registration_current = observed.as_ref().is_some_and(|identity| {
            identity.store_id.as_str() == mapping.store_id.as_str()
                && identity.store_identity.as_str() == mapping.store_identity.as_str()
                && native_lifecycle_root_is_present(&root).unwrap_or(false)
        });
        stores.push(LocatorStoreObservation {
            store_id: mapping.store_id.as_str().to_string(),
            root: path_text(&root),
            store_identity: mapping.store_identity.as_str().to_string(),
            registration_current,
        });
    }
    Ok(LocatorSnapshot { stores })
}

fn observe_current_store_identity(
    root: &Path,
    policy: &ShellPolicy,
) -> Result<Option<LifecycleStoreIdentity>, Box<dyn Error>> {
    if !path_is_directory(root)? || !native_lifecycle_root_is_present(root)? {
        return Ok(None);
    }
    let source_path = root.join(&policy.store_source_path);
    let projection_path = root.join(&policy.store_projection_path);
    if !path_is_regular_file(&source_path)? || !path_is_regular_file(&projection_path)? {
        return Ok(None);
    }
    let source = read_bounded_file(&source_path, CONFIG_READ_MAX_BYTES)?;
    let projection_bytes = read_bounded_file(&projection_path, CONFIG_READ_MAX_BYTES)?;
    let projection: LifecycleStoreIdentityProjection =
        match serde_json::from_slice(&projection_bytes) {
            Ok(projection) => projection,
            Err(_) => return Ok(None),
        };
    if lifecycle_store_source_identity(&source) != projection.source_identity {
        return Ok(None);
    }
    let admission = admit_lifecycle_store_identity(&projection);
    Ok(admission.identity)
}

fn platform_data_root() -> Option<PathBuf> {
    env::var_os(CAIRN_DATA_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os(XDG_DATA_HOME_ENV)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        })
        .or_else(|| {
            env::var_os(HOME_ENV)
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".local/share"))
        })
}

fn collect_direct_accepted_spec_snapshots(
    references: &[DirectStoreReference],
    locator: &LocatorSnapshot,
    policy: &ShellPolicy,
) -> Result<Vec<AcceptedSpecSnapshot>, Box<dyn Error>> {
    let mut snapshots = Vec::new();
    for reference in references {
        let matches = locator
            .stores
            .iter()
            .filter(|store| store.store_id == reference.store_id)
            .collect::<Vec<_>>();
        if matches.len() != 1 || !matches[0].registration_current {
            continue;
        }
        let store = matches[0];
        snapshots.push(AcceptedSpecSnapshot {
            store_id: store.store_id.clone(),
            store_identity: store.store_identity.clone(),
            specs: read_accepted_spec_metadata(Path::new(&store.root), policy)?,
        });
    }
    Ok(snapshots)
}

fn read_accepted_spec_metadata(
    store_root: &Path,
    policy: &ShellPolicy,
) -> Result<Vec<AcceptedSpecMetadata>, Box<dyn Error>> {
    let capability = CapabilityRoot::open(CapabilityRootKind::Repository, store_root)?;
    let accepted_root = PathBuf::from(".cairn/specs");
    let mut pending = vec![accepted_root.clone()];
    let mut spec_paths = Vec::new();
    let directory_entry_limit = policy.max_context_specs_per_store.saturating_add(1);

    while let Some(directory) = pending.pop() {
        let entries = capability.entries_bounded(&directory, directory_entry_limit)?;
        for entry in entries {
            let relative = directory.join(entry.file_name());
            if entry.file_type().is_dir() {
                pending.push(relative);
            } else if relative.file_name().is_some_and(|name| name == "spec.md") {
                if spec_paths.len() >= policy.max_context_specs_per_store {
                    return Err("accepted-spec index exceeds the generated policy bound".into());
                }
                spec_paths.push(relative);
            }
        }
    }
    spec_paths.sort();

    let mut metadata = Vec::with_capacity(spec_paths.len());
    for path in spec_paths {
        let markdown = String::from_utf8(capability.read_bounded(&path, SPEC_READ_MAX_BYTES)?)?;
        let title = spec_title(&markdown).unwrap_or_default();
        let purpose_summary = purpose_summary(&markdown);
        if title.len() > policy.max_spec_title_bytes {
            return Err(format!(
                "accepted spec title exceeds the policy bound: {}",
                path.display()
            )
            .into());
        }
        if purpose_summary.len() > policy.max_purpose_summary_bytes {
            return Err(format!(
                "accepted spec purpose exceeds the policy bound: {}",
                path.display()
            )
            .into());
        }
        metadata.push(AcceptedSpecMetadata {
            spec_id: accepted_spec_id(&accepted_root, &path)?,
            title,
            purpose_summary,
        });
    }
    Ok(metadata)
}

fn purpose_summary(markdown: &str) -> String {
    let mut in_purpose = false;
    let mut lines = Vec::new();
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed == "## Purpose" {
            in_purpose = true;
            continue;
        }
        if in_purpose && trimmed.starts_with('#') {
            break;
        }
        if in_purpose && !trimmed.is_empty() {
            lines.push(trimmed);
        }
    }
    lines.join(" ")
}

fn accepted_spec_id(accepted_root: &Path, path: &Path) -> Result<String, Box<dyn Error>> {
    let relative = path.strip_prefix(accepted_root)?;
    let parent = relative
        .parent()
        .filter(|parent| *parent != Path::new(""))
        .ok_or("accepted spec path has no store-local identity")?;
    let id = parent
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    if id.is_empty() || id == "." {
        return Err("accepted spec identity is empty".into());
    }
    Ok(id)
}

fn read_bounded_file(path: &Path, max_bytes: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let (root, file_name) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Repository, path, false)?;
    Ok(root.read_bounded(&file_name, max_bytes)?)
}

fn path_entry_exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("symlink path is not allowed: {}", path.display()),
                ));
            }
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn path_is_directory(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Ok(false);
            }
            Ok(metadata.is_dir())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn path_is_regular_file(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("symlink file is not allowed: {}", path.display()),
                ));
            }
            Ok(metadata.is_file())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn read_unique_selector(args: &[String], flag: &str) -> Result<Option<String>, Box<dyn Error>> {
    let mut selected = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == flag {
            if selected.is_some() {
                return Err(format!("duplicate root selector: {flag}").into());
            }
            let value = args
                .get(index.saturating_add(1))
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| format!("{flag} requires a value"))?;
            selected = Some(value.clone());
            index = index.saturating_add(OPTION_TOKEN_WIDTH);
        } else {
            index = index.saturating_add(1);
        }
    }
    Ok(selected)
}

fn rewrite_root_selectors(args: Vec<String>, root: &str) -> Vec<String> {
    let mut rewritten = Vec::with_capacity(args.len().saturating_add(OPTION_TOKEN_WIDTH));
    let mut index = 0;
    while index < args.len() {
        if args[index] == ROOT_FLAG || args[index] == LIFECYCLE_STORE_FLAG {
            index = index.saturating_add(OPTION_TOKEN_WIDTH);
        } else {
            rewritten.push(args[index].clone());
            index = index.saturating_add(1);
        }
    }
    rewritten.push(ROOT_FLAG.to_string());
    rewritten.push(root.to_string());
    rewritten
}

fn command_words_match(candidate: &str, tokens: &[&str]) -> bool {
    let words = candidate.split_whitespace().collect::<Vec<_>>();
    words.len() <= tokens.len()
        && words
            .iter()
            .zip(tokens.iter())
            .all(|(word, token)| word == token)
}

fn is_resolution_enabled_command(identity: Option<&str>) -> bool {
    matches!(
        identity,
        Some(
            "status"
                | "spec list"
                | "spec show"
                | "validate"
                | "context"
                | "workflow status"
                | "workflow instructions"
                | "doctor"
        )
    )
}

fn root_resolution_error(result: &RootResolutionResult) -> String {
    result
        .findings
        .iter()
        .map(|finding| format!("{}: {}", finding.code, finding.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn resolved_root_json(root: &cairn_core::ResolvedLifecycleRoot) -> Value {
    json!({
        "root": root.root,
        "source": root_resolution_source_name(root.source),
        "store_id": root.store_id,
        "store_identity": root.store_identity,
        "project_identity": root.project_identity,
    })
}

fn doctor_finding_json(finding: &LifecycleStoreDoctorFinding) -> Value {
    json!({
        "code": finding.code,
        "status": doctor_finding_status_name(finding.status),
        "subject": finding.subject,
        "message": finding.message,
        "repair_command": finding.repair_command,
        "remote_hint": finding.remote_hint,
        "manual_clone_hint": finding.remote_hint.as_ref().map(|remote| {
            format!("git clone '{remote}' '<checkout-path>'")
        }),
    })
}

fn root_resolution_source_name(source: RootResolutionSource) -> &'static str {
    match source {
        RootResolutionSource::ExplicitRoot => "explicit_root",
        RootResolutionSource::ExplicitLifecycleStore => "explicit_lifecycle_store",
        RootResolutionSource::NativeRoot => "native_root",
        RootResolutionSource::ProjectPointer => "project_pointer",
        RootResolutionSource::ClassicFallback => "classic_fallback",
    }
}

fn doctor_finding_status_name(status: DoctorFindingStatus) -> &'static str {
    match status {
        DoctorFindingStatus::Fail => "fail",
        DoctorFindingStatus::Warn => "warn",
        DoctorFindingStatus::Ok => "ok",
    }
}

fn registry_lookup_name(boundary: RegistryLookupBoundary) -> &'static str {
    match boundary {
        RegistryLookupBoundary::LocalLocatorOnly => "local_locator_only",
    }
}

fn reference_traversal_name(boundary: ReferenceTraversalBoundary) -> &'static str {
    match boundary {
        ReferenceTraversalBoundary::DirectOnly => "direct_only",
    }
}

fn reference_content_name(boundary: ReferenceContentBoundary) -> &'static str {
    match boundary {
        ReferenceContentBoundary::AcceptedSpecsOnly => "accepted_specs_only",
    }
}

fn authority_effect_name(effect: LifecycleAuthorityEffect) -> &'static str {
    match effect {
        LifecycleAuthorityEffect::None => "none",
    }
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
