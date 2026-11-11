#![allow(non_trait_imports, reason = "the durability fixtures target source-shape checks; naming lints have their own fixtures")]

use std::fs::File;
use std::io::Write;

pub enum PersistOutcome {
    Uncertain(String),
}

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

fn quarantine(_path: &str) {}

fn persist(file: &mut File, bytes: &[u8]) -> Result<(), StoreError> {
    file.write_all(bytes).map_err(|_error| store_error())?;
    match file.sync_all() {
        Ok(()) => Ok(()),
        Err(error) => {
            let _outcome = PersistOutcome::Uncertain(error.to_string());
            Err(store_error())
        }
    }
}

fn persist_with_quarantine(path: &str, file: &mut File, bytes: &[u8]) -> Result<(), StoreError> {
    file.write_all(bytes).map_err(|_error| store_error())?;
    if let Err(_error) = file.sync_all() {
        quarantine(path);
        return Err(store_error());
    }
    Ok(())
}

fn persist_commit(transaction: WriteTransaction) -> Result<(), StoreError> {
    transaction.commit().map_err(|_error| store_error())
}

fn main() {}
