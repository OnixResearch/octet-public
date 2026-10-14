#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(dead_code)]
#![allow(tigerstyle::no_panic)]
#![allow(tigerstyle::unchecked_narrowing)]
#![allow(tigerstyle::no_unwrap)]

pub unsafe fn read_public_raw(ptr: *const u8) -> u8 {
    unsafe { *ptr }
}

pub struct Buffer {
    ptr: *mut u8,
}

impl Buffer {
    pub unsafe fn from_public_raw(ptr: *mut u8) -> Self {
        Self { ptr }
    }

    pub fn from_checked(ptr: *mut u8) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { private_from_raw(ptr) })
    }
}

unsafe fn private_from_raw(ptr: *mut u8) -> Buffer {
    Buffer { ptr }
}

pub unsafe extern "C" fn exported_ffi(ptr: *const u8) -> u8 {
    unsafe { *ptr }
}

trait UnsafeFactory {
    unsafe fn from_trait_raw(ptr: *mut u8) -> Self;
}

impl UnsafeFactory for Buffer {
    unsafe fn from_trait_raw(ptr: *mut u8) -> Self {
        Self { ptr }
    }
}

#[cfg(test)]
pub unsafe fn public_unsafe_test_helper(ptr: *const u8) -> u8 {
    unsafe { *ptr }
}

fn main() {}
