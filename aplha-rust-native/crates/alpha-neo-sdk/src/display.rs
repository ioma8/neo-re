#[cfg(target_arch = "m68k")]
use core::arch::global_asm;

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
    "#
);

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn alpha_neo_clear_screen();
    fn alpha_neo_set_text_row(row: u32, col: u32, width: u32);
    fn alpha_neo_draw_char(byte: u32);
    fn alpha_neo_flush_text();
    fn alpha_neo_yield();
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
}
