#![allow(non_trait_imports, ignored_result, reason = "the durability fixtures target durability checks; the import, naming, and ignored-result lints have their own fixtures")]

use std::fs::File;
use std::io::Write;

pub struct StoreError {
    pub detail: String,
}

struct WriteTransaction;

impl WriteTransaction {
    fn commit(self) -> Result<(), StoreError> {
        Ok(())
    }
}

fn store_error() -> StoreError {
    StoreError {
        detail: String::from("uncertain"),
    }
}

fn persist_direct(file: &mut File, bytes: &[u8]) -> Result<(), StoreError> {
    file.write_all(bytes).map_err(|_error| store_error())?;
    file.sync_all();
    Ok(())
}

fn persist_ignored(file: &mut File) -> Result<(), StoreError> {
    let _ = file.sync_data();
    Ok(())
}

fn persist_empty_error(file: &mut File) -> Result<(), StoreError> {
    if let Err(_error) = file.sync_all() {}
    Ok(())
}

fn persist_missing_uncertainty(file: &mut File) -> Result<(), StoreError> {
    match file.sync_all() {
        Ok(()) => Ok(()),
        Err(_error) => Ok(()),
    }
}

fn persist_commit_ignored(transaction: WriteTransaction) -> Result<(), StoreError> {
    let _ = transaction.commit();
    Ok(())
}

fn main() {}
