#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

mod forth;

#[cfg(not(test))]
use core::panic::PanicInfo;

use alpha_neo_sdk::prelude::*;
use forth::Repl;

struct ForthMini;

impl Applet for ForthMini {
    const ID: u16 = 0xA131;

    fn on_focus(ctx: &mut Context) -> Status {
        let mut repl = Repl::new();
        draw(ctx, &repl);
        loop {
            ctx.keyboard().pump_events();
            if ctx.keyboard().is_ready() {
                let raw_key = ctx.keyboard().read_key();
                if is_exit_key(raw_key) {
                    return Status::APPLET_EXIT;
                }
                handle_key(raw_key, &mut repl);
                draw(ctx, &repl);
            }
            ctx.system().yield_once();
        }
    }
}

fn is_exit_key(raw: u32) -> bool {
    matches!(raw, 0x044B | 0x084B)
}

fn handle_key(raw: u32, repl: &mut Repl) {
    let byte = (raw & 0xff) as u8;
    match byte {
        b'\r' | b'\n' => repl.eval_line(),
        0x08 | 0x7f => repl.backspace(),
        b' '..=b'~' => repl.push_byte(byte),
        _ => {}
    }
}

fn draw(ctx: &mut Context, repl: &Repl) {
    ctx.screen().clear_row(1);
    screen_line!(ctx, 1, b"Forth Mini");
    ctx.screen().clear_row(2);
    ctx.screen().write_slice(2, &repl.stack_line());
    ctx.screen().clear_row(3);
    ctx.screen().write_slice(3, &repl.output[0]);
    ctx.screen().clear_row(4);
    ctx.screen().write_slice(4, &repl.output[1]);
    ctx.screen().clear_row(5);
    ctx.screen().write_slice(5, &repl.output[2]);
    ctx.screen().clear_row(6);
    ctx.screen().write_slice(6, &repl.prompt_line());
    ctx.screen().flush();
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
