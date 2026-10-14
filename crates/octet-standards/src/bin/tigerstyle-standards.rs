#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
//! CLI wrapper for `tigerstyle-standards`.

use std::process::ExitCode;

#[path = "../cli_shell.rs"]
mod cli_shell;

fn main() -> ExitCode {
    cli_shell::run_cli_from_env()
}
