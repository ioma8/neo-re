#![no_std]
#![no_main]

use core::panic::PanicInfo;

use alpha_neo_sdk::{NeoApplet, Status, dispatch, display, usb};

struct AlphaUsb;

impl NeoApplet for AlphaUsb {
    fn on_focus(_status: &mut u32) {
        display::clear();
        display::write_lines(
            2,
            &[
                "Now connect the NEO",
                "to your computer or",
                "smartphone via USB.",
            ],
        );
        display::idle_forever();
    }

    fn on_usb_plug(status: &mut u32) {
        usb::complete_hid_to_direct();
        usb::mark_direct_connected();
        *status = Status::UsbHandled as u32;
    }

    fn on_identity(status: &mut u32) {
        *status = 0xA130;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn alpha_usb_entry(message: u32, _param: u32, status_out: *mut u32) {
    // SAFETY: NEO OS calls the applet entrypoint with the message ABI and status pointer.
    unsafe { dispatch::<AlphaUsb>(message, status_out) };
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
