fn widen_byte(byte: u8) -> usize {
    byte as usize
}

fn widen_half(half: u16) -> usize {
    half as usize
}

fn widen_signed(delta: i16, byte: u8) -> (isize, isize) {
    (delta as isize, byte as isize)
}

fn main() {
    let _ = widen_byte(1);
    let _ = widen_half(2);
    let _ = widen_signed(-3, 4);
}
