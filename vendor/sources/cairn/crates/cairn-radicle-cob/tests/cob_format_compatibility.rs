use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    process,
};

use cairn_core::{
    apply_change_cob_operation, change_cob_content_root, change_cob_import_content_root,
    ChangeArtifactId, ChangeBaseRevision, ChangeCobAction, ChangeCobInit, ChangeCobOperation,
    CHANGE_COLLABORATION_SCHEMA_V1, RADICLE_CHANGE_COB_TYPE_NAME,
};
use git2::Repository;
use onix_cairn_radicle_cob::{
    create_change, inspect_change, update_change, CobRepository, CobStoreError, TYPENAME,
};
use radicle_crypto::{ssh::Keystore, KeyPair, PublicKey, Seed};

const CHANGE_ID: &str = "format-compatibility";
const WORKSPACE_ID: &str = "workspace:format-compatibility";
const POLICY_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PROPOSAL: &str = "# Exact COB bytes\n";
const CREATOR_PROPOSAL: &str = "# Creator proposal\n";
const DELEGATE_PROPOSAL: &str = "# Delegate proposal\n";
const RESOLVED_PROPOSAL: &str = "# Resolved proposal\n";
const PREVIEW_CREATOR_UPDATE_ID: &str = "preview:creator-update";
const PREVIEW_DELEGATE_UPDATE_ID: &str = "preview:delegate-update";
const PREVIEW_RESOLUTION_ID: &str = "preview:resolution";
const REPLICA_DIRECTORY: &str = "replica";
const GIT_OBJECTS_DIRECTORY: &str = ".git/objects";
const REPLICATION_REFLOG_MESSAGE: &str = "Replicate signed delegate COB ref for conflict fixture";
const TEST_DIRECTORY_PREFIX: &str = "cairn-cob-format-compatibility";
const GIT_AUTHOR_NAME: &str = "Cairn COB integration";
const GIT_AUTHOR_EMAIL: &str = "cob-integration@example.invalid";
const GIT_COMMIT_MESSAGE: &str = "Initialize integration repository";
const GIT_REF_MAIN: &str = "refs/heads/main";
const MANIFEST_BLOB_NAME: &str = "manifest";
const TEST_TIMESTAMP_SECONDS: i64 = 1_700_000_000;
const ED25519_SEED_BYTES: usize = 32;
const CREATOR_SEED_BYTE: u8 = 11;
const OTHER_SEED_BYTE: u8 = 12;

struct Fixture {
    root: PathBuf,
    repository: PathBuf,
    creator_keys: PathBuf,
    other_keys: PathBuf,
    creator: PublicKey,
    other: PublicKey,
}

impl Fixture {
    fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let root =
            std::env::temp_dir().join(format!("{TEST_DIRECTORY_PREFIX}-{name}-{}", process::id()));
        remove_directory(&root)?;
        fs::create_dir_all(&root)?;
        let repository = root.join("repository");
        initialize_repository(&repository)?;
        let creator_keys = root.join("creator-keys");
        let creator = initialize_keystore(&creator_keys, CREATOR_SEED_BYTE)?;
        let other_keys = root.join("other-keys");
        let other = initialize_keystore(&other_keys, OTHER_SEED_BYTE)?;
        Ok(Self {
            root,
            repository,
            creator_keys,
            other_keys,
            creator,
            other,
        })
    }

    fn init(&self) -> Result<ChangeCobInit, Box<dyn Error>> {
        let head = CobRepository::open(&self.repository)?.head_oid()?;
        Ok(ChangeCobInit {
            schema: CHANGE_COLLABORATION_SCHEMA_V1.to_string(),
            change_id: CHANGE_ID.to_string(),
            repository_id: WORKSPACE_ID.to_string(),
            base_git_oid: ChangeBaseRevision::new("git_oid", head.to_string())?.encode()?,
            policy_hash: POLICY_HASH.to_string(),
            delegates: BTreeSet::from([self.other.to_string()]),
            artifacts: BTreeMap::from([(ChangeArtifactId::proposal(), PROPOSAL.to_string())]),
        })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = remove_directory(&self.root);
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
    let repository = Repository::init(path)?;
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
    repository.set_head(GIT_REF_MAIN)?;
    assert_eq!(repository.head()?.target(), Some(commit_id));
    Ok(())
}

fn initialize_keystore(path: &Path, seed_byte: u8) -> Result<PublicKey, Box<dyn Error>> {
    let seed = Seed::new([seed_byte; ED25519_SEED_BYTES]);
    Ok(Keystore::new(&path).store(KeyPair::from_seed(seed), GIT_AUTHOR_NAME, None)?)
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_symlink() {
            return Err(format!("test fixture refuses symlink: {}", entry.path().display()).into());
        }
        if file_type.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else if file_type.is_file() {
            if target.exists() {
                continue;
            }
            fs::copy(entry.path(), target)?;
        } else {
            return Err(
                format!("unsupported test fixture entry: {}", entry.path().display()).into(),
            );
        }
    }
    Ok(())
}

fn cob_ref(object_id: &str, actor: &PublicKey) -> String {
    format!("refs/namespaces/{actor}/refs/cobs/{RADICLE_CHANGE_COB_TYPE_NAME}/{object_id}")
}

// r[impl cairn.radicle_cob_only.validation]
// r[impl cairn.radicle_two_peer_convergence.retired]
// r[verify cairn.radicle_two_peer_convergence.retired.local_only]
// r[verify cairn.radicle_cob_only.store.roundtrip]
// r[verify cairn.radicle_cob_only.compatibility.git_refs]
// r[verify cairn.radicle_cob_only.compatibility.signatures]
// r[verify cairn.radicle_change_cob.radicle_adapter.mapping]
#[test]
fn direct_store_writes_standard_signed_cob_commit_and_ref() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new("positive")?;
    let init = fixture.init()?;
    let expected_root = change_cob_import_content_root(&init, &fixture.creator.to_string())?;
    let created = create_change(
        &fixture.repository,
        &fixture.creator_keys,
        init,
        &expected_root,
    )?;

    let repository = Repository::open(&fixture.repository)?;
    let reference = repository.find_reference(&cob_ref(&created.object_id, &fixture.creator))?;
    assert_eq!(
        reference.target().map(|target| target.to_string()),
        Some(created.operation_id.clone())
    );
    let operation = repository.find_commit(created.operation_id.parse::<git2::Oid>()?)?;
    assert!(operation.tree()?.get_name(MANIFEST_BLOB_NAME).is_some());
    let reloaded = inspect_change(&fixture.repository, CHANGE_ID)?;
    assert_eq!(reloaded.object_id, created.object_id);
    assert_eq!(reloaded.state.creator, fixture.creator.to_string());
    assert_eq!(
        reloaded.state.delegates,
        BTreeSet::from([fixture.other.to_string()])
    );
    assert_eq!(
        cairn_core::change_cob_content_root(&reloaded.state)?,
        expected_root
    );
    Ok(())
}

// r[verify cairn.radicle_cob_only.store.roundtrip]
// r[verify cairn.radicle_cob_only.validation.direct]
// r[verify cairn.collaboration_update.radicle_execution.success]
// r[verify cairn.collaboration_resolution.radicle_execution.success]
#[test]
fn concurrent_signed_heads_resolve_without_losing_history() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new("resolution")?;
    let init = fixture.init()?;
    let expected_root = change_cob_import_content_root(&init, &fixture.creator.to_string())?;
    let created = create_change(
        &fixture.repository,
        &fixture.creator_keys,
        init,
        &expected_root,
    )?;
    let replica = fixture.root.join(REPLICA_DIRECTORY);
    copy_directory(&fixture.repository, &replica)?;
    let initial = inspect_change(&fixture.repository, CHANGE_ID)?;
    let root_basis = BTreeSet::from([created.operation_id.clone()]);

    let creator_actions = vec![ChangeCobAction::SetArtifact {
        artifact: ChangeArtifactId::proposal(),
        content: Some(CREATOR_PROPOSAL.to_string()),
        basis: root_basis.clone(),
    }];
    let mut creator_expected = initial.state.clone();
    apply_change_cob_operation(
        &mut creator_expected,
        ChangeCobOperation {
            id: PREVIEW_CREATOR_UPDATE_ID.to_string(),
            actor: fixture.creator.to_string(),
            is_delegate: false,
            actions: creator_actions.clone(),
        },
    )?;
    let creator_operation = update_change(
        &fixture.repository,
        &fixture.creator_keys,
        &created.object_id,
        CHANGE_ID,
        &fixture.creator.to_string(),
        &creator_actions,
        &change_cob_content_root(&creator_expected)?,
    )?;

    let delegate_actions = vec![ChangeCobAction::SetArtifact {
        artifact: ChangeArtifactId::proposal(),
        content: Some(DELEGATE_PROPOSAL.to_string()),
        basis: root_basis,
    }];
    let mut delegate_expected = initial.state;
    apply_change_cob_operation(
        &mut delegate_expected,
        ChangeCobOperation {
            id: PREVIEW_DELEGATE_UPDATE_ID.to_string(),
            actor: fixture.other.to_string(),
            is_delegate: true,
            actions: delegate_actions.clone(),
        },
    )?;
    let delegate_operation = update_change(
        &replica,
        &fixture.other_keys,
        &created.object_id,
        CHANGE_ID,
        &fixture.other.to_string(),
        &delegate_actions,
        &change_cob_content_root(&delegate_expected)?,
    )?;

    copy_directory(
        &replica.join(GIT_OBJECTS_DIRECTORY),
        &fixture.repository.join(GIT_OBJECTS_DIRECTORY),
    )?;
    let replica_repository = Repository::open(&replica)?;
    let delegate_ref = cob_ref(&created.object_id, &fixture.other);
    let delegate_target = replica_repository
        .find_reference(&delegate_ref)?
        .target()
        .ok_or("delegate COB ref target")?;
    Repository::open(&fixture.repository)?.reference(
        &delegate_ref,
        delegate_target,
        true,
        REPLICATION_REFLOG_MESSAGE,
    )?;

    let conflicted = inspect_change(&fixture.repository, CHANGE_ID)?;
    let heads = conflicted
        .state
        .artifacts
        .get(&ChangeArtifactId::proposal())
        .ok_or("proposal history")?
        .head_ids();
    assert_eq!(
        heads,
        BTreeSet::from([
            creator_operation.operation_id.clone(),
            delegate_operation.operation_id.clone(),
        ])
    );
    assert!(change_cob_content_root(&conflicted.state).is_err());

    let actions = vec![ChangeCobAction::SetArtifact {
        artifact: ChangeArtifactId::proposal(),
        content: Some(RESOLVED_PROPOSAL.to_string()),
        basis: heads,
    }];
    let mut expected_state = conflicted.state.clone();
    apply_change_cob_operation(
        &mut expected_state,
        ChangeCobOperation {
            id: PREVIEW_RESOLUTION_ID.to_string(),
            actor: fixture.other.to_string(),
            is_delegate: true,
            actions: actions.clone(),
        },
    )?;
    let expected_root = change_cob_content_root(&expected_state)?;
    let resolved = update_change(
        &fixture.repository,
        &fixture.other_keys,
        &created.object_id,
        CHANGE_ID,
        &fixture.other.to_string(),
        &actions,
        &expected_root,
    )?;
    let reloaded = inspect_change(&fixture.repository, CHANGE_ID)?;
    assert_eq!(change_cob_content_root(&reloaded.state)?, expected_root);

    let repository = CobRepository::open(&fixture.repository)?;
    let object_id = created.object_id.parse::<radicle_cob::ObjectId>()?;
    let stale_entry = delegate_operation
        .operation_id
        .parse::<radicle_cob::change::EntryId>()?;
    let before_ref = repository.namespace_ref_target(&fixture.other, &TYPENAME, &object_id)?;
    let stale_update = <CobRepository as radicle_cob::object::Storage>::update(
        &repository,
        &fixture.other,
        &TYPENAME,
        &object_id,
        &stale_entry,
    );
    assert!(matches!(
        stale_update,
        Err(CobStoreError::ReferenceDrift { .. })
    ));
    assert_eq!(
        repository.namespace_ref_target(&fixture.other, &TYPENAME, &object_id)?,
        before_ref
    );

    for operation in [
        created.operation_id,
        creator_operation.operation_id,
        delegate_operation.operation_id,
        resolved.operation_id,
    ] {
        assert!(reloaded.state.operations.contains(&operation));
    }
    Ok(())
}

// r[verify cairn.radicle_cob_only.store.negative]
// r[verify cairn.radicle_cob_only.compatibility.malformed]
// r[verify cairn.collaboration_update.radicle_execution.blocked]
// r[verify cairn.collaboration_resolution.radicle_execution.blocked]
#[test]
fn malformed_actor_history_is_rejected_instead_of_ignored() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new("negative")?;
    let init = fixture.init()?;
    let expected_root = change_cob_import_content_root(&init, &fixture.creator.to_string())?;
    let created = create_change(
        &fixture.repository,
        &fixture.creator_keys,
        init,
        &expected_root,
    )?;

    let repository = Repository::open(&fixture.repository)?;
    let mismatched_ref = cob_ref(&created.object_id, &fixture.other);
    repository.reference(
        &mismatched_ref,
        created.operation_id.parse::<git2::Oid>()?,
        true,
        "inject mismatched signed namespace",
    )?;
    assert!(inspect_change(&fixture.repository, CHANGE_ID).is_err());
    repository.find_reference(&mismatched_ref)?.delete()?;

    let ordinary_head = repository.head()?.target().ok_or("repository head")?;
    repository.reference(
        &mismatched_ref,
        ordinary_head,
        true,
        "inject malformed actor history",
    )?;
    assert!(inspect_change(&fixture.repository, CHANGE_ID).is_err());
    Ok(())
}
