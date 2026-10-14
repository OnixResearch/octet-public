const DEVICE_QUEUE_CAPACITY: usize = 16;

fn fixture() {
    let (_tx, _rx) = tokio::sync::mpsc::channel(DEVICE_QUEUE_CAPACITY);
}
