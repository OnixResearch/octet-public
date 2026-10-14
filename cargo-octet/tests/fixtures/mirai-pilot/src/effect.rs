use std::path::Path;

pub fn read_selected(path: &Path) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}
