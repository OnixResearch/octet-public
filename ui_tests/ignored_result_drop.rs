use std::io::Write;

struct Cleaner {
    buf: Vec<u8>,
}

impl Drop for Cleaner {
    fn drop(&mut self) {
        let _ = self.buf.write_all(b"bye");
    }
}

struct ManualCleaner {
    buf: Vec<u8>,
}

impl ManualCleaner {
    fn drop(&mut self) {
        let _ = self.buf.write_all(b"bye");
    }
}

fn main() {
    let _cleaner = Cleaner { buf: Vec::new() };
    let mut manual = ManualCleaner { buf: Vec::new() };
    manual.drop();
}
