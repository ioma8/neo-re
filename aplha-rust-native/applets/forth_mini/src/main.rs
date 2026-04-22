#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

mod forth;

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(target_arch = "m68k")]
use core::arch::global_asm;

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
        handle_input_byte(ctx, (ctx.param() & 0xff) as u8);
        Status::OK
    }

    fn on_key(ctx: &mut Context) -> Status {
        let raw_key = ctx.param();
        if is_exit_key(raw_key) {
            return Status::APPLET_EXIT;
        }
        if let Some(byte) = alpha_neo_sdk::keyboard::logical_key_to_byte(raw_key) {
            handle_input_byte(ctx, byte);
        }
        Status::OK
    }
}

fn is_exit_key(raw: u32) -> bool {
    matches!(raw, 0x044B | 0x084B)
}

fn handle_byte(byte: u8, repl: &mut Repl) {
    match byte {
        b'\r' | b'\n' => repl.eval_line(),
        0x08 | 0x7f => repl.backspace(),
        b' '..=b'~' => repl.push_byte(byte),
        _ => {}
    }
}

fn handle_input_byte(ctx: &mut Context, byte: u8) {
    with_repl(|repl| {
        handle_byte(byte, repl);
        draw(ctx, repl);
    });
}

fn with_repl(callback: impl FnOnce(&mut Repl)) {
    // SAFETY: A5 points at this applet's writable memory block for the active callback.
    unsafe {
        callback(&mut *repl_ptr());
    };
}

#[cfg(target_arch = "m68k")]
fn repl_ptr() -> *mut Repl {
    // SAFETY: Reads the applet memory base provided in A5 by the NEO OS.
    unsafe { alpha_neo_applet_memory_base() }
}

#[cfg(not(target_arch = "m68k"))]
fn repl_ptr() -> *mut Repl {
    core::ptr::null_mut()
}

fn draw(ctx: &mut Context, repl: &Repl) {
    ctx.screen().write_slice(1, &header_line());
    ctx.screen().write_slice(2, &repl.stack_line());
    ctx.screen().write_slice(3, &repl.output[2]);
    ctx.screen().write_slice(4, &repl.prompt_line());
    ctx.screen().flush();
}

fn header_line() -> [u8; 28] {
    let mut line = [b' '; 28];
    line[0] = b'F';
    line[1] = b'o';
    line[2] = b'r';
    line[3] = b't';
    line[4] = b'h';
    line[5] = b' ';
    line[6] = b'M';
    line[7] = b'i';
    line[8] = b'n';
    line[9] = b'i';
    line
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

export_applet!(ForthMini);

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}
