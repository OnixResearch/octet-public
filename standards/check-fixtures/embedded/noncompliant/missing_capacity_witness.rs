fn fixture() {
    let (_tx, _rx) = bounded_device_queue();
}

fn bounded_device_queue<T>() -> (QueueSender<T>, QueueReceiver<T>) {
    unimplemented!()
}

struct QueueSender<T>(core::marker::PhantomData<T>);
struct QueueReceiver<T>(core::marker::PhantomData<T>);
