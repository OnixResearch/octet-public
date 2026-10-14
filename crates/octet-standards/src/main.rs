#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
//! CLI wrapper for `octet-standards`.

use std::process::ExitCode;

mod cli_shell;

fn main() -> ExitCode {
    cli_shell::run_cli_from_env()
}
