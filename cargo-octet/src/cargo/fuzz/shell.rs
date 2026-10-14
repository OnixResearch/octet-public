use std::io::Read as _;

use super::*;

const CORPUS_DIRECTORY: &str = "fuzz/corpus/octet-profile-intake";
const ARTIFACT_DIRECTORY: &str = "fuzz/artifacts/octet-profile-intake";
const COMPILER_PROFILE: &str = "nix:fenix-nightly-2026-03-21-x86_64-unknown-linux-gnu";
const CRASH_PREFIX: &str = "crash-";
const MINIMIZED_PREFIX: &str = "minimized-";
const INTEGRATION_FAILURE_EXIT: i32 = 2;

struct Snapshot {
    entries: Vec<ArtifactIdentity>,
    total_bytes: u64,
}

impl Snapshot {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_ARTIFACT_COUNT),
            total_bytes: 0,
        }
    }

    fn push(&mut self, entry: ArtifactIdentity) -> Result<(), String> {
        if self.entries.len() >= MAX_ARTIFACT_COUNT {
            return Err("cargo-fuzz snapshot exceeds artifact count limit".to_owned());
        }
        self.total_bytes = self
            .total_bytes
            .checked_add(entry.length)
            .ok_or_else(|| "cargo-fuzz snapshot byte count overflow".to_owned())?;
        if self.total_bytes > MAX_ARTIFACT_BYTES {
            return Err("cargo-fuzz snapshot exceeds artifact byte limit".to_owned());
        }
        self.entries.push(entry);
        Ok(())
    }

    fn finish(mut self) -> Vec<ArtifactIdentity> {
        self.entries.sort();
        self.entries
    }
}

pub fn run(workspace_root: &std::path::Path) -> Result<(String, i32), String> {
    let initial_corpus = snapshot(workspace_root, CORPUS_DIRECTORY)?;
    let command_argv = command_argv();
    let (program, arguments) =
        command_argv.split_first().ok_or_else(|| "cargo-fuzz command argv is empty".to_owned())?;
    let output = std::process::Command::new(program)
        .args(arguments)
        .current_dir(workspace_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|error| format!("run reviewed cargo-fuzz command: {error}"))?;
    let log_identity = bounded_log_identity(&output.stdout, &output.stderr)?;
    let final_corpus = snapshot(workspace_root, CORPUS_DIRECTORY)?;
    let artifacts = snapshot(workspace_root, ARTIFACT_DIRECTORY)?;
    let exit_code = output.status.code().unwrap_or(INTEGRATION_FAILURE_EXIT);
    let crash_input = find_artifact(&artifacts, CRASH_PREFIX);
    let minimized_reproducer = find_artifact(&artifacts, MINIMIZED_PREFIX);
    let outcome = classify_outcome(exit_code, crash_input.as_ref());
    let replay_argv = minimized_reproducer.as_ref().map_or_else(Vec::new, |artifact| {
        vec![
            "cargo".to_owned(),
            "fuzz".to_owned(),
            "run".to_owned(),
            TARGET.to_owned(),
            artifact.path.clone(),
        ]
    });
    let details = ReceiptDetails {
        schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
        profile_id: PROFILE_ID.to_owned(),
        profile_identity: EXPECTED_PROFILE_IDENTITY.to_owned(),
        target: TARGET.to_owned(),
        engine: ENGINE.to_owned(),
        toolchain: TOOLCHAIN.to_owned(),
        compiler_identity: prefixed_identity(COMPILER_PROFILE.as_bytes()),
        sanitizer: SANITIZER.to_owned(),
        command_argv,
        budget: Budget {
            max_total_time_seconds: MAX_TOTAL_TIME_SECONDS,
            max_runs: MAX_RUNS,
            seed: FIXED_SEED,
        },
        initial_corpus_root: corpus_root(&initial_corpus),
        initial_corpus,
        final_corpus_root: corpus_root(&final_corpus),
        final_corpus,
        dictionary: None,
        outcome: outcome.to_owned(),
        crash_input,
        minimized_reproducer,
        replay_argv,
        bounded_log_identity: log_identity,
        evidence_tier: EVIDENCE_TIER.to_owned(),
        claimed_properties: Vec::new(),
        non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
    };
    let rendered =
        serde_json::to_string(&details).map_err(|error| format!("serialize cargo-fuzz run receipt: {error}"))?;
    Ok((rendered, exit_code))
}

fn snapshot(workspace_root: &std::path::Path, relative: &str) -> Result<Vec<ArtifactIdentity>, String> {
    let root = workspace_root.join(relative);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut pending = std::collections::VecDeque::with_capacity(MAX_ARTIFACT_COUNT);
    pending.push_back((root, relative.to_owned()));
    let mut snapshot = Snapshot::new();
    for _ in 0..MAX_ARTIFACT_COUNT {
        let Some((directory, logical_directory)) = pending.pop_front() else {
            return Ok(snapshot.finish());
        };
        let mut entries = std::fs::read_dir(&directory)
            .map_err(|error| format!("read cargo-fuzz directory {}: {error}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("read cargo-fuzz directory entry: {error}"))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            append_entry(entry, &logical_directory, &mut pending, &mut snapshot)?;
        }
    }
    Err("cargo-fuzz snapshot exceeds directory traversal limit".to_owned())
}

fn append_entry(
    entry: std::fs::DirEntry,
    logical_directory: &str,
    pending: &mut std::collections::VecDeque<(std::path::PathBuf, String)>,
    snapshot: &mut Snapshot,
) -> Result<(), String> {
    let file_type = entry.file_type().map_err(|error| format!("inspect cargo-fuzz entry: {error}"))?;
    if file_type.is_symlink() {
        return Err(format!("cargo-fuzz artifact {} is a symlink", entry.path().display()));
    }
    let name = entry.file_name().into_string().map_err(|_| "cargo-fuzz artifact name is not UTF-8".to_owned())?;
    let logical_path = format!("{logical_directory}/{name}");
    if file_type.is_dir() {
        if pending.len() >= MAX_ARTIFACT_COUNT {
            return Err("cargo-fuzz snapshot exceeds pending directory limit".to_owned());
        }
        pending.push_back((entry.path(), logical_path));
        return Ok(());
    }
    if !file_type.is_file() {
        return Err(format!("cargo-fuzz artifact {} is not a regular file", entry.path().display()));
    }
    snapshot.push(file_identity(&entry.path(), logical_path)?)
}

fn file_identity(path: &std::path::Path, logical_path: String) -> Result<ArtifactIdentity, String> {
    let length = std::fs::metadata(path)
        .map_err(|error| format!("inspect cargo-fuzz artifact {}: {error}", path.display()))?
        .len();
    if length > MAX_ARTIFACT_BYTES {
        return Err(format!("cargo-fuzz artifact {} exceeds byte limit", path.display()));
    }
    let capacity = usize::try_from(length).map_err(|_| "cargo-fuzz artifact length does not fit usize".to_owned())?;
    let mut bytes = Vec::with_capacity(capacity);
    std::fs::File::open(path)
        .map_err(|error| format!("open cargo-fuzz artifact {}: {error}", path.display()))?
        .take(MAX_ARTIFACT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read cargo-fuzz artifact {}: {error}", path.display()))?;
    let observed =
        u64::try_from(bytes.len()).map_err(|_| "cargo-fuzz artifact byte count does not fit u64".to_owned())?;
    if observed != length {
        return Err(format!("cargo-fuzz artifact {} changed during bounded read", path.display()));
    }
    Ok(ArtifactIdentity {
        path: logical_path,
        length,
        blake3: prefixed_identity(&bytes),
    })
}

fn bounded_log_identity(stdout: &[u8], stderr: &[u8]) -> Result<String, String> {
    let length = stdout.len().checked_add(stderr.len()).ok_or_else(|| "cargo-fuzz log length overflow".to_owned())?;
    let observed = u64::try_from(length).map_err(|_| "cargo-fuzz log length does not fit u64".to_owned())?;
    if observed > MAX_RECEIPT_BYTES {
        return Err("cargo-fuzz log exceeds bounded transcript limit".to_owned());
    }
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(stdout);
    bytes.extend_from_slice(stderr);
    Ok(prefixed_identity(&bytes))
}

fn find_artifact(artifacts: &[ArtifactIdentity], prefix: &str) -> Option<ArtifactIdentity> {
    artifacts
        .iter()
        .find(|artifact| artifact.path.rsplit('/').next().is_some_and(|name| name.starts_with(prefix)))
        .cloned()
}

fn classify_outcome(exit_code: i32, crash_input: Option<&ArtifactIdentity>) -> &'static str {
    if exit_code == 0 {
        "clean"
    } else if crash_input.is_some() {
        "crash"
    } else {
        "tool_failure"
    }
}

fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED_BYTES: &[u8] = b"seed";

    #[test]
    fn bounded_snapshot_hashes_files_and_rejects_symlinks() {
        let directory = tempfile::tempdir().expect("create cargo-fuzz shell fixture");
        let corpus = directory.path().join(CORPUS_DIRECTORY);
        std::fs::create_dir_all(&corpus).expect("create corpus");
        std::fs::write(corpus.join("seed"), SEED_BYTES).expect("write seed");

        let observed = snapshot(directory.path(), CORPUS_DIRECTORY).expect("snapshot corpus");
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].path, format!("{CORPUS_DIRECTORY}/seed"));
        assert_eq!(observed[0].length, u64::try_from(SEED_BYTES.len()).expect("seed length fits u64"));
        assert_eq!(observed[0].blake3, prefixed_identity(SEED_BYTES));

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(corpus.join("seed"), corpus.join("link")).expect("create symlink");
            assert!(snapshot(directory.path(), CORPUS_DIRECTORY).expect_err("symlink rejected").contains("symlink"));
        }
    }

    #[test]
    fn outcome_and_log_classification_have_positive_and_negative_paths() {
        let crash = ArtifactIdentity {
            path: format!("{ARTIFACT_DIRECTORY}/{CRASH_PREFIX}fixture"),
            length: u64::try_from(SEED_BYTES.len()).expect("seed length fits u64"),
            blake3: prefixed_identity(SEED_BYTES),
        };
        assert_eq!(classify_outcome(0, None), "clean");
        assert_eq!(classify_outcome(INTEGRATION_FAILURE_EXIT, Some(&crash)), "crash");
        assert_eq!(classify_outcome(INTEGRATION_FAILURE_EXIT, None), "tool_failure");
        assert!(bounded_log_identity(b"out", b"err").is_ok());
        let oversized = vec![b'x'; usize::try_from(MAX_RECEIPT_BYTES + 1).expect("log limit fits usize")];
        assert!(bounded_log_identity(&oversized, &[]).is_err());
    }
}
