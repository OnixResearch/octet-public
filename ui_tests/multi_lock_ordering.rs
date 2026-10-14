#![warn(multi_lock_ordering)]

use std::sync::Mutex;
use std::sync::Mutex as Guard;

// Should warn: 2 Mutex fields without ordering comment
struct Tracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should NOT warn: has lock ordering comment
struct OrderedTracker {
    // Lock ordering: always acquire `count` before `state`
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should warn: alias still resolves to a lock type.
struct AliasedTracker {
    count: Guard<u64>,
    state: Guard<u32>,
}

// Should warn: unrelated ordering comments do not document lock acquisition order.
struct AtomicsAndLocks {
    // Memory ordering is handled by a separate atomic flag.
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should NOT warn: single Mutex
struct SingleLock {
    data: Mutex<u32>,
}

fn main() {
    let _ = Tracker {
        count: Mutex::new(0),
        state: Mutex::new(0),
    };
    let _ = OrderedTracker {
        count: Mutex::new(0),
        state: Mutex::new(0),
    };
    let _ = AliasedTracker {
        count: Guard::new(0),
        state: Guard::new(0),
    };
    let _ = AtomicsAndLocks {
        count: Mutex::new(0),
        state: Mutex::new(0),
    };
    let _ = SingleLock {
        data: Mutex::new(0),
    };
}
