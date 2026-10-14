#![feature(rustc_private)]

use std::env;

use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().collect();
    dylint_driver::dylint_driver(&args)
}
