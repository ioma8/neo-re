#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MatrixKey(u8);

impl MatrixKey {
    pub(crate) const fn new(code: u8) -> Self {
        Self(code)
    }

    #[cfg(test)]
    pub(crate) const fn code(self) -> u8 {
        self.0
    }

    const fn row(self) -> u8 {
        self.0 & 0x0f
    }

    const fn column_mask(self) -> u8 {
        1 << (self.0 >> 4)
    }

    fn is_visible_on_row(self, row: Option<u8>) -> bool {
        row.is_none_or(|row| row == self.row())
    }
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
    use super::{Keyboard, MatrixKey, matrix_key_for_char};

    #[test]
    fn known_small_rom_password_keys_match_firmware_table() {
        assert_eq!(matrix_key_for_char('e').map(MatrixKey::code), Some(0x3a));
        assert_eq!(matrix_key_for_char('r').map(MatrixKey::code), Some(0x3d));
        assert_eq!(matrix_key_for_char('n').map(MatrixKey::code), Some(0x7f));
        assert_eq!(matrix_key_for_char('i').map(MatrixKey::code), Some(0x30));
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
