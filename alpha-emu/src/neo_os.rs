use crate::domain::{Lcd, UsbMode};

#[derive(Clone, Debug)]
pub struct NeoOs {
    pub lcd: Lcd,
    pub usb_mode: UsbMode,
    current_row: usize,
}

impl Default for NeoOs {
    fn default() -> Self {
        let mut os = Self {
            lcd: Lcd::default(),
            usb_mode: UsbMode::HidKeyboard,
            current_row: 0,
        };
        os.draw_empty_menu();
        os
    }
}

impl NeoOs {
    pub fn draw_empty_menu(&mut self) {
        self.lcd.clear();
        self.lcd.set_row(0, "SmartApplets");
        self.lcd.set_row(2, "No applet loaded.");
        self.lcd.set_row(4, "Use Open applet.");
        self.current_row = 0;
    }

    pub fn draw_applets_menu(&mut self, applet_name: &str, selected: bool) {
        self.lcd.clear();
        self.lcd.set_row(0, "SmartApplets");
        let marker = if selected { ">" } else { " " };
        self.lcd.set_row(2, format!("{marker} {applet_name}"));
        self.lcd.set_row(4, "Up/Down select");
        self.lcd.set_row(5, "Enter opens");
        self.current_row = 0;
    }

    pub fn draw_usb_attach_start(&mut self) {
        self.lcd.clear();
        self.lcd.set_row(1, "Making USB");
        self.lcd.set_row(2, "connection...");
        self.current_row = 0;
    }

    pub fn draw_usb_keyboard_attached(&mut self) {
        self.lcd.clear();
        self.lcd.set_row(1, "Connected to computer,");
        self.lcd.set_row(2, "emulating keyboard.");
        self.current_row = 0;
    }

    pub fn draw_direct_attached(&mut self) {
        self.lcd.clear();
        self.lcd.set_row(1, "Connected to");
        self.lcd.set_row(2, "NEO Manager.");
        self.current_row = 0;
    }

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
