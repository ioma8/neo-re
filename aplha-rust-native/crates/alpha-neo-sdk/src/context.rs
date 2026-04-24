use crate::{Status, display, keyboard, usb};

pub struct Context {
    param: u32,
}

impl Context {
    #[must_use]
    pub const fn new(param: u32) -> Self {
        Self { param }
    }

    #[must_use]
    pub const fn param(&self) -> u32 {
        self.param
    }

    pub const fn screen(&mut self) -> Screen {
        Screen
    }

    pub const fn system(&mut self) -> System {
        System
    }

    pub const fn keyboard(&mut self) -> Keyboard {
        Keyboard
    }

    pub const fn usb(&mut self) -> Usb {
        Usb
    }
}

pub struct Screen;

impl Screen {
    #[allow(
        clippy::inline_always,
        reason = "required to avoid GOT-backed literal pointers in SmartApplet output"
    )]
    #[inline(always)]
    pub fn clear(self) {
        display::clear();
    }

    #[allow(
        clippy::inline_always,
        reason = "required to keep byte literals immediate and relocatable"
    )]
    #[inline(always)]
    pub fn write_bytes<const N: usize>(self, row: u8, bytes: [u8; N]) {
        display::write_bytes(row, bytes);
    }

    #[allow(
        clippy::inline_always,
        reason = "required to keep applet UI rendering as direct trap calls"
    )]
    #[inline(always)]
    pub fn write_slice(self, row: u8, bytes: &[u8]) {
        display::write_slice(row, bytes);
    }

    #[allow(
        clippy::inline_always,
        reason = "required to keep applet UI rendering as direct trap calls"
    )]
    #[inline(always)]
    pub fn write_prefix<const N: usize>(self, row: u8, bytes: &[u8; N], len: usize) {
        display::write_prefix(row, bytes, len);
    }

    #[allow(
        clippy::inline_always,
        reason = "required to keep applet UI rendering as direct trap calls"
    )]
    #[inline(always)]
    pub fn write_chars<const N: usize>(self, row: u8, start_col: u8, bytes: &[u8; N], len: usize) {
        display::write_chars(row, start_col, bytes, len);
    }

    #[allow(
        clippy::inline_always,
        reason = "required to keep applet UI rendering as direct trap calls"
    )]
    #[inline(always)]
    pub fn clear_row(self, row: u8) {
        display::clear_row(row);
    }

    #[allow(
        clippy::inline_always,
        reason = "required to keep applet rendering as direct trap calls"
    )]
    #[inline(always)]
    pub fn flush(self) {
        display::flush();
    }
}

pub struct System;

impl System {
    #[allow(
        clippy::inline_always,
        reason = "required to keep the focus handler as direct PC-relative code"
    )]
    #[inline(always)]
    pub fn idle_forever(self) -> ! {
        display::idle_forever();
    }

    #[allow(
        clippy::inline_always,
        reason = "required to keep interactive applet loops as direct OS yield calls"
    )]
    #[inline(always)]
    pub fn yield_once(self) {
        display::yield_once();
    }
}

pub struct Keyboard;

impl Keyboard {
    #[must_use]
    pub fn is_ready(self) -> bool {
        keyboard::is_ready()
    }

    #[must_use]
    pub fn read_key(self) -> u32 {
        keyboard::read_key()
    }

    #[must_use]
    pub fn read_byte(self) -> Option<u8> {
        keyboard::logical_key_to_byte(keyboard::read_key())
    }

    pub fn pump_events(self) {
        keyboard::pump_events();
    }
}

pub struct Usb;

impl Usb {
    #[must_use]
    pub const fn is_keyboard_connection(self) -> bool {
        true
    }

    #[allow(
        clippy::inline_always,
        reason = "required to keep USB callback control flow PC-relative"
    )]
    #[inline(always)]
    pub fn switch_to_direct(self) {
        usb::complete_hid_to_direct();
        usb::mark_direct_connected();
    }
}

pub struct Identity;

impl Identity {
    #[must_use]
    pub const fn applet_id(id: u16) -> Status {
        Status::raw(id as u32)
    }
}
