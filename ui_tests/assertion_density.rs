#![warn(assertion_density)]

fn sparse_function(data: &[u8]) -> usize {
    // Should warn: 20+ lines with 0 assertions
    let a = data.len();
    let b = a + 1;
    let c = b + 2;
    let d = c + 3;
    let e = d + 4;
    let f = e + 5;
    let g = f + 6;
    let h = g + 7;
    let i = h + 8;
    let j = i + 9;
    let k = j + 10;
    let l = k + 11;
    let m = l + 12;
    let n = m + 13;
    let o = n + 14;
    let p = o + 15;
    let q = p + 16;
    let r = q + 17;
    let s = r + 18;
    let t = s + 19;
    t
}

fn dense_function(data: &[u8]) -> usize {
    // Should NOT warn: has assertions
    let a = data.len();
    assert!(a > 0);
    let b = a + 1;
    let c = b + 2;
    let d = c + 3;
    let e = d + 4;
    let f = e + 5;
    let g = f + 6;
    let h = g + 7;
    let i = h + 8;
    let j = i + 9;
    let k = j + 10;
    let l = k + 11;
    let m = l + 12;
    let n = m + 13;
    let o = n + 14;
    let p = o + 15;
    let q = p + 16;
    let r = q + 17;
    let s = r + 18;
    assert!(s > 0);
    let t = s + 19;
    t
}

fn short_function(x: u32) -> u32 {
    // Should NOT warn: under line threshold
    x + 1
}

fn main() {
    sparse_function(&[1]);
    dense_function(&[1]);
    short_function(1);
}
