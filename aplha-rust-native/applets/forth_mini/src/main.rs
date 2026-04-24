#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

mod forth;

#[cfg(target_arch = "m68k")]
use core::arch::global_asm;
#[cfg(not(test))]
use core::panic::PanicInfo;

use alpha_neo_sdk::prelude::*;
use forth::Repl;

struct ForthMini;

#[cfg(target_arch = "m68k")]
global_asm!(
    r#"
    .global alpha_neo_applet_memory_base
alpha_neo_applet_memory_base:
    move.l %a5,%a0
    adda.l #0x300,%a0
    rts
    "#
);

#[cfg(target_arch = "m68k")]
unsafe extern "C" {
    fn alpha_neo_applet_memory_base() -> *mut Repl;
}

impl Applet for ForthMini {
    const ID: u16 = 0xA131;

    fn on_focus(ctx: &mut Context) -> Status {
        with_repl(|repl| {
            *repl = Repl::new();
            ctx.screen().clear();
            draw(ctx, repl);
        });
        Status::OK
    }

    fn on_char(ctx: &mut Context) -> Status {
        let byte = (ctx.param() & 0xff) as u8;
        if !handle_input_byte(byte) {
            return Status::UNHANDLED;
        }
        with_repl(|repl| draw(ctx, repl));
        Status::OK
    }

    fn on_key(ctx: &mut Context) -> Status {
        let raw_key = ctx.param();
        if is_exit_key(raw_key) {
            return Status::APPLET_EXIT;
        }
        let Some(byte) = alpha_neo_sdk::keyboard::logical_key_to_byte(raw_key) else {
            return Status::UNHANDLED;
        };
        if !handle_input_key_byte(byte) {
            return Status::UNHANDLED;
        }
        with_repl(|repl| draw(ctx, repl));
        Status::OK
    }
}

fn is_exit_key(raw: u32) -> bool {
    matches!(raw, 0x044B | 0x084B)
}

fn handle_input_byte(byte: u8) -> bool {
    if !is_char_input_byte(byte) {
        return false;
    }
    with_repl(|repl| match byte {
        b' '..=b'~' => repl.accept_printable(byte),
        b'\r' | b'\n' => repl.enter(),
        0x08 | 0x7f => repl.backspace(),
        _ => {}
    });
    true
}

fn handle_input_key_byte(byte: u8) -> bool {
    if !is_key_input_byte(byte) {
        return false;
    }
    with_repl(|repl| match byte {
        b' '..=b'~' => repl.accept_printable(byte),
        0x08 | 0x7f => repl.backspace(),
        _ => {}
    });
    true
}

fn handle_polled_byte(byte: u8) -> bool {
    if !matches!(byte, b' '..=b'~' | 0x08 | 0x7f) {
        return false;
    }
    with_repl(|repl| match byte {
        b' '..=b'~' => repl.accept_printable(byte),
        0x08 | 0x7f => repl.backspace(),
        _ => {}
    });
    true
}

const fn is_char_input_byte(byte: u8) -> bool {
    matches!(byte, b' '..=b'~' | b'\r' | b'\n' | 0x08 | 0x7f)
}

const fn is_key_input_byte(byte: u8) -> bool {
    matches!(byte, b' '..=b'~' | 0x08 | 0x7f)
}

fn with_repl(callback: impl FnOnce(&mut Repl)) {
    // SAFETY: The applet owns a single REPL state block and firmware dispatch is single-threaded.
    unsafe {
        callback(&mut *repl_ptr());
    }
}

#[cfg(target_arch = "m68k")]
fn repl_ptr() -> *mut Repl {
    // SAFETY: Returns the applet-owned writable memory block reserved by the firmware.
    unsafe { alpha_neo_applet_memory_base() }
}

#[cfg(not(target_arch = "m68k"))]
fn repl_ptr() -> *mut Repl {
    core::ptr::null_mut()
}

fn draw(ctx: &mut Context, repl: &Repl) {
    let line1 = repl.line_transcript(1);
    let line2 = repl.line_transcript(2);
    let prompt = repl.line_prompt();
    ctx.screen().write_slice(1, &line1);
    ctx.screen().write_slice(2, &line2);
    ctx.screen().write_slice(3, &prompt);
    ctx.screen().write_slice(4, &prompt);
    ctx.screen().flush();
}

#[cfg(test)]
mod tests {
    use super::{is_char_input_byte, is_key_input_byte};

    #[test]
    fn on_key_handles_printable_prompt_input() {
        assert!(is_key_input_byte(b'1'));
        assert!(is_key_input_byte(b'x'));
        assert!(is_key_input_byte(0x08));
    }

    #[test]
    fn on_key_does_not_claim_enter() {
        assert!(!is_key_input_byte(b'\r'));
        assert!(!is_key_input_byte(b'\n'));
    }

    #[test]
    fn on_char_still_handles_enter() {
        assert!(is_char_input_byte(b'\r'));
        assert!(is_char_input_byte(b'\n'));
    }
}

#[cfg(target_arch = "m68k")]
#[unsafe(no_mangle)]
pub extern "C" fn __mulsi3(lhs: i32, rhs: i32) -> i32 {
    let negative = (lhs < 0) ^ (rhs < 0);
    let mut a = lhs.unsigned_abs();
    let mut b = rhs.unsigned_abs();
    let mut result = 0_u32;
    while b != 0 {
        if b & 1 != 0 {
            result = result.wrapping_add(a);
        }
        a = a.wrapping_shl(1);
        b >>= 1;
    }
    if negative {
        result.wrapping_neg().cast_signed()
    } else {
        result.cast_signed()
    }
}

#[cfg(target_arch = "m68k")]
fn udivmod32(numerator: u32, denominator: u32) -> (u32, u32) {
    if denominator == 0 {
        return (0, numerator);
    }
    let mut quotient = 0u32;
    let mut remainder = 0u32;
    let mut bit = 32u32;
    while bit != 0 {
        bit -= 1;
        remainder = remainder.wrapping_shl(1);
        remainder |= (numerator >> bit) & 1;
        if remainder >= denominator {
            remainder = remainder.wrapping_sub(denominator);
            quotient |= 1u32 << bit;
        }
    }
    (quotient, remainder)
}

#[cfg(target_arch = "m68k")]
#[unsafe(no_mangle)]
pub extern "C" fn __udivsi3(numerator: u32, denominator: u32) -> u32 {
    udivmod32(numerator, denominator).0
}

#[cfg(target_arch = "m68k")]
#[unsafe(no_mangle)]
pub extern "C" fn __umodsi3(numerator: u32, denominator: u32) -> u32 {
    udivmod32(numerator, denominator).1
}

#[cfg(target_arch = "m68k")]
#[unsafe(no_mangle)]
pub extern "C" fn __modsi3(lhs: i32, rhs: i32) -> i32 {
    if rhs == 0 {
        return lhs;
    }
    let negative = lhs < 0;
    let (.., remainder) = udivmod32(lhs.unsigned_abs(), rhs.unsigned_abs());
    if negative {
        remainder.wrapping_neg().cast_signed()
    } else {
        remainder.cast_signed()
    }
}

#[cfg(target_arch = "m68k")]
#[unsafe(no_mangle)]
pub extern "C" fn __divsi3(lhs: i32, rhs: i32) -> i32 {
    if rhs == 0 {
        return 0;
    }
    let negative = (lhs < 0) ^ (rhs < 0);
    let (quotient, _) = udivmod32(lhs.unsigned_abs(), rhs.unsigned_abs());
    if negative {
        quotient.wrapping_neg().cast_signed()
    } else {
        quotient.cast_signed()
    }
}

#[cfg(target_arch = "m68k")]
core::arch::global_asm!(
    r#"
    .section .text.alpha_usb_entry,"ax"
    .global alpha_usb_entry
alpha_usb_entry:
    move.l 12(%sp),-(%sp)
    move.l 12(%sp),-(%sp)
    move.l 12(%sp),-(%sp)
    .short 0x4EBA
    .short alpha_neo_process_message - .
    lea 12(%sp),%sp
    rts
    .text
    "#
);

#[unsafe(no_mangle)]
/// # Safety
///
/// Called by the generated m68k entry shell with a status pointer supplied by the NEO OS.
/// The pointer must be null or valid for one volatile `u32` write.
pub unsafe extern "C" fn alpha_neo_process_message(message: u32, param: u32, status_out: *mut u32) {
    if message == 0 {
        let mut ctx = Context::new(0);
        ctx.keyboard().pump_events();
        while ctx.keyboard().is_ready() {
            let Some(byte) = ctx.keyboard().read_byte() else {
                break;
            };
            let _ = handle_polled_byte(byte);
        }
        with_repl(|repl| draw(&mut ctx, repl));
        if !status_out.is_null() {
            unsafe { status_out.write_volatile(Status::OK.as_raw()) };
        }
        return;
    }
    unsafe { alpha_neo_sdk::dispatch::<ForthMini>(message, param, status_out) };
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
