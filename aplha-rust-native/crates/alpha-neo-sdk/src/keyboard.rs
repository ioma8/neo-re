#[cfg(target_arch = "m68k")]
use core::arch::global_asm;
#[cfg(target_arch = "m68k")]
use core::sync::atomic::{Ordering, compiler_fence};

#[cfg(target_arch = "m68k")]
global_asm!(
    r#"
    .global alpha_neo_read_key_code
alpha_neo_read_key_code:
    .short 0xA094
    rts

    .global alpha_neo_is_key_ready
alpha_neo_is_key_ready:
    .short 0xA09C
    rts

    .global alpha_neo_pump_ui_events
alpha_neo_pump_ui_events:
    .short 0xA0A4
    rts
    "#
);

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn alpha_neo_read_key_code() -> u32;
    fn alpha_neo_is_key_ready() -> u32;
    fn alpha_neo_pump_ui_events();
}

#[must_use]
#[allow(
    clippy::inline_always,
    reason = "required to avoid 68020 long-branch wrapper thunks in SmartApplet output"
)]
#[inline(always)]
pub fn is_ready() -> bool {
    #[cfg(not(target_arch = "m68k"))]
    {
        false
    }
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS key-ready trap and reads its scalar return value.
    unsafe {
        let value = alpha_neo_is_key_ready();
        compiler_fence(Ordering::SeqCst);
        value != 0
    }
}

#[must_use]
#[allow(
    clippy::inline_always,
    reason = "required to avoid 68020 long-branch wrapper thunks in SmartApplet output"
)]
#[inline(always)]
pub fn read_key() -> u32 {
    #[cfg(not(target_arch = "m68k"))]
    {
        0
    }
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS read-key trap and reads its scalar return value.
    unsafe {
        let value = alpha_neo_read_key_code();
        compiler_fence(Ordering::SeqCst);
        value
    }
}

#[must_use]
pub fn logical_key_to_byte(raw: u32) -> Option<u8> {
    match raw & 0xff {
        0x00 => Some(0x5d),
        0x02 => Some(0x5b),
        0x03 => Some(0x08),
        0x07 => Some(0x74),
        0x09 => Some(0x79),
        0x0b => Some(0x27),
        0x10 => Some(0x67),
        0x11 => Some(0x68),
        0x12 => Some(0x6b),
        0x13 => Some(0x6c),
        0x14 => Some(0x3b),
        0x15 => Some(0x5c),
        0x16 => Some(0x64),
        0x17 => Some(0x73),
        0x18 => Some(0x61),
        0x19 => Some(0x66),
        0x1b => Some(0x6a),
        0x1c => Some(0x69),
        0x1d => Some(0x6f),
        0x1e => Some(0x70),
        0x20 => Some(0x65),
        0x21 => Some(0x77),
        0x22 => Some(0x71),
        0x23 => Some(0x72),
        0x24 => Some(0x75),
        0x25 => Some(0x3d),
        0x28 => Some(0x2d),
        0x2e => Some(0x60),
        0x2f => Some(0x35),
        0x30 => Some(0x36),
        0x31 => Some(0x38),
        0x32 => Some(0x39),
        0x33 => Some(0x30),
        0x36 => Some(0x33),
        0x37 => Some(0x32),
        0x38 => Some(0x31),
        0x39 => Some(0x34),
        0x3a => Some(0x37),
        0x3b => Some(0x2c),
        0x3d => Some(0x2e),
        0x40 => Some(0x0d),
        0x41 => Some(0x63),
        0x42 => Some(0x78),
        0x43 => Some(0x7a),
        0x44 => Some(0x76),
        0x46 => Some(0x6d),
        0x47 => Some(0x2f),
        0x4c => Some(0x20),
        0x4e => Some(0x62),
        0x4f => Some(0x6e),
        _ => None,
    }
}

#[allow(
    clippy::inline_always,
    reason = "required to avoid 68020 long-branch wrapper thunks in SmartApplet output"
)]
#[inline(always)]
pub fn pump_events() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS event pump trap with no borrowed Rust state crossing the ABI.
    unsafe {
        alpha_neo_pump_ui_events();
        compiler_fence(Ordering::SeqCst);
    };
}

#[cfg(test)]
mod tests {
    use super::logical_key_to_byte;

    #[test]
    fn decodes_logical_key_codes_from_read_key_trap() {
        let expected = [
            (0x38, b'1'),
            (0x37, b'2'),
            (0x36, b'3'),
            (0x39, b'4'),
            (0x2f, b'5'),
            (0x30, b'6'),
            (0x22, b'q'),
            (0x21, b'w'),
            (0x20, b'e'),
            (0x23, b'r'),
            (0x40, b'\r'),
            (0x03, 0x08),
        ];
        for (logical, byte) in expected {
            assert_eq!(logical_key_to_byte(logical), Some(byte));
        }
    }
}
