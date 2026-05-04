pub mod firmware;
pub mod firmware_session;
pub mod gui;
pub mod keyboard;
pub mod lcd;
pub mod text_screen;
mod memory;
pub mod recovery_seed;

pub(crate) fn read_be32(bytes: &[u8], addr: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *bytes.get(addr)?,
        *bytes.get(addr + 1)?,
        *bytes.get(addr + 2)?,
        *bytes.get(addr + 3)?,
    ]))
}

pub(crate) fn write_be32(bytes: &mut [u8], addr: usize, value: u32) {
    debug_assert!(
        addr + 4 <= bytes.len(),
        "write_be32 out of bounds: addr=0x{addr:x}, len={}",
        bytes.len()
    );
    let value = value.to_be_bytes();
    bytes[addr..addr + 4].copy_from_slice(&value);
}
