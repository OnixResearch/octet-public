#![allow(non_trait_imports, reason = "the durability fixtures target source-shape checks; naming lints have their own fixtures")]

use std::fs::File;

pub struct StoreError {
    pub detail: String,
}

fn store_error() -> StoreError {
    StoreError {
        detail: String::from("uncertain"),
    }
}

fn persist(file: &mut File) -> Result<(), StoreError> {
    if let Err(_error) = file.sync_all() {
        return Err(store_error());
    }
    file.sync_all().map_err(|_error| store_error())?;
    Ok(())
}

fn main() {}
