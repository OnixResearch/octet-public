fn fixed_to_usize(x: u32) -> usize {
    // Should warn: as usize from fixed-width type
    x as usize
}

fn usize_to_usize(x: usize) -> usize {
    // Should NOT warn: already platform-sized
    x
}

fn widening_to_u64(x: u32) -> u64 {
    // Should NOT warn: not casting to usize/isize
    x as u64
}

fn main() {
    fixed_to_usize(42);
    usize_to_usize(42);
    widening_to_u64(42);
}
