/// 68000-safe replacement for compiler-builtins' public `memcpy` wrapper.
///
/// # Safety
///
/// `dest` and `src` must be valid for `count` bytes and must not overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    let mut index = 0;
    while index < count {
        // SAFETY: The caller provides valid non-overlapping buffers for `count` bytes.
        unsafe {
            dest.add(index).write(src.add(index).read());
        }
        index += 1;
    }
    dest
}

/// 68000-safe replacement for compiler-builtins' public `memset` wrapper.
///
/// # Safety
///
/// `dest` must be valid for `count` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, value: i32, count: usize) -> *mut u8 {
    let mut index = 0;
    while index < count {
        // SAFETY: The caller provides a valid output buffer for `count` bytes.
        unsafe {
            dest.add(index).write(value as u8);
        }
        index += 1;
    }
    dest
}

/// 68000-safe replacement for compiler-builtins' public `memmove` wrapper.
///
/// # Safety
///
/// `dest` and `src` must be valid for `count` bytes. Regions may overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    if (dest as usize) <= (src as usize) {
        let mut index = 0;
        while index < count {
            // SAFETY: The caller provides valid buffers for `count` bytes.
            unsafe {
                dest.add(index).write(src.add(index).read());
            }
            index += 1;
        }
    } else {
        let mut index = count;
        while index > 0 {
            index -= 1;
            // SAFETY: The caller provides valid buffers for `count` bytes.
            unsafe {
                dest.add(index).write(src.add(index).read());
            }
        }
    }
    dest
}
