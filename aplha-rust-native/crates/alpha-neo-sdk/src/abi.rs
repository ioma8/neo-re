#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Message {
    Init = 0x18,
    SetFocus = 0x19,
    Char = 0x20,
    Key = 0x21,
    Identity = 0x26,
    UsbMacInit = 0x10001,
    UsbPcInit = 0x20001,
    UsbPlug = 0x30001,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Ok = 0,
    Unhandled = 0x04,
    UsbHandled = 0x11,
}

/// Dispatch one NEO applet message to the Rust applet implementation.
///
/// # Safety
///
/// `status_out` must be null or a valid writable pointer provided by the NEO OS.
pub unsafe fn dispatch<A: crate::NeoApplet>(message: u32, status_out: *mut u32) {
    let mut status = 0_u32;
    match message {
        value if value == Message::SetFocus as u32 => A::on_focus(&mut status),
        value if value == Message::UsbPlug as u32 => A::on_usb_plug(&mut status),
        value if value == Message::Identity as u32 => A::on_identity(&mut status),
        _ => status = Status::Unhandled as u32,
    }

    if !status_out.is_null() {
        // SAFETY: NEO OS passes a valid status output pointer for applet message dispatch.
        unsafe { status_out.write_volatile(status) };
    }
}
