use std::fs;
use std::path::PathBuf;

use super::publish_shard;

const CONCURRENT_WRITERS: usize = 2;
const SHARD_BYTES: &[u8] = b"identical shard";

fn directory(case: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("octet-shard-publication-{}-{case}", std::process::id()));
    fs::create_dir(&root).expect("reserve test directory");
    root
}

#[test]
fn concurrent_identical_shards_have_independent_staging() {
    let root = directory("concurrent");
    let output = root.join("shard.json");
    std::thread::scope(|scope| {
        let mut jobs = Vec::new();
        for index in 0..CONCURRENT_WRITERS {
            let temporary = root.join(format!("shard-{index}.tmp"));
            let output = &output;
            jobs.push(scope.spawn(move || publish_shard(&temporary, output, SHARD_BYTES)));
        }
        for job in jobs {
            job.join().expect("publisher thread").expect("publish shard");
        }
    });
    assert_eq!(fs::read(&output).expect("published bytes"), SHARD_BYTES);
    fs::remove_dir_all(root).expect("remove owned test directory");
}

#[test]
fn occupied_staging_file_is_not_overwritten() {
    let root = directory("collision");
    let temporary = root.join("shard.tmp");
    let output = root.join("shard.json");
    fs::write(&temporary, SHARD_BYTES).expect("existing staged bytes");
    let error = publish_shard(&temporary, &output, b"replacement").expect_err("stage collision");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&temporary).expect("preserved bytes"), SHARD_BYTES);
    assert!(!output.exists());
    fs::remove_dir_all(root).expect("remove owned test directory");
}
