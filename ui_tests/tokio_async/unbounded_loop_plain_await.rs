use tokio::sync::mpsc;

async fn wait_once(mut rx: mpsc::Receiver<u8>) {
    let _ = rx.recv().await;
}

fn main() {}
