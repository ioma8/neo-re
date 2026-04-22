#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatrixKey(u8);

impl MatrixKey {
    pub(crate) const fn new(code: u8) -> Self {
        Self(code)
    }

    pub const fn code(self) -> u8 {
        self.0
    }

    pub(crate) const fn row(self) -> u8 {
        self.0 & 0x0f
    }

    const fn column_mask(self) -> u8 {
        1 << (self.0 >> 4)
    }

    fn is_visible_on_rows(self, rows: Option<u16>) -> bool {
        rows.is_none_or(|rows| rows & (1 << self.row()) != 0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatrixCell {
    pub raw: MatrixKey,
    pub row: u8,
    pub col: u8,
    pub logical: u8,
}

const MATRIX_EMPTY: u8 = 0xff;

// Logical codes as found in `os3kneorom.os3kos` at firmware base 0x00400000,
// table `0x0044c37b`. low nibble is row; bit index is column.
const MATRIX_LOGICAL: [[u8; 8]; 16] = [
    [0x00, 0x0a, 0x12, 0x1c, 0x25, 0x31, 0x3b, MATRIX_EMPTY],
    [
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        0x26,
        MATRIX_EMPTY,
        0x50,
        MATRIX_EMPTY,
    ],
    [
        0x01,
        MATRIX_EMPTY,
        0x13,
        0x1d,
        0x27,
        0x32,
        0x3d,
        MATRIX_EMPTY,
    ],
    [0x02, 0x0b, 0x14, 0x1e, 0x28, 0x33, MATRIX_EMPTY, 0x47],
    [
        MATRIX_EMPTY,
        0x0c,
        MATRIX_EMPTY,
        0x1f,
        MATRIX_EMPTY,
        0x34,
        MATRIX_EMPTY,
        0x48,
    ],
    [
        MATRIX_EMPTY,
        0x0d,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        0x3e,
        0x49,
    ],
    [
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        0x29,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        0x4a,
    ],
    [
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        0x2a,
        MATRIX_EMPTY,
        0x3f,
        0x4b,
    ],
    [
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
    ],
    [0x03, 0x0e, 0x15, MATRIX_EMPTY, 0x2b, 0x35, 0x40, 0x4c],
    [0x04, 0x0f, 0x16, 0x20, 0x2c, 0x36, 0x41, MATRIX_EMPTY],
    [
        0x05,
        MATRIX_EMPTY,
        0x17,
        0x21,
        0x2d,
        0x37,
        0x42,
        MATRIX_EMPTY,
    ],
    [0x06, MATRIX_EMPTY, 0x18, 0x22, 0x2e, 0x38, 0x43, 0x4d],
    [0x07, 0x10, 0x19, 0x23, 0x2f, 0x39, 0x44, 0x4e],
    [
        0x08,
        MATRIX_EMPTY,
        0x1a,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        MATRIX_EMPTY,
        0x45,
        MATRIX_EMPTY,
    ],
    [0x09, 0x11, 0x1b, 0x24, 0x30, 0x3a, 0x46, 0x4f],
];

// Logical-key to USB HID usage table found in the full OS firmware at
// `os3kneorom.os3kos` file offset 0x3c32a / mapped address 0x0043c32a.
// It is indexed by the logical values returned from MATRIX_LOGICAL.
const LOGICAL_HID_USAGE: [u8; 0x51] = [
    0x30, 0x40, 0x2f, 0x2a, 0x3c, 0x39, 0x2b, 0x17, //
    0xe1, 0x1c, 0x3f, 0x34, 0xe3, 0x51, 0x3e, 0x3d, //
    0x0a, 0x0b, 0x0e, 0x0f, 0x33, 0x31, 0x07, 0x16, //
    0x04, 0x09, 0xff, 0x0d, 0x0c, 0x12, 0x13, 0x4a, //
    0x08, 0x1a, 0x14, 0x15, 0x18, 0x2e, 0xe2, 0x41, //
    0x2d, 0x58, 0xff, 0x42, 0x3b, 0x3a, 0x35, 0x22, //
    0x23, 0x25, 0x26, 0x27, 0x45, 0x43, 0x20, 0x1f, //
    0x1e, 0x21, 0x24, 0x36, 0xe6, 0x37, 0x4d, 0x44, //
    0x28, 0x06, 0x1b, 0x1d, 0x19, 0xe5, 0x10, 0x38, //
    0x29, 0x50, 0x4f, 0x52, 0x2c, 0xe0, 0x05, 0x11, //
    0x4c,
];

pub fn matrix_cells() -> Vec<MatrixCell> {
    let mut cells = Vec::new();
    for (row, row_data) in MATRIX_LOGICAL.iter().enumerate() {
        for (col, logical) in row_data.iter().enumerate() {
            if *logical == MATRIX_EMPTY {
                continue;
            }
            let raw = ((col as u8) << 4) | (row as u8);
            cells.push(MatrixCell {
                raw: MatrixKey::new(raw),
                row: row as u8,
                col: col as u8,
                logical: *logical,
            });
        }
    }
    cells
}

fn matrix_code_to_logical(value: u8) -> Option<u8> {
    let row = value & 0x0f;
    let col = value >> 4;
    if row > 0x0f || col > 7 {
        return None;
    }
    let logical = MATRIX_LOGICAL[row as usize][col as usize];
    if logical == MATRIX_EMPTY {
        None
    } else {
        Some(logical)
    }
}

pub(crate) fn matrix_key_for_code(value: u8) -> Option<MatrixKey> {
    matrix_code_to_logical(value).map(|_| MatrixKey::new(value))
}

pub(crate) fn matrix_code_to_char(value: u8) -> Option<char> {
    hid_usage_to_char(matrix_code_to_hid_usage(value)?)
}

pub fn matrix_key_label(value: u8) -> String {
    if let Some(label) = physical_key_label(value) {
        label.to_string()
    } else if let Some(ch) = matrix_code_to_char(value) {
        ch.to_string()
    } else {
        if let Some(logical) = matrix_code_to_logical(value) {
            format!("0x{value:02x} (L0x{logical:02x})")
        } else {
            format!("0x{value:02x}")
        }
    }
}

#[cfg(test)]
pub(crate) fn matrix_key_is_character(value: u8) -> bool {
    matrix_code_to_char(value).is_some()
}

fn matrix_code_to_hid_usage(value: u8) -> Option<u8> {
    let logical = matrix_code_to_logical(value)?;
    let usage = LOGICAL_HID_USAGE[logical as usize];
    if usage == 0xff { None } else { Some(usage) }
}

fn hid_usage_to_char(usage: u8) -> Option<char> {
    match usage {
        0x04..=0x1d => Some((b'a' + (usage - 0x04)) as char),
        0x1e..=0x26 => Some((b'1' + (usage - 0x1e)) as char),
        0x27 => Some('0'),
        0x2c => Some(' '),
        0x2d => Some('-'),
        0x2e => Some('='),
        0x2f => Some('['),
        0x30 => Some(']'),
        0x31 => Some('\\'),
        0x33 => Some(';'),
        0x34 => Some('\''),
        0x35 => Some('`'),
        0x36 => Some(','),
        0x37 => Some('.'),
        0x38 => Some('/'),
        _ => None,
    }
}

fn physical_key_label(value: u8) -> Option<&'static str> {
    match value {
        0x4b => Some("File 1"),
        0x4a => Some("File 2"),
        0x0a => Some("File 3"),
        0x1a => Some("File 4"),
        0x19 => Some("File 5"),
        0x10 => Some("File 6"),
        0x02 => Some("File 7"),
        0x42 => Some("File 8"),
        0x49 => Some("Print"),
        0x59 => Some("Spell Check"),
        0x67 => Some("Find"),
        0x54 => Some("Clear File"),
        0x34 => Some("Home"),
        0x65 => Some("End"),
        0x46 => Some("Applets"),
        0x47 => Some("Send"),
        _ => hid_usage_to_special_label(matrix_code_to_hid_usage(value)?),
    }
}

fn hid_usage_to_special_label(usage: u8) -> Option<&'static str> {
    match usage {
        0x28 => Some("Enter"),
        0x29 => Some("Esc"),
        0x2a => Some("Backspace"),
        0x2b => Some("Tab"),
        0x39 => Some("Caps Lock"),
        0x4a => Some("Home"),
        0x4c => Some("Delete"),
        0x4d => Some("End"),
        0x4f => Some("Right"),
        0x50 => Some("Left"),
        0x51 => Some("Down"),
        0x52 => Some("Up"),
        0xe0 => Some("Ctrl"),
        0xe1 => Some("Shift"),
        0xe2 => Some("Alt/Option"),
        0xe3 => Some("Command"),
        0xe5 => Some("Right Shift"),
        0xe6 => Some("Right Alt"),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
struct KeyPhase {
    key_codes: [Option<MatrixKey>; 4],
    reads: usize,
    all_rows: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Keyboard {
    held: Vec<MatrixKey>,
    script: Vec<KeyPhase>,
    phase: usize,
    reads_in_phase: usize,
    selected_rows: Option<u16>,
}

impl Keyboard {
    pub(crate) fn press(&mut self, key: MatrixKey) {
        if !self.held.contains(&key) {
            self.held.push(key);
        }
    }

    pub(crate) fn release(&mut self, key: MatrixKey) {
        self.held.retain(|held| *held != key);
    }

    pub(crate) fn tap(&mut self, key: MatrixKey) {
        self.push_phase([Some(key), None, None, None], 8, false);
        self.push_phase([None, None, None, None], 64, false);
    }

    pub(crate) fn tap_chord(&mut self, keys: &[MatrixKey]) {
        let mut key_codes = [None, None, None, None];
        for (slot, key) in key_codes.iter_mut().zip(keys.iter().copied()) {
            *slot = Some(key);
        }
        self.push_phase(key_codes, 8, false);
        self.push_phase([None, None, None, None], 64, false);
    }

    pub(crate) fn tap_long(&mut self, key: MatrixKey) {
        self.push_phase([Some(key), None, None, None], 50_000, false);
        self.push_phase([None, None, None, None], 5_000, false);
    }

    pub(crate) fn tap_all_rows(&mut self, key: MatrixKey) {
        self.push_phase([Some(key), None, None, None], 50_000, true);
        self.push_phase([None, None, None, None], 5_000, true);
    }

    pub(crate) fn hold_small_rom_entry_chord(&mut self) {
        self.hold_keys_all_rows(
            &[
                MatrixKey::new(0x6e),
                MatrixKey::new(0x60),
                MatrixKey::new(0x62),
                MatrixKey::new(0x73),
            ],
            4,
        );
    }

    pub(crate) fn hold_keys_all_rows(&mut self, keys: &[MatrixKey], reads: usize) {
        let mut key_codes = [None, None, None, None];
        for (slot, key) in key_codes.iter_mut().zip(keys.iter().copied()) {
            *slot = Some(key);
        }
        self.push_phase(key_codes, reads, true);
    }

    pub(crate) fn hold_keys_exact_rows(&mut self, keys: &[MatrixKey], reads: usize) {
        let mut key_codes = [None, None, None, None];
        for (slot, key) in key_codes.iter_mut().zip(keys.iter().copied()) {
            *slot = Some(key);
        }
        self.push_phase(key_codes, reads, false);
    }

    pub(crate) fn select_row(&mut self, row: u8) {
        self.select_rows(1 << (row & 0x0f));
    }

    pub(crate) fn select_rows(&mut self, rows: u16) {
        self.selected_rows = Some(rows & 0xffff);
    }

    pub(crate) fn read_matrix_input(&mut self) -> u8 {
        let mut active_columns = 0;
        for script_key in self.read_script_keys() {
            let visible_row = if script_key.all_rows {
                None
            } else {
                self.selected_rows
            };
            if script_key.key.is_visible_on_rows(visible_row) {
                active_columns |= script_key.column_mask();
            }
        }
        for key in self.held.iter().copied() {
            if key.is_visible_on_rows(self.selected_rows) {
                active_columns |= key.column_mask();
            }
        }
        !active_columns
    }

    fn read_script_keys(&mut self) -> Vec<ScriptKey> {
        let Some(phase) = self.script.get(self.phase).copied() else {
            return Vec::new();
        };
        self.reads_in_phase = self.reads_in_phase.saturating_add(1);
        if self.reads_in_phase >= phase.reads {
            self.phase = self.phase.saturating_add(1);
            self.reads_in_phase = 0;
        }
        phase
            .key_codes
            .into_iter()
            .flatten()
            .map(|key| ScriptKey {
                key,
                all_rows: phase.all_rows,
            })
            .collect()
    }

    fn push_phase(&mut self, key_codes: [Option<MatrixKey>; 4], reads: usize, all_rows: bool) {
        self.script.push(KeyPhase {
            key_codes,
            reads,
            all_rows,
        });
    }
}

#[derive(Clone, Copy, Debug)]
struct ScriptKey {
    key: MatrixKey,
    all_rows: bool,
}

impl ScriptKey {
    fn column_mask(self) -> u8 {
        self.key.column_mask()
    }
}

pub fn matrix_key_for_char(value: char) -> Option<MatrixKey> {
    let normalized = value.to_ascii_lowercase();
    for row in 0..16u8 {
        for col in 0..8u8 {
            let raw = (col << 4) | row;
            if matrix_code_to_char(raw) == Some(normalized) {
                return Some(MatrixKey::new(raw));
            }
        }
    }
    None
}

fn _assert_full_matrix_tables() {
    let count = matrix_cells().len();
    assert!(
        count == 80,
        "firmware-derived matrix currently exposes 80 physical keys, got {count}"
    );
    assert!(
        LOGICAL_HID_USAGE.len() == 0x51,
        "logical HID usage map must remain 0x51 entries"
    );
}

#[cfg(test)]
fn _validate_display_map() {
    for row in MATRIX_LOGICAL {
        for logical in row {
            if logical == MATRIX_EMPTY {
                continue;
            }
            assert!(
                (logical as usize) < LOGICAL_HID_USAGE.len(),
                "logical code 0x{logical:02x} must be in HID usage table"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        _assert_full_matrix_tables, matrix_cells, matrix_key_for_char, matrix_key_for_code,
        matrix_key_label,
    };
    use super::{_validate_display_map, Keyboard, MatrixKey};

    #[test]
    fn known_small_rom_password_keys_match_firmware_table() {
        assert_eq!(matrix_key_for_char('e').map(MatrixKey::code), Some(0x3a));
        assert_eq!(matrix_key_for_char('r').map(MatrixKey::code), Some(0x3d));
        assert_eq!(matrix_key_for_char('n').map(MatrixKey::code), Some(0x7f));
        assert_eq!(matrix_key_for_char('i').map(MatrixKey::code), Some(0x30));
    }

    #[test]
    fn firmware_char_map_is_consistent() {
        _assert_full_matrix_tables();
        _validate_display_map();
    }

    #[test]
    fn char_lookup_falls_back_to_candidate_firmware_labels_when_available() {
        assert_eq!(matrix_key_for_char('e').map(MatrixKey::code), Some(0x3a));
        assert_eq!(matrix_key_for_char('d').map(MatrixKey::code), Some(0x2a));
        assert_eq!(matrix_key_for_char('b').map(MatrixKey::code), Some(0x7d));
        assert_eq!(matrix_key_for_char('`').map(MatrixKey::code), Some(0x4c));
        assert_eq!(matrix_key_for_char(' ').map(MatrixKey::code), Some(0x79));
        assert_eq!(matrix_key_for_char('/').map(MatrixKey::code), Some(0x73));
    }

    #[test]
    fn special_key_labels_use_firmware_hid_bridge() {
        assert_eq!(matrix_key_label(0x69), "Enter");
        assert_eq!(matrix_key_label(0x15), "Down");
        assert_eq!(matrix_key_label(0x77), "Up");
        assert_eq!(matrix_key_label(0x75), "Left");
        assert_eq!(matrix_key_label(0x76), "Right");
        assert_eq!(matrix_key_label(0x46), "Applets");
        assert_eq!(matrix_key_label(0x47), "Send");
    }

    #[test]
    fn matrix_cells_is_derived_from_firmware_keyboard_map() {
        let cells = matrix_cells();
        assert!(
            cells
                .iter()
                .any(|cell| cell.row == 0x0f && cell.col == 4 && cell.raw.code() == 0x4f)
        );
        assert!(cells.iter().any(|cell| cell.raw.code() == 0x7f));
    }

    #[test]
    fn matrix_key_for_code_only_accepts_non_empty_matrix_slots() {
        assert_eq!(matrix_key_for_code(0x3a).map(MatrixKey::code), Some(0x3a));
        assert_eq!(matrix_key_for_code(0x00).map(MatrixKey::code), Some(0x00));
        assert_eq!(matrix_key_for_code(0x08), None);
    }

    #[test]
    fn small_rom_entry_chord_is_visible_during_boot_gate_only() {
        let mut keyboard = Keyboard::default();
        keyboard.select_row(0x00);
        keyboard.hold_small_rom_entry_chord();

        for _ in 0..4 {
            assert_ne!(keyboard.read_matrix_input(), 0xff);
        }
        assert_eq!(keyboard.read_matrix_input(), 0xff);
    }

    #[test]
    fn all_row_text_tap_is_visible_independent_of_selected_row() {
        let mut keyboard = Keyboard::default();
        keyboard.select_row(0x00);
        keyboard.tap_all_rows(MatrixKey::new(0x3a));

        assert_eq!(keyboard.read_matrix_input(), 0xf7);
    }

    #[test]
    fn tap_appended_after_expired_boot_chord_becomes_visible() {
        let mut keyboard = Keyboard::default();
        keyboard.hold_keys_all_rows(&[MatrixKey::new(0x0e), MatrixKey::new(0x0c)], 1);
        assert_eq!(keyboard.read_matrix_input(), 0xfe);
        assert_eq!(keyboard.read_matrix_input(), 0xff);

        keyboard.tap(MatrixKey::new(0x15));
        keyboard.select_row(0x05);

        assert_eq!(keyboard.read_matrix_input(), 0xfd);
    }

    #[test]
    fn held_key_is_active_low_only_on_selected_row() {
        let mut keyboard = Keyboard::default();
        keyboard.press(MatrixKey::new(0x3a));

        keyboard.select_row(0x0d);
        assert_eq!(keyboard.read_matrix_input(), 0xff);

        keyboard.select_row(0x0a);
        assert_eq!(keyboard.read_matrix_input(), 0xf7);
    }

    #[test]
    fn held_keys_are_visible_on_any_selected_row() {
        let mut keyboard = Keyboard::default();
        keyboard.press(MatrixKey::new(0x14));
        keyboard.press(MatrixKey::new(0x2c));

        keyboard.select_rows((1 << 0x04) | (1 << 0x0c));
        let value = keyboard.read_matrix_input();

        assert_eq!(value & 0x02, 0x00);
        assert_eq!(value & 0x04, 0x00);
    }
}
