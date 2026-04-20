#![no_std]

use core::panic::PanicInfo;

use alpha_neo_sdk::SDK_VERSION;

#[unsafe(no_mangle)]
pub extern "C" fn alpha_usb_entry() -> u16 {
    SDK_VERSION
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
