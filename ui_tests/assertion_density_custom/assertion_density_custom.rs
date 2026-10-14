#![warn(assertion_density)]

fn under_custom_line_threshold() -> u32 {
    let s1 = 1;
    let s2 = 2;
    let s3 = 3;
    let s4 = 4;
    let s5 = 5;
    let s6 = 6;
    let s7 = 7;
    let s8 = 8;
    let s9 = 9;
    let s10 = 10;
    let s11 = 11;
    let s12 = 12;
    let s13 = 13;
    let s14 = 14;
    let s15 = 15;
    let s16 = 16;
    let s17 = 17;
    let s18 = 18;
    let s19 = 19;
    let s20 = 20;
    let s21 = 21;
    let s22 = 22;
    let s23 = 23;
    s1
}

fn insufficient_for_custom_assertion_count() -> u32 {
    let w1 = 1;
    let w2 = 2;
    let w3 = 3;
    let w4 = 4;
    let w5 = 5;
    let w6 = 6;
    let w7 = 7;
    let w8 = 8;
    let w9 = 9;
    let w10 = 10;
    let w11 = 11;
    let w12 = 12;
    let w13 = 13;
    let w14 = 14;
    assert!(w1 > 0);
    assert!(w2 > 0);
    let x15 = 15;
    let x16 = 16;
    let x17 = 17;
    let x18 = 18;
    let x19 = 19;
    let x20 = 20;
    let x21 = 21;
    let x22 = 22;
    let x23 = 23;
    let x24 = 24;
    let x25 = 25;
    let x26 = 26;
    let x27 = 27;
    let x28 = 28;
    let x29 = 29;
    let x30 = 30;
    x30
}

fn main() {
    under_custom_line_threshold();
    insufficient_for_custom_assertion_count();
}
