#[cfg(target_arch = "m68k")]
use core::arch::global_asm;

#[cfg(target_arch = "m68k")]
global_asm!(
    r#"
    .global alpha_neo_complete_hid_to_direct
alpha_neo_complete_hid_to_direct:
    .short 0x42A7
    .short 0x42A7
    .short 0x4878, 0x0001
    .short 0x4EB9
    .long  0x0041F9A0
    .short 0x4878, 0x0064
    .short 0x4EB9
    .long  0x00424780
    .short 0x13FC, 0x0001, 0x0001, 0x3CF9
    .short 0x4EB9
    .long  0x0044044E
    .short 0x4878, 0x0064
    .short 0x4EB9
    .long  0x00424780
    .short 0x4FEF, 0x0014
    .short 0x4EB9
    .long  0x0044047C
    rts

    .global alpha_neo_mark_direct_connected
alpha_neo_mark_direct_connected:
    .short 0x4EB9
    .long  0x00410B26
    rts
    "#
);

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn alpha_neo_complete_hid_to_direct();
    fn alpha_neo_mark_direct_connected();
}

pub fn complete_hid_to_direct() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the validated Alpha USB absolute OS sequence.
    unsafe {
        alpha_neo_complete_hid_to_direct();
    }
}

pub fn mark_direct_connected() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the validated NEO OS direct-connected marker.
    unsafe {
        alpha_neo_mark_direct_connected();
    }
}
