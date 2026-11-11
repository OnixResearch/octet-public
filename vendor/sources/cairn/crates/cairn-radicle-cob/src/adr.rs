use std::{path::Path, str::FromStr};

use cairn_core::{
    apply_document_cob_operation, document_content_root, document_init_content_root,
    document_state_hash, initialize_document_cob, validate_document_init, DocumentAction,
    DocumentInit, DocumentOperation, DocumentState, ADR_COB_TYPE_NAME, BASE_REVISION_KIND_GIT_OID,
};
use nonempty::NonEmpty;
use radicle_cob::{
    change::Entry, CollaborativeObject, Create, Evaluate, ObjectId, TypeName, Update, Version,
};
use radicle_crypto::{signature::Keypair as _, ssh::keystore::MemorySigner, PublicKey};
use radicle_oid::Oid;
use serde_json::Error as JsonError;
use thiserror::Error;

use crate::store::CobRepository;
use crate::CobMutationOutcome;

pub const CREATE_MESSAGE_PREFIX: &str = "Import ADR";
pub const UPDATE_MESSAGE_PREFIX: &str = "Update ADR";
pub const PREFLIGHT_OPERATION_ID: &str = "cairn-adr-preflight";

// r[impl cairn.document_collaboration.radicle_adapter]
// r[impl cairn.adr_cob.backends]
// r[impl cairn.adr_cob.profile]
// r[impl cairn.adr_cob.materialization]
// r[impl cairn.adr_cob.materialization.conflict]
// r[impl cairn.adr_cob.identity]
// r[impl cairn.adr_cob.identity.provider]
// r[impl cairn.adr_cob.identity.non_claims]
// r[impl cairn.adr_cob.lineage]
// r[impl cairn.adr_cob.interop]
// r[impl cairn.adr_cob.interop.negative]
// r[impl cairn.adr_cob.validation]
// r[impl cairn.adr_cob.validation.release]

pub static TYPE: std::sync::LazyLock<TypeName> =
    std::sync::LazyLock::new(|| match TypeName::from_str(ADR_COB_TYPE_NAME) {
        Ok(type_name) => type_name,
        Err(error) => panic!("invalid static ADR COB type name: {error}"),
    });

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairnAdrCob {
    state: DocumentState,
}

impl CairnAdrCob {
    pub fn state(&self) -> &DocumentState {
        &self.state
    }
    pub fn into_state(self) -> DocumentState {
        self.state
    }
}

// r[verify cairn.adr_cob.backends]
impl Evaluate<CobRepository> for CairnAdrCob {
    type Error = AdrCobError;

    fn init(entry: &Entry, _store: &CobRepository) -> Result<Self, Self::Error> {
        validate_manifest(entry)?;
        let actions = decode_actions(entry)?;
        validate_root_base(entry, &actions)?;
        let operation = operation(entry, actions);
        Ok(Self {
            state: initialize_document_cob(operation)?,
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
        apply_document_cob_operation(&mut self.state, operation)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrCobInspection {
    pub repository_head_oid: String,
    pub object_id: String,
    pub state: DocumentState,
}

#[derive(Debug, Error)]
pub enum AdrCobError {
    #[error(transparent)]
    Store(#[from] crate::CobStoreError),
    #[error("COB operation failed: {0}")]
    Operation(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Cairn rejected the ADR COB operation: {0}")]
    Core(#[from] cairn_core::DocumentCobError),
    #[error("ADR action JSON is invalid: {0}")]
    ActionJson(#[from] JsonError),
    #[error("unexpected COB type {actual}; expected {expected}")]
    TypeMismatch { expected: String, actual: String },
    #[error("unsupported COB manifest version {actual}; expected {expected}")]
    VersionMismatch { expected: usize, actual: usize },
    #[error("root action base Git object {base} is missing from related objects")]
    MissingRelatedBase { base: String },
    #[error("ADR {document_id} already has COB object {object_id}")]
    AlreadyExists {
        document_id: String,
        object_id: String,
    },
    #[error("ADR {document_id} has no COB")]
    Missing { document_id: String },
    #[error("ADR {document_id} has multiple COBs: {object_ids:?}")]
    Duplicate {
        document_id: String,
        object_ids: Vec<String>,
    },
    #[error("COB object {object_id} is missing")]
    ObjectMissing { object_id: String },
    #[error("keystore path is a symlink: {path}")]
    SymlinkKeystore { path: String },
    #[error("failed to load explicit COB signer: {0}")]
    Signer(#[from] MemorySignerError),
    #[error("signed actor {actor} does not match planned actor {planned}")]
    ActorDrift { actor: String, planned: String },
    #[error("content root drifted: expected {expected}, observed {observed}")]
    ContentRootDrift { expected: String, observed: String },
    #[error("invalid Git object ID {value}")]
    InvalidGitOid { value: String },
    #[error("local COB filesystem operation failed: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("signer unavailable: {0}")]
    SignerUnavailable(String),
}

use radicle_crypto::ssh::keystore::MemorySignerError;

pub fn load_signer(keystore_path: &Path) -> Result<MemorySigner, AdrCobError> {
    crate::load_signer(keystore_path)
        .map_err(|error| AdrCobError::SignerUnavailable(error.to_string()))
}

// r[verify cairn.adr_cob.interop.outward]
pub fn inspect_adr(
    repository_path: &Path,
    document_id: &str,
) -> Result<AdrCobInspection, AdrCobError> {
    let repository = CobRepository::open(repository_path)?;
    inspect_open_adr(&repository, document_id)
}

fn inspect_open_adr(
    repository: &CobRepository,
    document_id: &str,
) -> Result<AdrCobInspection, AdrCobError> {
    let mut matches = Vec::new();
    for object_id in repository.object_ids(&TYPE)? {
        let Some(object) = load_object(repository, object_id)? else {
            return Err(AdrCobError::ObjectMissing {
                object_id: object_id.to_string(),
            });
        };
        if object.object.state.document_id == document_id {
            matches.push(object);
        }
    }
    matches.sort_by(|left, right| left.id.cmp(&right.id));
    if matches.len() > 1 {
        return Err(AdrCobError::Duplicate {
            document_id: document_id.to_string(),
            object_ids: matches.iter().map(|object| object.id.to_string()).collect(),
        });
    }
    let Some(object) = matches.pop() else {
        return Err(AdrCobError::Missing {
            document_id: document_id.to_string(),
        });
    };
    Ok(AdrCobInspection {
        repository_head_oid: repository.head_oid()?.to_string(),
        object_id: object.id.to_string(),
        state: object.object.into_state(),
    })
}

// r[verify cairn.adr_cob.interop]
// r[verify cairn.adr_cob.materialization]
// r[verify cairn.adr_cob.backends]
pub fn create_adr(
    repository_path: &Path,
    keystore_path: &Path,
    init: DocumentInit,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, AdrCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    match inspect_open_adr(&repository, &init.document_id) {
        Ok(existing) => {
            return Err(AdrCobError::AlreadyExists {
                document_id: init.document_id,
                object_id: existing.object_id,
            });
        }
        Err(AdrCobError::Missing { .. }) => {}
        Err(error) => return Err(error),
    }
    validate_document_init(&init)?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    let planned_root = document_init_content_root(&init)?;
    if planned_root != expected_content_root {
        return Err(AdrCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let base = required_base_oid(&init)?;
    let action = DocumentAction::Init { state: init };
    let contents = encode_actions([&action])?;
    let created = radicle_cob::create::<CairnAdrCob, _, _>(
        &repository,
        &signer,
        None,
        vec![base],
        &public_key,
        Create {
            contents,
            type_name: TYPE.clone(),
            message: format!("{CREATE_MESSAGE_PREFIX} {}", action_document_id(&action)),
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

pub fn update_adr(
    repository_path: &Path,
    keystore_path: &Path,
    object_id: &str,
    document_id: &str,
    planned_actor: &str,
    actions: &[DocumentAction],
    expected_content_root: &str,
) -> Result<CobMutationOutcome, AdrCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    if actor != planned_actor {
        return Err(AdrCobError::ActorDrift {
            actor,
            planned: planned_actor.to_string(),
        });
    }
    let before = load_required_object(&repository, object_id)?;
    let mut planned_state = before.object.state.clone();
    apply_document_cob_operation(
        &mut planned_state,
        DocumentOperation {
            id: PREFLIGHT_OPERATION_ID.to_string(),
            actor: actor.clone(),
            role: None,
            actions: actions.to_vec(),
        },
    )?;
    let planned_root = document_content_root(&planned_state)?;
    if planned_root != expected_content_root {
        return Err(AdrCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let parsed_object = ObjectId::from_str(object_id).map_err(|_| AdrCobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    let prior_target = repository.namespace_ref_target(&public_key, &TYPE, &parsed_object)?;
    let updated = radicle_cob::update::<CairnAdrCob, _, _>(
        &repository,
        &signer,
        None,
        Vec::new(),
        &public_key,
        Update {
            changes: encode_actions(actions.iter())?,
            object_id: parsed_object,
            type_name: TYPE.clone(),
            message: format!("{UPDATE_MESSAGE_PREFIX} {document_id}"),
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
) -> Result<CollaborativeObject<CairnAdrCob>, AdrCobError> {
    let parsed = ObjectId::from_str(object_id).map_err(|_| AdrCobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    load_object(repository, parsed)?.ok_or_else(|| AdrCobError::ObjectMissing {
        object_id: object_id.to_string(),
    })
}

fn load_object(
    repository: &CobRepository,
    object_id: ObjectId,
) -> Result<Option<CollaborativeObject<CairnAdrCob>>, AdrCobError> {
    radicle_cob::get::<CairnAdrCob, _>(repository, &TYPE, &object_id).map_err(operation_error)
}

fn mutation_outcome_or_restore(
    repository: &CobRepository,
    namespace: &PublicKey,
    prior_target: Option<Oid>,
    new_target: Oid,
    object: CollaborativeObject<CairnAdrCob>,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, AdrCobError> {
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
                return Err(AdrCobError::Store(rollback));
            }
            Err(mutation)
        }
    }
}

fn mutation_outcome(
    object: CollaborativeObject<CairnAdrCob>,
    operation_id: Oid,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, AdrCobError> {
    let observed = document_content_root(object.object.state())?;
    if observed != expected_content_root {
        return Err(AdrCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed,
        });
    }
    Ok(CobMutationOutcome {
        object_id: object.id.to_string(),
        operation_id: operation_id.to_string(),
        actor,
        content_root: expected_content_root.to_string(),
        state_hash: document_state_hash(object.object.state())?,
    })
}

fn validate_manifest(entry: &Entry) -> Result<(), AdrCobError> {
    if entry.manifest.type_name != *TYPE {
        return Err(AdrCobError::TypeMismatch {
            expected: TYPE.to_string(),
            actual: entry.manifest.type_name.to_string(),
        });
    }
    let expected = usize::from(Version::default());
    let actual = usize::from(entry.manifest.version);
    if actual != expected {
        return Err(AdrCobError::VersionMismatch { expected, actual });
    }
    Ok(())
}

fn decode_actions(entry: &Entry) -> Result<Vec<DocumentAction>, AdrCobError> {
    entry
        .contents()
        .iter()
        .map(|blob| serde_json::from_slice(blob))
        .collect::<Result<Vec<_>, _>>()
        .map_err(AdrCobError::from)
}

fn encode_actions<'a>(
    actions: impl IntoIterator<Item = &'a DocumentAction>,
) -> Result<NonEmpty<Vec<u8>>, AdrCobError> {
    let values = actions
        .into_iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()?;
    NonEmpty::from_vec(values).ok_or(cairn_core::DocumentCobError::EmptyOperation.into())
}

fn operation(entry: &Entry, actions: Vec<DocumentAction>) -> DocumentOperation {
    DocumentOperation {
        id: entry.id().to_string(),
        actor: entry.author().to_string(),
        role: None,
        actions,
    }
}

fn validate_root_base(entry: &Entry, actions: &[DocumentAction]) -> Result<(), AdrCobError> {
    let [DocumentAction::Init { state }] = actions else {
        return Err(cairn_core::DocumentCobError::InvalidRootShape.into());
    };
    let base = required_base_oid(state)?;
    if !entry.related.contains(&base) {
        return Err(AdrCobError::MissingRelatedBase {
            base: base.to_string(),
        });
    }
    Ok(())
}

fn required_base_oid(init: &DocumentInit) -> Result<Oid, AdrCobError> {
    let revision = cairn_core::decode_base_revision(&init.base_revision).map_err(|error| {
        cairn_core::DocumentCobError::Factory(format!("base revision invalid: {error}"))
    })?;
    if revision.kind != BASE_REVISION_KIND_GIT_OID {
        return Err(cairn_core::DocumentCobError::InvalidField {
            field: "base_revision".to_string(),
            reason: "ADR COB requires a git_oid base revision".to_string(),
        }
        .into());
    }
    Oid::from_str(&revision.value).map_err(|_| AdrCobError::InvalidGitOid {
        value: revision.value,
    })
}

fn action_document_id(action: &DocumentAction) -> &str {
    match action {
        DocumentAction::Init { state } => &state.document_id,
        _ => "unknown",
    }
}

fn operation_error<E>(error: E) -> AdrCobError
where
    E: std::error::Error + Send + Sync + 'static,
{
    AdrCobError::Operation(Box::new(error))
}
