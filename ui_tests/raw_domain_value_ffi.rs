#![warn(raw_domain_value)]

#[repr(C)]
struct FfiRecord {
    // Should not warn: the C-compatible representation is a declared exception.
    token: u64,
}

fn main() {
    let record = FfiRecord { token: 1 };
    let _ = record.token;
}
