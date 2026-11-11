use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    process::{self, Command},
};

use git2::Repository;
use radicle_crypto::{ssh::Keystore, KeyPair, PublicKey, Seed};
use serde::Deserialize;

mod evaluator;
mod reference;
pub(crate) use evaluator::ReferenceState;
pub(crate) use reference::ReferenceRepository;

pub(crate) const BUNDLE_BYTES: &[u8] =
    include_bytes!("../../../../fixtures/radicle-cob-interoperability/external-cairn-cob.bundle");
pub(crate) const PRODUCER_SOURCE_BYTES: &[u8] =
    include_bytes!("../../../../fixtures/radicle-cob-interoperability/producer/src/main.rs");
pub(crate) const PRODUCER_MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../fixtures/radicle-cob-interoperability/producer/Cargo.toml");
pub(crate) const PRODUCER_LOCK_BYTES: &[u8] =
    include_bytes!("../../../../fixtures/radicle-cob-interoperability/producer/Cargo.lock");
pub(crate) const REFERENCE_STORAGE_SOURCE: &str = include_str!("reference.rs");
pub(crate) const REFERENCE_EVALUATOR_SOURCE: &str = include_str!("evaluator.rs");
const PROVENANCE_JSON: &str =
    include_str!("../../../../fixtures/radicle-cob-interoperability/provenance.json");
const TEST_DIRECTORY_PREFIX: &str = "cairn-cob-interoperability";
const GIT_AUTHOR_NAME: &str = "Cairn COB interoperability";
const GIT_AUTHOR_EMAIL: &str = "cob-interoperability@example.invalid";
const GIT_REF_MAIN: &str = "refs/heads/main";
const ED25519_SEED_BYTES: usize = 32;
const GIT_FETCH_FAILURE: &str = "failed to fetch static COB bundle";

#[derive(Debug, Deserialize)]
pub(crate) struct Provenance {
    pub(crate) schema: String,
    pub(crate) producer: ProducerProvenance,
    pub(crate) fixture: FixtureProvenance,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProducerProvenance {
    pub(crate) implementation: String,
    pub(crate) radicle_version: String,
    pub(crate) radicle_cob_version: String,
    pub(crate) radicle_crypto_version: String,
    pub(crate) radicle_oid_version: String,
    pub(crate) git2_version: String,
    pub(crate) generation_command: String,
    pub(crate) bundle_command: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FixtureProvenance {
    pub(crate) bundle_path: String,
    pub(crate) bundle_blake3: String,
    pub(crate) producer_source_path: String,
    pub(crate) producer_source_blake3: String,
    pub(crate) producer_manifest_path: String,
    pub(crate) producer_manifest_blake3: String,
    pub(crate) producer_lock_path: String,
    pub(crate) producer_lock_blake3: String,
    pub(crate) actor: String,
    pub(crate) delegate: String,
    pub(crate) delegate_seed_byte: u8,
    pub(crate) base_oid: String,
    pub(crate) object_id: String,
    pub(crate) operation_id: String,
    pub(crate) reference: String,
    pub(crate) change_id: String,
    pub(crate) repository_id: String,
    pub(crate) type_name: String,
    pub(crate) proposal: String,
}

pub(crate) struct ExternalFixture {
    root: PathBuf,
    pub(crate) repository: PathBuf,
    pub(crate) delegate_keystore: PathBuf,
    pub(crate) delegate: PublicKey,
}

impl ExternalFixture {
    pub(crate) fn new(name: &str, provenance: &Provenance) -> Result<Self, Box<dyn Error>> {
        let root =
            std::env::temp_dir().join(format!("{TEST_DIRECTORY_PREFIX}-{name}-{}", process::id()));
        remove_directory(&root)?;
        fs::create_dir_all(&root)?;
        let repository = root.join("repository");
        materialize_bundle(&repository, &root.join("fixture.bundle"))?;
        let delegate_keystore = root.join("delegate-keys");
        let delegate =
            initialize_keystore(&delegate_keystore, provenance.fixture.delegate_seed_byte)?;
        if delegate.to_string() != provenance.fixture.delegate {
            return Err("fixture delegate seed does not reproduce declared key".into());
        }
        Ok(Self {
            root,
            repository,
            delegate_keystore,
            delegate,
        })
    }
}

impl Drop for ExternalFixture {
    fn drop(&mut self) {
        let _ = remove_directory(&self.root);
    }
}

pub(crate) fn provenance() -> Result<Provenance, serde_json::Error> {
    serde_json::from_str(PROVENANCE_JSON)
}

pub(crate) fn verify_fixture_identity(
    bundle: &[u8],
    producer_source: &[u8],
    producer_manifest: &[u8],
    producer_lock: &[u8],
    provenance: &Provenance,
) -> Result<(), String> {
    verify_hash("bundle", bundle, &provenance.fixture.bundle_blake3)?;
    verify_hash(
        "producer source",
        producer_source,
        &provenance.fixture.producer_source_blake3,
    )?;
    verify_hash(
        "producer manifest",
        producer_manifest,
        &provenance.fixture.producer_manifest_blake3,
    )?;
    verify_hash(
        "producer lock",
        producer_lock,
        &provenance.fixture.producer_lock_blake3,
    )
}

pub(crate) fn initialize_repository(path: &Path) -> Result<Repository, Box<dyn Error>> {
    let repository = Repository::init(path)?;
    set_local_identity(&repository)?;
    let tree_id = repository.treebuilder(None)?.write()?;
    let signature = git2::Signature::now(GIT_AUTHOR_NAME, GIT_AUTHOR_EMAIL)?;
    let commit_id = {
        let tree = repository.find_tree(tree_id)?;
        repository.commit(
            Some(GIT_REF_MAIN),
            &signature,
            &signature,
            "Initialize Cairn interoperability repository",
            &tree,
            &[],
        )?
    };
    repository.set_head(GIT_REF_MAIN)?;
    if repository.head()?.target() != Some(commit_id) {
        return Err("initialized repository HEAD mismatch".into());
    }
    Ok(repository)
}

pub(crate) fn initialize_keystore(path: &Path, seed_byte: u8) -> Result<PublicKey, Box<dyn Error>> {
    let seed = Seed::new([seed_byte; ED25519_SEED_BYTES]);
    Ok(Keystore::new(&path).store(KeyPair::from_seed(seed), GIT_AUTHOR_NAME, None)?)
}

pub(crate) fn remove_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn materialize_bundle(repository_path: &Path, bundle_path: &Path) -> Result<(), Box<dyn Error>> {
    fs::write(bundle_path, BUNDLE_BYTES)?;
    let repository = Repository::init_bare(repository_path)?;
    set_local_identity(&repository)?;
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_path)
        .arg("fetch")
        .arg(bundle_path)
        .arg("+refs/heads/*:refs/heads/*")
        .arg("+refs/namespaces/*:refs/namespaces/*")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "{GIT_FETCH_FAILURE}: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    repository.set_head(GIT_REF_MAIN)?;
    Ok(())
}

fn set_local_identity(repository: &Repository) -> Result<(), git2::Error> {
    let mut config = repository.config()?.open_level(git2::ConfigLevel::Local)?;
    config.set_str("user.name", GIT_AUTHOR_NAME)?;
    config.set_str("user.email", GIT_AUTHOR_EMAIL)
}

fn verify_hash(label: &str, bytes: &[u8], expected: &str) -> Result<(), String> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        ))
    }
}
