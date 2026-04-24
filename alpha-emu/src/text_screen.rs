#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextScreen {
    rows: [[u8; 64]; 8],
    row: usize,
    col: usize,
    active: bool,
}

impl Default for TextScreen {
    fn default() -> Self {
        Self {
            rows: [[b' '; 64]; 8],
            row: 0,
            col: 0,
            active: false,
        }
    }
}

impl TextScreen {
    pub fn clear(&mut self) {
        self.rows = [[b' '; 64]; 8];
        self.row = 0;
        self.col = 0;
        self.active = true;
    }

    pub fn set_cursor(&mut self, row: u32, col: u32, _width: u32) {
        self.row = row.saturating_sub(1) as usize;
        self.col = col.saturating_sub(1) as usize;
        self.active = true;
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
}
