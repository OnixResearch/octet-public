use tokio::sync::mpsc;

async fn shutdown(tx: mpsc::Sender<u8>) {
    let _ = tx.send(1).await;
}

struct Closer {
    tx: mpsc::Sender<u8>,
}

impl Closer {
    async fn close(&self) {
        let _ = self.tx.send(1).await;
    }
}

fn main() {}
