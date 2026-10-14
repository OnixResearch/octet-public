#![allow(unbounded_channel)]

use std::sync::mpsc;

fn shutdown(tx: mpsc::Sender<u8>) {
    let _ = tx.send(1);
}

struct Closer {
    tx: mpsc::Sender<u8>,
}

impl Closer {
    fn close(&self) {
        let _ = self.tx.send(1);
    }
}

struct FakeSender;

impl FakeSender {
    fn send(&self, _value: u8) -> Result<(), ()> {
        Ok(())
    }
}

fn shutdown_fake(tx: FakeSender) {
    let _ = tx.send(1);
}

fn publish(tx: mpsc::Sender<u8>) {
    let _ = tx.send(1);
}

fn main() {
    let (tx, _rx) = mpsc::channel();
    shutdown(tx.clone());
    Closer { tx: tx.clone() }.close();
    publish(tx);
    shutdown_fake(FakeSender);
}
