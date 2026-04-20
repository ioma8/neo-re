use m68000::MemoryAccess;
use thiserror::Error;

use crate::firmware::FirmwareRuntime;

const MEMORY_SIZE: usize = 0x0080_0000;
const ROM_BASE: usize = 0x0040_0000;
const LOW_MMIO_START: u32 = 0x0000_f000;
const LOW_MMIO_END: u32 = 0x0001_0000;
const MMIO_BASE: u32 = 0xffff_0000;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("firmware image does not fit emulator memory")]
    ImageTooLarge,
}

#[derive(Clone, Debug)]
pub(crate) struct EmuMemory {
    bytes: Vec<u8>,
    mmio_accesses: Vec<String>,
}

impl EmuMemory {
    pub(crate) fn load_small_rom(firmware: &FirmwareRuntime) -> Result<Self, MemoryError> {
        if firmware.image().len() > MEMORY_SIZE
            || ROM_BASE.saturating_add(firmware.image().len()) > MEMORY_SIZE
        {
            return Err(MemoryError::ImageTooLarge);
        }
        let mut bytes = vec![0; MEMORY_SIZE];
        bytes[..firmware.image().len()].copy_from_slice(firmware.image());
        bytes[ROM_BASE..ROM_BASE + firmware.image().len()].copy_from_slice(firmware.image());
        Ok(Self {
            bytes,
            mmio_accesses: Vec::new(),
        })
    }

    pub(crate) fn drain_mmio_accesses(&mut self) -> Vec<String> {
        std::mem::take(&mut self.mmio_accesses)
    }

    fn record_mmio(&mut self, access: impl Into<String>) {
        self.mmio_accesses.push(access.into());
        if self.mmio_accesses.len() > 64 {
            self.mmio_accesses.remove(0);
        }
    }
}

impl MemoryAccess for EmuMemory {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        if is_mmio(addr) {
            self.record_mmio(format!("read8 0x{addr:08x}"));
            return Some(0);
        }
        self.bytes.get(addr as usize).copied()
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        if is_mmio(addr) {
            self.record_mmio(format!("read16 0x{addr:08x}"));
            return Some(0);
        }
        let addr = addr as usize;
        Some(u16::from_be_bytes([
            *self.bytes.get(addr)?,
            *self.bytes.get(addr + 1)?,
        ]))
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        if is_mmio(addr) {
            self.record_mmio(format!("write8 0x{addr:08x}=0x{value:02x}"));
            return Some(());
        }
        *self.bytes.get_mut(addr as usize)? = value;
        Some(())
    }

    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        if is_mmio(addr) {
            self.record_mmio(format!("write16 0x{addr:08x}=0x{value:04x}"));
            return Some(());
        }
        let addr = addr as usize;
        let bytes = value.to_be_bytes();
        *self.bytes.get_mut(addr)? = bytes[0];
        *self.bytes.get_mut(addr + 1)? = bytes[1];
        Some(())
    }

    fn reset_instruction(&mut self) {}
}

fn is_mmio(addr: u32) -> bool {
    addr >= MMIO_BASE || (LOW_MMIO_START..LOW_MMIO_END).contains(&addr)
}
