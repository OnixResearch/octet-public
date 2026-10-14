use std::sync::mpsc::channel as std_channel;

use crossbeam::channel::unbounded as crossbeam_unbounded;
use tokio::sync::mpsc::unbounded_channel as tokio_unbounded_channel;

fn std_unbounded() {
    let _ = std_channel::<u8>();
}

fn crossbeam_unbounded_ctor() {
    let _ = crossbeam_unbounded::<u8>();
}

fn tokio_unbounded_ctor() {
    let _ = tokio_unbounded_channel::<u8>();
}

fn tokio_bounded_is_allowed() {
    let _ = tokio::sync::mpsc::channel::<u8>(32);
}

fn main() {
    std_unbounded();
    crossbeam_unbounded_ctor();
    tokio_unbounded_ctor();
    tokio_bounded_is_allowed();
}
