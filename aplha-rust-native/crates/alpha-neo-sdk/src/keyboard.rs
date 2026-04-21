#[cfg(target_arch = "m68k")]
use core::arch::global_asm;
#[cfg(target_arch = "m68k")]
use core::sync::atomic::{Ordering, compiler_fence};

#[cfg(target_arch = "m68k")]
global_asm!(
    r#"
    .global alpha_neo_read_key_code
alpha_neo_read_key_code:
    .short 0xA094
    rts

    .global alpha_neo_is_key_ready
alpha_neo_is_key_ready:
    .short 0xA09C
    rts

    .global alpha_neo_pump_ui_events
alpha_neo_pump_ui_events:
    .short 0xA0A4
    rts
    "#
);

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn alpha_neo_read_key_code() -> u32;
    fn alpha_neo_is_key_ready() -> u32;
    fn alpha_neo_pump_ui_events();
}

#[must_use]
#[allow(
    clippy::inline_always,
    reason = "required to avoid 68020 long-branch wrapper thunks in SmartApplet output"
)]
#[inline(always)]
pub fn is_ready() -> bool {
    #[cfg(not(target_arch = "m68k"))]
    {
        false
    }
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS key-ready trap and reads its scalar return value.
    unsafe {
        let value = alpha_neo_is_key_ready();
        compiler_fence(Ordering::SeqCst);
        value != 0
    }
}

#[must_use]
#[allow(
    clippy::inline_always,
    reason = "required to avoid 68020 long-branch wrapper thunks in SmartApplet output"
)]
#[inline(always)]
pub fn read_key() -> u32 {
    #[cfg(not(target_arch = "m68k"))]
    {
        0
    }
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS read-key trap and reads its scalar return value.
    unsafe {
        let value = alpha_neo_read_key_code();
        compiler_fence(Ordering::SeqCst);
        value
    }
}

#[allow(
    clippy::inline_always,
    reason = "required to avoid 68020 long-branch wrapper thunks in SmartApplet output"
)]
#[inline(always)]
pub fn pump_events() {
    #[cfg(not(target_arch = "m68k"))]
    {}
    #[cfg(target_arch = "m68k")]
    // SAFETY: Calls the NEO OS event pump trap with no borrowed Rust state crossing the ABI.
    unsafe {
        alpha_neo_pump_ui_events();
        compiler_fence(Ordering::SeqCst);
    };
}
