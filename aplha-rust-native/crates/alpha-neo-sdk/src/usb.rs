type OsFn0 = unsafe extern "C" fn();
type OsFn1 = unsafe extern "C" fn(u32);

const COMPLETE_USB_STAGE_1: usize = 0x0041_F9A0;
const DELAY_MS: usize = 0x0042_4780;
const COMPLETE_USB_STAGE_2: usize = 0x0044_044E;
const COMPLETE_USB_STAGE_3: usize = 0x0044_047C;
const MARK_DIRECT_CONNECTED: usize = 0x0041_0B26;
const USB_DIRECT_FLAG: *mut u8 = 0x0001_3CF9 as *mut u8;

pub fn complete_hid_to_direct() {
    call1(COMPLETE_USB_STAGE_1, 1);
    call1(DELAY_MS, 100);
    // SAFETY: This mirrors the validated Alpha USB applet write to the NEO USB mode flag.
    unsafe { USB_DIRECT_FLAG.write_volatile(1) };
    call0(COMPLETE_USB_STAGE_2);
    call1(DELAY_MS, 100);
    call0(COMPLETE_USB_STAGE_3);
}

pub fn mark_direct_connected() {
    call0(MARK_DIRECT_CONNECTED);
}

fn call0(address: usize) {
    // SAFETY: Addresses are fixed NEO OS entrypoints validated by the working Alpha USB applet.
    let function: OsFn0 = unsafe { core::mem::transmute(address) };
    // SAFETY: The selected OS entrypoint takes no arguments.
    unsafe { function() };
}

fn call1(address: usize, arg: u32) {
    // SAFETY: Addresses are fixed NEO OS entrypoints validated by the working Alpha USB applet.
    let function: OsFn1 = unsafe { core::mem::transmute(address) };
    // SAFETY: The selected OS entrypoint takes one scalar argument.
    unsafe { function(arg) };
}
