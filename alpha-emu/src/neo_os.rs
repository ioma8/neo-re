use crate::domain::{Lcd, UsbMode};

#[derive(Clone, Debug)]
pub struct NeoOs {
    pub lcd: Lcd,
    pub usb_mode: UsbMode,
    current_row: usize,
}

impl Default for NeoOs {
    fn default() -> Self {
        Self {
            lcd: Lcd::default(),
            usb_mode: UsbMode::HidKeyboard,
            current_row: 0,
        }
    }
}

impl NeoOs {
    pub fn clear_screen(&mut self) {
        self.lcd.clear();
    }

    pub fn set_row_text(&mut self, row: u32, text: String) {
        let index = row.saturating_sub(1) as usize;
        self.lcd.set_row(index, text);
        self.current_row = index;
    }

    pub fn append_char(&mut self, ch: u8) {
        let mut text = self
            .lcd
            .rows()
            .get(self.current_row)
            .cloned()
            .unwrap_or_default();
        text.push(ch as char);
        self.lcd.set_row(self.current_row, text);
    }

    pub fn switch_to_direct(&mut self) {
        self.usb_mode = UsbMode::Direct;
    }
}
