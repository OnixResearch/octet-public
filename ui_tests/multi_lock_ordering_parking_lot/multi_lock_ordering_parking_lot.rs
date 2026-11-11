#![warn(multi_lock_ordering)]
#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::sync::Arc;

// Should warn: `parking_lot` locks are `lock_api` type aliases.
struct ParkingTracker {
    count: parking_lot::Mutex<u64>,
    state: parking_lot::RwLock<u32>,
}

// Should warn: shared `parking_lot` locks behind `Arc` still count.
struct SharedParkingTracker {
    count: Arc<parking_lot::Mutex<u64>>,
    state: Arc<parking_lot::RwLock<u32>>,
}

// Should NOT warn: has lock ordering comment.
struct OrderedParkingTracker {
    // Lock ordering: always acquire `count` before `state`
    count: parking_lot::Mutex<u64>,
    state: Arc<parking_lot::RwLock<u32>>,
}

// Should NOT warn: single lock.
struct SingleParkingLock {
    data: Arc<parking_lot::Mutex<u32>>,
}

fn main() {
    let _ = ParkingTracker {
        count: parking_lot::Mutex::new(0),
        state: parking_lot::RwLock::new(0),
    };
    let _ = SharedParkingTracker {
        count: Arc::new(parking_lot::Mutex::new(0)),
        state: Arc::new(parking_lot::RwLock::new(0)),
    };
    let _ = OrderedParkingTracker {
        count: parking_lot::Mutex::new(0),
        state: Arc::new(parking_lot::RwLock::new(0)),
    };
    let _ = SingleParkingLock {
        data: Arc::new(parking_lot::Mutex::new(0)),
    };
}
