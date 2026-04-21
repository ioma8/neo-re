use m68000::MemoryAccess;
use std::collections::BTreeMap;
use std::path::Path;
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
const ASIC_REGISTER_START: u32 = 0x0200_0000;
const ASIC_REGISTER_END: u32 = 0x0200_0008;
const STOCK_APPLET_BASE: usize = 0x0047_0000;

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
    timebase_counter: u32,
    timer_counter: u16,
    applet_storage_end: Option<u32>,
    f411_row_select_enabled: bool,
}

impl EmuMemory {
    pub(crate) fn load_small_rom(firmware: &FirmwareRuntime) -> Result<Self, MemoryError> {
        if firmware.image().len() > MEMORY_SIZE {
            return Err(MemoryError::ImageTooLarge);
        }
        let mut bytes = vec![0; MEMORY_SIZE];
        if firmware.is_neo_system_image() {
            let start = 0x0041_0000usize;
            let end = start.saturating_add(firmware.image().len());
            if end > MEMORY_SIZE {
                return Err(MemoryError::ImageTooLarge);
            }
            bytes[start..end].copy_from_slice(firmware.image());
            if let Some(system_package) = find_last_system_package(firmware.image()) {
                write_be32(&mut bytes, 0x0000_0e0a, 0x0041_0000 + system_package as u32);
            }
            load_stock_applets(&mut bytes);
            write_bytes(&mut bytes, 0x0000_0400, b"I am not corrupted!\0");
            write_be16(&mut bytes, 0x0000_35f4, 0x2675);
            write_be32(&mut bytes, 0x0000_35e2, 1);
            write_be16(&mut bytes, 0x0000_35e6, 0xa000);
            write_be32(&mut bytes, 0x0000_35ec, 1);
            write_be16(&mut bytes, 0x0000_35f8, 0x2675);
            write_be32(&mut bytes, 0x0000_7dd8, 0x0000_0830);
        } else {
            if ROM_BASE.saturating_add(firmware.image().len()) > MEMORY_SIZE {
                return Err(MemoryError::ImageTooLarge);
            }
            bytes[..firmware.image().len()].copy_from_slice(firmware.image());
            bytes[ROM_BASE..ROM_BASE + firmware.image().len()].copy_from_slice(firmware.image());
        }
        let is_neo_system_image = firmware.is_neo_system_image();
        let applet_storage_end = find_applet_storage_end(&bytes);
        Ok(Self {
            bytes,
            lcd: Lcd::new(),
            keyboard: Keyboard::default(),
            mmio_bytes: BTreeMap::new(),
            mmio_accesses: Vec::new(),
            mmio_logging: true,
            timebase_counter: 0,
            timer_counter: 0,
            applet_storage_end,
            f411_row_select_enabled: !is_neo_system_image,
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

    pub(crate) fn tap_key_long(&mut self, key: MatrixKey) {
        self.keyboard.tap_long(key);
    }

    pub(crate) fn tap_key_all_rows(&mut self, key: MatrixKey) {
        self.keyboard.tap_all_rows(key);
    }

    pub(crate) fn hold_small_rom_entry_chord(&mut self) {
        self.keyboard.hold_small_rom_entry_chord();
    }

    pub(crate) fn hold_boot_keys_all_rows(&mut self, keys: &[MatrixKey], reads: usize) {
        self.keyboard.hold_keys_all_rows(keys, reads);
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

    pub(crate) fn peek_word(&self, addr: u32) -> Option<u16> {
        let addr = addr as usize;
        Some(u16::from_be_bytes([
            *self.bytes.get(addr)?,
            *self.bytes.get(addr + 1)?,
        ]))
    }

    pub(crate) fn peek_long(&self, addr: u32) -> Option<u32> {
        let addr = addr as usize;
        Some(u32::from_be_bytes([
            *self.bytes.get(addr)?,
            *self.bytes.get(addr + 1)?,
            *self.bytes.get(addr + 2)?,
            *self.bytes.get(addr + 3)?,
        ]))
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
        if addr == 0xf202 {
            let value = self.mmio_bytes.get(&addr).copied().unwrap_or(0).wrapping_add(1);
            self.mmio_bytes.insert(addr, value);
            return value;
        }
        if addr == 0xf449 {
            return self.mmio_bytes.get(&addr).copied().unwrap_or(0) | 0x20;
        }
        *self.mmio_bytes.get(&addr).unwrap_or(&0)
    }

    fn read_mmio_word(&mut self, addr: u32) -> u16 {
        match normalize_mmio(addr) {
            0xfb00 => return (self.packed_timebase() >> 16) as u16,
            0xfb02 => return self.packed_timebase() as u16,
            0xfb1a => {
                self.timebase_counter = self.timebase_counter.wrapping_add(1);
                return 0;
            }
            0xf608 => {
                self.timer_counter = self.timer_counter.wrapping_add(1);
                return self.timer_counter;
            }
            _ => {}
        }
        u16::from_be_bytes([self.read_mmio_byte(addr), self.read_mmio_byte(addr + 1)])
    }

    fn write_mmio_byte(&mut self, addr: u32, value: u8) {
        let normalized = normalize_mmio(addr);
        if normalized == 0xf411 && self.f411_row_select_enabled {
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

    fn packed_timebase(&self) -> u32 {
        let seconds = self.timebase_counter;
        let second = seconds % 60;
        let minute = (seconds / 60) % 60;
        let hour = (seconds / 3600) % 24;
        (hour << 24) | (minute << 16) | second
    }
}

fn find_last_system_package(image: &[u8]) -> Option<usize> {
    image
        .windows(4)
        .enumerate()
        .filter_map(|(offset, window)| (window == [0xc0, 0xff, 0xee, 0xad]).then_some(offset))
        .last()
}

fn write_be32(bytes: &mut [u8], addr: usize, value: u32) {
    let value = value.to_be_bytes();
    bytes[addr..addr + 4].copy_from_slice(&value);
}

fn write_be16(bytes: &mut [u8], addr: usize, value: u16) {
    let value = value.to_be_bytes();
    bytes[addr..addr + 2].copy_from_slice(&value);
}

fn write_bytes(bytes: &mut [u8], addr: usize, value: &[u8]) {
    bytes[addr..addr + value.len()].copy_from_slice(value);
}

fn load_stock_applets(bytes: &mut [u8]) {
    let applet_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../analysis/device-dumps/applets");
    let Ok(entries) = std::fs::read_dir(applet_dir) else {
        return;
    };

    let mut applets = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "os3kapp")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with('A') && !name.starts_with("AF"))
        })
        .collect::<Vec<_>>();
    applets.sort();

    let mut cursor = STOCK_APPLET_BASE;
    let applet_start = cursor;
    for path in applets.into_iter().take(31) {
        let Ok(image) = std::fs::read(&path) else {
            continue;
        };
        if cursor + image.len() > bytes.len() || image.len() < 0x16 {
            continue;
        }
        bytes[cursor..cursor + image.len()].copy_from_slice(&image);
        cursor += image.len();
    }
    if cursor > applet_start {
        write_be32(bytes, 0x0000_0e8a, applet_start as u32);
        write_be32(bytes, 0x0000_0e8e, cursor as u32);
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
        if is_watched_ram(addr) {
            self.record_mmio(format!("write8 ram 0x{addr:08x}=0x{value:02x}"));
        }
        *self.bytes.get_mut(addr as usize)? = value;
        Some(())
    }

    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        if self.applet_storage_end.is_some() && (addr == 0x0000_0e8e || addr == 0x0000_0e90) {
            self.record_mmio(format!(
                "ignored applet storage bound write16 ram 0x{addr:08x}=0x{value:04x}"
            ));
            return Some(());
        }
        if is_mmio(addr) {
            self.write_mmio_word(addr, value);
            self.record_mmio(format!(
                "write16 0x{addr:08x}/0x{:04x}=0x{value:04x}",
                normalize_mmio(addr)
            ));
            return Some(());
        }
        if is_watched_ram(addr) {
            self.record_mmio(format!("write16 ram 0x{addr:08x}=0x{value:04x}"));
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
        || (ASIC_REGISTER_START..ASIC_REGISTER_END).contains(&addr)
}

fn is_watched_ram(addr: u32) -> bool {
    addr == 0x0000_0414
        || (0x0000_0e00..=0x0000_0eff).contains(&addr)
        || (0x0000_3550..=0x0000_357f).contains(&addr)
        || (0x0000_35e0..=0x0000_35ff).contains(&addr)
        || (0x0000_3e80..=0x0000_3e9f).contains(&addr)
}

fn normalize_mmio(addr: u32) -> u32 {
    if addr >= MMIO_BASE {
        addr & 0xffff
    } else {
        addr
    }
}

fn find_applet_storage_end(bytes: &[u8]) -> Option<u32> {
    let start = STOCK_APPLET_BASE;
    let mut cursor = start;
    while cursor + 8 <= bytes.len()
        && bytes.get(cursor..cursor + 4) == Some([0xc0, 0xff, 0xee, 0xad].as_slice())
    {
        let length = u32::from_be_bytes(bytes[cursor + 4..cursor + 8].try_into().ok()?) as usize;
        if length == 0 || cursor + length > bytes.len() {
            break;
        }
        cursor += length;
    }
    (cursor > start).then_some(cursor as u32)
}

fn gpio_keyboard_row_select(addr: u32, value: u8) -> Option<u8> {
    match addr {
        0xf410 => {
            const ROWS: &[(u8, u8)] = &[
                (0x80, 0x00),
                (0x40, 0x01),
                (0x20, 0x02),
                (0x10, 0x03),
                (0x08, 0x04),
                (0x04, 0x05),
                (0x02, 0x06),
                (0x01, 0x07),
            ];
            ROWS.iter()
                .find_map(|(bit, row)| (value & bit != 0).then_some(*row))
        }
        0xf408 => (value & 0x20 != 0).then_some(0x09),
        0xf440 => {
            const ROWS: &[(u8, u8)] = &[
                (0x80, 0x0a),
                (0x40, 0x0b),
                (0x20, 0x0c),
                (0x10, 0x0d),
                (0x08, 0x0e),
                (0x04, 0x0f),
            ];
            ROWS.iter()
                .find_map(|(bit, row)| (value & bit != 0).then_some(*row))
        }
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
