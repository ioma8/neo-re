use crate::{Applet, Context};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Message {
    Init,
    SetFocus,
    Char,
    Key,
    Identity,
    UsbMacInit,
    UsbPcInit,
    UsbPlug,
    Unknown(u32),
}

impl Message {
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0x18 => Self::Init,
            0x19 => Self::SetFocus,
            0x20 => Self::Char,
            0x21 => Self::Key,
            0x26 => Self::Identity,
            0x1_0001 => Self::UsbMacInit,
            0x2_0001 => Self::UsbPcInit,
            0x3_0001 => Self::UsbPlug,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status(u32);

impl Status {
    pub const OK: Self = Self(0);
    pub const UNHANDLED: Self = Self(0x04);
    pub const APPLET_EXIT: Self = Self(0x07);
    pub const USB_HANDLED: Self = Self(0x11);

    #[must_use]
    pub const fn raw(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

/// Dispatch one NEO applet message to the Rust applet implementation.
///
/// # Safety
///
/// `status_out` must be null or a valid writable pointer provided by the NEO OS.
pub unsafe fn dispatch<A: Applet>(message: u32, param: u32, status_out: *mut u32) {
    let mut ctx = Context::new(param);
    let status = match Message::from_raw(message) {
        Message::SetFocus => A::on_focus(&mut ctx),
        Message::Char => A::on_char(&mut ctx),
        Message::Key => A::on_key(&mut ctx),
        Message::UsbPlug => A::on_usb_plug(&mut ctx),
        Message::UsbMacInit => A::on_usb_mac_init(&mut ctx),
        Message::UsbPcInit => A::on_usb_pc_init(&mut ctx),
        Message::Identity => A::on_identity(&mut ctx),
        _ => Status::UNHANDLED,
    };

    if !status_out.is_null() {
        // SAFETY: NEO OS passes a valid status output pointer for applet message dispatch.
        unsafe { status_out.write_volatile(status.as_raw()) };
    }
}
