use std::{
    error::Error,
    fs,
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use cairn_core::{
    empty_global_registry_snapshot, verify_global_registry_snapshot, GlobalRegistrySnapshot,
};

use super::policy::GlobalRegistryPolicy;

pub(super) trait RegistryStorageAdapter {
    fn id(&self) -> &'static str;
    fn load(
        &self,
        path: &Path,
        scope_id: &str,
        policy: &GlobalRegistryPolicy,
    ) -> Result<GlobalRegistrySnapshot, Box<dyn Error>>;
    fn commit(
        &mut self,
        path: &Path,
        expected_snapshot_hash: &str,
        snapshot: &GlobalRegistrySnapshot,
        policy: &GlobalRegistryPolicy,
    ) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Default)]
pub(super) struct MemoryRegistryStorage {
    snapshot: Option<GlobalRegistrySnapshot>,
}

impl RegistryStorageAdapter for MemoryRegistryStorage {
    fn id(&self) -> &'static str {
        "memory.v1"
    }

    fn load(
        &self,
        _path: &Path,
        scope_id: &str,
        policy: &GlobalRegistryPolicy,
    ) -> Result<GlobalRegistrySnapshot, Box<dyn Error>> {
        Ok(self
            .snapshot
            .clone()
            .unwrap_or_else(|| empty_global_registry_snapshot(scope_id, &policy.core)))
    }

    fn commit(
        &mut self,
        _path: &Path,
        expected_snapshot_hash: &str,
        snapshot: &GlobalRegistrySnapshot,
        policy: &GlobalRegistryPolicy,
    ) -> Result<(), Box<dyn Error>> {
        let empty = empty_global_registry_snapshot(&snapshot.scope_id, &policy.core);
        let observed = self
            .snapshot
            .as_ref()
            .map(|value| value.snapshot_hash.as_str())
            .unwrap_or(&empty.snapshot_hash);
        require_expected_snapshot(expected_snapshot_hash, observed)?;
        self.snapshot = Some(snapshot.clone());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(super) struct PortableRegistryStorage;

impl RegistryStorageAdapter for PortableRegistryStorage {
    fn id(&self) -> &'static str {
        "portable-json.v1"
    }

    fn load(
        &self,
        path: &Path,
        scope_id: &str,
        policy: &GlobalRegistryPolicy,
    ) -> Result<GlobalRegistrySnapshot, Box<dyn Error>> {
        if !path.exists() {
            return Ok(empty_global_registry_snapshot(scope_id, &policy.core));
        }
        reject_symlink(path)?;
        let value = read_snapshot(path, policy.max_artifact_bytes)?;
        if value.scope_id != scope_id {
            return Err(format!(
                "registry scope mismatch: expected {scope_id}, observed {}",
                value.scope_id
            )
            .into());
        }
        let report = verify_global_registry_snapshot(&value, &policy.core);
        report.snapshot.ok_or_else(|| {
            format!("portable registry verification failed: {:?}", report.issues).into()
        })
    }

    fn commit(
        &mut self,
        path: &Path,
        expected_snapshot_hash: &str,
        snapshot: &GlobalRegistrySnapshot,
        policy: &GlobalRegistryPolicy,
    ) -> Result<(), Box<dyn Error>> {
        let _lock = PortableWriteLock::acquire(path)?;
        if path.exists() {
            reject_symlink(path)?;
            let observed = read_snapshot(path, policy.max_artifact_bytes)?;
            require_expected_snapshot(expected_snapshot_hash, &observed.snapshot_hash)?;
        } else {
            let empty = empty_global_registry_snapshot(&snapshot.scope_id, &policy.core);
            require_expected_snapshot(expected_snapshot_hash, &empty.snapshot_hash)?;
        }
        atomic_write_json(path, snapshot)
    }
}

struct PortableWriteLock {
    path: PathBuf,
}

impl PortableWriteLock {
    fn acquire(store: &Path) -> Result<Self, Box<dyn Error>> {
        let parent = store.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let file_name = store
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("registry store path must have a UTF-8 file name")?;
        let path = parent.join(format!(".{file_name}.lock"));
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| format!("registry portable write lock unavailable: {error}"))?;
        Ok(Self { path })
    }
}

impl Drop for PortableWriteLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub(super) fn storage_adapter(id: &str) -> Result<Box<dyn RegistryStorageAdapter>, Box<dyn Error>> {
    match id {
        "memory.v1" => Ok(Box::<MemoryRegistryStorage>::default()),
        "portable-json.v1" => Ok(Box::<PortableRegistryStorage>::default()),
        "redb.v1" => redb_adapter(),
        _ => Err(format!("unsupported registry storage adapter: {id}").into()),
    }
}

pub(super) fn read_snapshot(
    path: &Path,
    max_bytes: u64,
) -> Result<GlobalRegistrySnapshot, Box<dyn Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err("registry snapshot path must not be a symlink".into());
    }
    if metadata.len() > max_bytes {
        return Err(format!(
            "registry artifact exceeds bounded read limit: {} > {max_bytes}",
            metadata.len()
        )
        .into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(super) fn atomic_write_json<T: serde::Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), Box<dyn Error>> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("registry output path must have a UTF-8 file name")?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn require_expected_snapshot(expected: &str, observed: &str) -> Result<(), Box<dyn Error>> {
    if expected == observed {
        Ok(())
    } else {
        Err(format!("registry snapshot conflict: expected {expected}, observed {observed}").into())
    }
}

fn reject_symlink(path: &Path) -> Result<(), Box<dyn Error>> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        Err("registry storage path must not be a symlink".into())
    } else {
        Ok(())
    }
}

#[cfg(feature = "redb-registry")]
fn redb_adapter() -> Result<Box<dyn RegistryStorageAdapter>, Box<dyn Error>> {
    Ok(Box::<redb_storage::RedbRegistryStorage>::default())
}

#[cfg(not(feature = "redb-registry"))]
fn redb_adapter() -> Result<Box<dyn RegistryStorageAdapter>, Box<dyn Error>> {
    Err("redb.v1 adapter is not compiled; enable the redb-registry feature".into())
}

#[cfg(feature = "redb-registry")]
mod redb_storage {
    use std::{error::Error, path::Path};

    use cairn_core::{
        empty_global_registry_snapshot, verify_global_registry_snapshot, GlobalRegistrySnapshot,
    };
    use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

    use super::{require_expected_snapshot, GlobalRegistryPolicy, RegistryStorageAdapter};

    const SNAPSHOT_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("snapshot");
    const NODE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("nodes");
    const TAG_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("tags");
    const FORWARD_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("forward_edges");
    const REVERSE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("reverse_edges");
    const SOURCE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("sources");
    const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
    const CURRENT_KEY: &str = "current";
    const STORAGE_SCHEMA_KEY: &str = "storage_schema";

    #[derive(Debug, Default)]
    pub(super) struct RedbRegistryStorage;

    impl RegistryStorageAdapter for RedbRegistryStorage {
        fn id(&self) -> &'static str {
            "redb.v1"
        }

        fn load(
            &self,
            path: &Path,
            scope_id: &str,
            policy: &GlobalRegistryPolicy,
        ) -> Result<GlobalRegistrySnapshot, Box<dyn Error>> {
            if !path.exists() {
                return Ok(empty_global_registry_snapshot(scope_id, &policy.core));
            }
            let database = Database::open(path)?;
            let read = database.begin_read()?;
            let metadata = read.open_table(METADATA_TABLE)?;
            let observed_schema = metadata
                .get(STORAGE_SCHEMA_KEY)?
                .map(|value| String::from_utf8(value.value().to_vec()))
                .transpose()?
                .ok_or("redb registry storage schema is missing")?;
            if observed_schema != policy.storage_schema {
                return Err(
                    format!("unsupported redb registry storage schema: {observed_schema}").into(),
                );
            }
            let snapshots = read.open_table(SNAPSHOT_TABLE)?;
            let bytes = snapshots
                .get(CURRENT_KEY)?
                .ok_or("redb registry current snapshot is missing")?
                .value()
                .to_vec();
            let snapshot: GlobalRegistrySnapshot = serde_json::from_slice(&bytes)?;
            if snapshot.scope_id != scope_id {
                return Err(format!(
                    "registry scope mismatch: expected {scope_id}, observed {}",
                    snapshot.scope_id
                )
                .into());
            }
            let report = verify_global_registry_snapshot(&snapshot, &policy.core);
            report.snapshot.ok_or_else(|| {
                format!("redb registry verification failed: {:?}", report.issues).into()
            })
        }

        fn commit(
            &mut self,
            path: &Path,
            expected_snapshot_hash: &str,
            snapshot: &GlobalRegistrySnapshot,
            policy: &GlobalRegistryPolicy,
        ) -> Result<(), Box<dyn Error>> {
            let existed = path.exists();
            if !existed {
                let empty = empty_global_registry_snapshot(&snapshot.scope_id, &policy.core);
                require_expected_snapshot(expected_snapshot_hash, &empty.snapshot_hash)?;
            }
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let database = Database::create(path)?;
            let write = database.begin_write()?;
            if existed {
                let observed_bytes = {
                    let snapshots = write.open_table(SNAPSHOT_TABLE)?;
                    let value = snapshots
                        .get(CURRENT_KEY)?
                        .map(|guard| guard.value().to_vec());
                    value
                };
                let observed: GlobalRegistrySnapshot = serde_json::from_slice(
                    &observed_bytes.ok_or("redb registry current snapshot is missing")?,
                )?;
                require_expected_snapshot(expected_snapshot_hash, &observed.snapshot_hash)?;
                let observed_schema = {
                    let metadata = write.open_table(METADATA_TABLE)?;
                    let value = metadata
                        .get(STORAGE_SCHEMA_KEY)?
                        .map(|guard| String::from_utf8(guard.value().to_vec()))
                        .transpose()?;
                    value
                }
                .ok_or("redb registry storage schema is missing")?;
                if observed_schema != policy.storage_schema {
                    return Err(format!(
                        "unsupported redb registry storage schema: {observed_schema}"
                    )
                    .into());
                }
            }
            {
                let mut snapshots = write.open_table(SNAPSHOT_TABLE)?;
                snapshots.retain(|_, _| false)?;
                let snapshot_bytes = serde_json::to_vec(snapshot)?;
                snapshots.insert(CURRENT_KEY, snapshot_bytes.as_slice())?;
                let mut metadata = write.open_table(METADATA_TABLE)?;
                metadata.retain(|_, _| false)?;
                metadata.insert(STORAGE_SCHEMA_KEY, policy.storage_schema.as_bytes())?;
                let mut nodes = write.open_table(NODE_TABLE)?;
                nodes.retain(|_, _| false)?;
                for node in &snapshot.nodes {
                    let bytes = serde_json::to_vec(node)?;
                    nodes.insert(node.stable_id.as_str(), bytes.as_slice())?;
                }
                let mut tags = write.open_table(TAG_TABLE)?;
                tags.retain(|_, _| false)?;
                for tag in &snapshot.tags {
                    let bytes = serde_json::to_vec(tag)?;
                    tags.insert(tag.assignment_id.as_str(), bytes.as_slice())?;
                }
                let mut forward = write.open_table(FORWARD_TABLE)?;
                let mut reverse = write.open_table(REVERSE_TABLE)?;
                forward.retain(|_, _| false)?;
                reverse.retain(|_, _| false)?;
                for edge in &snapshot.edges {
                    let bytes = serde_json::to_vec(edge)?;
                    let forward_key = format!("{}\u{0}{}", edge.from_id, edge.stable_id);
                    let reverse_key = format!("{}\u{0}{}", edge.to_id, edge.stable_id);
                    forward.insert(forward_key.as_str(), bytes.as_slice())?;
                    reverse.insert(reverse_key.as_str(), bytes.as_slice())?;
                }
                let mut sources = write.open_table(SOURCE_TABLE)?;
                sources.retain(|_, _| false)?;
                for source in &snapshot.sources {
                    let bytes = serde_json::to_vec(source)?;
                    sources.insert(source.source_id.as_str(), bytes.as_slice())?;
                }
            }
            write.commit()?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use cairn_core::{
        build_global_registry_snapshot, RegistryCandidateRecords, RegistryCorePolicy,
        RegistrySourceRecord,
    };

    use super::*;
    use crate::global_registry::policy::GlobalRegistryPolicy;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const MAX_TEST_ARTIFACT_BYTES: u64 = 4 * 1024 * 1024;

    fn policy() -> GlobalRegistryPolicy {
        GlobalRegistryPolicy {
            schema: "cairn.global-registry-snapshot.v1".to_string(),
            storage_schema: "cairn.global-registry-storage.v1".to_string(),
            max_artifact_bytes: MAX_TEST_ARTIFACT_BYTES,
            adapters: BTreeMap::new(),
            graph_profiles: BTreeMap::new(),
            core: RegistryCorePolicy {
                reserved_tag_namespaces: vec!["cairn.policy".to_string()],
                allowed_vocabulary_owners: vec!["cairn".to_string(), "valence".to_string()],
                required_non_claims: vec!["registry storage proves no source claims".to_string()],
            },
        }
    }

    fn candidate_snapshot() -> GlobalRegistrySnapshot {
        build_global_registry_snapshot(
            "workspace:test",
            RegistryCandidateRecords {
                sources: vec![RegistrySourceRecord {
                    source_id: "source:test".to_string(),
                    family: "test".to_string(),
                    schema: "test.v1".to_string(),
                    content_hash: HASH_A.to_string(),
                    vocabulary_owner: "cairn".to_string(),
                    source_ref: "fixture:test".to_string(),
                    non_claims: vec!["test source proves no claims".to_string()],
                    record_hash: String::new(),
                }],
                ..RegistryCandidateRecords::default()
            },
            &policy().core,
        )
        .snapshot
        .unwrap()
    }

    fn path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "cairn-global-registry-{name}-{}",
            std::process::id()
        ))
    }

    fn exercise_adapter(
        adapter: &mut dyn RegistryStorageAdapter,
        store: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let policy = policy();
        let observed = adapter.load(store, "workspace:test", &policy)?;
        let candidate = candidate_snapshot();
        adapter.commit(store, &observed.snapshot_hash, &candidate, &policy)?;
        assert_eq!(adapter.load(store, "workspace:test", &policy)?, candidate);
        assert!(adapter
            .commit(store, HASH_A, &observed, &policy)
            .unwrap_err()
            .to_string()
            .contains("snapshot conflict"));
        assert_eq!(adapter.load(store, "workspace:test", &policy)?, candidate);
        Ok(())
    }

    #[test]
    // r[verify cairn.global_registry.storage]
    // r[verify cairn.global_registry.storage.memory]
    // r[verify cairn.global_registry.adapters.storage]
    fn memory_storage_conforms_and_rejects_stale_writes() -> Result<(), Box<dyn Error>> {
        exercise_adapter(&mut MemoryRegistryStorage::default(), Path::new("unused"))
    }

    #[test]
    // r[verify cairn.global_registry.portability.export]
    // r[verify cairn.global_registry.portability.verify]
    fn portable_storage_conforms_and_preserves_prior_state() -> Result<(), Box<dyn Error>> {
        let path = path("portable.json");
        if path.exists() {
            fs::remove_file(&path)?;
        }
        exercise_adapter(&mut PortableRegistryStorage, &path)?;
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    // r[verify cairn.global_registry.adapters.storage]
    // r[verify cairn.global_registry.validation.positive]
    fn memory_and_portable_backends_reload_identical_snapshots() -> Result<(), Box<dyn Error>> {
        let policy = policy();
        let candidate = candidate_snapshot();
        let mut memory = MemoryRegistryStorage::default();
        let memory_observed = memory.load(Path::new("unused"), "workspace:test", &policy)?;
        memory.commit(
            Path::new("unused"),
            &memory_observed.snapshot_hash,
            &candidate,
            &policy,
        )?;
        let path = path("backend-parity.json");
        if path.exists() {
            fs::remove_file(&path)?;
        }
        let mut portable = PortableRegistryStorage;
        let portable_observed = portable.load(&path, "workspace:test", &policy)?;
        portable.commit(&path, &portable_observed.snapshot_hash, &candidate, &policy)?;
        assert_eq!(
            memory.load(Path::new("unused"), "workspace:test", &policy)?,
            portable.load(&path, "workspace:test", &policy)?
        );
        fs::remove_file(path)?;
        Ok(())
    }

    #[cfg(feature = "redb-registry")]
    #[test]
    // r[verify cairn.global_registry.storage.redb]
    // r[verify cairn.global_registry.storage.redb_failure]
    fn redb_storage_conforms_transactionally() -> Result<(), Box<dyn Error>> {
        use redb::{Database, ReadableDatabase, TableDefinition};

        const SNAPSHOTS: TableDefinition<&str, &[u8]> = TableDefinition::new("snapshot");
        const METADATA: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
        const CURRENT: &str = "current";
        const STORAGE_SCHEMA: &str = "storage_schema";

        let path = path("redb.db");
        if path.exists() {
            fs::remove_file(&path)?;
        }
        let mut adapter = redb_storage::RedbRegistryStorage;
        exercise_adapter(&mut adapter, &path)?;
        let database = Database::open(&path)?;
        let baseline = {
            let read = database.begin_read()?;
            let table = read.open_table(SNAPSHOTS)?;
            let value = table
                .get(CURRENT)?
                .ok_or("current snapshot")?
                .value()
                .to_vec();
            value
        };
        {
            let interrupted = database.begin_write()?;
            {
                let mut table = interrupted.open_table(SNAPSHOTS)?;
                table.insert(CURRENT, b"interrupted".as_slice())?;
            }
        }
        let after_interruption = {
            let read = database.begin_read()?;
            let table = read.open_table(SNAPSHOTS)?;
            let value = table
                .get(CURRENT)?
                .ok_or("current snapshot")?
                .value()
                .to_vec();
            value
        };
        assert_eq!(after_interruption, baseline);
        {
            let migration = database.begin_write()?;
            {
                let mut metadata = migration.open_table(METADATA)?;
                metadata.insert(STORAGE_SCHEMA, b"unsupported.v99".as_slice())?;
            }
            migration.commit()?;
        }
        drop(database);
        assert!(adapter
            .load(&path, "workspace:test", &policy())
            .unwrap_err()
            .to_string()
            .contains("unsupported redb registry storage schema"));
        let database = Database::open(&path)?;
        let preserved = {
            let read = database.begin_read()?;
            let table = read.open_table(SNAPSHOTS)?;
            let value = table
                .get(CURRENT)?
                .ok_or("current snapshot")?
                .value()
                .to_vec();
            value
        };
        assert_eq!(preserved, baseline);
        drop(database);
        fs::remove_file(path)?;
        Ok(())
    }
}
