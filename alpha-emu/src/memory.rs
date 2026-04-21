use m68000::MemoryAccess;
use std::collections::BTreeMap;
use thiserror::Error;

use crate::firmware::FirmwareRuntime;
use crate::keyboard::{Keyboard, MatrixKey};
use crate::lcd::{Lcd, LcdSnapshot};

const MEMORY_SIZE: usize = 0x0080_0000;
const ROM_BASE: usize = 0x0040_0000;
const LOW_MMIO_START: u32 = 0x0000_f000;
const LOW_MMIO_END: u32 = 0x0001_0000;
const MMIO_BASE: u32 = 0xffff_0000;
const LCD_RIGHT_START: u32 = 0x0100_0000;
const LCD_RIGHT_END: u32 = 0x0100_0002;
const LCD_LEFT_START: u32 = 0x0100_8000;
const LCD_LEFT_END: u32 = 0x0100_8002;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("firmware image does not fit emulator memory")]
    ImageTooLarge,
}

#[derive(Clone, Debug)]
pub(crate) struct EmuMemory {
    bytes: Vec<u8>,
    lcd: Lcd,
    keyboard: Keyboard,
    mmio_bytes: BTreeMap<u32, u8>,
    mmio_accesses: Vec<String>,
    mmio_logging: bool,
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
            lcd: Lcd::new(),
            keyboard: Keyboard::default(),
            mmio_bytes: BTreeMap::new(),
            mmio_accesses: Vec::new(),
            mmio_logging: true,
        })
    }

    pub(crate) fn press_key(&mut self, key: MatrixKey) {
        self.keyboard.press(key);
    }

    pub(crate) fn release_key(&mut self, key: MatrixKey) {
        self.keyboard.release(key);
    }

    pub(crate) fn tap_key(&mut self, key: MatrixKey) {
        self.keyboard.tap(key);
    }

    pub(crate) fn tap_key_all_rows(&mut self, key: MatrixKey) {
        self.keyboard.tap_all_rows(key);
    }

    pub(crate) fn hold_small_rom_entry_chord(&mut self) {
        self.keyboard.hold_small_rom_entry_chord();
    }

    pub(crate) fn drain_mmio_accesses(&mut self) -> Vec<String> {
        std::mem::take(&mut self.mmio_accesses)
    }

    pub(crate) fn lcd_snapshot(&self) -> LcdSnapshot {
        self.lcd.snapshot()
    }

    pub(crate) fn set_mmio_logging(&mut self, enabled: bool) -> bool {
        let previous = self.mmio_logging;
        self.mmio_logging = enabled;
        previous
    }

    fn record_mmio(&mut self, access: impl Into<String>) {
        if !self.mmio_logging {
            return;
        }
        self.mmio_accesses.push(access.into());
        if self.mmio_accesses.len() > 4096 {
            self.mmio_accesses.remove(0);
        }
    }

    fn read_mmio_byte(&mut self, addr: u32) -> u8 {
        let addr = normalize_mmio(addr);
        if addr == 0xf419 {
            return self.keyboard.read_matrix_input();
        }
        *self.mmio_bytes.get(&addr).unwrap_or(&0)
    }

    fn read_mmio_word(&mut self, addr: u32) -> u16 {
        u16::from_be_bytes([self.read_mmio_byte(addr), self.read_mmio_byte(addr + 1)])
    }

    fn write_mmio_byte(&mut self, addr: u32, value: u8) {
        let normalized = normalize_mmio(addr);
        if normalized == 0xf411 {
            self.keyboard.select_row(value);
        } else if let Some(row) = gpio_keyboard_row_select(normalized, value) {
            self.keyboard.select_row(row);
        }
        match addr {
            0x0100_8000 => self.lcd.write_command(0, value),
            0x0100_8001 => self.lcd.write_data(0, value),
            0x0100_0000 => self.lcd.write_command(1, value),
            0x0100_0001 => self.lcd.write_data(1, value),
            _ => {}
        }
        self.mmio_bytes.insert(normalize_mmio(addr), value);
    }

    fn write_mmio_word(&mut self, addr: u32, value: u16) {
        let bytes = value.to_be_bytes();
        self.write_mmio_byte(addr, bytes[0]);
        self.write_mmio_byte(addr + 1, bytes[1]);
    }
}

impl MemoryAccess for EmuMemory {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        if is_mmio(addr) {
            let value = self.read_mmio_byte(addr);
            self.record_mmio(format!(
                "read8 0x{addr:08x}/0x{:04x}->0x{value:02x}",
                normalize_mmio(addr)
            ));
            return Some(value);
        }
        self.bytes.get(addr as usize).copied()
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        if is_mmio(addr) {
            let value = self.read_mmio_word(addr);
            self.record_mmio(format!(
                "read16 0x{addr:08x}/0x{:04x}->0x{value:04x}",
                normalize_mmio(addr)
            ));
            return Some(value);
        }
        let addr = addr as usize;
        Some(u16::from_be_bytes([
            *self.bytes.get(addr)?,
            *self.bytes.get(addr + 1)?,
        ]))
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        if is_mmio(addr) {
            self.write_mmio_byte(addr, value);
            self.record_mmio(format!(
                "write8 0x{addr:08x}/0x{:04x}=0x{value:02x}",
                normalize_mmio(addr)
            ));
            return Some(());
        }
        *self.bytes.get_mut(addr as usize)? = value;
        Some(())
    }

    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        if is_mmio(addr) {
            self.write_mmio_word(addr, value);
            self.record_mmio(format!(
                "write16 0x{addr:08x}/0x{:04x}=0x{value:04x}",
                normalize_mmio(addr)
            ));
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
    addr >= MMIO_BASE
        || (LOW_MMIO_START..LOW_MMIO_END).contains(&addr)
        || (LCD_LEFT_START..LCD_LEFT_END).contains(&addr)
        || (LCD_RIGHT_START..LCD_RIGHT_END).contains(&addr)
}

fn normalize_mmio(addr: u32) -> u32 {
    if addr >= MMIO_BASE {
        addr & 0xffff
    } else {
        addr
    }
}

fn gpio_keyboard_row_select(addr: u32, value: u8) -> Option<u8> {
    match (addr, value) {
        (0xf410, 0x80) => Some(0x00),
        (0xf410, 0x40) => Some(0x01),
        (0xf410, 0x20) => Some(0x02),
        (0xf410, 0x10) => Some(0x03),
        (0xf410, 0x08) => Some(0x04),
        (0xf410, 0x04) => Some(0x05),
        (0xf410, 0x02) => Some(0x06),
        (0xf410, 0x01) => Some(0x07),
        (0xf408, 0x20) => Some(0x09),
        (0xf440, 0x80) => Some(0x0a),
        (0xf440, 0x40) => Some(0x0b),
        (0xf440, 0x20) => Some(0x0c),
        (0xf440, 0x10) => Some(0x0d),
        (0xf440, 0x08) => Some(0x0e),
        (0xf440, 0x04) => Some(0x0f),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use m68000::MemoryAccess;

    use super::EmuMemory;
    use crate::firmware::FirmwareRuntime;

    #[test]
    fn sign_extended_mmio_aliases_share_register_state() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom_default()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;

        assert_eq!(memory.set_byte(0xffff_f429, 0x5a), Some(()));

        assert_eq!(memory.get_byte(0x0000_f429), Some(0x5a));
        Ok(())
    }

    #[test]
    fn lcd_controller_command_and_data_ports_are_mapped() -> Result<(), Box<dyn std::error::Error>>
    {
        let firmware = FirmwareRuntime::load_small_rom_default()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;

        assert_eq!(memory.set_byte(0x0100_8000, 0xb0), Some(()));
        assert_eq!(memory.set_byte(0x0100_8001, 0xff), Some(()));
        assert_eq!(memory.set_byte(0x0100_0000, 0xa3), Some(()));
        assert_eq!(memory.set_byte(0x0100_0001, 0x55), Some(()));

        assert_eq!(memory.get_byte(0x0100_8000), Some(0xb0));
        assert_eq!(memory.get_byte(0x0100_8001), Some(0xff));
        assert_eq!(memory.get_byte(0x0100_0000), Some(0xa3));
        assert_eq!(memory.get_byte(0x0100_0001), Some(0x55));
        Ok(())
    }

    #[test]
    fn lcd_port_0x01008000_maps_to_left_half() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom_default()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;

        assert_eq!(memory.set_byte(0x0100_8000, 0xb0), Some(()));
        assert_eq!(memory.set_byte(0x0100_8001, 0x01), Some(()));

        let snapshot = memory.lcd_snapshot();
        assert!(snapshot.pixels[0]);
        assert!(!snapshot.pixels[132]);
        Ok(())
    }

    #[test]
    fn keyboard_input_defaults_to_no_pressed_key() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom_default()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;

        assert_eq!(memory.get_byte(0xffff_f419), Some(0xff));
        Ok(())
    }

    #[test]
    fn gpio_row_select_exposes_nonzero_row_letter_keys() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom_default()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;
        let cases = [
            (0x3c, 0xf440, 0x20, 0xf7), // Q row 0x0c, col 3
            (0x3b, 0xf440, 0x40, 0xf7), // W row 0x0b, col 3
            (0x3a, 0xf440, 0x80, 0xf7), // E row 0x0a, col 3
            (0x3d, 0xf440, 0x10, 0xf7), // R row 0x0d, col 3
            (0x0d, 0xf440, 0x10, 0xfe), // T row 0x0d, col 0
        ];

        for (key, row_addr, row_value, expected_input) in cases {
            memory.press_key(crate::keyboard::MatrixKey::new(key));
            assert_eq!(memory.set_byte(row_addr, row_value), Some(()));
            assert_eq!(memory.get_byte(0xffff_f419), Some(expected_input));
            memory.release_key(crate::keyboard::MatrixKey::new(key));
        }
        Ok(())
    }
}
