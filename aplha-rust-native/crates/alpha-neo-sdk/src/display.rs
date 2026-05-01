#[cfg(target_arch = "m68k")]
use core::arch::global_asm;
#[cfg(target_arch = "m68k")]
use core::sync::atomic::{Ordering, compiler_fence};

#[cfg(target_arch = "m68k")]
global_asm!(
    r#"
    .global alpha_neo_clear_screen
alpha_neo_clear_screen:
    .short 0xA000
    rts

    .global alpha_neo_set_text_row
alpha_neo_set_text_row:
    .short 0xA004
    rts

    .global alpha_neo_draw_char
alpha_neo_draw_char:
    .short 0xA010
    rts

    .global alpha_neo_flush_text
alpha_neo_flush_text:
    .short 0xA098
    rts

    .global alpha_neo_yield
alpha_neo_yield:
    .short 0xA25C
    rts

    .global alpha_neo_write_chars
alpha_neo_write_chars:
    movem.l %d2-%d6/%a2,-(%a7)
    move.l 28(%a7),%d4
    move.l 32(%a7),%d5
    move.l 36(%a7),%a2
    move.l 40(%a7),%d6
    move.l %d6,%d0
    beq 2f
1:
    suba.l #12,%a7
    move.l %d4,(%a7)
    move.l %d5,4(%a7)
    move.l #1,8(%a7)
    .short 0xA004
    adda.l #12,%a7
    move.b (%a2)+,%d0
    and.l #0xff,%d0
    move.l %d0,-(%a7)
    .short 0xA010
    adda.l #4,%a7
    add.l #1,%d5
    sub.l #1,%d6
    bne 1b
2:
    movem.l (%a7)+,%d2-%d6/%a2
    rts
    "#
);

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn alpha_neo_clear_screen();
    fn alpha_neo_set_text_row(row: u32, col: u32, width: u32);
    fn alpha_neo_draw_char(byte: u32);
    fn alpha_neo_flush_text();
    fn alpha_neo_yield();
    fn alpha_neo_write_chars(row: u32, start_col: u32, bytes: *const u8, len: u32);
}

#[allow(
    clippy::inline_always,
    reason = "required to avoid 68020 long-branch wrapper thunks in SmartApplet output"
)]
#[inline(always)]
pub fn clear() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS clear-screen trap with no borrowed Rust state crossing the ABI.
    unsafe {
        alpha_neo_clear_screen();
    };
    #[cfg(target_arch = "m68k")]
    compiler_fence(Ordering::SeqCst);
}

#[allow(
    clippy::inline_always,
    reason = "required to avoid GOT-backed byte literal arrays in applet code"
)]
#[inline(always)]
pub fn write_bytes<const N: usize>(row: u8, bytes: [u8; N]) {
    set_row(row);
    for byte in bytes {
        draw_char(byte);
    }
}

#[allow(
    clippy::inline_always,
    reason = "required to render stack-local applet buffers through direct trap calls"
)]
#[inline(always)]
pub fn write_slice(row: u8, bytes: &[u8]) {
    set_row(row);
    for byte in bytes {
        draw_char(*byte);
    }
}

#[allow(
    clippy::inline_always,
    reason = "required to render stack-local applet buffers through direct trap calls"
)]
#[inline(always)]
pub fn write_prefix<const N: usize>(row: u8, bytes: &[u8; N], len: usize) {
    if len == 0 {
        return;
    }
    let width = len.min(N);
    set_row_width(row, width);
    let mut index = 0;
    while index < width {
        draw_char(bytes[index]);
        index += 1;
    }
}

#[allow(
    clippy::inline_always,
    reason = "required to render prompt characters through direct trap calls"
)]
#[inline(always)]
pub fn write_chars<const N: usize>(row: u8, start_col: u8, bytes: &[u8; N], len: usize) {
    #[cfg(not(target_arch = "m68k"))]
    let _ = (row, start_col, bytes, len);
    #[cfg(target_arch = "m68k")]
    // SAFETY: The assembly helper reads exactly `len.min(N)` bytes from the supplied buffer.
    unsafe {
        alpha_neo_write_chars(
            u32::from(row),
            u32::from(start_col),
            bytes.as_ptr(),
            len.min(N) as u32,
        );
    };
    #[cfg(target_arch = "m68k")]
    compiler_fence(Ordering::SeqCst);
}

#[allow(
    clippy::inline_always,
    reason = "required to render applet UI through direct trap calls"
)]
#[inline(always)]
pub fn clear_row(row: u8) {
    set_row(row);
    let mut count = 0;
    while count < 28 {
        draw_char(b' ');
        count += 1;
    }
}

#[allow(
    clippy::inline_always,
    reason = "required to keep the focus idle loop inside relocatable applet code"
)]
#[inline(always)]
pub fn idle_forever() -> ! {
    flush();
    loop {
        yield_once();
    }
}

#[allow(
    clippy::inline_always,
    reason = "required to keep interactive applet loops inside relocatable applet code"
)]
#[inline(always)]
pub fn yield_once() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Yields cooperatively to the NEO OS; it does not return ownership of any data.
    unsafe {
        alpha_neo_yield();
    };
    #[cfg(target_arch = "m68k")]
    compiler_fence(Ordering::SeqCst);
}

#[allow(
    clippy::inline_always,
    reason = "required so row setup uses the caller's stack frame directly"
)]
#[inline(always)]
fn set_row(row: u8) {
    #[cfg(not(target_arch = "m68k"))]
    let _ = row;
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS row-selection trap with a scalar row argument.
    unsafe {
        alpha_neo_set_text_row(u32::from(row), 1, 28);
    };
    #[cfg(target_arch = "m68k")]
    compiler_fence(Ordering::SeqCst);
}

#[allow(
    clippy::inline_always,
    reason = "required so textbox sessions can resume at a precise cursor position"
)]
#[inline(always)]
pub fn set_cursor(row: u8, col: u8, width: usize) {
    #[cfg(not(target_arch = "m68k"))]
    let _ = (row, col, width);
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS row-selection trap with scalar cursor arguments.
    unsafe {
        alpha_neo_set_text_row(u32::from(row), u32::from(col), width.min(28) as u32);
    };
    #[cfg(target_arch = "m68k")]
    compiler_fence(Ordering::SeqCst);
}

#[allow(
    clippy::inline_always,
    reason = "required so row setup uses the caller's stack frame directly"
)]
#[inline(always)]
fn set_row_width(row: u8, width: usize) {
    #[cfg(not(target_arch = "m68k"))]
    let _ = (row, width);
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS row-selection trap with scalar row and width arguments.
    unsafe {
        alpha_neo_set_text_row(u32::from(row), 1, width as u32);
    };
    #[cfg(target_arch = "m68k")]
    compiler_fence(Ordering::SeqCst);
}

#[allow(
    clippy::inline_always,
    reason = "required so byte drawing uses immediate values instead of a data table"
)]
#[inline(always)]
fn draw_char(byte: u8) {
    #[cfg(not(target_arch = "m68k"))]
    let _ = byte;
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS character drawing trap with a scalar byte argument.
    unsafe {
        alpha_neo_draw_char(u32::from(byte));
    };
    #[cfg(target_arch = "m68k")]
    compiler_fence(Ordering::SeqCst);
}


#[allow(
    clippy::inline_always,
    reason = "required to keep the focus idle loop free of extra applet calls"
)]
#[inline(always)]
pub fn flush() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS flush trap with no borrowed Rust state crossing the ABI.
    unsafe {
        alpha_neo_flush_text();
    };
    #[cfg(target_arch = "m68k")]
    compiler_fence(Ordering::SeqCst);
}
