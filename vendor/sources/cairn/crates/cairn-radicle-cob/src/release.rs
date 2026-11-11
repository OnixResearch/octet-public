use std::{path::Path, str::FromStr};

use cairn_core::{
    apply_release_operation, initialize_release_decision, release_candidate_root,
    release_content_root, release_state_hash, validate_release_init, ReleaseDecisionAction,
    ReleaseDecisionInit, ReleaseDecisionOperation, ReleaseDecisionState,
    RELEASE_DECISION_COB_TYPE_NAME,
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

pub const CREATE_MESSAGE_PREFIX: &str = "Import release decision";
pub const UPDATE_MESSAGE_PREFIX: &str = "Update release decision";
pub const PREFLIGHT_OPERATION_ID: &str = "cairn-rd-preflight";

// r[impl cairn.release_decision_cob.backends]
// r[impl cairn.release_decision_cob.candidate]
// r[impl cairn.release_decision_cob.candidate.provider]
// r[impl cairn.release_decision_cob.candidate.immutable]
// r[impl cairn.release_decision_cob.backends.boundary]
// r[impl cairn.release_decision_cob.interop]
// r[impl cairn.release_decision_cob.interop.negative]
// r[impl cairn.release_decision_cob.policy]
// r[impl cairn.release_decision_cob.policy.deterministic]
// r[impl cairn.release_decision_cob.policy.invalid]
// r[verify cairn.release_decision_cob.interop.outward]

pub static TYPE: std::sync::LazyLock<TypeName> =
    std::sync::LazyLock::new(
        || match TypeName::from_str(RELEASE_DECISION_COB_TYPE_NAME) {
            Ok(type_name) => type_name,
            Err(error) => panic!("invalid static release-decision COB type name: {error}"),
        },
    );

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairnReleaseDecisionCob {
    state: ReleaseDecisionState,
}

impl CairnReleaseDecisionCob {
    pub fn state(&self) -> &ReleaseDecisionState {
        &self.state
    }
}

impl Evaluate<CobRepository> for CairnReleaseDecisionCob {
    type Error = ReleaseCobError;

    fn init(entry: &Entry, _store: &CobRepository) -> Result<Self, Self::Error> {
        validate_manifest(entry)?;
        let actions = decode_actions(entry)?;
        validate_root_base(entry, &actions)?;
        let operation = operation(entry, actions);
        Ok(Self {
            state: initialize_release_decision(operation)?,
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
        apply_release_operation(&mut self.state, operation(entry, actions))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseDecisionInspection {
    pub repository_head_oid: String,
    pub object_id: String,
    pub state: ReleaseDecisionState,
}

#[derive(Debug, Error)]
pub enum ReleaseCobError {
    #[error(transparent)]
    Store(#[from] crate::CobStoreError),
    #[error("COB operation failed: {0}")]
    Operation(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Cairn rejected the release-decision COB operation: {0}")]
    Core(#[from] cairn_core::ReleaseDecisionError),
    #[error("action JSON is invalid: {0}")]
    ActionJson(#[from] JsonError),
    #[error("unexpected COB type {actual}; expected {expected}")]
    TypeMismatch { expected: String, actual: String },
    #[error("unsupported COB manifest version {actual}; expected {expected}")]
    VersionMismatch { expected: usize, actual: usize },
    #[error("root action base Git object {base} is missing from related objects")]
    MissingRelatedBase { base: String },
    #[error("decision {decision_id} already has COB object {object_id}")]
    AlreadyExists {
        decision_id: String,
        object_id: String,
    },
    #[error("decision {decision_id} has no COB")]
    Missing { decision_id: String },
    #[error("decision {decision_id} has multiple COBs: {object_ids:?}")]
    Duplicate {
        decision_id: String,
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
) -> Result<radicle_crypto::ssh::keystore::MemorySigner, ReleaseCobError> {
    crate::load_signer(keystore_path)
        .map_err(|error| ReleaseCobError::SignerUnavailable(error.to_string()))
}

// r[verify cairn.release_decision_cob.interop.outward]
pub fn inspect_release(
    repository_path: &Path,
    decision_id: &str,
) -> Result<ReleaseDecisionInspection, ReleaseCobError> {
    let repository = CobRepository::open(repository_path)?;
    inspect_open_release(&repository, decision_id)
}

fn inspect_open_release(
    repository: &CobRepository,
    decision_id: &str,
) -> Result<ReleaseDecisionInspection, ReleaseCobError> {
    let mut matches = Vec::new();
    for object_id in repository.object_ids(&TYPE)? {
        let Some(object) = load_object(repository, object_id)? else {
            return Err(ReleaseCobError::ObjectMissing {
                object_id: object_id.to_string(),
            });
        };
        if object.object.state.decision_id == decision_id {
            matches.push(object);
        }
    }
    matches.sort_by(|left, right| left.id.cmp(&right.id));
    if matches.len() > 1 {
        return Err(ReleaseCobError::Duplicate {
            decision_id: decision_id.to_string(),
            object_ids: matches.iter().map(|o| o.id.to_string()).collect(),
        });
    }
    let Some(object) = matches.pop() else {
        return Err(ReleaseCobError::Missing {
            decision_id: decision_id.to_string(),
        });
    };
    Ok(ReleaseDecisionInspection {
        repository_head_oid: repository.head_oid()?.to_string(),
        object_id: object.id.to_string(),
        state: object.object.state.clone(),
    })
}

pub fn create_release(
    repository_path: &Path,
    keystore_path: &Path,
    init: ReleaseDecisionInit,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, ReleaseCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    match inspect_open_release(&repository, &init.decision_id) {
        Ok(existing) => {
            return Err(ReleaseCobError::AlreadyExists {
                decision_id: init.decision_id,
                object_id: existing.object_id,
            });
        }
        Err(ReleaseCobError::Missing { .. }) => {}
        Err(error) => return Err(error),
    }
    validate_release_init(&init)?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    let init_operation = ReleaseDecisionOperation {
        id: "init".to_string(),
        actor: actor.clone(),
        role: None,
        actions: vec![ReleaseDecisionAction::Init {
            state: init.clone(),
        }],
    };
    let init_candidate_root = release_candidate_root(&init.candidate, &init.policy_identity)?;
    if init_candidate_root != init.candidate_root {
        return Err(ReleaseCobError::ContentRootDrift {
            expected: init.candidate_root.clone(),
            observed: init_candidate_root,
        });
    }
    let planned_root = release_content_root(&initialize_release_decision(init_operation)?)?;
    if planned_root != expected_content_root {
        return Err(ReleaseCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let base = required_base_oid(&init)?;
    let action = ReleaseDecisionAction::Init { state: init };
    let contents = encode_actions([&action])?;
    let created = radicle_cob::create::<CairnReleaseDecisionCob, _, _>(
        &repository,
        &signer,
        None,
        vec![base],
        &public_key,
        Create {
            contents,
            type_name: TYPE.clone(),
            message: format!("{CREATE_MESSAGE_PREFIX} {}", decision_id_of(&action)),
            embeds: Vec::new(),
            version: Version::default(),
        },
    )
    .map_err(operation_error)?;
    mutation_outcome_or_restore(
        &repository,
        &public_key,
        None,
        *created.id,
        created,
        actor,
        expected_content_root,
    )
}

pub fn update_release(
    repository_path: &Path,
    keystore_path: &Path,
    object_id: &str,
    decision_id: &str,
    planned_actor: &str,
    actions: &[ReleaseDecisionAction],
    expected_content_root: &str,
) -> Result<CobMutationOutcome, ReleaseCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    if actor != planned_actor {
        return Err(ReleaseCobError::ActorDrift {
            actor,
            planned: planned_actor.to_string(),
        });
    }
    let before = load_required_object(&repository, object_id)?;
    let mut planned_state = before.object.state.clone();
    apply_release_operation(
        &mut planned_state,
        ReleaseDecisionOperation {
            id: PREFLIGHT_OPERATION_ID.to_string(),
            actor: actor.clone(),
            role: None,
            actions: actions.to_vec(),
        },
    )?;
    let planned_root = release_content_root(&planned_state)?;
    if planned_root != expected_content_root {
        return Err(ReleaseCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let parsed_object =
        ObjectId::from_str(object_id).map_err(|_| ReleaseCobError::InvalidGitOid {
            value: object_id.to_string(),
        })?;
    let prior_target = repository.namespace_ref_target(&public_key, &TYPE, &parsed_object)?;
    let updated = radicle_cob::update::<CairnReleaseDecisionCob, _, _>(
        &repository,
        &signer,
        None,
        Vec::new(),
        &public_key,
        Update {
            changes: encode_actions(actions.iter())?,
            object_id: parsed_object,
            type_name: TYPE.clone(),
            message: format!("{UPDATE_MESSAGE_PREFIX} {decision_id}"),
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
) -> Result<CollaborativeObject<CairnReleaseDecisionCob>, ReleaseCobError> {
    let parsed = ObjectId::from_str(object_id).map_err(|_| ReleaseCobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    load_object(repository, parsed)?.ok_or_else(|| ReleaseCobError::ObjectMissing {
        object_id: object_id.to_string(),
    })
}

fn load_object(
    repository: &CobRepository,
    object_id: ObjectId,
) -> Result<Option<CollaborativeObject<CairnReleaseDecisionCob>>, ReleaseCobError> {
    radicle_cob::get::<CairnReleaseDecisionCob, _>(repository, &TYPE, &object_id)
        .map_err(operation_error)
}

fn mutation_outcome_or_restore(
    repository: &CobRepository,
    namespace: &PublicKey,
    prior_target: Option<Oid>,
    new_target: Oid,
    object: CollaborativeObject<CairnReleaseDecisionCob>,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, ReleaseCobError> {
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
                return Err(ReleaseCobError::Store(rollback));
            }
            Err(mutation)
        }
    }
}

fn mutation_outcome(
    object: CollaborativeObject<CairnReleaseDecisionCob>,
    operation_id: Oid,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, ReleaseCobError> {
    let observed = release_content_root(object.object.state())?;
    if observed != expected_content_root {
        return Err(ReleaseCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed,
        });
    }
    Ok(CobMutationOutcome {
        object_id: object.id.to_string(),
        operation_id: operation_id.to_string(),
        actor,
        content_root: expected_content_root.to_string(),
        state_hash: release_state_hash(object.object.state())?,
    })
}

fn validate_manifest(entry: &Entry) -> Result<(), ReleaseCobError> {
    if entry.manifest.type_name != *TYPE {
        return Err(ReleaseCobError::TypeMismatch {
            expected: TYPE.to_string(),
            actual: entry.manifest.type_name.to_string(),
        });
    }
    let expected = usize::from(Version::default());
    let actual = usize::from(entry.manifest.version);
    if actual != expected {
        return Err(ReleaseCobError::VersionMismatch { expected, actual });
    }
    Ok(())
}

fn decode_actions(entry: &Entry) -> Result<Vec<ReleaseDecisionAction>, ReleaseCobError> {
    entry
        .contents()
        .iter()
        .map(|blob| serde_json::from_slice(blob))
        .collect::<Result<Vec<_>, _>>()
        .map_err(ReleaseCobError::from)
}

fn encode_actions<'a>(
    actions: impl IntoIterator<Item = &'a ReleaseDecisionAction>,
) -> Result<NonEmpty<Vec<u8>>, ReleaseCobError> {
    let values = actions
        .into_iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()?;
    NonEmpty::from_vec(values).ok_or(cairn_core::ReleaseDecisionError::EmptyOperation.into())
}

fn operation(entry: &Entry, actions: Vec<ReleaseDecisionAction>) -> ReleaseDecisionOperation {
    ReleaseDecisionOperation {
        id: entry.id().to_string(),
        actor: entry.author().to_string(),
        role: None,
        actions,
    }
}

fn validate_root_base(
    entry: &Entry,
    actions: &[ReleaseDecisionAction],
) -> Result<(), ReleaseCobError> {
    let [ReleaseDecisionAction::Init { state }] = actions else {
        return Err(cairn_core::ReleaseDecisionError::InvalidRootShape.into());
    };
    let base = required_base_oid(state)?;
    if !entry.related.contains(&base) {
        return Err(ReleaseCobError::MissingRelatedBase {
            base: base.to_string(),
        });
    }
    Ok(())
}

fn required_base_oid(init: &ReleaseDecisionInit) -> Result<Oid, ReleaseCobError> {
    // The workspace_id carries the git_oid base revision.
    let (kind, value) = init.workspace_id.split_once(':').unwrap_or(("", ""));
    if kind != "git_oid" {
        return Err(ReleaseCobError::InvalidGitOid {
            value: init.workspace_id.clone(),
        });
    }
    Oid::from_str(value).map_err(|_| ReleaseCobError::InvalidGitOid {
        value: value.to_string(),
    })
}

fn decision_id_of(action: &ReleaseDecisionAction) -> &str {
    match action {
        ReleaseDecisionAction::Init { state } => &state.decision_id,
        _ => "unknown",
    }
}

fn operation_error<E>(error: E) -> ReleaseCobError
where
    E: std::error::Error + Send + Sync + 'static,
{
    ReleaseCobError::Operation(Box::new(error))
}
