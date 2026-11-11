#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use tokio::sync::mpsc;

async fn wait_once(mut rx: mpsc::Receiver<u8>) {
    let _ = rx.recv().await;
}

fn main() {}
