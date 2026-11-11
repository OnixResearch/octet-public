use std::{error::Error, fs, path::PathBuf, str::FromStr};

use nonempty::NonEmpty;
use radicle::{prelude::RepoId, storage::git::Repository as RadicleRepository};
use radicle_cob::{change::Entry, Create, TypeName, Version};
use radicle_crypto::{
    ssh::{keystore::MemorySigner, Keystore},
    KeyPair, Seed,
};
use serde_json::json;

const OUTPUT_ARGUMENT_INDEX: usize = 1;
const EXPECTED_ARGUMENT_COUNT: usize = 2;
const GIT_AUTHOR_NAME: &str = "External Radicle COB reference";
const GIT_AUTHOR_EMAIL: &str = "cob-reference@example.invalid";
const GIT_REF_MAIN: &str = "refs/heads/main";
const GIT_COMMIT_MESSAGE: &str = "Initialize external COB fixture";
const COB_COMMIT_MESSAGE: &str = "Create external Cairn COB fixture";
const TYPE_NAME: &str = "xyz.onixresearch.cairn-change";
const CHANGE_SCHEMA: &str = "cairn.change-collaboration.v1";
const CHANGE_ID: &str = "external-cob-fixture";
const WORKSPACE_ID: &str = "workspace:external-cob-fixture";
const POLICY_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PROPOSAL: &str = "# External reference proposal\n";
const BASE_REVISION_KIND: &str = "git_oid";
const BASE_COMMIT_TIMESTAMP_SECONDS: i64 = 1_514_817_556;
const ED25519_SEED_BYTES: usize = 32;
const CREATOR_SEED_BYTE: u8 = 41;
const DELEGATE_SEED_BYTE: u8 = 42;
const CREATOR_KEYSTORE_SUFFIX: &str = "creator-keys";
const DELEGATE_KEYSTORE_SUFFIX: &str = "delegate-keys";

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments.len() != EXPECTED_ARGUMENT_COUNT {
        return Err("usage: cairn-cob-reference-producer OUTPUT_REPOSITORY".into());
    }
    let output = PathBuf::from(&arguments[OUTPUT_ARGUMENT_INDEX]);
    if output.exists() {
        fs::remove_dir_all(&output)?;
    }
    let backend = git2::Repository::init_bare(&output)?;
    let mut config = backend.config()?.open_level(git2::ConfigLevel::Local)?;
    config.set_str("user.name", GIT_AUTHOR_NAME)?;
    config.set_str("user.email", GIT_AUTHOR_EMAIL)?;
    let base = initialize_base(&backend)?;
    let repo_id = RepoId::from(base);
    let repository = RadicleRepository {
        id: repo_id,
        backend,
    };

    let creator_keystore_path = output.with_extension(CREATOR_KEYSTORE_SUFFIX);
    let delegate_keystore_path = output.with_extension(DELEGATE_KEYSTORE_SUFFIX);
    remove_directory_if_present(&creator_keystore_path)?;
    remove_directory_if_present(&delegate_keystore_path)?;
    let creator_keystore = Keystore::new(&creator_keystore_path);
    let creator_id = creator_keystore.store(
        KeyPair::from_seed(Seed::new([CREATOR_SEED_BYTE; ED25519_SEED_BYTES])),
        GIT_AUTHOR_NAME,
        None,
    )?;
    let creator = MemorySigner::load(&creator_keystore, None)?;
    let delegate_id = Keystore::new(&delegate_keystore_path).store(
        KeyPair::from_seed(Seed::new([DELEGATE_SEED_BYTE; ED25519_SEED_BYTES])),
        GIT_AUTHOR_NAME,
        None,
    )?;
    let type_name = TypeName::from_str(TYPE_NAME)?;
    let base_revision = format!("{BASE_REVISION_KIND}:{base}");
    let action = serde_json::to_vec(&json!({
        "type": "init",
        "state": {
            "schema": CHANGE_SCHEMA,
            "change_id": CHANGE_ID,
            "repository_id": WORKSPACE_ID,
            "base_git_oid": base_revision,
            "policy_hash": POLICY_HASH,
            "delegates": [delegate_id.to_string()],
            "artifacts": {"proposal": PROPOSAL}
        }
    }))?;
    let created = radicle_cob::create::<NonEmpty<Entry>, _, _>(
        &repository,
        &creator,
        None,
        vec![base.into()],
        &creator_id,
        Create {
            contents: NonEmpty::new(action),
            type_name: type_name.clone(),
            message: COB_COMMIT_MESSAGE.to_string(),
            embeds: Vec::new(),
            version: Version::default(),
        },
    )?;
    let reference = format!(
        "refs/namespaces/{creator_id}/refs/cobs/{type_name}/{}",
        created.id
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "actor": creator_id.to_string(),
            "base_oid": base.to_string(),
            "change_id": CHANGE_ID,
            "delegate": delegate_id.to_string(),
            "object_id": created.id.to_string(),
            "operation_id": created.id.to_string(),
            "reference": reference,
            "repository_id": WORKSPACE_ID,
            "type_name": TYPE_NAME
        }))?
    );
    Ok(())
}

fn remove_directory_if_present(path: &std::path::Path) -> Result<(), std::io::Error> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn initialize_base(repository: &git2::Repository) -> Result<git2::Oid, git2::Error> {
    let tree_id = repository.treebuilder(None)?.write()?;
    let signature = git2::Signature::new(
        GIT_AUTHOR_NAME,
        GIT_AUTHOR_EMAIL,
        &git2::Time::new(BASE_COMMIT_TIMESTAMP_SECONDS, 0),
    )?;
    let tree = repository.find_tree(tree_id)?;
    repository.commit(
        Some(GIT_REF_MAIN),
        &signature,
        &signature,
        GIT_COMMIT_MESSAGE,
        &tree,
        &[],
    )
}
