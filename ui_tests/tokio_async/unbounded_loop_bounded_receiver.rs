#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use tokio::sync::mpsc;

async fn recv_loop(mut rx: mpsc::Receiver<u8>) {
    while let Some(msg) = rx.recv().await {
        let _ = msg;
    }
}

fn main() {}
