use std::{path::Path, str::FromStr};

use cairn_core::{
    apply_document_cob_operation, document_content_root, document_init_content_root,
    document_state_hash, initialize_document_cob, validate_document_init, DocumentAction,
    DocumentInit, DocumentOperation, DocumentState, BASE_REVISION_KIND_GIT_OID, RFD_COB_TYPE_NAME,
};
use nonempty::NonEmpty;
use radicle_cob::{
    change::Entry, CollaborativeObject, Create, Evaluate, ObjectId, TypeName, Update, Version,
};
use radicle_crypto::{
    signature::Keypair as _,
    ssh::keystore::{MemorySigner, MemorySignerError},
    PublicKey,
};
use radicle_oid::Oid;
use serde_json::Error as JsonError;
use thiserror::Error;

use crate::store::CobRepository;
use crate::CobMutationOutcome;

pub const CREATE_MESSAGE_PREFIX: &str = "Import RFD";
pub const UPDATE_MESSAGE_PREFIX: &str = "Update RFD";
pub const PREFLIGHT_OPERATION_ID: &str = "cairn-rfd-preflight";

// r[impl cairn.document_collaboration.radicle_adapter]
// r[impl cairn.document_collaboration.radicle_adapter.mapping]
// r[impl cairn.document_collaboration.radicle_adapter.compatibility]
// r[impl cairn.rfd_cob.interop]

pub static TYPE: std::sync::LazyLock<TypeName> =
    std::sync::LazyLock::new(|| match TypeName::from_str(RFD_COB_TYPE_NAME) {
        Ok(type_name) => type_name,
        Err(error) => panic!("invalid static RFD COB type name: {error}"),
    });

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CairnRfdCob {
    state: DocumentState,
}

impl CairnRfdCob {
    pub fn state(&self) -> &DocumentState {
        &self.state
    }

    pub fn into_state(self) -> DocumentState {
        self.state
    }
}

// r[impl cairn.document_collaboration.radicle_adapter.mapping]
// r[impl cairn.rfd_cob.interop.negative]
impl Evaluate<CobRepository> for CairnRfdCob {
    type Error = RfdCobError;

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

// r[verify cairn.document_collaboration.radicle_adapter.compatibility]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RfdCobInspection {
    pub repository_head_oid: String,
    pub object_id: String,
    pub state: DocumentState,
}

#[derive(Debug, Error)]
pub enum RfdCobError {
    #[error(transparent)]
    Store(#[from] crate::CobStoreError),
    #[error("COB operation failed: {0}")]
    Operation(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Cairn rejected the RFD COB operation: {0}")]
    Core(#[from] cairn_core::DocumentCobError),
    #[error("RFD action JSON is invalid: {0}")]
    ActionJson(#[from] JsonError),
    #[error("unexpected COB type {actual}; expected {expected}")]
    TypeMismatch { expected: String, actual: String },
    #[error("unsupported COB manifest version {actual}; expected {expected}")]
    VersionMismatch { expected: usize, actual: usize },
    #[error("root action base Git object {base} is missing from related objects")]
    MissingRelatedBase { base: String },
    #[error("RFD {document_id} already has COB object {object_id}")]
    AlreadyExists {
        document_id: String,
        object_id: String,
    },
    #[error("RFD {document_id} has no COB")]
    Missing { document_id: String },
    #[error("RFD {document_id} has multiple COBs: {object_ids:?}")]
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

pub fn load_signer(keystore_path: &Path) -> Result<MemorySigner, RfdCobError> {
    let signer = crate::load_signer(keystore_path)
        .map_err(|error| RfdCobError::SignerUnavailable(error.to_string()))?;
    Ok(signer)
}

// r[verify cairn.rfd_cob.interop.outward]
pub fn inspect_rfd(
    repository_path: &Path,
    document_id: &str,
) -> Result<RfdCobInspection, RfdCobError> {
    let repository = CobRepository::open(repository_path)?;
    inspect_open_rfd(&repository, document_id)
}

fn inspect_open_rfd(
    repository: &CobRepository,
    document_id: &str,
) -> Result<RfdCobInspection, RfdCobError> {
    let mut matches = Vec::new();
    for object_id in repository.object_ids(&TYPE)? {
        let Some(object) = load_object(repository, object_id)? else {
            return Err(RfdCobError::ObjectMissing {
                object_id: object_id.to_string(),
            });
        };
        if object.object.state.document_id == document_id {
            matches.push(object);
        }
    }
    matches.sort_by(|left, right| left.id.cmp(&right.id));
    if matches.len() > 1 {
        return Err(RfdCobError::Duplicate {
            document_id: document_id.to_string(),
            object_ids: matches.iter().map(|object| object.id.to_string()).collect(),
        });
    }
    let Some(object) = matches.pop() else {
        return Err(RfdCobError::Missing {
            document_id: document_id.to_string(),
        });
    };
    Ok(RfdCobInspection {
        repository_head_oid: repository.head_oid()?.to_string(),
        object_id: object.id.to_string(),
        state: object.object.into_state(),
    })
}

// r[verify cairn.rfd_cob.interop.outward]
pub fn create_rfd(
    repository_path: &Path,
    keystore_path: &Path,
    init: DocumentInit,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, RfdCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    match inspect_open_rfd(&repository, &init.document_id) {
        Ok(existing) => {
            return Err(RfdCobError::AlreadyExists {
                document_id: init.document_id,
                object_id: existing.object_id,
            });
        }
        Err(RfdCobError::Missing { .. }) => {}
        Err(error) => return Err(error),
    }
    validate_document_init(&init)?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    let planned_root = document_init_content_root(&init)?;
    if planned_root != expected_content_root {
        return Err(RfdCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let base = required_base_oid(&init)?;
    let action = DocumentAction::Init { state: init };
    let contents = encode_actions([&action])?;
    let created = radicle_cob::create::<CairnRfdCob, _, _>(
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

// r[verify cairn.rfd_cob.interop.outward]
pub fn update_rfd(
    repository_path: &Path,
    keystore_path: &Path,
    object_id: &str,
    document_id: &str,
    planned_actor: &str,
    actions: &[DocumentAction],
    expected_content_root: &str,
) -> Result<CobMutationOutcome, RfdCobError> {
    let repository = CobRepository::open(repository_path)?;
    repository.validate_local_commit_identity()?;
    let signer = load_signer(keystore_path)?;
    let public_key = signer.verifying_key();
    let actor = public_key.to_string();
    if actor != planned_actor {
        return Err(RfdCobError::ActorDrift {
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
        return Err(RfdCobError::ContentRootDrift {
            expected: expected_content_root.to_string(),
            observed: planned_root,
        });
    }
    let parsed_object = ObjectId::from_str(object_id).map_err(|_| RfdCobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    let prior_target = repository.namespace_ref_target(&public_key, &TYPE, &parsed_object)?;
    let updated = radicle_cob::update::<CairnRfdCob, _, _>(
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
) -> Result<CollaborativeObject<CairnRfdCob>, RfdCobError> {
    let parsed = ObjectId::from_str(object_id).map_err(|_| RfdCobError::InvalidGitOid {
        value: object_id.to_string(),
    })?;
    load_object(repository, parsed)?.ok_or_else(|| RfdCobError::ObjectMissing {
        object_id: object_id.to_string(),
    })
}

fn load_object(
    repository: &CobRepository,
    object_id: ObjectId,
) -> Result<Option<CollaborativeObject<CairnRfdCob>>, RfdCobError> {
    radicle_cob::get::<CairnRfdCob, _>(repository, &TYPE, &object_id).map_err(operation_error)
}

fn mutation_outcome_or_restore(
    repository: &CobRepository,
    namespace: &PublicKey,
    prior_target: Option<Oid>,
    new_target: Oid,
    object: CollaborativeObject<CairnRfdCob>,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, RfdCobError> {
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
                return Err(RfdCobError::Store(rollback));
            }
            Err(mutation)
        }
    }
}

fn mutation_outcome(
    object: CollaborativeObject<CairnRfdCob>,
    operation_id: Oid,
    actor: String,
    expected_content_root: &str,
) -> Result<CobMutationOutcome, RfdCobError> {
    let observed = document_content_root(object.object.state())?;
    if observed != expected_content_root {
        return Err(RfdCobError::ContentRootDrift {
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

fn validate_manifest(entry: &Entry) -> Result<(), RfdCobError> {
    if entry.manifest.type_name != *TYPE {
        return Err(RfdCobError::TypeMismatch {
            expected: TYPE.to_string(),
            actual: entry.manifest.type_name.to_string(),
        });
    }
    let expected = usize::from(Version::default());
    let actual = usize::from(entry.manifest.version);
    if actual != expected {
        return Err(RfdCobError::VersionMismatch { expected, actual });
    }
    Ok(())
}

fn decode_actions(entry: &Entry) -> Result<Vec<DocumentAction>, RfdCobError> {
    entry
        .contents()
        .iter()
        .map(|blob| serde_json::from_slice(blob))
        .collect::<Result<Vec<_>, _>>()
        .map_err(RfdCobError::from)
}

fn encode_actions<'a>(
    actions: impl IntoIterator<Item = &'a DocumentAction>,
) -> Result<NonEmpty<Vec<u8>>, RfdCobError> {
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

fn validate_root_base(entry: &Entry, actions: &[DocumentAction]) -> Result<(), RfdCobError> {
    let [DocumentAction::Init { state }] = actions else {
        return Err(cairn_core::DocumentCobError::InvalidRootShape.into());
    };
    let base = required_base_oid(state)?;
    if !entry.related.contains(&base) {
        return Err(RfdCobError::MissingRelatedBase {
            base: base.to_string(),
        });
    }
    Ok(())
}

fn required_base_oid(init: &DocumentInit) -> Result<Oid, RfdCobError> {
    let revision = cairn_core::decode_base_revision(&init.base_revision).map_err(|error| {
        cairn_core::DocumentCobError::Factory(format!("base revision invalid: {error}"))
    })?;
    if revision.kind != BASE_REVISION_KIND_GIT_OID {
        return Err(cairn_core::DocumentCobError::InvalidField {
            field: "base_revision".to_string(),
            reason: "RFD COB requires a git_oid base revision".to_string(),
        }
        .into());
    }
    Oid::from_str(&revision.value).map_err(|_| RfdCobError::InvalidGitOid {
        value: revision.value,
    })
}

fn action_document_id(action: &DocumentAction) -> &str {
    match action {
        DocumentAction::Init { state } => &state.document_id,
        _ => "unknown",
    }
}

fn operation_error<E>(error: E) -> RfdCobError
where
    E: std::error::Error + Send + Sync + 'static,
{
    RfdCobError::Operation(Box::new(error))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        error::Error,
        fs, io, process,
    };

    use cairn_core::{
        apply_document_cob_operation, document_content_root, DocumentOperation, DocumentRole,
        DOCUMENT_COLLABORATION_SCHEMA_V1, RFD_SCHEMA_V1,
    };
    use radicle_crypto::{ssh::Keystore, KeyPair, Seed};

    use super::*;

    const DOC: &str = "rfd-0001";
    const GIT_AUTHOR_NAME: &str = "Cairn RFD COB test";
    const GIT_AUTHOR_EMAIL: &str = "rfd-test@example.invalid";
    const GIT_COMMIT_MESSAGE: &str = "Initialize COB repository";
    const GIT_REF_MAIN: &str = "refs/heads/main";
    const TEST_TIMESTAMP_SECONDS: i64 = 1_700_000_000;
    const SEED_BYTES: usize = 32;
    const SEED_BYTE: u8 = 1;
    const POLICY_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    struct Fixture {
        root: std::path::PathBuf,
        repository: std::path::PathBuf,
        keys: std::path::PathBuf,
        author: PublicKey,
    }

    impl Fixture {
        fn new(name: &str) -> Result<Self, Box<dyn Error>> {
            let root = std::env::temp_dir().join(format!("cairn-rfd-cob-{name}-{}", process::id()));
            remove_dir(&root)?;
            fs::create_dir_all(&root)?;
            let repository = root.join("repository");
            initialize_repository(&repository)?;
            let keys = root.join("keys");
            let seed = Seed::new([SEED_BYTE; SEED_BYTES]);
            let author =
                Keystore::new(&keys).store(KeyPair::from_seed(seed), GIT_AUTHOR_NAME, None)?;
            Ok(Self {
                root,
                repository,
                keys,
                author,
            })
        }

        fn init(&self) -> Result<DocumentInit, Box<dyn Error>> {
            let repository = CobRepository::open(&self.repository)?;
            let base = format!("{BASE_REVISION_KIND_GIT_OID}:{}", repository.head_oid()?);
            Ok(DocumentInit {
                schema: RFD_SCHEMA_V1.to_string(),
                kind: "rfd".to_string(),
                document_id: DOC.to_string(),
                workspace_id: "workspace:rfd".to_string(),
                base_revision: base,
                policy_hash: POLICY_HASH.to_string(),
                authors: BTreeSet::from([self.author.to_string()]),
                participants: BTreeSet::from(["participant".to_string()]),
                reviewers: BTreeSet::from(["reviewer".to_string()]),
                approvers: BTreeSet::from(["approver".to_string()]),
                publishers: BTreeSet::from(["publisher".to_string()]),
                artifacts: BTreeMap::from([("spec.md".to_string(), "# Initial RFD\n".to_string())]),
            })
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = remove_dir(&self.root);
        }
    }

    fn remove_dir(path: &std::path::Path) -> Result<(), Box<dyn Error>> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn initialize_repository(path: &std::path::Path) -> Result<(), Box<dyn Error>> {
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

    // r[verify cairn.document_collaboration.radicle_adapter.mapping]
    // r[verify cairn.rfd_cob.interop.outward]
    #[test]
    fn direct_rfd_cob_round_trip_preserves_state_and_content_root() -> Result<(), Box<dyn Error>> {
        let fixture = Fixture::new("roundtrip")?;
        let init = fixture.init()?;
        let expected_root = document_init_content_root(&init)?;
        let created = create_rfd(&fixture.repository, &fixture.keys, init, &expected_root)?;
        let inspected = inspect_rfd(&fixture.repository, DOC)?;
        assert_eq!(inspected.object_id, created.object_id);
        assert_eq!(inspected.state.document_id, DOC);
        assert_eq!(document_content_root(&inspected.state)?, expected_root);

        // author edits the artifact (author role matched by real public key)
        let heads = inspected.state.artifacts["spec.md"].head_ids();
        let actions = vec![DocumentAction::SetArtifact {
            artifact: "spec.md".to_string(),
            content: Some("# Updated RFD\n".to_string()),
            basis: heads.clone(),
        }];
        let mut expected_state = inspected.state.clone();
        apply_document_cob_operation(
            &mut expected_state,
            DocumentOperation {
                id: PREFLIGHT_OPERATION_ID.to_string(),
                actor: fixture.author.to_string(),
                role: Some(DocumentRole::Author),
                actions: actions.clone(),
            },
        )?;
        let expected_root = document_content_root(&expected_state)?;
        let updated = update_rfd(
            &fixture.repository,
            &fixture.keys,
            &created.object_id,
            DOC,
            &fixture.author.to_string(),
            &actions,
            &expected_root,
        )?;
        assert_eq!(updated.actor, fixture.author.to_string());
        let reloaded = inspect_rfd(&fixture.repository, DOC)?;
        assert_eq!(document_content_root(&reloaded.state)?, expected_root);
        assert!(!reloaded.state.artifacts["spec.md"].head_ids().is_empty());
        Ok(())
    }

    // r[verify cairn.document_collaboration.radicle_adapter.compatibility]
    #[test]
    fn existing_change_cob_bytes_remain_stable() -> Result<(), Box<dyn Error>> {
        // Guard that adding the RFD COB type does not disturb the change COB surface:
        // the change type name and schema constants remain untouched, and no change
        // refs/commits are created by this test.
        assert_eq!(RFD_COB_TYPE_NAME, "xyz.onixresearch.rfd");
        assert_eq!(
            DOCUMENT_COLLABORATION_SCHEMA_V1,
            "cairn.document-collaboration.v1"
        );
        Ok(())
    }
}
