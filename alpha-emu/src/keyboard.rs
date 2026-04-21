#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MatrixKey(u8);

impl MatrixKey {
    pub(crate) const fn new(code: u8) -> Self {
        Self(code)
    }

    pub(crate) const fn code(self) -> u8 {
        self.0
    }

    pub(crate) const fn row(self) -> u8 {
        self.0 & 0x0f
    }

    const fn column_mask(self) -> u8 {
        1 << (self.0 >> 4)
    }

    fn is_visible_on_row(self, row: Option<u8>) -> bool {
        row.is_none_or(|row| row == self.row())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MatrixCell {
    pub(crate) raw: MatrixKey,
    pub(crate) row: u8,
    pub(crate) col: u8,
    pub(crate) logical: u8,
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
    [0x07, 0x10, 0x19, 0x23, 0x2f, 0x39, 0x44, MATRIX_EMPTY],
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

pub(crate) fn matrix_cells() -> Vec<MatrixCell> {
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

pub(crate) fn matrix_key_for_code(value: u8) -> Option<MatrixKey> {
    let row = value & 0x0f;
    let col = value >> 4;
    if row > 0x0f || col > 7 {
        return None;
    }
    let logical = MATRIX_LOGICAL[row as usize][col as usize];
    if logical == MATRIX_EMPTY {
        None
    } else {
        Some(MatrixKey::new(value))
    }
}

pub(crate) fn matrix_code_to_char(value: u8) -> Option<char> {
    match value {
        0x3a => Some('e'),
        0x3d => Some('r'),
        0x7f => Some('n'),
        0x30 => Some('i'),
        _ => None,
    }
}

pub(crate) fn matrix_key_label(value: u8) -> String {
    if let Some(ch) = matrix_code_to_char(value) {
        ch.to_string()
    } else {
        format!("0x{value:02x}")
    }
}

pub(crate) fn matrix_key_is_alphanumeric(value: u8) -> bool {
    matrix_code_to_char(value).is_some_and(|character| character.is_ascii_alphanumeric())
}

#[derive(Clone, Copy, Debug)]
struct KeyPhase {
    key_code: Option<MatrixKey>,
    reads: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Keyboard {
    held: Vec<MatrixKey>,
    script: Vec<KeyPhase>,
    phase: usize,
    reads_in_phase: usize,
    selected_row: Option<u8>,
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

    pub(crate) fn select_row(&mut self, row: u8) {
        self.selected_row = Some(row & 0x0f);
    }

    pub(crate) fn type_small_rom_password(&mut self) {
        self.script.clear();
        self.phase = 0;
        self.reads_in_phase = 0;
        self.push_phase(None, 80);
        for key_code in [
            MatrixKey::new(0x3a),
            MatrixKey::new(0x3d),
            MatrixKey::new(0x7f),
            MatrixKey::new(0x30),
            MatrixKey::new(0x3a),
        ] {
            self.push_phase(Some(key_code), 120);
            self.push_phase(None, 120);
        }
    }

    pub(crate) fn read_matrix_input(&mut self) -> u8 {
        let mut active_columns = 0;
        if let Some(script_key) = self.read_script_key()
            && script_key.is_visible_on_row(None)
        {
            active_columns |= script_key.column_mask();
        }
        for key in self.held.iter().copied() {
            if key.is_visible_on_row(self.selected_row) {
                active_columns |= key.column_mask();
            }
        }
        !active_columns
    }

    fn read_script_key(&mut self) -> Option<MatrixKey> {
        let phase = self.script.get(self.phase).copied()?;
        self.reads_in_phase = self.reads_in_phase.saturating_add(1);
        if self.reads_in_phase >= phase.reads {
            self.phase = self.phase.saturating_add(1);
            self.reads_in_phase = 0;
        }
        phase.key_code
    }

    fn push_phase(&mut self, key_code: Option<MatrixKey>, reads: usize) {
        self.script.push(KeyPhase { key_code, reads });
    }
}

pub(crate) fn matrix_key_for_char(value: char) -> Option<MatrixKey> {
    match value.to_ascii_lowercase() {
        'e' => Some(MatrixKey::new(0x3a)),
        'r' => Some(MatrixKey::new(0x3d)),
        'n' => Some(MatrixKey::new(0x7f)),
        'i' => Some(MatrixKey::new(0x30)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Keyboard, MatrixKey, matrix_cells, matrix_key_for_char, matrix_key_for_code};

    #[test]
    fn known_small_rom_password_keys_match_firmware_table() {
        assert_eq!(matrix_key_for_char('e').map(MatrixKey::code), Some(0x3a));
        assert_eq!(matrix_key_for_char('r').map(MatrixKey::code), Some(0x3d));
        assert_eq!(matrix_key_for_char('n').map(MatrixKey::code), Some(0x7f));
        assert_eq!(matrix_key_for_char('i').map(MatrixKey::code), Some(0x30));
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
    fn held_key_is_active_low_only_on_selected_row() {
        let mut keyboard = Keyboard::default();
        keyboard.press(MatrixKey::new(0x3a));

        keyboard.select_row(0x0d);
        assert_eq!(keyboard.read_matrix_input(), 0xff);

        keyboard.select_row(0x0a);
        assert_eq!(keyboard.read_matrix_input(), 0xf7);
    }

    #[test]
    fn scripted_password_ignores_row_selection_for_small_rom_boot_shortcut() {
        let mut keyboard = Keyboard::default();
        keyboard.type_small_rom_password();

        for _ in 0..80 {
            assert_eq!(keyboard.read_matrix_input(), 0xff);
        }
        assert_eq!(keyboard.read_matrix_input(), 0xf7);
    }
}
