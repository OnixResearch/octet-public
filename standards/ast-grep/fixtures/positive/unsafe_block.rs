unsafe fn read_pointer(pointer: *const u8) -> u8 {
    unsafe { *pointer }
}
