#![warn(multi_lock_ordering)]
#![allow(non_trait_imports, renamed_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

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

// Should warn: an unrelated memory-ordering comment inside the struct is not enough.
struct AtomicsAndLocks {
    // Memory ordering is handled by a separate atomic flag.
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should NOT warn: single Mutex
struct SingleLock {
    data: Mutex<u32>,
}

// Should NOT warn: the doc comment below documents acquisition order.

/// Lock order: always acquire `count` before `state`.
struct DocumentedTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should NOT warn: the plain comment directly above the attributes documents it.

// Lock ordering: acquire `count` before `state`.
#[allow(dead_code)]
struct CommentedTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should warn: a documented order on another item does not carry over.

// Lock ordering: acquire `count` before `state`.
const UNRELATED: u32 = 0;
struct DetachedCommentTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should NOT warn: an explicit doc attribute documents it.
#[doc = "Lock acquisition: `count` before `state`."]
struct DocAttributeTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should warn: locks shared behind `Arc` still count.
struct SharedTracker {
    count: std::sync::Arc<Mutex<u64>>,
    state: std::sync::Arc<std::sync::RwLock<u32>>,
}

// Should warn: a clock ordering comment does not document the mutexes.
struct ClockTracker {
    // Clock ordering follows the wall-clock source.
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should warn: `block`, `unlock`, `recorder`, and `border` only contain the words.
/// Unlock the block recorder at the border.
struct RecorderTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should NOT warn: the note above a parsed attribute such as `#[repr(C)]` documents it.
// Lock ordering: acquire `count` before `state`.
#[repr(C)]
struct ReprTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should NOT warn: the three comments below use `locking`, `locked`, and `ordered`.

// Locking order: `count` before `state`.
struct LockingOrderTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Fields are locked in order: `count`, then `state`.
struct LockedInOrderTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Both mutexes are locked as ordered here: `count`, then `state`.
struct LockedAsOrderedTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
}

// Should warn: `deadlock` is not `lock`, even next to `ordered`.

// Deadlock-free: `state` is ordered after `count`.
struct DeadlockNoteTracker {
    count: Mutex<u64>,
    state: Mutex<u32>,
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
