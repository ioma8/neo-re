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

    .global alpha_neo_translate_key_to_char
alpha_neo_translate_key_to_char:
    .short 0xA164
    rts

    .global alpha_neo_wait_for_key
alpha_neo_wait_for_key:
    .short 0xA088
    rts

    .global alpha_neo_text_box
alpha_neo_text_box:
    .short 0xA084
    rts
    "#
);

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn alpha_neo_read_key_code() -> u32;
    fn alpha_neo_is_key_ready() -> u32;
    fn alpha_neo_pump_ui_events();
    fn alpha_neo_translate_key_to_char(key: u32) -> u32;
    fn alpha_neo_wait_for_key() -> u32;
    fn alpha_neo_text_box(
        buffer: *mut u8,
        len: *mut u8,
        max_len: u32,
        exit_keys: *const u8,
        password: u32,
    ) -> u32;
}

const KEY_MOD_CAPS_LOCK: u32 = 0x0200;
pub const KEY_ESC: u8 = 0x48;
pub const KEY_LEFT: u8 = 0x49;
pub const KEY_RIGHT: u8 = 0x4a;
pub const KEY_UP: u8 = 0x4b;
pub const KEY_DOWN: u8 = 0x0d;
pub const KEY_ENTER: u8 = 0x40;
pub const TEXTBOX_EXIT_KEY_COUNT: usize = 14;
pub const MULTILINE_TEXTBOX_EXIT_KEY_COUNT: usize = 15;

#[must_use]
pub const fn textbox_exit_keys() -> [u8; TEXTBOX_EXIT_KEY_COUNT] {
    [
        KEY_LEFT, KEY_RIGHT, KEY_UP, KEY_DOWN, 0x2d, 0x2c, 0x04, 0x0f, 0x0e, 0x0a, 0x01, 0x27,
        KEY_ESC, 0xff,
    ]
}

#[must_use]
pub const fn multiline_textbox_exit_keys() -> [u8; MULTILINE_TEXTBOX_EXIT_KEY_COUNT] {
    [
        KEY_LEFT, KEY_RIGHT, KEY_UP, KEY_DOWN, KEY_ENTER, 0x2d, 0x2c, 0x04, 0x0f, 0x0e, 0x0a,
        0x01, 0x27, KEY_ESC, 0xff,
    ]
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

#[must_use]
pub fn matrix_code_to_logical(raw: u8) -> Option<u8> {
    match raw {
        0x00 => Some(0x00),
        0x10 => Some(0x0a),
        0x20 => Some(0x12),
        0x30 => Some(0x1c),
        0x40 => Some(0x25),
        0x50 => Some(0x31),
        0x60 => Some(0x3b),
        0x41 => Some(0x26),
        0x61 => Some(0x50),
        0x02 => Some(0x01),
        0x22 => Some(0x13),
        0x32 => Some(0x1d),
        0x42 => Some(0x27),
        0x52 => Some(0x32),
        0x62 => Some(0x3d),
        0x03 => Some(0x02),
        0x13 => Some(0x0b),
        0x23 => Some(0x14),
        0x33 => Some(0x1e),
        0x43 => Some(0x28),
        0x53 => Some(0x33),
        0x73 => Some(0x47),
        0x14 => Some(0x0c),
        0x34 => Some(0x1f),
        0x54 => Some(0x34),
        0x74 => Some(0x48),
        0x15 => Some(0x0d),
        0x65 => Some(0x3e),
        0x75 => Some(0x49),
        0x46 => Some(0x29),
        0x76 => Some(0x4a),
        0x47 => Some(0x2a),
        0x67 => Some(0x3f),
        0x77 => Some(0x4b),
        0x09 => Some(0x03),
        0x19 => Some(0x0e),
        0x29 => Some(0x15),
        0x49 => Some(0x2b),
        0x59 => Some(0x35),
        0x69 => Some(0x40),
        0x79 => Some(0x4c),
        0x0a => Some(0x04),
        0x1a => Some(0x0f),
        0x2a => Some(0x16),
        0x3a => Some(0x20),
        0x4a => Some(0x2c),
        0x5a => Some(0x36),
        0x6a => Some(0x41),
        0x0b => Some(0x05),
        0x2b => Some(0x17),
        0x3b => Some(0x21),
        0x4b => Some(0x2d),
        0x5b => Some(0x37),
        0x6b => Some(0x42),
        0x0c => Some(0x06),
        0x2c => Some(0x18),
        0x3c => Some(0x22),
        0x4c => Some(0x2e),
        0x5c => Some(0x38),
        0x6c => Some(0x43),
        0x7c => Some(0x4d),
        0x0d => Some(0x07),
        0x1d => Some(0x10),
        0x2d => Some(0x19),
        0x3d => Some(0x23),
        0x4d => Some(0x2f),
        0x5d => Some(0x39),
        0x6d => Some(0x44),
        0x7d => Some(0x4e),
        0x0e => Some(0x08),
        0x2e => Some(0x1a),
        0x6e => Some(0x45),
        0x0f => Some(0x09),
        0x1f => Some(0x11),
        0x2f => Some(0x1b),
        0x3f => Some(0x24),
        0x4f => Some(0x30),
        0x5f => Some(0x3a),
        0x6f => Some(0x46),
        0x7f => Some(0x4f),
        _ => None,
    }
}

#[must_use]
pub fn matrix_code_to_byte(raw: u8) -> Option<u8> {
    matrix_code_to_logical(raw).and_then(|logical| logical_key_to_byte(u32::from(logical)))
}

#[must_use]
pub fn translate_key_to_byte(raw: u32) -> Option<u8> {
    #[cfg(not(target_arch = "m68k"))]
    {
        let _ = raw;
        None
    }
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS key-to-char trap and reads its scalar return value.
    unsafe {
        let value = alpha_neo_translate_key_to_char(raw);
        compiler_fence(Ordering::SeqCst);
        let byte = value as u8;
        (byte != 0).then_some(byte)
    }
}

#[must_use]
pub const fn normalize_textbox_exit(raw: u32) -> u8 {
    (raw & !KEY_MOD_CAPS_LOCK) as u8
}

#[must_use]
pub const fn file_slot_for_exit_key(raw: u8) -> Option<usize> {
    match raw {
        0x2d => Some(1),
        0x2c => Some(2),
        0x04 => Some(3),
        0x0f => Some(4),
        0x0e => Some(5),
        0x0a => Some(6),
        0x01 => Some(7),
        0x27 => Some(8),
        _ => None,
    }
}

#[must_use]
pub fn wait_for_key() -> u32 {
    #[cfg(not(target_arch = "m68k"))]
    {
        0
    }
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS blocking wait-for-key trap and reads its scalar return value.
    unsafe {
        let value = alpha_neo_wait_for_key();
        compiler_fence(Ordering::SeqCst);
        value
    }
}

#[must_use]
pub fn text_box(
    buffer: &mut [u8],
    len: &mut u8,
    max_len: u16,
    exit_keys: &[u8],
    password: bool,
) -> u32 {
    #[cfg(not(target_arch = "m68k"))]
    {
        let _ = (buffer, len, max_len, exit_keys, password);
        0
    }
    #[cfg(target_arch = "m68k")]
    // SAFETY: Passes valid pointers to the NEO OS textbox trap for the duration of the call.
    unsafe {
        let value = alpha_neo_text_box(
            buffer.as_mut_ptr(),
            len as *mut u8,
            u32::from(max_len),
            exit_keys.as_ptr(),
            u32::from(password),
        );
        compiler_fence(Ordering::SeqCst);
        value
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
    use super::{
        file_slot_for_exit_key, logical_key_to_byte, matrix_code_to_byte,
        matrix_code_to_logical, multiline_textbox_exit_keys, normalize_textbox_exit,
    };

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

    #[test]
    fn decodes_matrix_codes_to_logical_values() {
        assert_eq!(matrix_code_to_logical(0x2c), Some(0x18));
        assert_eq!(matrix_code_to_logical(0x75), Some(0x49));
        assert_eq!(matrix_code_to_logical(0x76), Some(0x4a));
        assert_eq!(matrix_code_to_logical(0x15), Some(0x0d));
    }

    #[test]
    fn decodes_matrix_codes_to_bytes() {
        assert_eq!(matrix_code_to_byte(0x2c), Some(b'a'));
        assert_eq!(matrix_code_to_byte(0x3a), Some(b'e'));
        assert_eq!(matrix_code_to_byte(0x69), Some(b'\r'));
        assert_eq!(matrix_code_to_byte(0x09), Some(0x08));
    }

    #[test]
    fn normalizes_textbox_exit_by_stripping_modifier_bits() {
        assert_eq!(normalize_textbox_exit(0x0249), 0x49);
        assert_eq!(normalize_textbox_exit(0x020d), 0x0d);
        assert_eq!(normalize_textbox_exit(0x0048), 0x48);
    }

    #[test]
    fn maps_textbox_file_exit_keys_to_slots() {
        assert_eq!(file_slot_for_exit_key(0x2d), Some(1));
        assert_eq!(file_slot_for_exit_key(0x2c), Some(2));
        assert_eq!(file_slot_for_exit_key(0x04), Some(3));
        assert_eq!(file_slot_for_exit_key(0x0f), Some(4));
        assert_eq!(file_slot_for_exit_key(0x0e), Some(5));
        assert_eq!(file_slot_for_exit_key(0x0a), Some(6));
        assert_eq!(file_slot_for_exit_key(0x01), Some(7));
        assert_eq!(file_slot_for_exit_key(0x27), Some(8));
        assert_eq!(file_slot_for_exit_key(0x48), None);
    }

    #[test]
    fn textbox_exit_key_list_matches_validated_alpha_word_style_flow() {
        assert_eq!(
            super::textbox_exit_keys(),
            [0x49, 0x4a, 0x4b, 0x0d, 0x2d, 0x2c, 0x04, 0x0f, 0x0e, 0x0a, 0x01, 0x27, 0x48, 0xff]
        );
    }

    #[test]
    fn multiline_textbox_exit_key_list_includes_enter() {
        assert_eq!(
            multiline_textbox_exit_keys(),
            [0x49, 0x4a, 0x4b, 0x0d, 0x40, 0x2d, 0x2c, 0x04, 0x0f, 0x0e, 0x0a, 0x01, 0x27, 0x48, 0xff]
        );
    }
}
