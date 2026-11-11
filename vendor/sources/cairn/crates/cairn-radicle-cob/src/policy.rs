use std::{path::Path, str::FromStr};

use cairn_core::{
    apply_exception_operation, exception_content_root, exception_request_root,
    exception_state_hash, initialize_exception, validate_exception_init, PolicyExceptionAction,
    PolicyExceptionInit, PolicyExceptionOperation, PolicyExceptionState,
    POLICY_EXCEPTION_COB_TYPE_NAME,
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

pub const CREATE_MESSAGE_PREFIX: &str = "Import policy exception";
pub const UPDATE_MESSAGE_PREFIX: &str = "Update policy exception";
pub const PREFLIGHT_OPERATION_ID: &str = "cairn-pe-preflight";

pub static TYPE: std::sync::LazyLock<TypeName> =
    std::sync::LazyLock::new(
        || match TypeName::from_str(POLICY_EXCEPTION_COB_TYPE_NAME) {
            Ok(type_name) => type_name,
            Err(error) => panic!("invalid static policy-exception COB type name: {error}"),
        },
    );

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairnPolicyExceptionCob {
    state: PolicyExceptionState,
}

impl CairnPolicyExceptionCob {
    pub fn state(&self) -> &PolicyExceptionState {
        &self.state
    }
}

// r[impl cairn.policy_exception_cob.backends]
// r[impl cairn.policy_exception_cob.scope]
// r[impl cairn.policy_exception_cob.policy]
// r[impl cairn.policy_exception_cob.policy.deterministic]
// r[impl cairn.policy_exception_cob.policy.invalid]
// r[impl cairn.policy_exception_cob.profile]
// r[impl cairn.policy_exception_cob.interop]
// r[impl cairn.policy_exception_cob.interop.negative]
// r[impl cairn.policy_exception_cob.backends.boundary]
impl Evaluate<CobRepository> for CairnPolicyExceptionCob {
    type Error = PolicyExceptionCobError;

    fn init(entry: &Entry, _store: &CobRepository) -> Result<Self, Self::Error> {
        validate_manifest(entry)?;
        let actions = decode_actions(entry)?;
        validate_root_base(entry, &actions)?;
        Ok(Self {
            state: initialize_exception(operation(entry, actions))?,
        })
    }

    fn apply<'a, I: Iterator<Item = (&'a Oid, &'a Entry)>>(
        &mut self,
        entry: &Entry,
        _concurrent: I,
        _store: &CobRepository,
    ) -> Result<(), Self::Error> {
        validate_manifest(entry)?;
        apply_exception_operation(&mut self.state, operation(entry, decode_actions(entry)?))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyExceptionInspection {
    pub repository_head_oid: String,
    pub object_id: String,
    pub state: PolicyExceptionState,
}

#[derive(Debug, Error)]
pub enum PolicyExceptionCobError {
    #[error(transparent)]
    Store(#[from] crate::CobStoreError),
    #[error("COB operation failed: {0}")]
    Operation(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Cairn rejected the policy-exception COB operation: {0}")]
    Core(#[from] cairn_core::PolicyExceptionError),
    #[error("action JSON is invalid: {0}")]
    ActionJson(#[from] JsonError),
    #[error("unexpected COB type {actual}; expected {expected}")]
    TypeMismatch { expected: String, actual: String },
    #[error("unsupported COB manifest version {actual}; expected {expected}")]
    VersionMismatch { expected: usize, actual: usize },
    #[error("root action base Git object {base} is missing from related objects")]
    MissingRelatedBase { base: String },
    #[error("exception {exception_id} already has COB object {object_id}")]
    AlreadyExists {
        exception_id: String,
        object_id: String,
    },
    #[error("exception {exception_id} has no COB")]
    Missing { exception_id: String },
    #[error("exception {exception_id} has multiple COBs: {object_ids:?}")]
    Duplicate {
        exception_id: String,
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
) -> Result<radicle_crypto::ssh::keystore::MemorySigner, PolicyExceptionCobError> {
    crate::load_signer(keystore_path)
        .map_err(|error| PolicyExceptionCobError::SignerUnavailable(error.to_string()))
}

// r[verify cairn.policy_exception_cob.interop.outward]
pub fn inspect_exception(
    repository_path: &Path,
    exception_id: &str,
) -> Result<PolicyExceptionInspection, PolicyExceptionCobError> {
    let repository = CobRepository::open(repository_path)?;
    let mut matches = Vec::new();
    for object_id in repository.object_ids(&TYPE)? {
        let Some(object) = load_object(&repository, object_id)? else {
            return Err(PolicyExceptionCobError::ObjectMissing {
                object_id: object_id.to_string(),
            });
        };
        if object.object.state.exception_id == exception_id {
            matches.push(object);
        }
    }
    matches.sort_by(|left, right| left.id.cmp(&right.id));
    if matches.len() > 1 {
        return Err(PolicyExceptionCobError::Duplicate {
            exception_id: exception_id.to_string(),
            object_ids: matches.iter().map(|o| o.id.to_string()).collect(),
        });
    }
    let Some(object) = matches.pop() else {
        return Err(PolicyExceptionCobError::Missing {
            exception_id: exception_id.to_string(),
        });
    };
    Ok(PolicyExceptionInspection {
        repository_head_oid: repository.head_oid()?.to_string(),
        object_id: object.id.to_string(),
        state: object.object.state.clone(),
    })
}

pub fn create_exception(
    repository_path: &Path,
    keystore_path: &Path,
    init: PolicyExceptionInit,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, PolicyExceptionCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    validate_exception_init(&init)?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    let init_operation = PolicyExceptionOperation {
        id: "init".to_string(),
        actor: actor.clone(),
        role: None,
        actions: vec![PolicyExceptionAction::Init {
            state: Box::new(init.clone()),
        }],
    };
    let root = exception_request_root(&init.request)?;
    if root != init.request.request_root {
        return Err(PolicyExceptionCobError::ContentRootDrift {
            expected: init.request.request_root.clone(),
            observed: root,
        });
    }
    let planned_root = exception_content_root(&initialize_exception(init_operation)?)?;
    if planned_root != expected_content_root {
        return Err(PolicyExceptionCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let base = required_base_oid(&init)?;
    let action = PolicyExceptionAction::Init {
        state: Box::new(init),
    };
    let created = radicle_cob::create::<CairnPolicyExceptionCob, _, _>(
        &repository,
        &signer,
        None,
        vec![base],
        &public_key,
        Create {
            contents: encode_actions([&action])?,
            type_name: TYPE.clone(),
            message: format!("{CREATE_MESSAGE_PREFIX} {}", exception_id_of(&action)),
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

pub fn update_exception(
    repository_path: &Path,
    keystore_path: &Path,
    object_id: &str,
    exception_id: &str,
    planned_actor: &str,
    actions: &[PolicyExceptionAction],
    expected_content_root: &str,
) -> Result<CobMutationOutcome, PolicyExceptionCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    if actor != planned_actor {
        return Err(PolicyExceptionCobError::ActorDrift {
            actor,
            planned: planned_actor.to_string(),
        });
    }
    let before = load_required_object(&repository, object_id)?;
    let mut planned_state = before.object.state.clone();
    apply_exception_operation(
        &mut planned_state,
        PolicyExceptionOperation {
            id: PREFLIGHT_OPERATION_ID.to_string(),
            actor: actor.clone(),
            role: None,
            actions: actions.to_vec(),
        },
    )?;
    let planned_root = exception_content_root(&planned_state)?;
    if planned_root != expected_content_root {
        return Err(PolicyExceptionCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let parsed_object =
        ObjectId::from_str(object_id).map_err(|_| PolicyExceptionCobError::InvalidGitOid {
            value: object_id.to_string(),
        })?;
    let prior_target = repository.namespace_ref_target(&public_key, &TYPE, &parsed_object)?;
    let updated = radicle_cob::update::<CairnPolicyExceptionCob, _, _>(
        &repository,
        &signer,
        None,
        Vec::new(),
        &public_key,
        Update {
            changes: encode_actions(actions.iter())?,
            object_id: parsed_object,
            type_name: TYPE.clone(),
            message: format!("{UPDATE_MESSAGE_PREFIX} {exception_id}"),
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
) -> Result<CollaborativeObject<CairnPolicyExceptionCob>, PolicyExceptionCobError> {
    let parsed =
        ObjectId::from_str(object_id).map_err(|_| PolicyExceptionCobError::InvalidGitOid {
            value: object_id.to_string(),
        })?;
    load_object(repository, parsed)?.ok_or_else(|| PolicyExceptionCobError::ObjectMissing {
        object_id: object_id.to_string(),
    })
}

fn load_object(
    repository: &CobRepository,
    object_id: ObjectId,
) -> Result<Option<CollaborativeObject<CairnPolicyExceptionCob>>, PolicyExceptionCobError> {
    radicle_cob::get::<CairnPolicyExceptionCob, _>(repository, &TYPE, &object_id)
        .map_err(operation_error)
}

fn mutation_outcome_or_restore(
    repository: &CobRepository,
    namespace: &PublicKey,
    prior_target: Option<Oid>,
    new_target: Oid,
    object: CollaborativeObject<CairnPolicyExceptionCob>,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, PolicyExceptionCobError> {
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
                return Err(PolicyExceptionCobError::Store(rollback));
            }
            Err(mutation)
        }
    }
}

fn mutation_outcome(
    object: CollaborativeObject<CairnPolicyExceptionCob>,
    operation_id: Oid,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, PolicyExceptionCobError> {
    let observed = exception_content_root(object.object.state())?;
    if observed != expected_content_root {
        return Err(PolicyExceptionCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed,
        });
    }
    Ok(CobMutationOutcome {
        object_id: object.id.to_string(),
        operation_id: operation_id.to_string(),
        actor,
        content_root: expected_content_root.to_string(),
        state_hash: exception_state_hash(object.object.state())?,
    })
}

fn validate_manifest(entry: &Entry) -> Result<(), PolicyExceptionCobError> {
    if entry.manifest.type_name != *TYPE {
        return Err(PolicyExceptionCobError::TypeMismatch {
            expected: TYPE.to_string(),
            actual: entry.manifest.type_name.to_string(),
        });
    }
    let expected = usize::from(Version::default());
    let actual = usize::from(entry.manifest.version);
    if actual != expected {
        return Err(PolicyExceptionCobError::VersionMismatch { expected, actual });
    }
    Ok(())
}

fn decode_actions(entry: &Entry) -> Result<Vec<PolicyExceptionAction>, PolicyExceptionCobError> {
    entry
        .contents()
        .iter()
        .map(|blob| serde_json::from_slice(blob))
        .collect::<Result<Vec<_>, _>>()
        .map_err(PolicyExceptionCobError::from)
}

fn encode_actions<'a>(
    actions: impl IntoIterator<Item = &'a PolicyExceptionAction>,
) -> Result<NonEmpty<Vec<u8>>, PolicyExceptionCobError> {
    let values = actions
        .into_iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()?;
    NonEmpty::from_vec(values).ok_or(cairn_core::PolicyExceptionError::EmptyOperation.into())
}

fn operation(entry: &Entry, actions: Vec<PolicyExceptionAction>) -> PolicyExceptionOperation {
    PolicyExceptionOperation {
        id: entry.id().to_string(),
        actor: entry.author().to_string(),
        role: None,
        actions,
    }
}

fn validate_root_base(
    entry: &Entry,
    actions: &[PolicyExceptionAction],
) -> Result<(), PolicyExceptionCobError> {
    let [PolicyExceptionAction::Init { state }] = actions else {
        return Err(cairn_core::PolicyExceptionError::InvalidRootShape.into());
    };
    let base = required_base_oid(state)?;
    if !entry.related.contains(&base) {
        return Err(PolicyExceptionCobError::MissingRelatedBase {
            base: base.to_string(),
        });
    }
    Ok(())
}

fn required_base_oid(init: &PolicyExceptionInit) -> Result<Oid, PolicyExceptionCobError> {
    let (kind, value) = init.workspace_id.split_once(':').unwrap_or(("", ""));
    if kind != "git_oid" {
        return Err(PolicyExceptionCobError::InvalidGitOid {
            value: init.workspace_id.clone(),
        });
    }
    Oid::from_str(value).map_err(|_| PolicyExceptionCobError::InvalidGitOid {
        value: value.to_string(),
    })
}

fn exception_id_of(action: &PolicyExceptionAction) -> &str {
    match action {
        PolicyExceptionAction::Init { state } => &state.exception_id,
        _ => "unknown",
    }
}

fn operation_error<E>(error: E) -> PolicyExceptionCobError
where
    E: std::error::Error + Send + Sync + 'static,
{
    PolicyExceptionCobError::Operation(Box::new(error))
}
