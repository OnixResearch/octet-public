use tokio::sync::mpsc;

async fn event_loop(mut rx: mpsc::Receiver<u8>) {
    loop {
        let _ = rx.recv().await;
    }
}

fn main() {}
