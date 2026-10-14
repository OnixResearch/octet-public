use std::{error::Error, path::PathBuf};

use radicle_crypto::{ssh::Keystore, KeyPair, Seed};
use serde_json::json;

// Public fixture seeds: never use these identities for production authority.
const CREATOR_SEED: u8 = 51;
const DELEGATE_SEED: u8 = 52;

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(std::env::args_os().nth(1).ok_or("usage: octet-cob-fixture-keys OUTPUT_DIRECTORY")?);
    let creator = Keystore::new(&root.join("creator")).store(
        KeyPair::from_seed(Seed::new([CREATOR_SEED; 32])),
        "Octet synthetic fixture creator",
        None,
    )?;
    let delegate = Keystore::new(&root.join("delegate")).store(
        KeyPair::from_seed(Seed::new([DELEGATE_SEED; 32])),
        "Octet synthetic fixture delegate",
        None,
    )?;
    println!("{}", json!({"creator": creator.to_string(), "delegate": delegate.to_string()}));
    Ok(())
}
