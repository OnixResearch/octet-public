//! Helpers shared by unit tests that execute fixture scripts.

#[cfg(unix)]
const OWNER_ONLY_EXECUTABLE: u32 = 0o700;
#[cfg(unix)]
const WRITER_SHELL: &str = "/bin/sh";
#[cfg(unix)]
const WRITER_SCRIPT: &str = "printf '%s' \"$2\" > \"$1\"";

/// Writes `contents` to `path` as an owner-only executable without this process ever holding a
/// write descriptor for the file.
///
/// `cargo test` runs tests as threads of one process. A sibling test that forks to spawn a child
/// copies every open descriptor into that child, which keeps them until it calls `exec`. A script
/// written with `std::fs::write` can therefore still be open for writing in such a child when this
/// test executes it, and `execve` fails with `ETXTBSY` ("Text file busy"). A short-lived shell
/// writes the file instead: its descriptor is gone once it exits, and no other process held one.
#[cfg(unix)]
pub(crate) fn write_executable_script(path: &std::path::Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt as _;

    let status = std::process::Command::new(WRITER_SHELL)
        .args(["-c", WRITER_SCRIPT, "write-executable-script"])
        .arg(path)
        .arg(contents)
        .env_clear()
        .status()
        .unwrap_or_else(|error| panic!("spawn writer for {}: {error}", path.display()));
    assert!(status.success(), "write {}: {status}", path.display());
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(OWNER_ONLY_EXECUTABLE))
        .unwrap_or_else(|error| panic!("make {} executable: {error}", path.display()));
}
