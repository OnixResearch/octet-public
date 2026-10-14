use tokio::sync::mpsc;

async fn recv_loop(mut rx: mpsc::UnboundedReceiver<u8>) {
    while let Some(msg) = rx.recv().await {
        let _ = msg;
    }
}

fn main() {}
