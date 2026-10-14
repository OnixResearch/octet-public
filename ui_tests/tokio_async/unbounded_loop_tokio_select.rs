use tokio::sync::mpsc;

async fn event_loop(mut rx: mpsc::Receiver<u8>) {
    loop {
        tokio::select! {
            maybe = rx.recv() => {
                if maybe.is_none() {
                    break;
                }
            }
        }
    }
}

fn main() {}
