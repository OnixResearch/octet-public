#![deny(clippy::unwrap_used)]

mod adr;
mod policy;
mod release;
mod rfd;
mod store;
mod vo;

pub use crate::adr::{
    create_adr, inspect_adr, update_adr, AdrCobError, AdrCobInspection, CairnAdrCob,
};
pub use crate::policy::{
    create_exception, inspect_exception, update_exception, CairnPolicyExceptionCob,
    PolicyExceptionCobError, PolicyExceptionInspection,
};
pub use crate::release::{
    create_release, inspect_release, update_release, CairnReleaseDecisionCob, ReleaseCobError,
    ReleaseDecisionInspection,
};
pub use crate::rfd::{
    create_rfd, inspect_rfd, update_rfd, CairnRfdCob, RfdCobError, RfdCobInspection,
};
pub use crate::vo::{
    create_vo, inspect_vo, update_vo, CairnVerificationObligationCob, VoCobError, VoCobInspection,
};

use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use cairn_core::{
    apply_change_cob_operation, change_cob_content_root, change_cob_import_content_root,
    change_cob_state_hash, decode_base_revision, initialize_change_cob, ChangeCobAction,
    ChangeCobError, ChangeCobInit, ChangeCobOperation, ChangeCobState, BASE_REVISION_KIND_GIT_OID,
    RADICLE_CHANGE_COB_SCHEMA_V1, RADICLE_CHANGE_COB_TYPE_NAME,
};
use nonempty::NonEmpty;
use radicle_cob::{
    change::Entry, CollaborativeObject, Create, Evaluate, ObjectId, TypeName, Update, Version,
};
use radicle_crypto::{
    signature::Keypair as _,
    ssh::keystore::{Keystore, MemorySigner, MemorySignerError},
    PublicKey,
};
use radicle_oid::Oid;
use serde_json::Error as JsonError;
use thiserror::Error;

pub use store::{CobRepository, CobStoreError};

pub const RADICLE_COB_LIBRARY_VERSION: &str = "0.20.0";
pub const RADICLE_CRYPTO_LIBRARY_VERSION: &str = "0.17.0";
pub const RADICLE_OID_LIBRARY_VERSION: &str = "0.2.0";
pub const GIT2_LIBRARY_VERSION: &str = "0.20.4";
pub const GIT_REF_FORMAT_LIBRARY_VERSION: &str = "0.6.0";

const CREATE_MESSAGE_PREFIX: &str = "Import Cairn change";
const UPDATE_MESSAGE_PREFIX: &str = "Update Cairn change";
const PREFLIGHT_OPERATION_ID: &str = "cairn-cob-preflight";
const MUTATION_LOCK_DIRECTORY: &str = "cairn/locks/cob";
const MUTATION_LOCK_CONTENT: &[u8] = b"cairn-cob-mutation\n";

// r[impl cairn.radicle_cob_only.dependencies]
// r[impl cairn.radicle_cob_only.store]
// r[impl cairn.radicle_cob_only.authority]
// r[impl cairn.radicle_cob_only.compatibility]
// r[impl cairn.radicle_change_cob.radicle_adapter]

pub static TYPENAME: LazyLock<TypeName> =
    LazyLock::new(|| match TypeName::from_str(RADICLE_CHANGE_COB_TYPE_NAME) {
        Ok(type_name) => type_name,
        Err(error) => panic!("invalid static Cairn COB type name: {error}"),
    });

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairnChangeCob {
    state: ChangeCobState,
}

impl CairnChangeCob {
    pub fn state(&self) -> &ChangeCobState {
        &self.state
    }

    pub fn into_state(self) -> ChangeCobState {
        self.state
    }
}

// r[impl cairn.radicle_change_cob.radicle_adapter.mapping]
impl Evaluate<CobRepository> for CairnChangeCob {
    type Error = CobError;

    fn init(entry: &Entry, _store: &CobRepository) -> Result<Self, Self::Error> {
        validate_manifest(entry)?;
        let actions = decode_actions(entry)?;
        validate_root_base(entry, &actions)?;
        let operation = operation(entry, actions, false);
        Ok(Self {
            state: initialize_change_cob(operation)?,
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
        let actor = entry.author().to_string();
        let is_delegate = self.state.delegates.contains(&actor);
        apply_change_cob_operation(&mut self.state, operation(entry, actions, is_delegate))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobChangeInspection {
    pub repository_head_oid: String,
    pub object_id: String,
    pub state: ChangeCobState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobMutationOutcome {
    pub object_id: String,
    pub operation_id: String,
    pub actor: String,
    pub content_root: String,
    pub state_hash: String,
}

#[derive(Debug, Error)]
pub enum CobError {
    #[error(transparent)]
    Store(#[from] CobStoreError),
    #[error("COB operation failed: {0}")]
    Operation(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("COB action JSON is invalid: {0}")]
    ActionJson(#[from] JsonError),
    #[error("Cairn rejected the COB operation: {0}")]
    Core(#[from] ChangeCobError),
    #[error("unexpected COB type {actual}; expected {expected}")]
    TypeMismatch { expected: String, actual: String },
    #[error("unsupported COB manifest version {actual}; expected {expected}")]
    VersionMismatch { expected: usize, actual: usize },
    #[error("root action base Git object {base} is missing from related objects")]
    MissingRelatedBase { base: String },
    #[error("COB-only collaboration requires a git_oid base revision, observed {kind}")]
    UnsupportedBaseRevisionKind { kind: String },
    #[error("invalid Git object ID {value}")]
    InvalidGitOid { value: String },
    #[error("change {change_id} already has Cairn COB object {object_id}")]
    ChangeAlreadyExists {
        change_id: String,
        object_id: String,
    },
    #[error("change {change_id} has no Cairn COB")]
    ChangeMissing { change_id: String },
    #[error("change {change_id} has multiple Cairn COBs: {object_ids:?}")]
    DuplicateChanges {
        change_id: String,
        object_ids: Vec<String>,
    },
    #[error("COB object {object_id} is missing")]
    ObjectMissing { object_id: String },
    #[error("keystore path is a symlink: {path}")]
    SymlinkKeystore { path: String },
    #[error("local COB filesystem operation failed: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("failed to load explicit COB signer: {0}")]
    Signer(#[from] MemorySignerError),
    #[error("signed actor {actor} is neither creator nor immutable delegate")]
    UnauthorizedActor { actor: String },
    #[error("signed actor {actor} does not match planned actor {planned}")]
    ActorDrift { actor: String, planned: String },
    #[error("COB content root changed: expected {expected}, observed {observed}")]
    ContentRootDrift { expected: String, observed: String },
    #[error("COB postcondition failed ({mutation}) and reference rollback failed ({rollback})")]
    PostconditionRollback {
        mutation: Box<CobError>,
        rollback: CobStoreError,
    },
    #[error("another local COB mutation is active for change {change_id}; after confirming no mutation is running, remove stale lock {path}")]
    ConcurrentMutation { change_id: String, path: String },
    #[error("failed to initialize COB mutation lock {path}: {source}; cleanup: {cleanup}")]
    LockInitialize {
        path: String,
        #[source]
        source: std::io::Error,
        cleanup: String,
    },
    #[error("failed to release COB mutation lock {path}: {source}")]
    LockRelease {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

pub struct CobMutationLock {
    path: PathBuf,
    display_path: String,
    released: bool,
}

impl CobMutationLock {
    pub fn acquire(repository_path: &Path, change_id: &str) -> Result<Self, CobError> {
        let repository = CobRepository::open(repository_path)?;
        let directory = repository.raw().path().join(MUTATION_LOCK_DIRECTORY);
        fs::create_dir_all(&directory)?;
        let lock_name = format!("{change_id}.lock");
        let path = directory.join(&lock_name);
        let display_path = Path::new(MUTATION_LOCK_DIRECTORY)
            .join(lock_name)
            .display()
            .to_string();
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(CobError::ConcurrentMutation {
                    change_id: change_id.to_string(),
                    path: display_path,
                });
            }
            Err(error) => return Err(error.into()),
        };
        if let Err(source) = file
            .write_all(MUTATION_LOCK_CONTENT)
            .and_then(|()| file.sync_all())
        {
            let cleanup = match fs::remove_file(&path) {
                Ok(()) => "removed incomplete lock".to_string(),
                Err(error) => format!("failed to remove incomplete lock: {error}"),
            };
            return Err(CobError::LockInitialize {
                path: display_path,
                source,
                cleanup,
            });
        }
        Ok(Self {
            path,
            display_path,
            released: false,
        })
    }

    pub fn release(mut self) -> Result<(), CobError> {
        self.remove()
    }

    fn remove(&mut self) -> Result<(), CobError> {
        if self.released {
            return Ok(());
        }
        fs::remove_file(&self.path).map_err(|source| CobError::LockRelease {
            path: self.display_path.clone(),
            source,
        })?;
        self.released = true;
        Ok(())
    }
}

impl Drop for CobMutationLock {
    fn drop(&mut self) {
        if !self.released {
            let _ = fs::remove_file(&self.path);
            self.released = true;
        }
    }
}

pub fn load_signer(keystore_path: &Path) -> Result<MemorySigner, CobError> {
    let metadata = fs::symlink_metadata(keystore_path)?;
    if metadata.file_type().is_symlink() {
        return Err(CobError::SymlinkKeystore {
            path: keystore_path.display().to_string(),
        });
    }
    Ok(MemorySigner::load(&Keystore::new(&keystore_path), None)?)
}

pub fn signer_actor(keystore_path: &Path) -> Result<String, CobError> {
    Ok(load_signer(keystore_path)?.verifying_key().to_string())
}

pub fn inspect_change(
    repository_path: &Path,
    change_id: &str,
) -> Result<CobChangeInspection, CobError> {
    let repository = CobRepository::open(repository_path)?;
    inspect_open_change(&repository, change_id)
}

pub fn inspect_open_change(
    repository: &CobRepository,
    change_id: &str,
) -> Result<CobChangeInspection, CobError> {
    let mut matches = Vec::new();
    for object_id in repository.object_ids(&TYPENAME)? {
        let Some(object) = load_object(repository, object_id)? else {
            return Err(CobError::ObjectMissing {
                object_id: object_id.to_string(),
            });
        };
        if object.object.state.change_id == change_id {
            matches.push(object);
        }
    }
    matches.sort_by(|left, right| left.id.cmp(&right.id));
    if matches.len() > 1 {
        return Err(CobError::DuplicateChanges {
            change_id: change_id.to_string(),
            object_ids: matches.iter().map(|object| object.id.to_string()).collect(),
        });
    }
    let Some(object) = matches.pop() else {
        return Err(CobError::ChangeMissing {
            change_id: change_id.to_string(),
        });
    };
    Ok(CobChangeInspection {
        repository_head_oid: repository.head_oid()?.to_string(),
        object_id: object.id.to_string(),
        state: object.object.into_state(),
    })
}

pub fn find_change(
    repository_path: &Path,
    change_id: &str,
) -> Result<Option<CobChangeInspection>, CobError> {
    match inspect_change(repository_path, change_id) {
        Ok(inspection) => Ok(Some(inspection)),
        Err(CobError::ChangeMissing { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn create_change(
    repository_path: &Path,
    keystore_path: &Path,
    init: ChangeCobInit,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, CobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    match inspect_open_change(&repository, &init.change_id) {
        Ok(existing) => {
            return Err(CobError::ChangeAlreadyExists {
                change_id: init.change_id,
                object_id: existing.object_id,
            });
        }
        Err(CobError::ChangeMissing { .. }) => {}
        Err(error) => return Err(error),
    }
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    let planned_root = change_cob_import_content_root(&init, &actor)?;
    if planned_root != expected_content_root {
        return Err(CobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let base = required_base_oid(&init)?;
    let action = ChangeCobAction::Init { state: init };
    let contents = encode_actions([&action])?;
    let created = radicle_cob::create::<CairnChangeCob, _, _>(
        &repository,
        &signer,
        None,
        vec![base],
        &public_key,
        Create {
            contents,
            type_name: TYPENAME.clone(),
            message: format!("{CREATE_MESSAGE_PREFIX} {}", action_change_id(&action)),
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

pub fn update_change(
    repository_path: &Path,
    keystore_path: &Path,
    object_id: &str,
    change_id: &str,
    planned_actor: &str,
    actions: &[ChangeCobAction],
    expected_content_root: &str,
) -> Result<CobMutationOutcome, CobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    if actor != planned_actor {
        return Err(CobError::ActorDrift {
            actor,
            planned: planned_actor.to_string(),
        });
    }
    let before = load_required_object(&repository, object_id)?;
    let is_delegate = before.object.state.delegates.contains(&actor);
    if actor != before.object.state.creator && !is_delegate {
        return Err(CobError::UnauthorizedActor { actor });
    }
    let mut planned_state = before.object.state.clone();
    apply_change_cob_operation(
        &mut planned_state,
        ChangeCobOperation {
            id: PREFLIGHT_OPERATION_ID.to_string(),
            actor: actor.clone(),
            is_delegate,
            actions: actions.to_vec(),
        },
    )?;
    let planned_root = change_cob_content_root(&planned_state)?;
    if planned_root != expected_content_root {
        return Err(CobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let parsed_object = ObjectId::from_str(object_id).map_err(|_| CobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    let prior_target = repository.namespace_ref_target(&public_key, &TYPENAME, &parsed_object)?;
    let updated = radicle_cob::update::<CairnChangeCob, _, _>(
        &repository,
        &signer,
        None,
        Vec::new(),
        &public_key,
        Update {
            changes: encode_actions(actions.iter())?,
            object_id: parsed_object,
            type_name: TYPENAME.clone(),
            message: format!("{UPDATE_MESSAGE_PREFIX} {change_id}"),
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
) -> Result<CollaborativeObject<CairnChangeCob>, CobError> {
    let parsed = ObjectId::from_str(object_id).map_err(|_| CobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    load_object(repository, parsed)?.ok_or_else(|| CobError::ObjectMissing {
        object_id: object_id.to_string(),
    })
}

fn load_object(
    repository: &CobRepository,
    object_id: ObjectId,
) -> Result<Option<CollaborativeObject<CairnChangeCob>>, CobError> {
    radicle_cob::get::<CairnChangeCob, _>(repository, &TYPENAME, &object_id)
        .map_err(operation_error)
}

fn mutation_outcome_or_restore(
    repository: &CobRepository,
    namespace: &PublicKey,
    prior_target: Option<Oid>,
    new_target: Oid,
    object: CollaborativeObject<CairnChangeCob>,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, CobError> {
    let object_id = object.id;
    match mutation_outcome(object, new_target, actor, expected_content_root) {
        Ok(outcome) => Ok(outcome),
        Err(mutation) => {
            if let Err(rollback) = repository.restore_namespace_ref(
                namespace,
                &TYPENAME,
                &object_id,
                new_target,
                prior_target,
            ) {
                return Err(CobError::PostconditionRollback {
                    mutation: Box::new(mutation),
                    rollback,
                });
            }
            Err(mutation)
        }
    }
}

fn mutation_outcome(
    object: CollaborativeObject<CairnChangeCob>,
    operation_id: Oid,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, CobError> {
    let observed = change_cob_content_root(object.object.state())?;
    if observed != expected_content_root {
        return Err(CobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed,
        });
    }
    Ok(CobMutationOutcome {
        object_id: object.id.to_string(),
        operation_id: operation_id.to_string(),
        actor,
        content_root: expected_content_root.to_string(),
        state_hash: change_cob_state_hash(object.object.state())?,
    })
}

fn validate_manifest(entry: &Entry) -> Result<(), CobError> {
    if entry.manifest.type_name != *TYPENAME {
        return Err(CobError::TypeMismatch {
            expected: TYPENAME.to_string(),
            actual: entry.manifest.type_name.to_string(),
        });
    }
    let expected = usize::from(Version::default());
    let actual = usize::from(entry.manifest.version);
    if actual != expected {
        return Err(CobError::VersionMismatch { expected, actual });
    }
    Ok(())
}

fn decode_actions(entry: &Entry) -> Result<Vec<ChangeCobAction>, CobError> {
    entry
        .contents()
        .iter()
        .map(|blob| serde_json::from_slice(blob))
        .collect::<Result<Vec<_>, _>>()
        .map_err(CobError::from)
}

fn encode_actions<'a>(
    actions: impl IntoIterator<Item = &'a ChangeCobAction>,
) -> Result<NonEmpty<Vec<u8>>, CobError> {
    let values = actions
        .into_iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()?;
    NonEmpty::from_vec(values).ok_or(ChangeCobError::EmptyOperation.into())
}

fn operation(
    entry: &Entry,
    actions: Vec<ChangeCobAction>,
    is_delegate: bool,
) -> ChangeCobOperation {
    ChangeCobOperation {
        id: entry.id().to_string(),
        actor: entry.author().to_string(),
        is_delegate,
        actions,
    }
}

fn validate_root_base(entry: &Entry, actions: &[ChangeCobAction]) -> Result<(), CobError> {
    let [ChangeCobAction::Init { state }] = actions else {
        return Err(ChangeCobError::InvalidRootShape.into());
    };
    let base = required_base_oid(state)?;
    if !entry.related.contains(&base) {
        return Err(CobError::MissingRelatedBase {
            base: base.to_string(),
        });
    }
    Ok(())
}

fn required_base_oid(init: &ChangeCobInit) -> Result<Oid, CobError> {
    let value = if init.schema == RADICLE_CHANGE_COB_SCHEMA_V1 {
        init.base_git_oid.clone()
    } else {
        let revision = decode_base_revision(&init.base_git_oid)?;
        if revision.kind != BASE_REVISION_KIND_GIT_OID {
            return Err(CobError::UnsupportedBaseRevisionKind {
                kind: revision.kind,
            });
        }
        revision.value
    };
    Oid::from_str(&value).map_err(|_| CobError::InvalidGitOid { value })
}

fn action_change_id(action: &ChangeCobAction) -> &str {
    match action {
        ChangeCobAction::Init { state } => &state.change_id,
        ChangeCobAction::SetArtifact { .. }
        | ChangeCobAction::AttachEvidence { .. }
        | ChangeCobAction::SetLifecycle { .. } => "unknown",
    }
}

fn operation_error<E>(error: E) -> CobError
where
    E: std::error::Error + Send + Sync + 'static,
{
    CobError::Operation(Box::new(error))
}

pub fn embedded_authority(state: &ChangeCobState, actor: &str) -> (bool, bool) {
    (actor == state.creator, state.delegates.contains(actor))
}

pub fn parse_delegate_ids(
    creator: &str,
    delegates: &[String],
) -> Result<BTreeSet<String>, CobError> {
    let creator = PublicKey::from_str(creator).map_err(|_| ChangeCobError::InvalidField {
        field: "creator".to_string(),
        reason: "must be a Radicle Ed25519 public key".to_string(),
    })?;
    let parsed = delegates
        .iter()
        .map(|delegate| {
            PublicKey::from_str(delegate).map_err(|_| ChangeCobError::InvalidField {
                field: "delegate".to_string(),
                reason: format!("invalid Radicle Ed25519 public key: {delegate}"),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_delegates(&creator, parsed)
}

pub fn validate_delegates(
    creator: &PublicKey,
    delegates: impl IntoIterator<Item = PublicKey>,
) -> Result<BTreeSet<String>, CobError> {
    let creator = creator.to_string();
    let delegates = delegates
        .into_iter()
        .map(|delegate| delegate.to_string())
        .collect::<BTreeSet<_>>();
    if delegates.contains(&creator) {
        return Err(ChangeCobError::InvalidField {
            field: "delegates".to_string(),
            reason: "creator must not also be an embedded delegate".to_string(),
        }
        .into());
    }
    Ok(delegates)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, error::Error, io, process};

    use cairn_core::{
        apply_change_cob_operation, change_cob_import_content_root, change_cob_lifecycle_view,
        ChangeArtifactId, ChangeBaseRevision, ChangeCobOperation, ChangeLifecycleStatus,
        ChangeLifecycleView, CHANGE_COLLABORATION_SCHEMA_V1,
    };
    use radicle_crypto::{ssh::Keystore, KeyPair, Seed};

    use super::*;

    const CHANGE_ID: &str = "radicle-cob-only";
    const WORKSPACE_ID: &str = "workspace:cob-only";
    const POLICY_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const INITIAL_PROPOSAL: &str = "# Initial\n";
    const UPDATED_PROPOSAL: &str = "# Updated by delegate\n";
    const TEST_DIRECTORY_PREFIX: &str = "cairn-radicle-cob-only";
    const GIT_AUTHOR_NAME: &str = "Cairn COB test";
    const GIT_AUTHOR_EMAIL: &str = "cob-test@example.invalid";
    const GIT_COMMIT_MESSAGE: &str = "Initialize COB repository";
    const GIT_REF_MAIN: &str = "refs/heads/main";
    const TEST_TIMESTAMP_SECONDS: i64 = 1_700_000_000;
    const ED25519_SEED_BYTES: usize = 32;
    const CREATOR_SEED_BYTE: u8 = 1;
    const DELEGATE_SEED_BYTE: u8 = 2;
    const OUTSIDER_SEED_BYTE: u8 = 3;
    const INVALID_ROOT_HEX_CHAR: char = 'c';
    const PREVIEW_OPERATION_ID: &str = "preview:update";
    const PREVIEW_LIFECYCLE_OPERATION_ID: &str = "preview:lifecycle";
    const WORKSPACE_LOCKFILE: &str = include_str!("../../../Cargo.lock");
    const UMBRELLA_RADICLE_PACKAGE: &str = "radicle";
    const RADICLE_NODE_PACKAGE: &str = "radicle-node";
    const RADICLE_COB_PACKAGE: &str = "radicle-cob";
    const RADICLE_CRYPTO_PACKAGE: &str = "radicle-crypto";
    const RADICLE_OID_PACKAGE: &str = "radicle-oid";
    const GIT2_PACKAGE: &str = "git2";
    const GIT_REF_FORMAT_PACKAGE: &str = "git-ref-format-core";

    struct Fixture {
        root: PathBuf,
        repository: PathBuf,
        creator_keys: PathBuf,
        delegate_keys: PathBuf,
        outsider_keys: PathBuf,
        creator: PublicKey,
        delegate: PublicKey,
    }

    impl Fixture {
        fn new(name: &str) -> Result<Self, Box<dyn Error>> {
            let root = std::env::temp_dir()
                .join(format!("{TEST_DIRECTORY_PREFIX}-{name}-{}", process::id()));
            remove_directory(&root)?;
            fs::create_dir_all(&root)?;
            let repository = root.join("repository");
            initialize_repository(&repository)?;
            let creator_keys = root.join("creator-keys");
            let delegate_keys = root.join("delegate-keys");
            let outsider_keys = root.join("outsider-keys");
            let creator = initialize_keystore(&creator_keys, CREATOR_SEED_BYTE)?;
            let delegate = initialize_keystore(&delegate_keys, DELEGATE_SEED_BYTE)?;
            let _outsider = initialize_keystore(&outsider_keys, OUTSIDER_SEED_BYTE)?;
            Ok(Self {
                root,
                repository,
                creator_keys,
                delegate_keys,
                outsider_keys,
                creator,
                delegate,
            })
        }

        fn init(&self) -> Result<ChangeCobInit, Box<dyn Error>> {
            let repository = CobRepository::open(&self.repository)?;
            let base_revision = ChangeBaseRevision::new(
                BASE_REVISION_KIND_GIT_OID,
                repository.head_oid()?.to_string(),
            )?
            .encode()?;
            Ok(ChangeCobInit {
                schema: CHANGE_COLLABORATION_SCHEMA_V1.to_string(),
                change_id: CHANGE_ID.to_string(),
                repository_id: WORKSPACE_ID.to_string(),
                base_git_oid: base_revision,
                policy_hash: POLICY_HASH.to_string(),
                delegates: BTreeSet::from([self.delegate.to_string()]),
                artifacts: BTreeMap::from([(
                    ChangeArtifactId::proposal(),
                    INITIAL_PROPOSAL.to_string(),
                )]),
            })
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = remove_directory(&self.root);
        }
    }

    fn lockfile_has_package(name: &str) -> bool {
        WORKSPACE_LOCKFILE.contains(&format!("\nname = \"{name}\"\n"))
    }

    fn lockfile_has_package_version(name: &str, version: &str) -> bool {
        WORKSPACE_LOCKFILE.contains(&format!("\nname = \"{name}\"\nversion = \"{version}\"\n"))
    }

    // r[verify cairn.radicle_cob_only.dependencies.excludes_runtime]
    // r[verify cairn.radicle_cob_only.dependencies.required_primitives]
    #[test]
    fn workspace_lock_uses_direct_cob_primitives_without_runtime_crates() {
        assert!(!lockfile_has_package(UMBRELLA_RADICLE_PACKAGE));
        assert!(!lockfile_has_package(RADICLE_NODE_PACKAGE));
        for required in [
            RADICLE_COB_PACKAGE,
            RADICLE_CRYPTO_PACKAGE,
            RADICLE_OID_PACKAGE,
            GIT2_PACKAGE,
            GIT_REF_FORMAT_PACKAGE,
        ] {
            assert!(
                lockfile_has_package(required),
                "missing direct primitive: {required}"
            );
        }
        for (name, version) in [
            (RADICLE_COB_PACKAGE, RADICLE_COB_LIBRARY_VERSION),
            (RADICLE_CRYPTO_PACKAGE, RADICLE_CRYPTO_LIBRARY_VERSION),
            (RADICLE_OID_PACKAGE, RADICLE_OID_LIBRARY_VERSION),
            (GIT2_PACKAGE, GIT2_LIBRARY_VERSION),
            (GIT_REF_FORMAT_PACKAGE, GIT_REF_FORMAT_LIBRARY_VERSION),
        ] {
            assert!(
                lockfile_has_package_version(name, version),
                "receipt version does not match lockfile: {name} {version}"
            );
        }
    }

    fn remove_directory(path: &Path) -> Result<(), Box<dyn Error>> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn initialize_repository(path: &Path) -> Result<(), Box<dyn Error>> {
        let repository = git2::Repository::init(path)?;
        let mut config = repository.config()?.open_level(git2::ConfigLevel::Local)?;
        config.set_str("user.name", GIT_AUTHOR_NAME)?;
        config.set_str("user.email", GIT_AUTHOR_EMAIL)?;
        let tree_id = repository.treebuilder(None)?.write()?;
        let signature = git2::Signature::new(
            GIT_AUTHOR_NAME,
            GIT_AUTHOR_EMAIL,
            &git2::Time::new(TEST_TIMESTAMP_SECONDS, 0),
        )?;
        let commit_id = {
            let tree = repository.find_tree(tree_id)?;
            repository.commit(
                Some(GIT_REF_MAIN),
                &signature,
                &signature,
                GIT_COMMIT_MESSAGE,
                &tree,
                &[],
            )?
        };
        repository.reference(GIT_REF_MAIN, commit_id, true, GIT_COMMIT_MESSAGE)?;
        repository.set_head(GIT_REF_MAIN)?;
        Ok(())
    }

    fn initialize_keystore(path: &Path, seed_byte: u8) -> Result<PublicKey, Box<dyn Error>> {
        let seed = Seed::new([seed_byte; ED25519_SEED_BYTES]);
        Ok(Keystore::new(&path).store(KeyPair::from_seed(seed), GIT_AUTHOR_NAME, None)?)
    }

    // r[verify cairn.radicle_cob_only.store.roundtrip]
    // r[verify cairn.radicle_cob_only.authority.allowed]
    #[test]
    fn direct_cob_round_trip_accepts_immutable_delegate_update() -> Result<(), Box<dyn Error>> {
        let fixture = Fixture::new("roundtrip")?;
        let init = fixture.init()?;
        let expected_root = change_cob_import_content_root(&init, &fixture.creator.to_string())?;
        let created = create_change(
            &fixture.repository,
            &fixture.creator_keys,
            init,
            &expected_root,
        )?;
        let inspected = inspect_change(&fixture.repository, CHANGE_ID)?;
        assert_eq!(inspected.object_id, created.object_id);
        assert_eq!(inspected.state.creator, fixture.creator.to_string());
        assert_eq!(
            inspected.state.delegates,
            BTreeSet::from([fixture.delegate.to_string()])
        );

        let basis = inspected
            .state
            .artifacts
            .get(&ChangeArtifactId::proposal())
            .ok_or("proposal history")?
            .head_ids();
        let actions = vec![ChangeCobAction::SetArtifact {
            artifact: ChangeArtifactId::proposal(),
            content: Some(UPDATED_PROPOSAL.to_string()),
            basis,
        }];
        let mut expected_state = inspected.state.clone();
        apply_change_cob_operation(
            &mut expected_state,
            ChangeCobOperation {
                id: PREVIEW_OPERATION_ID.to_string(),
                actor: fixture.delegate.to_string(),
                is_delegate: true,
                actions: actions.clone(),
            },
        )?;
        let expected_root = change_cob_content_root(&expected_state)?;
        let updated = update_change(
            &fixture.repository,
            &fixture.delegate_keys,
            &created.object_id,
            CHANGE_ID,
            &fixture.delegate.to_string(),
            &actions,
            &expected_root,
        )?;
        assert_eq!(updated.actor, fixture.delegate.to_string());
        let reloaded = inspect_change(&fixture.repository, CHANGE_ID)?;
        assert_eq!(change_cob_content_root(&reloaded.state)?, expected_root);
        assert_eq!(
            reloaded
                .state
                .artifacts
                .get(&ChangeArtifactId::proposal())
                .ok_or("proposal history")?
                .head_ids(),
            BTreeSet::from([updated.operation_id])
        );

        let lifecycle_actions = vec![ChangeCobAction::SetLifecycle {
            status: ChangeLifecycleStatus::ArchiveRequested,
            subject_root: expected_root.clone(),
            evidence: BTreeSet::new(),
            basis: reloaded.state.lifecycle.head_ids(),
        }];
        let mut expected_lifecycle = reloaded.state;
        apply_change_cob_operation(
            &mut expected_lifecycle,
            ChangeCobOperation {
                id: PREVIEW_LIFECYCLE_OPERATION_ID.to_string(),
                actor: fixture.delegate.to_string(),
                is_delegate: true,
                actions: lifecycle_actions.clone(),
            },
        )?;
        let lifecycle_update = update_change(
            &fixture.repository,
            &fixture.delegate_keys,
            &created.object_id,
            CHANGE_ID,
            &fixture.delegate.to_string(),
            &lifecycle_actions,
            &expected_root,
        )?;
        assert_eq!(lifecycle_update.content_root, expected_root);
        let lifecycle = inspect_change(&fixture.repository, CHANGE_ID)?;
        assert!(matches!(
            change_cob_lifecycle_view(&lifecycle.state)?,
            ChangeLifecycleView::Current(record)
                if record.status == ChangeLifecycleStatus::ArchiveRequested
        ));
        Ok(())
    }

    // r[verify cairn.radicle_cob_only.authority.denied]
    // r[verify cairn.radicle_cob_only.store.negative]
    // r[verify cairn.radicle_change_cob.radicle_adapter.reject]
    #[test]
    fn outsider_and_concurrent_lock_fail_before_ref_update() -> Result<(), Box<dyn Error>> {
        let fixture = Fixture::new("negative")?;
        let repository = CobRepository::open(&fixture.repository)?;
        let mut local_config = repository
            .raw()
            .config()?
            .open_level(git2::ConfigLevel::Local)?;
        local_config.remove("user.name")?;
        assert!(matches!(
            repository.validate_local_commit_identity(),
            Err(CobStoreError::MissingLocalCommitIdentity { field: "user.name" })
        ));
        local_config.set_str("user.name", GIT_AUTHOR_NAME)?;
        assert!(matches!(
            validate_delegates(&fixture.creator, [fixture.creator]),
            Err(CobError::Core(ChangeCobError::InvalidField { ref field, .. }))
                if field == "delegates"
        ));
        let init = fixture.init()?;
        let expected_root = change_cob_import_content_root(&init, &fixture.creator.to_string())?;
        let invalid_root = INVALID_ROOT_HEX_CHAR
            .to_string()
            .repeat(cairn_core::BLAKE3_HEX_LENGTH);
        assert!(matches!(
            create_change(
                &fixture.repository,
                &fixture.creator_keys,
                init.clone(),
                &invalid_root,
            ),
            Err(CobError::ContentRootDrift { .. })
        ));
        assert!(find_change(&fixture.repository, CHANGE_ID)?.is_none());
        let created = create_change(
            &fixture.repository,
            &fixture.creator_keys,
            init,
            &expected_root,
        )?;
        let outsider = signer_actor(&fixture.outsider_keys)?;
        let before = inspect_change(&fixture.repository, CHANGE_ID)?;
        let rollback_basis = before
            .state
            .artifacts
            .get(&ChangeArtifactId::proposal())
            .ok_or("proposal history")?
            .head_ids();
        assert!(matches!(
            update_change(
                &fixture.repository,
                &fixture.delegate_keys,
                &created.object_id,
                CHANGE_ID,
                &fixture.delegate.to_string(),
                &[ChangeCobAction::SetArtifact {
                    artifact: ChangeArtifactId::proposal(),
                    content: Some(UPDATED_PROPOSAL.to_string()),
                    basis: rollback_basis,
                }],
                &invalid_root,
            ),
            Err(CobError::ContentRootDrift { .. })
        ));
        assert_eq!(inspect_change(&fixture.repository, CHANGE_ID)?, before);
        let basis = before
            .state
            .artifacts
            .get(&ChangeArtifactId::proposal())
            .ok_or("proposal history")?
            .head_ids();
        let denied = update_change(
            &fixture.repository,
            &fixture.outsider_keys,
            &created.object_id,
            CHANGE_ID,
            &outsider,
            &[ChangeCobAction::SetArtifact {
                artifact: ChangeArtifactId::proposal(),
                content: Some("# Unauthorized\n".to_string()),
                basis,
            }],
            &expected_root,
        );
        assert!(matches!(denied, Err(CobError::UnauthorizedActor { .. })));
        assert_eq!(inspect_change(&fixture.repository, CHANGE_ID)?, before);

        let guard = CobMutationLock::acquire(&fixture.repository, CHANGE_ID)?;
        assert!(matches!(
            CobMutationLock::acquire(&fixture.repository, CHANGE_ID),
            Err(CobError::ConcurrentMutation { .. })
        ));
        guard.release()?;
        let released = CobMutationLock::acquire(&fixture.repository, CHANGE_ID)?;
        released.release()?;
        {
            let _dropped = CobMutationLock::acquire(&fixture.repository, CHANGE_ID)?;
        }
        let after_drop = CobMutationLock::acquire(&fixture.repository, CHANGE_ID)?;
        after_drop.release()?;
        Ok(())
    }
}
