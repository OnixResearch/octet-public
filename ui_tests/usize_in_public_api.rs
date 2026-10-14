struct Buffer([u8; 4]);

pub fn get_item(index: usize) -> Option<u8> {
    Some(index as u8)
}

pub fn platform_len() -> usize {
    4
}

fn private_helper(index: usize) -> usize {
    index
}

impl Buffer {
    pub fn slot(&self, idx: usize) -> usize {
        idx
    }
}

impl std::ops::Index<usize> for Buffer {
    type Output = u8;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

fn main() {
    let buf = Buffer([1, 2, 3, 4]);
    let _ = get_item(1);
    let _ = platform_len();
    let _ = private_helper(1);
    let _ = buf.slot(1);
    let _ = buf[1];
}
