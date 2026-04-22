#![no_std]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

pub mod abi;
pub mod context;
pub mod display;
pub mod keyboard;
pub mod runtime;
pub mod usb;

pub mod prelude {
    pub use crate::{Applet, Context, Message, Status, export_applet, screen_line};
}

pub use abi::{Message, Status, dispatch};
pub use context::Context;

pub trait Applet {
    const ID: u16;

    fn on_focus(_ctx: &mut Context) -> Status {
        Status::OK
    }

    fn on_char(_ctx: &mut Context) -> Status {
        Status::UNHANDLED
    }

    fn on_key(_ctx: &mut Context) -> Status {
        Status::UNHANDLED
    }

    fn on_usb_plug(_ctx: &mut Context) -> Status {
        Status::UNHANDLED
    }

    fn on_usb_mac_init(_ctx: &mut Context) -> Status {
        Status::UNHANDLED
    }

    fn on_usb_pc_init(_ctx: &mut Context) -> Status {
        Status::UNHANDLED
    }

    fn on_identity(_ctx: &mut Context) -> Status {
        Status::raw(u32::from(Self::ID))
    }
}

#[macro_export]
macro_rules! export_applet {
    ($applet:ty) => {
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
        pub unsafe extern "C" fn alpha_neo_process_message(
            message: u32,
            param: u32,
            status_out: *mut u32,
        ) {
            // SAFETY: The generated m68k entry shell forwards the NEO-provided status pointer.
            unsafe { $crate::dispatch::<$applet>(message, param, status_out) };
        }
    };
}

#[macro_export]
macro_rules! screen_line {
    ($ctx:expr, $row:expr, $bytes:literal) => {{
        $ctx.screen().write_bytes($row, *$bytes);
    }};
}
