#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use tokio::sync::mpsc;

async fn event_loop(mut rx: mpsc::Receiver<u8>) {
    loop {
        let _ = rx.recv().await;
    }
}

fn main() {}
