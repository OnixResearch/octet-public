mod tokio {
    pub struct JoinHandle;

    pub fn spawn<F>(_future: F) -> JoinHandle {
        JoinHandle
    }

    pub mod task {
        pub struct JoinHandle;

        pub fn spawn<F>(_future: F) -> JoinHandle {
            JoinHandle
        }
    }
}

mod async_std {
    pub mod task {
        pub struct JoinHandle;

        pub fn spawn<F>(_future: F) -> JoinHandle {
            JoinHandle
        }
    }
}

fn start_core_work() {
    let handle = std::thread::spawn(|| {});
    let _joined = handle.join();
    let _tokio_handle = tokio::spawn(async {});
    let _tokio_task_handle = tokio::task::spawn(async {});
    let _async_std_handle = async_std::task::spawn(async {});
}

fn main() {
    start_core_work();
}
