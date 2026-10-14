const DEVICE_QUEUE_CAPACITY: usize = 16;

fn fixture() {
    let (_tx, _rx) = bounded_device_queue(); // capacity_witness=DEVICE_QUEUE_CAPACITY; backpressure=await_send
}

fn bounded_device_queue<T>() -> (QueueSender<T>, QueueReceiver<T>) {
    unimplemented!()
}

struct QueueSender<T>(core::marker::PhantomData<T>);
struct QueueReceiver<T>(core::marker::PhantomData<T>);
