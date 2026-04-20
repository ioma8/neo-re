#![no_std]
#![cfg_attr(target_arch = "m68k", feature(asm_experimental_arch))]

pub mod abi;
pub mod display;
pub mod usb;

pub use abi::{Message, Status, dispatch};

pub trait NeoApplet {
    fn on_focus(status: &mut u32) {
        *status = Status::Ok as u32;
    }

    fn on_usb_plug(status: &mut u32) {
        *status = Status::Unhandled as u32;
    }

    fn on_identity(status: &mut u32) {
        *status = 0;
    }
}
