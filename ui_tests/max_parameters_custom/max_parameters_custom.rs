fn six_parameters(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) {
    let _ = (a, b, c, d, e, f);
}

fn eight_parameters(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8) {
    let _ = (a, b, c, d, e, f, g, h);
}

fn main() {
    six_parameters(1, 2, 3, 4, 5, 6);
    eight_parameters(1, 2, 3, 4, 5, 6, 7, 8);
}
