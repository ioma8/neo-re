#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

mod editor;
mod storage;

#[cfg(target_arch = "m68k")]
use core::arch::global_asm;
#[cfg(not(test))]
use core::panic::PanicInfo;

use alpha_neo_sdk::prelude::*;
use editor::{Document, SCREEN_COLS, SCREEN_ROWS, SlotNavigation, Viewport};

struct BasicWriter;

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
    fn alpha_neo_applet_memory_base() -> *mut AppState;
}

impl Applet for BasicWriter {
    const ID: u16 = 0xA132;

    fn on_focus(ctx: &mut Context) -> Status {
        with_state(|state| {
            *state = AppState::new();
            let _ = storage::load_slot(state.active_slot, &mut state.document);
            ctx.screen().clear();
            draw_document(ctx, &state.document);
        });
        Status::OK
    }

    fn on_char(ctx: &mut Context) -> Status {
        let byte = (ctx.param() & 0xff) as u8;
        with_state(|state| {
            let _ = apply_action(
                state,
                match byte {
                    b'\r' | b'\n' => InputAction::Insert(b'\n'),
                    0x08 | 0x7f => InputAction::Backspace,
                    b' '..=b'~' => InputAction::Insert(byte),
                    _ => InputAction::Ignore,
                },
            );
            let _ = storage::save_slot(state.active_slot, &state.document);
            draw_document(ctx, &state.document);
        });
        Status::OK
    }

    fn on_key(ctx: &mut Context) -> Status {
        if is_exit_key(ctx.param()) {
            return Status::APPLET_EXIT;
        }
        with_state(|state| {
            let _ = apply_action(state, input_action_for_key(ctx.param()));
            let _ = storage::save_slot(state.active_slot, &state.document);
            draw_document(ctx, &state.document);
        });
        Status::OK
    }
}

fn is_exit_key(raw: u32) -> bool {
    matches!(raw, 0x044B | 0x084B | 0x29)
}

export_applet!(BasicWriter);

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputAction {
    Insert(u8),
    Backspace,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    SwitchFile(usize),
    Ignore,
}

#[repr(C)]
struct AppState {
    active_slot: usize,
    document: Document,
    navigation: SlotNavigation,
}

impl AppState {
    const fn new() -> Self {
        Self {
            active_slot: 1,
            document: Document::new(),
            navigation: SlotNavigation::new(),
        }
    }
}

fn apply_action(state: &mut AppState, action: InputAction) -> bool {
    match action {
        InputAction::Insert(byte) => state.document.insert_byte(byte),
        InputAction::Backspace => state.document.backspace(),
        InputAction::MoveLeft => {
            state.document.move_left();
            false
        }
        InputAction::MoveRight => {
            state.document.move_right();
            false
        }
        InputAction::MoveUp => {
            state.document.move_up();
            false
        }
        InputAction::MoveDown => {
            state.document.move_down();
            false
        }
        InputAction::SwitchFile(slot) => {
            let _ = storage::save_slot(state.active_slot, &state.document);
            state
                .navigation
                .store(state.active_slot, state.document.cursor(), state.document.viewport());
            state.active_slot = slot;
            if let Some((cursor, viewport)) = state.navigation.restore(slot) {
                let _ = storage::load_slot(slot, &mut state.document);
                state.document.set_cursor(cursor);
                state.document.set_viewport(viewport);
            } else {
                state.document = Document::new();
                let _ = storage::load_slot(slot, &mut state.document);
                state.document.set_viewport(Viewport { row: 0 });
            }
            false
        }
        InputAction::Ignore => false,
    }
}

fn with_state(callback: impl FnOnce(&mut AppState)) {
    // SAFETY: The applet owns a single state block and firmware dispatch is single-threaded.
    unsafe {
        callback(&mut *state_ptr());
    }
}

#[cfg(target_arch = "m68k")]
fn state_ptr() -> *mut AppState {
    // SAFETY: Returns the applet-owned writable memory block reserved by the firmware.
    unsafe { alpha_neo_applet_memory_base() }
}

#[cfg(not(target_arch = "m68k"))]
fn state_ptr() -> *mut AppState {
    core::ptr::null_mut()
}

fn input_action_for_key(raw: u32) -> InputAction {
    match raw & 0xff {
        0x49 => InputAction::MoveLeft,
        0x4a => InputAction::MoveRight,
        0x4b => InputAction::MoveUp,
        0x0d => InputAction::MoveDown,
        0x2d => InputAction::SwitchFile(1),
        0x2c => InputAction::SwitchFile(2),
        0x04 => InputAction::SwitchFile(3),
        0x0f => InputAction::SwitchFile(4),
        0x0e => InputAction::SwitchFile(5),
        0x0a => InputAction::SwitchFile(6),
        0x01 => InputAction::SwitchFile(7),
        0x27 => InputAction::SwitchFile(8),
        _ => match alpha_neo_sdk::keyboard::logical_key_to_byte(raw) {
            Some(0x08 | 0x7f) => InputAction::Backspace,
            Some(byte @ b' '..=b'~') => InputAction::Insert(byte),
            Some(b'\r' | b'\n') => InputAction::Insert(b'\n'),
            _ => InputAction::Ignore,
        },
    }
}

fn draw_document(ctx: &mut Context, document: &Document) {
    let mut row_index = 0;
    while row_index < SCREEN_ROWS {
        let mut row = [b' '; SCREEN_COLS];
        document.render_row(row_index, &mut row);
        ctx.screen().write_prefix((row_index + 1) as u8, &row, SCREEN_COLS);
        row_index += 1;
    }
    ctx.screen().flush();
}

#[cfg(test)]
mod tests {
    use super::{InputAction, input_action_for_key};

    #[test]
    fn maps_arrow_logical_keys_to_navigation() {
        assert_eq!(input_action_for_key(0x49), InputAction::MoveLeft);
        assert_eq!(input_action_for_key(0x4a), InputAction::MoveRight);
        assert_eq!(input_action_for_key(0x4b), InputAction::MoveUp);
        assert_eq!(input_action_for_key(0x0d), InputAction::MoveDown);
    }

    #[test]
    fn maps_file_logical_keys_to_slots() {
        assert_eq!(input_action_for_key(0x2d), InputAction::SwitchFile(1));
        assert_eq!(input_action_for_key(0x2c), InputAction::SwitchFile(2));
        assert_eq!(input_action_for_key(0x04), InputAction::SwitchFile(3));
        assert_eq!(input_action_for_key(0x0f), InputAction::SwitchFile(4));
        assert_eq!(input_action_for_key(0x0e), InputAction::SwitchFile(5));
        assert_eq!(input_action_for_key(0x0a), InputAction::SwitchFile(6));
        assert_eq!(input_action_for_key(0x01), InputAction::SwitchFile(7));
        assert_eq!(input_action_for_key(0x27), InputAction::SwitchFile(8));
    }

    #[test]
    fn maps_printable_and_backspace_keys() {
        assert_eq!(input_action_for_key(0x38), InputAction::Insert(b'1'));
        assert_eq!(input_action_for_key(0x40), InputAction::Insert(b'\n'));
        assert_eq!(input_action_for_key(0x03), InputAction::Backspace);
    }

    #[test]
    fn recognizes_applet_exit_keys() {
        assert!(super::is_exit_key(0x044B));
        assert!(super::is_exit_key(0x084B));
        assert!(super::is_exit_key(0x29));
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
