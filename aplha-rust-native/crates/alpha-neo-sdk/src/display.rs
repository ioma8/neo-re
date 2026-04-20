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
    fn alpha_neo_set_text_row(row: u32);
    fn alpha_neo_draw_char(byte: u32);
    fn alpha_neo_flush_text();
    fn alpha_neo_yield();
}

pub fn clear() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS clear-screen trap with no borrowed Rust state crossing the ABI.
    unsafe {
        alpha_neo_clear_screen();
    };
}

pub fn write_lines(start_row: u8, lines: &[&str]) {
    for (index, line) in lines.iter().enumerate() {
        let Ok(row_offset) = u8::try_from(index) else {
            return;
        };
        let Some(row) = start_row.checked_add(row_offset) else {
            return;
        };
        set_row(row);
        for byte in line.bytes() {
            draw_char(byte);
        }
    }
}

pub fn idle_forever() -> ! {
    flush();
    loop {
        #[cfg(not(target_arch = "m68k"))]
        {}
        #[cfg(target_arch = "m68k")]
        // SAFETY: Yields cooperatively to the NEO OS; it does not return ownership of any data.
        unsafe {
            alpha_neo_yield();
        };
    }
}

fn set_row(row: u8) {
    #[cfg(not(target_arch = "m68k"))]
    let _ = row;
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS row-selection trap with a scalar row argument.
    unsafe {
        alpha_neo_set_text_row(u32::from(row));
    };
}

fn draw_char(byte: u8) {
    #[cfg(not(target_arch = "m68k"))]
    let _ = byte;
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS character drawing trap with a scalar byte argument.
    unsafe {
        alpha_neo_draw_char(u32::from(byte));
    };
}

fn flush() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS flush trap with no borrowed Rust state crossing the ABI.
    unsafe {
        alpha_neo_flush_text();
    };
}
