// `[octet]` wins; the shadowed legacy sections are reported, not parsed.
fn six(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> u8 {
    a ^ b ^ c ^ d ^ e ^ f
}

fn main() {
    let _ = six(1, 2, 3, 4, 5, 6);
}
