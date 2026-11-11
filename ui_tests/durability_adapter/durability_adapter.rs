#![allow(non_trait_imports, reason = "the durability fixtures target source-shape checks; naming lints have their own fixtures")]

use std::fs::File;
use std::io::Write;

pub struct StoreError {
    pub detail: String,
}

pub struct Publication;

impl Publication {
    fn commit_durable(&mut self) -> Result<(), StoreError> {
        Ok(())
    }
}

fn store_error() -> StoreError {
    StoreError {
        detail: String::from("uncertain"),
    }
}

fn publish(publication: &mut Publication, file: &mut File, bytes: &[u8]) -> Result<(), StoreError> {
    file.write_all(bytes).map_err(|_error| store_error())?;
    file.flush().map_err(|_error| store_error())?;
    publication.commit_durable().map_err(|_error| store_error())?;
    Ok(())
}

fn main() {}
