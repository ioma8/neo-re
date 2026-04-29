pub const FILE_COUNT: usize = 8;
pub const MAX_FILE_BYTES: usize = 4096;
pub const SCREEN_ROWS: usize = 4;
pub const SCREEN_COLS: usize = 28;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    pub row: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NavigationState {
    cursor: usize,
    viewport: Viewport,
    present: bool,
}

#[repr(C)]
pub struct SlotNavigation {
    states: [NavigationState; FILE_COUNT],
}

impl SlotNavigation {
    #[must_use]
    pub const fn new() -> Self {
        const EMPTY: NavigationState = NavigationState {
            cursor: 0,
            viewport: Viewport { row: 0 },
            present: false,
        };
        Self {
            states: [EMPTY; FILE_COUNT],
        }
    }

    pub fn store(&mut self, slot: usize, cursor: usize, viewport: Viewport) {
        if (1..=FILE_COUNT).contains(&slot) {
            // SAFETY: Slot range was checked above and maps to 0..FILE_COUNT.
            unsafe {
                *self.states.get_unchecked_mut(slot - 1) = NavigationState {
                    cursor,
                    viewport,
                    present: true,
                };
            }
        }
    }

    #[must_use]
    pub fn restore(&self, slot: usize) -> Option<(usize, Viewport)> {
        if !(1..=FILE_COUNT).contains(&slot) {
            return None;
        }
        // SAFETY: Slot range was checked above and maps to 0..FILE_COUNT.
        let state = unsafe { *self.states.get_unchecked(slot - 1) };
        if state.present {
            Some((state.cursor, state.viewport))
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct Document {
    bytes: [u8; MAX_FILE_BYTES],
    len: usize,
    cursor: usize,
    viewport: Viewport,
}

impl Document {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bytes: [0; MAX_FILE_BYTES],
            len: 0,
            cursor: 0,
            viewport: Viewport { row: 0 },
        }
    }

    #[must_use]
    #[allow(dead_code, reason = "used by storage load path in later implementation task")]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut document = Self::new();
        for byte in bytes {
            if is_supported_file_byte(*byte) {
                let _ = document.insert_byte(*byte);
            }
        }
        document
    }

    #[must_use]
    #[allow(dead_code, reason = "used by storage save path in later implementation task")]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: `self.len` is maintained as <= MAX_FILE_BYTES.
        unsafe { core::slice::from_raw_parts(self.bytes.as_ptr(), self.len) }
    }

    #[must_use]
    #[allow(dead_code, reason = "used by storage bounds tests")]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    #[must_use]
    pub const fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.len);
        self.ensure_cursor_visible();
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.ensure_cursor_visible();
    }

    #[allow(dead_code, reason = "used by storage load path in later implementation task")]
    pub fn move_to_end(&mut self) {
        self.cursor = self.len;
        self.ensure_cursor_visible();
    }

    pub fn insert_byte(&mut self, byte: u8) -> bool {
        if !is_supported_file_byte(byte) || self.len == MAX_FILE_BYTES {
            return false;
        }
        let mut index = self.len;
        while index > self.cursor {
            let previous = self.byte_at(index - 1);
            self.set_byte(index, previous);
            index -= 1;
        }
        self.set_byte(self.cursor, byte);
        self.len += 1;
        self.cursor += 1;
        self.ensure_cursor_visible();
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let mut index = self.cursor - 1;
        while index + 1 < self.len {
            let next = self.byte_at(index + 1);
            self.set_byte(index, next);
            index += 1;
        }
        self.len -= 1;
        self.cursor -= 1;
        self.ensure_cursor_visible();
        true
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.ensure_cursor_visible();
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.len);
        self.ensure_cursor_visible();
    }

    pub fn move_up(&mut self) {
        let position = self.visual_position(self.cursor);
        if position.row == 0 {
            return;
        }
        self.cursor = self.cursor_for_visual_position(position.row - 1, position.col);
        self.ensure_cursor_visible();
    }

    pub fn move_down(&mut self) {
        let position = self.visual_position(self.cursor);
        self.cursor = self.cursor_for_visual_position(position.row + 1, position.col);
        self.ensure_cursor_visible();
    }

    pub fn ensure_cursor_visible(&mut self) {
        let row = self.visual_position(self.cursor).row;
        if row < self.viewport.row {
            self.viewport.row = row;
        } else if row >= self.viewport.row + SCREEN_ROWS {
            self.viewport.row = row + 1 - SCREEN_ROWS;
        }
    }

    pub fn render_row(&self, screen_row: usize, output: &mut [u8; SCREEN_COLS]) {
        let wanted_row = self.viewport.row + screen_row;
        let mut index = 0;
        while index < SCREEN_COLS {
            set_output_byte(output, index, b' ');
            index += 1;
        }

        let mut cursor_marked = false;
        let mut byte_index = 0;
        while byte_index <= self.len {
            let position = self.visual_position(byte_index);
            if position.row > wanted_row {
                break;
            }
            if position.row == wanted_row && position.col < SCREEN_COLS {
                if byte_index == self.cursor {
                    set_output_byte(output, position.col, b'|');
                    cursor_marked = true;
                } else if byte_index < self.len && self.byte_at(byte_index) != b'\n' {
                    set_output_byte(output, position.col, self.byte_at(byte_index));
                }
            }
            if byte_index == self.len {
                break;
            }
            byte_index += 1;
        }

        if !cursor_marked && screen_row == 0 {
            set_output_byte(output, 0, b'|');
        }
    }

    fn byte_at(&self, index: usize) -> u8 {
        debug_assert!(index < self.len);
        // SAFETY: Callers only read initialized document bytes.
        unsafe { *self.bytes.get_unchecked(index) }
    }

    fn set_byte(&mut self, index: usize, value: u8) {
        debug_assert!(index < MAX_FILE_BYTES);
        // SAFETY: Callers only write inside the fixed document buffer.
        unsafe {
            *self.bytes.get_unchecked_mut(index) = value;
        }
    }

    fn visual_position(&self, cursor: usize) -> VisualPosition {
        let mut row = 0;
        let mut col = 0;
        let limit = cursor.min(self.len);
        let mut index = 0;
        while index < limit {
            if self.byte_at(index) == b'\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
                if col == SCREEN_COLS {
                    row += 1;
                    col = 0;
                }
            }
            index += 1;
        }
        VisualPosition { row, col }
    }

    fn cursor_for_visual_position(&self, wanted_row: usize, wanted_col: usize) -> usize {
        let mut index = 0;
        while index <= self.len {
            let position = self.visual_position(index);
            if position.row > wanted_row
                || (position.row == wanted_row && position.col >= wanted_col.min(SCREEN_COLS - 1))
            {
                return index;
            }
            if index == self.len {
                break;
            }
            index += 1;
        }
        self.len
    }
}

fn set_output_byte(output: &mut [u8; SCREEN_COLS], index: usize, value: u8) {
    debug_assert!(index < SCREEN_COLS);
    // SAFETY: Callers pass columns in 0..SCREEN_COLS.
    unsafe {
        *output.get_unchecked_mut(index) = value;
    }
}

#[derive(Clone, Copy)]
struct VisualPosition {
    row: usize,
    col: usize,
}

#[must_use]
const fn is_supported_file_byte(byte: u8) -> bool {
    matches!(byte, b'\n' | b' '..=b'~')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_printable_byte_at_cursor() {
        let mut document = Document::new();
        assert!(document.insert_byte(b'a'));
        assert!(document.insert_byte(b'c'));
        document.move_left();
        assert!(document.insert_byte(b'b'));

        assert_eq!(document.as_bytes(), b"abc");
        assert_eq!(document.cursor(), 2);
    }

    #[test]
    fn handles_newline_and_backspace() {
        let mut document = Document::from_bytes(b"a\nb");
        document.move_to_end();
        document.backspace();
        document.backspace();

        assert_eq!(document.as_bytes(), b"a");
        assert_eq!(document.cursor(), 1);
    }

    #[test]
    fn drops_unsupported_bytes_on_load() {
        let document = Document::from_bytes(b"a\x00b\tc\x7fd");

        assert_eq!(document.as_bytes(), b"abcd");
        assert_eq!(document.cursor(), 4);
    }

    #[test]
    fn rejects_insert_when_file_is_full() {
        let mut document = Document::new();
        for _ in 0..MAX_FILE_BYTES {
            assert!(document.insert_byte(b'x'));
        }

        assert!(!document.insert_byte(b'y'));
        assert_eq!(document.len(), MAX_FILE_BYTES);
    }

    #[test]
    fn moves_left_and_right_across_newline() {
        let mut document = Document::from_bytes(b"ab\ncd");
        document.move_to_end();
        document.move_left();
        document.move_left();
        document.move_left();
        assert_eq!(document.cursor(), 2);
        document.move_right();
        assert_eq!(document.cursor(), 3);
    }

    #[test]
    fn moves_up_and_down_across_wrapped_rows() {
        let mut document = Document::from_bytes(b"abcdefghijklmnopqrstuvwxyz0123456789");
        document.set_cursor(30);
        document.move_up();
        assert_eq!(document.cursor(), 2);
        document.move_down();
        assert_eq!(document.cursor(), 30);
    }

    #[test]
    fn scrolls_viewport_to_keep_cursor_visible() {
        let mut document = Document::from_bytes(b"111111111111111111111111111122222222222222222222222222223333333333333333333333333334444444444444444444444444455555555555555555555555555");
        document.move_to_end();
        document.ensure_cursor_visible();

        assert_eq!(document.viewport().row, 1);
    }

    #[test]
    fn slot_navigation_state_survives_switches_in_ram() {
        let mut slots = SlotNavigation::new();
        slots.store(1, 7, Viewport { row: 2 });

        assert_eq!(slots.restore(1), Some((7, Viewport { row: 2 })));
        assert_eq!(slots.restore(2), None);
    }

    #[test]
    fn fresh_restart_has_no_slot_navigation_state() {
        let slots = SlotNavigation::new();
        let document = Document::from_bytes(b"persisted");

        assert_eq!(slots.restore(1), None);
        assert_eq!(document.cursor(), b"persisted".len());
    }

    #[test]
    fn render_row_marks_visible_cursor() {
        let document = Document::from_bytes(b"abc");
        let mut row = [0; SCREEN_COLS];

        document.render_row(0, &mut row);

        assert_eq!(&row[..4], b"abc|");
    }
}
