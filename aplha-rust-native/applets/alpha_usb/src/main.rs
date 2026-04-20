#![no_std]
#![no_main]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

use core::panic::PanicInfo;

use alpha_neo_sdk::prelude::*;

struct AlphaUsb;

impl Applet for AlphaUsb {
    const ID: u16 = 0xA130;

    fn on_focus(ctx: &mut Context) -> Status {
        ctx.screen().clear();
        screen_line!(ctx, 2, b"Now connect the NEO");
        screen_line!(ctx, 3, b"to your computer or");
        screen_line!(ctx, 4, b"smartphone via USB.");
        ctx.system().idle_forever()
    }

    fn on_usb_plug(ctx: &mut Context) -> Status {
        if ctx.usb().is_keyboard_connection() {
            ctx.usb().switch_to_direct();
            Status::USB_HANDLED
        } else {
            Status::UNHANDLED
        }
    }

    fn on_usb_mac_init(_ctx: &mut Context) -> Status {
        Status::USB_HANDLED
    }

    fn on_usb_pc_init(_ctx: &mut Context) -> Status {
        Status::USB_HANDLED
    }
}

export_applet!(AlphaUsb);

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
