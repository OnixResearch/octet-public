use std::{path::Path, str::FromStr};

use cairn_core::{
    apply_workflow_cob_operation, initialize_workflow_cob, validate_workflow_init,
    workflow_content_root, workflow_state_hash, WorkflowAction, WorkflowInit, WorkflowOperation,
    WorkflowState, BASE_REVISION_KIND_GIT_OID, VERIFICATION_OBLIGATION_COB_TYPE_NAME,
};
use nonempty::NonEmpty;
use radicle_cob::{
    change::Entry, CollaborativeObject, Create, Evaluate, ObjectId, TypeName, Update, Version,
};
use radicle_crypto::{signature::Keypair as _, PublicKey};
use radicle_oid::Oid;
use serde_json::Error as JsonError;
use thiserror::Error;

use crate::store::CobRepository;
use crate::CobMutationOutcome;

pub const CREATE_MESSAGE_PREFIX: &str = "Import verification obligation";
pub const UPDATE_MESSAGE_PREFIX: &str = "Update verification obligation";
pub const PREFLIGHT_OPERATION_ID: &str = "cairn-vo-preflight";

// r[impl cairn.workflow_collaboration.radicle_adapter]
// r[impl cairn.workflow_collaboration.radicle_adapter]

pub static TYPE: std::sync::LazyLock<TypeName> =
    std::sync::LazyLock::new(
        || match TypeName::from_str(VERIFICATION_OBLIGATION_COB_TYPE_NAME) {
            Ok(type_name) => type_name,
            Err(error) => panic!("invalid static verification-obligation COB type name: {error}"),
        },
    );

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairnVerificationObligationCob {
    state: WorkflowState,
}

impl CairnVerificationObligationCob {
    pub fn state(&self) -> &WorkflowState {
        &self.state
    }
}

impl Evaluate<CobRepository> for CairnVerificationObligationCob {
    type Error = VoCobError;

    fn init(entry: &Entry, _store: &CobRepository) -> Result<Self, Self::Error> {
        validate_manifest(entry)?;
        let actions = decode_actions(entry)?;
        validate_root_base(entry, &actions)?;
        let operation = operation(entry, actions);
        Ok(Self {
            state: initialize_workflow_cob(operation)?,
        })
    }

    fn apply<'a, I: Iterator<Item = (&'a Oid, &'a Entry)>>(
        &mut self,
        entry: &Entry,
        _concurrent: I,
        _store: &CobRepository,
    ) -> Result<(), Self::Error> {
        validate_manifest(entry)?;
        let actions = decode_actions(entry)?;
        let operation = operation(entry, actions);
        apply_workflow_cob_operation(&mut self.state, operation)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoCobInspection {
    pub repository_head_oid: String,
    pub object_id: String,
    pub state: WorkflowState,
}

#[derive(Debug, Error)]
pub enum VoCobError {
    #[error(transparent)]
    Store(#[from] crate::CobStoreError),
    #[error("COB operation failed: {0}")]
    Operation(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Cairn rejected the verification-obligation COB operation: {0}")]
    Core(#[from] cairn_core::WorkflowCobError),
    #[error("action JSON is invalid: {0}")]
    ActionJson(#[from] JsonError),
    #[error("unexpected COB type {actual}; expected {expected}")]
    TypeMismatch { expected: String, actual: String },
    #[error("unsupported COB manifest version {actual}; expected {expected}")]
    VersionMismatch { expected: usize, actual: usize },
    #[error("root action base Git object {base} is missing from related objects")]
    MissingRelatedBase { base: String },
    #[error("obligation {obligation_id} already has COB object {object_id}")]
    AlreadyExists {
        obligation_id: String,
        object_id: String,
    },
    #[error("obligation {obligation_id} has no COB")]
    Missing { obligation_id: String },
    #[error("obligation {obligation_id} has multiple COBs: {object_ids:?}")]
    Duplicate {
        obligation_id: String,
        object_ids: Vec<String>,
    },
    #[error("COB object {object_id} is missing")]
    ObjectMissing { object_id: String },
    #[error("keystore path is a symlink: {path}")]
    SymlinkKeystore { path: String },
    #[error("signed actor {actor} does not match planned actor {planned}")]
    ActorDrift { actor: String, planned: String },
    #[error("content root drifted: expected {expected}, observed {observed}")]
    ContentRootDrift { expected: String, observed: String },
    #[error("invalid Git object ID {value}")]
    InvalidGitOid { value: String },
    #[error("signer unavailable: {0}")]
    SignerUnavailable(String),
}

pub fn load_signer(
    keystore_path: &Path,
) -> Result<radicle_crypto::ssh::keystore::MemorySigner, VoCobError> {
    crate::load_signer(keystore_path)
        .map_err(|error| VoCobError::SignerUnavailable(error.to_string()))
}

// r[verify cairn.verification_obligation_cob.interop.outward]
pub fn inspect_vo(
    repository_path: &Path,
    obligation_id: &str,
) -> Result<VoCobInspection, VoCobError> {
    let repository = CobRepository::open(repository_path)?;
    inspect_open_vo(&repository, obligation_id)
}

fn inspect_open_vo(
    repository: &CobRepository,
    obligation_id: &str,
) -> Result<VoCobInspection, VoCobError> {
    let mut matches = Vec::new();
    for object_id in repository.object_ids(&TYPE)? {
        let Some(object) = load_object(repository, object_id)? else {
            return Err(VoCobError::ObjectMissing {
                object_id: object_id.to_string(),
            });
        };
        if object.object.state.obligation_id == obligation_id {
            matches.push(object);
        }
    }
    matches.sort_by(|left, right| left.id.cmp(&right.id));
    if matches.len() > 1 {
        return Err(VoCobError::Duplicate {
            obligation_id: obligation_id.to_string(),
            object_ids: matches.iter().map(|o| o.id.to_string()).collect(),
        });
    }
    let Some(object) = matches.pop() else {
        return Err(VoCobError::Missing {
            obligation_id: obligation_id.to_string(),
        });
    };
    Ok(VoCobInspection {
        repository_head_oid: repository.head_oid()?.to_string(),
        object_id: object.id.to_string(),
        state: object.object.state.clone(),
    })
}

// r[verify cairn.workflow_collaboration.radicle_adapter]
// r[verify cairn.workflow_collaboration.radicle_adapter]
// r[verify cairn.verification_obligation_cob.validation]
// r[verify cairn.workflow_collaboration.radicle_adapter.mapping]
// r[verify cairn.workflow_collaboration.radicle_adapter.mapping]
// r[verify cairn.workflow_collaboration.radicle_adapter.compatibility]
// r[verify cairn.workflow_collaboration.radicle_adapter.boundary]
// r[verify cairn.verification_obligation_cob.interop]
// r[verify cairn.verification_obligation_cob.interop.outward]
// r[verify cairn.verification_obligation_cob.interop.negative]
// r[verify cairn.workflow_collaboration.semantic_core.identity]
// r[verify cairn.workflow_collaboration.receipts]
// r[verify cairn.verification_obligation_cob.receipts]
pub fn create_vo(
    repository_path: &Path,
    keystore_path: &Path,
    init: WorkflowInit,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, VoCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    match inspect_open_vo(&repository, &init.obligation_id) {
        Ok(existing) => {
            return Err(VoCobError::AlreadyExists {
                obligation_id: init.obligation_id,
                object_id: existing.object_id,
            });
        }
        Err(VoCobError::Missing { .. }) => {}
        Err(error) => return Err(error),
    }
    validate_workflow_init(&init)?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    let init_operation = WorkflowOperation {
        id: "init".to_string(),
        actor: actor.clone(),
        role: None,
        actions: vec![WorkflowAction::Init {
            state: init.clone(),
        }],
    };
    let planned_root = workflow_content_root(&initialize_workflow_cob(init_operation)?)?;
    if planned_root != expected_content_root {
        return Err(VoCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let base = required_base_oid(&init)?;
    let action = WorkflowAction::Init { state: init };
    let contents = encode_actions([&action])?;
    let created = radicle_cob::create::<CairnVerificationObligationCob, _, _>(
        &repository,
        &signer,
        None,
        vec![base],
        &public_key,
        Create {
            contents,
            type_name: TYPE.clone(),
            message: format!("{CREATE_MESSAGE_PREFIX} {}", obligation_id_of(&action)),
            embeds: Vec::new(),
            version: Version::default(),
        },
    )
    .map_err(operation_error)?;
    let new_target = *created.id;
    mutation_outcome_or_restore(
        &repository,
        &public_key,
        None,
        new_target,
        created,
        actor,
        expected_content_root,
    )
}

pub fn update_vo(
    repository_path: &Path,
    keystore_path: &Path,
    object_id: &str,
    obligation_id: &str,
    planned_actor: &str,
    actions: &[WorkflowAction],
    expected_content_root: &str,
) -> Result<CobMutationOutcome, VoCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    if actor != planned_actor {
        return Err(VoCobError::ActorDrift {
            actor,
            planned: planned_actor.to_string(),
        });
    }
    let before = load_required_object(&repository, object_id)?;
    let mut planned_state = before.object.state.clone();
    apply_workflow_cob_operation(
        &mut planned_state,
        WorkflowOperation {
            id: PREFLIGHT_OPERATION_ID.to_string(),
            actor: actor.clone(),
            role: None,
            actions: actions.to_vec(),
        },
    )?;
    let planned_root = workflow_content_root(&planned_state)?;
    if planned_root != expected_content_root {
        return Err(VoCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let parsed_object = ObjectId::from_str(object_id).map_err(|_| VoCobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    let prior_target = repository.namespace_ref_target(&public_key, &TYPE, &parsed_object)?;
    let updated = radicle_cob::update::<CairnVerificationObligationCob, _, _>(
        &repository,
        &signer,
        None,
        Vec::new(),
        &public_key,
        Update {
            changes: encode_actions(actions.iter())?,
            object_id: parsed_object,
            type_name: TYPE.clone(),
            message: format!("{UPDATE_MESSAGE_PREFIX} {obligation_id}"),
            embeds: Vec::new(),
        },
    )
    .map_err(operation_error)?;
    mutation_outcome_or_restore(
        &repository,
        &public_key,
        prior_target,
        updated.head,
        updated.object,
        actor,
        expected_content_root,
    )
}

fn load_required_object(
    repository: &CobRepository,
    object_id: &str,
) -> Result<CollaborativeObject<CairnVerificationObligationCob>, VoCobError> {
    let parsed = ObjectId::from_str(object_id).map_err(|_| VoCobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    load_object(repository, parsed)?.ok_or_else(|| VoCobError::ObjectMissing {
        object_id: object_id.to_string(),
    })
}

fn load_object(
    repository: &CobRepository,
    object_id: ObjectId,
) -> Result<Option<CollaborativeObject<CairnVerificationObligationCob>>, VoCobError> {
    radicle_cob::get::<CairnVerificationObligationCob, _>(repository, &TYPE, &object_id)
        .map_err(operation_error)
}

fn mutation_outcome_or_restore(
    repository: &CobRepository,
    namespace: &PublicKey,
    prior_target: Option<Oid>,
    new_target: Oid,
    object: CollaborativeObject<CairnVerificationObligationCob>,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, VoCobError> {
    let object_id = object.id;
    match mutation_outcome(object, new_target, actor, expected_content_root) {
        Ok(outcome) => Ok(outcome),
        Err(mutation) => {
            if let Err(rollback) = repository.restore_namespace_ref(
                namespace,
                &TYPE,
                &object_id,
                new_target,
                prior_target,
            ) {
                return Err(VoCobError::Store(rollback));
            }
            Err(mutation)
        }
    }
}

fn mutation_outcome(
    object: CollaborativeObject<CairnVerificationObligationCob>,
    operation_id: Oid,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, VoCobError> {
    let observed = workflow_content_root(object.object.state())?;
    if observed != expected_content_root {
        return Err(VoCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed,
        });
    }
    Ok(CobMutationOutcome {
        object_id: object.id.to_string(),
        operation_id: operation_id.to_string(),
        actor,
        content_root: expected_content_root.to_string(),
        state_hash: workflow_state_hash(object.object.state())?,
    })
}

fn validate_manifest(entry: &Entry) -> Result<(), VoCobError> {
    if entry.manifest.type_name != *TYPE {
        return Err(VoCobError::TypeMismatch {
            expected: TYPE.to_string(),
            actual: entry.manifest.type_name.to_string(),
        });
    }
    let expected = usize::from(Version::default());
    let actual = usize::from(entry.manifest.version);
    if actual != expected {
        return Err(VoCobError::VersionMismatch { expected, actual });
    }
    Ok(())
}

fn decode_actions(entry: &Entry) -> Result<Vec<WorkflowAction>, VoCobError> {
    entry
        .contents()
        .iter()
        .map(|blob| serde_json::from_slice(blob))
        .collect::<Result<Vec<_>, _>>()
        .map_err(VoCobError::from)
}

fn encode_actions<'a>(
    actions: impl IntoIterator<Item = &'a WorkflowAction>,
) -> Result<NonEmpty<Vec<u8>>, VoCobError> {
    let values = actions
        .into_iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()?;
    NonEmpty::from_vec(values).ok_or(cairn_core::WorkflowCobError::EmptyOperation.into())
}

fn operation(entry: &Entry, actions: Vec<WorkflowAction>) -> WorkflowOperation {
    WorkflowOperation {
        id: entry.id().to_string(),
        actor: entry.author().to_string(),
        role: None,
        actions,
    }
}

fn validate_root_base(entry: &Entry, actions: &[WorkflowAction]) -> Result<(), VoCobError> {
    let [WorkflowAction::Init { state }] = actions else {
        return Err(cairn_core::WorkflowCobError::InvalidRootShape.into());
    };
    let base = required_base_oid(state)?;
    if !entry.related.contains(&base) {
        return Err(VoCobError::MissingRelatedBase {
            base: base.to_string(),
        });
    }
    Ok(())
}

fn required_base_oid(init: &WorkflowInit) -> Result<Oid, VoCobError> {
    let revision = cairn_core::decode_base_revision(&init.base_revision)
        .map_err(|error| cairn_core::WorkflowCobError::Factory(format!("{error}")))?;
    if revision.kind != BASE_REVISION_KIND_GIT_OID {
        return Err(cairn_core::WorkflowCobError::InvalidField {
            field: "base_revision".to_string(),
            reason: "verification-obligation COB requires a git_oid base revision".to_string(),
        }
        .into());
    }
    Oid::from_str(&revision.value).map_err(|_| VoCobError::InvalidGitOid {
        value: revision.value,
    })
}

fn obligation_id_of(action: &WorkflowAction) -> &str {
    match action {
        WorkflowAction::Init { state } => &state.obligation_id,
        _ => "unknown",
    }
}

fn operation_error<E>(error: E) -> VoCobError
where
    E: std::error::Error + Send + Sync + 'static,
{
    VoCobError::Operation(Box::new(error))
}
