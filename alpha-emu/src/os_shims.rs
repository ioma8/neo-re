use std::collections::VecDeque;

use crate::domain::{Lcd, UsbMode};

#[derive(Clone, Debug)]
pub struct NeoOsShims {
    pub lcd: Lcd,
    pub usb_mode: UsbMode,
    current_row: usize,
    current_col: usize,
    key_queue: VecDeque<u8>,
}

impl Default for NeoOsShims {
    fn default() -> Self {
        let mut os = Self {
            lcd: Lcd::default(),
            usb_mode: UsbMode::HidKeyboard,
            current_row: 0,
            current_col: 0,
            key_queue: VecDeque::new(),
        };
        os.draw_empty_menu();
        os
    }
}

impl NeoOsShims {
    // Firmware-owned screens. These are not drawn by applet machine code.
    pub fn draw_empty_menu(&mut self) {
        self.lcd.clear();
        self.lcd.set_row(0, "SmartApplets");
        self.lcd.set_row(2, "No applet loaded.");
        self.lcd.set_row(4, "Use Open applet.");
        self.current_row = 0;
        self.current_col = 0;
    }

    pub fn draw_applets_menu(&mut self, applet_name: &str, selected: bool) {
        self.lcd.clear();
        self.lcd.set_row(0, "SmartApplets");
        let marker = if selected { ">" } else { " " };
        self.lcd.set_row(2, format!("{marker} {applet_name}"));
        self.lcd.set_row(4, "Up/Down select");
        self.lcd.set_row(5, "Enter opens");
        self.current_row = 0;
        self.current_col = 0;
    }

    pub fn draw_usb_attach_start(&mut self) {
        self.lcd.clear();
        self.lcd.set_row(1, "Making USB");
        self.lcd.set_row(2, "connection...");
        self.current_row = 0;
        self.current_col = 0;
    }

    pub fn draw_usb_keyboard_attached(&mut self) {
        self.lcd.clear();
        self.lcd.set_row(1, "Connected to computer,");
        self.lcd.set_row(2, "emulating keyboard.");
        self.current_row = 0;
        self.current_col = 0;
    }

    pub fn draw_direct_attached(&mut self) {
        self.lcd.clear();
        self.lcd.set_row(1, "Connected to");
        self.lcd.set_row(2, "NEO Manager.");
        self.current_row = 0;
        self.current_col = 0;
    }

    // A-line display traps. These are invoked by interpreted applet machine code.
    pub fn clear_screen(&mut self) {
        self.lcd.clear();
        self.current_row = 0;
        self.current_col = 0;
    }

    pub fn set_text_region(&mut self, row: u32, col: u32, width: u32) {
        let index = row.saturating_sub(1) as usize;
        self.current_row = index;
        self.current_col = col.saturating_sub(1) as usize;
        self.lcd.clear_span(index, self.current_col, width as usize);
    }

    pub fn append_char(&mut self, ch: u8) {
        let printable = match ch {
            b'\t' | b'\n' | b'\r' | 0x00..=0x1f => b' ',
            0x7f => b' ',
            _ => ch,
        };
        self.lcd
            .put_char(self.current_row, self.current_col, printable as char);
        self.current_col = self.current_col.saturating_add(1);
    }

    pub fn append_text(&mut self, value: &str) {
        for ch in value.bytes() {
            self.append_char(ch);
        }
    }

    pub fn switch_to_direct(&mut self) {
        self.usb_mode = UsbMode::Direct;
    }

    pub fn push_key(&mut self, key: u8) {
        self.key_queue.push_back(key);
    }

    pub fn is_key_ready(&self) -> bool {
        !self.key_queue.is_empty()
    }

    pub fn read_key(&mut self) -> u8 {
        self.key_queue.pop_front().unwrap_or(0)
    }
}
