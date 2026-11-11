#![allow(non_trait_imports, reason = "the durability fixtures target source-shape checks; naming lints have their own fixtures")]

use std::fs::File;
use std::io::Write;

pub struct StoreError {
    pub detail: String,
}

fn store_error() -> StoreError {
    StoreError {
        detail: String::from("uncertain"),
    }
}

fn reconcile(_path: &str) -> Result<(), StoreError> { Ok(()) }

fn persist_reconcile(path: &str, file: &mut File, bytes: &[u8]) -> Result<(), StoreError> {
    file.write_all(bytes).map_err(|_error| store_error())?;
    if let Err(_error) = file.sync_all() {
        return Err(store_error());
    }
    reconcile(path).map_err(|_error| store_error())?;
    file.sync_all().map_err(|_error| store_error())?;
    Ok(())
}

fn persist_reopen(path: &str, bytes: &[u8]) -> Result<(), StoreError> {
    let mut file = File::create(path).map_err(|_error| store_error())?;
    file.write_all(bytes).map_err(|_error| store_error())?;
    if let Err(_error) = file.sync_all() {
        let mut reopened = File::open(path).map_err(|_error| store_error())?;
        reopened.sync_all().map_err(|_error| store_error())?;
        return Ok(());
    }
    Ok(())
}

fn main() {}
