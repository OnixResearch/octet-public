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

fn single_debug_assert(data: &[u8]) -> usize {
    // Should warn: one `debug_assert!` is one assertion, not two
    let a = data.len();
    debug_assert!(a > 0);
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

struct Parser;

impl Parser {
    // Should warn: inherent methods are checked like free functions
    fn sparse_method(&self, data: &[u8]) -> usize {
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
}

trait SparseDefault {
    // Should NOT warn: required trait methods have no body
    fn required_method(&self, data: &[u8]) -> usize;

    // Should warn: provided trait methods have a body
    fn sparse_default_method(&self, data: &[u8]) -> usize {
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
}

fn comment_marker_in_string(data: &[u8]) -> usize {
    // Should NOT warn: `/*` inside a string literal does not start a comment
    let marker = "/*";
    let a = data.len() + marker.len();
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
    assert!(r > 0);
    r
}

#[cfg(test)]
fn test_only_sparse(data: &[u8]) -> usize {
    // Should NOT warn: `cfg(test)` code is test-only (the fixture runs with `--test`, which sets `cfg(test)`)
    let a = data.len();
    let a = a + 1;
    let a = a + 2;
    let a = a + 3;
    let a = a + 4;
    let a = a + 5;
    let a = a + 6;
    let a = a + 7;
    let a = a + 8;
    let a = a + 9;
    let a = a + 10;
    let a = a + 11;
    let a = a + 12;
    let a = a + 13;
    let a = a + 14;
    let a = a + 15;
    let a = a + 16;
    let a = a + 17;
    let a = a + 18;
    let a = a + 19;
    a
}

#[cfg(all(test, unix))]
fn test_and_unix_sparse(data: &[u8]) -> usize {
    // Should NOT warn: `all(test, unix)` requires `test`
    let a = data.len();
    let a = a + 1;
    let a = a + 2;
    let a = a + 3;
    let a = a + 4;
    let a = a + 5;
    let a = a + 6;
    let a = a + 7;
    let a = a + 8;
    let a = a + 9;
    let a = a + 10;
    let a = a + 11;
    let a = a + 12;
    let a = a + 13;
    let a = a + 14;
    let a = a + 15;
    let a = a + 16;
    let a = a + 17;
    let a = a + 18;
    let a = a + 19;
    a
}

#[cfg(any(test, unix))]
fn test_or_unix_sparse(data: &[u8]) -> usize {
    // Should warn: `any(test, unix)` also holds in production builds
    let a = data.len();
    let a = a + 1;
    let a = a + 2;
    let a = a + 3;
    let a = a + 4;
    let a = a + 5;
    let a = a + 6;
    let a = a + 7;
    let a = a + 8;
    let a = a + 9;
    let a = a + 10;
    let a = a + 11;
    let a = a + 12;
    let a = a + 13;
    let a = a + 14;
    let a = a + 15;
    let a = a + 16;
    let a = a + 17;
    let a = a + 18;
    let a = a + 19;
    a
}

#[test]
fn harness_test_sparse() {
    // Should NOT warn: `#[test]` functions are test-only
    let a = 0_usize;
    let a = a + 1;
    let a = a + 2;
    let a = a + 3;
    let a = a + 4;
    let a = a + 5;
    let a = a + 6;
    let a = a + 7;
    let a = a + 8;
    let a = a + 9;
    let a = a + 10;
    let a = a + 11;
    let a = a + 12;
    let a = a + 13;
    let a = a + 14;
    let a = a + 15;
    let a = a + 16;
    let a = a + 17;
    let a = a + 18;
    let a = a + 19;
    let _ = a;
}

fn helper_beside_test_sparse() -> usize {
    // Should warn: helpers next to `#[test]` functions are still checked
    let a = 0_usize;
    let a = a + 1;
    let a = a + 2;
    let a = a + 3;
    let a = a + 4;
    let a = a + 5;
    let a = a + 6;
    let a = a + 7;
    let a = a + 8;
    let a = a + 9;
    let a = a + 10;
    let a = a + 11;
    let a = a + 12;
    let a = a + 13;
    let a = a + 14;
    let a = a + 15;
    let a = a + 16;
    let a = a + 17;
    let a = a + 18;
    let a = a + 19;
    a
}
