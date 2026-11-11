struct Header {
    index: u32,
}

fn asserted_index(header: &Header, data: &[u8]) -> u8 {
    let idx: u32 = header.index;
    // Should NOT warn: the cast is the bounds check for the index below.
    assert!((idx as usize) < data.len());
    // Should NOT warn: bounded by the prior `assert!`.
    data[idx as usize]
}

fn if_guarded_index(idx: u32, data: &[u8]) -> Option<u8> {
    // Should NOT warn: both casts form a len-bounded index.
    if data.len() > idx as usize { Some(data[idx as usize]) } else { None }
}

fn short_circuit_guarded_index(header: &Header, data: &[u8]) -> bool {
    // Should NOT warn: `&&` evaluates the index only after the bounds check.
    (header.index as usize) < data.len() && data[header.index as usize] == 0
}

fn unguarded_index(idx: u32, data: &[u8]) -> u8 {
    // Should warn: nothing bounds the index.
    data[idx as usize]
}

fn guard_on_other_slice(idx: u32, data: &[u8], other: &[u8]) -> u8 {
    // Should warn: the check bounds `other`, not `data`.
    assert!((idx as usize) < other.len());
    // Should warn: the check bounds `other`, not `data`.
    data[idx as usize]
}

fn index_before_check(idx: u32, data: &[u8]) -> u8 {
    // Should warn: the index runs before the check.
    let value = data[idx as usize];
    // Should warn: no later index depends on this check.
    assert!((idx as usize) < data.len());
    value
}

fn main() {
    let header = Header { index: 0 };
    let data = [1, 2, 3];
    let _ = asserted_index(&header, &data);
    let _ = if_guarded_index(1, &data);
    let _ = short_circuit_guarded_index(&header, &data);
    let _ = unguarded_index(0, &data);
    let _ = guard_on_other_slice(0, &data, &data);
    let _ = index_before_check(0, &data);
}
