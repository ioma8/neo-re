use m68000::MemoryAccess;

use crate::applet_runner::RunnerError;
use crate::os3kapp::Os3kApp;

const MEMORY_SIZE: usize = 0x0080_0000;
pub(crate) const APPLET_RAM_BASE: u32 = 0x0050_0000;

#[derive(Debug)]
pub(crate) struct EmuMemory {
    bytes: Vec<u8>,
}

impl EmuMemory {
    pub(crate) fn load(app: &Os3kApp) -> Result<Self, RunnerError> {
        if app.image.len() > MEMORY_SIZE {
            return Err(RunnerError::ImageTooLarge);
        }
        let mut bytes = vec![0; MEMORY_SIZE];
        bytes[..app.image.len()].copy_from_slice(&app.image);
        let applet_ram_end = APPLET_RAM_BASE as usize
            + app.base_memory_size as usize
            + app.extra_memory_size as usize
            + 0x4000;
        if applet_ram_end > bytes.len() {
            return Err(RunnerError::ImageTooLarge);
        }
        Ok(Self { bytes })
    }
}

impl MemoryAccess for EmuMemory {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        self.bytes.get(addr as usize).copied()
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        let addr = addr as usize;
        Some(u16::from_be_bytes([
            *self.bytes.get(addr)?,
            *self.bytes.get(addr + 1)?,
        ]))
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        *self.bytes.get_mut(addr as usize)? = value;
        Some(())
    }

    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        let addr = addr as usize;
        let bytes = value.to_be_bytes();
        *self.bytes.get_mut(addr)? = bytes[0];
        *self.bytes.get_mut(addr + 1)? = bytes[1];
        Some(())
    }

    fn reset_instruction(&mut self) {}
}
