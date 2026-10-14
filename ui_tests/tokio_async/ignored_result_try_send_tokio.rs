use tokio::sync::mpsc;

fn shutdown(tx: mpsc::Sender<u8>) {
    let _ = tx.try_send(1);
}

struct Closer {
    tx: mpsc::Sender<u8>,
}

impl Closer {
    fn close(&self) {
        let _ = self.tx.try_send(1);
    }
}

fn main() {}
