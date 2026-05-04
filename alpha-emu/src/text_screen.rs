#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextScreen {
    rows: [[u8; 64]; 8],
    row: usize,
    col: usize,
    active: bool,
    cursor_visible: bool,
}

impl Default for TextScreen {
    fn default() -> Self {
        Self {
            rows: [[b' '; 64]; 8],
            row: 0,
            col: 0,
            active: false,
            cursor_visible: false,
        }
    }
}

impl TextScreen {
    pub fn clear(&mut self) {
        self.rows = [[b' '; 64]; 8];
        self.row = 0;
        self.col = 0;
        self.active = true;
        self.cursor_visible = false;
    }

    pub fn set_cursor(&mut self, row: u32, col: u32, _width: u32) {
        self.row = row.saturating_sub(1) as usize;
        self.col = col.saturating_sub(1) as usize;
        self.active = true;
    }

    pub fn set_cursor_mode(&mut self, mode: u32) {
        self.active = true;
        self.cursor_visible = mode == 0x0f;
    }

    pub fn draw_char(&mut self, byte: u8) {
        if self.row < self.rows.len() && self.col < self.rows[self.row].len() {
            self.rows[self.row][self.col] = byte;
            self.active = true;
        }
        self.col = self.col.saturating_add(1);
    }

    pub fn draw_c_string(&mut self, bytes: &[u8]) {
        for byte in bytes.iter().copied() {
            if byte == 0 {
                break;
            }
            self.draw_char(byte);
        }
    }

    pub fn render(&self) -> Option<String> {
        if !self.active {
            return None;
        }
        let lines = self
            .rows
            .iter()
            .map(|row| {
                let sanitized = row
                    .iter()
                    .map(|byte| match *byte {
                        0x20..=0x7e => *byte,
                        _ => b' ',
                    })
                    .collect::<Vec<_>>();
                let end = sanitized
                    .iter()
                    .rposition(|byte| *byte != b' ')
                    .map_or(0, |index| index + 1);
                String::from_utf8_lossy(&sanitized[..end]).into_owned()
            })
            .collect::<Vec<_>>();
        let last_nonempty = lines.iter().rposition(|line| !line.is_empty()).unwrap_or(0);
        Some(lines[..=last_nonempty].join("\n"))
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    pub fn cursor_row(&self) -> usize {
        self.row
    }

    pub fn cursor_col(&self) -> usize {
        self.col
    }
}

#[cfg(test)]
mod tests {
    use super::TextScreen;

    #[test]
    fn render_returns_none_when_inactive() {
        let screen = TextScreen::default();
        assert_eq!(screen.render(), None);
    }

    #[test]
    fn render_returns_text_after_draw_char() {
        let mut screen = TextScreen::default();
        screen.clear();
        for b in b"hello" {
            screen.draw_char(*b);
        }
        let output = screen.render().unwrap();
        assert!(output.contains("hello"));
    }

    #[test]
    fn render_trims_trailing_spaces() {
        let mut screen = TextScreen::default();
        screen.clear();
        for b in b"abc" {
            screen.draw_char(*b);
        }
        let output = screen.render().unwrap();
        assert_eq!(output.lines().next(), Some("abc"));
    }

    #[test]
    fn set_cursor_and_set_cursor_mode_activate() {
        let mut screen = TextScreen::default();
        // Render is None when inactive
        assert_eq!(screen.render(), None);
        assert!(!screen.cursor_visible());

        screen.set_cursor(2, 3, 64);
        screen.set_cursor_mode(0x0f);
        assert!(screen.cursor_visible());
        assert_eq!(screen.cursor_row(), 1);
        assert_eq!(screen.cursor_col(), 2);
    }

    #[test]
    fn draw_char_at_explicit_cursor_position() {
        let mut screen = TextScreen::default();
        screen.clear();
        screen.set_cursor(2, 5, 64);
        screen.draw_char(b'X');
        let output = screen.render().unwrap();
        assert_eq!(output.lines().nth(1).map(|line| line.as_bytes()[4]), Some(b'X'));
    }

    #[test]
    fn draw_c_string_stops_at_null() {
        let mut screen = TextScreen::default();
        screen.clear();
        screen.draw_c_string(b"ab\0cd");
        let output = screen.render().unwrap();
        assert!(output.contains("ab"));
        assert!(!output.contains("cd"));
    }

    #[test]
    fn sanitizes_non_ascii_bytes_in_render() {
        let mut screen = TextScreen::default();
        screen.clear();
        screen.draw_char(0x00);
        screen.draw_char(0x7f);
        let output = screen.render().unwrap();
        assert_eq!(output.trim(), "");
    }
}
