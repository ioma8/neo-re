use m68000::MemoryAccess;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use thiserror::Error;

use crate::firmware::FirmwareRuntime;
use crate::keyboard::{Keyboard, MatrixKey};
use crate::lcd::{Lcd, LcdSnapshot};
use crate::{read_be32, write_be32};

const MEMORY_SIZE: usize = 0x0080_0000;
const ROM_BASE: usize = 0x0040_0000;
// 0x0000_0e0a: firmware pointer to the last system package in the ROM image
// 0x0000_0e8a/0x0000_0e8e: applet storage start/end bounds
// 0x0000_7dd8: firmware keyboard stack pointer area (configurable stack depth)
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
const CPU_HZ: u64 = 33_000_000;
const PLL_CLK32_EDGE_HZ: u64 = 32_768;
const PLL_CLK32_CYCLES_PER_EDGE: u64 = CPU_HZ / PLL_CLK32_EDGE_HZ;
const TIMER_PRESCALER: u64 = 0x20 + 1;
const TIMER_COUNTER_DENOMINATOR: u64 = CPU_HZ * TIMER_PRESCALER;
const EXTRA_APPLET_PATHS: &[&str] = &[
    "../exports/applets/alpha-usb-native.os3kapp",
    "../exports/applets/forth-mini.os3kapp",
    "../exports/applets/basic-writer.os3kapp",
    "../exports/applets/write-or-die.os3kapp",
    "../exports/applets/floppy-bird.os3kapp",
    "../exports/applets/snake.os3kapp",
    "../exports/applets/raycaster.os3kapp",
];

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("firmware image does not fit emulator memory")]
    ImageTooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppletMemoryValidation {
    pub(crate) count: usize,
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) alpha_usb_native: Option<u32>,
    pub(crate) forth_mini: Option<u32>,
    pub(crate) basic_writer: Option<u32>,
    pub(crate) valid: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AppletLaunchInfo {
    pub(crate) slot: u32,
    pub(crate) base: u32,
    pub(crate) entry: u32,
    pub(crate) entry_offset: u32,
    pub(crate) id: u16,
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
    timer_counter: u32,
    pll_clk32_cycles: u64,
    timer_counter_phase: u64,
    applet_storage_end: Option<u32>,
    f411_row_select_enabled: bool,
    gpio_keyboard_rows: u16,
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
            pll_clk32_cycles: 0,
            timer_counter_phase: 0,
            applet_storage_end,
            f411_row_select_enabled: !is_neo_system_image,
            gpio_keyboard_rows: 0,
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

    pub(crate) fn tap_key_chord(&mut self, keys: &[MatrixKey]) {
        self.keyboard.tap_chord(keys);
    }

    pub(crate) fn tap_key_chord_for_reads(
        &mut self,
        keys: &[MatrixKey],
        press_reads: usize,
        release_reads: usize,
    ) {
        self.keyboard
            .tap_chord_for_reads(keys, press_reads, release_reads);
    }

    pub(crate) fn tap_key_chord_for_cycles(
        &mut self,
        keys: &[MatrixKey],
        press_cycles: u64,
        release_cycles: u64,
    ) {
        self.keyboard
            .tap_chord_for_cycles(keys, press_cycles, release_cycles);
    }

    pub(crate) fn tap_key_chord_debug(&mut self, keys: &[MatrixKey]) {
        self.keyboard.tap_chord_debug(keys);
    }

    pub(crate) fn tap_key_long(&mut self, key: MatrixKey) {
        self.keyboard.tap_long(key);
    }

    pub(crate) fn tap_key_debug(&mut self, key: MatrixKey) {
        self.keyboard.tap_debug(key);
    }

    pub(crate) fn tap_key_all_rows(&mut self, key: MatrixKey) {
        self.keyboard.tap_all_rows(key);
    }

    pub(crate) fn tap_key_all_rows_debug(&mut self, key: MatrixKey) {
        self.keyboard.tap_all_rows_debug(key);
    }


    pub(crate) fn hold_small_rom_entry_chord(&mut self) {
        self.keyboard.hold_small_rom_entry_chord();
    }

    pub(crate) fn hold_boot_keys_all_rows(&mut self, keys: &[MatrixKey], reads: usize) {
        self.keyboard.hold_keys_all_rows(keys, reads);
    }

    pub(crate) fn hold_boot_keys_exact_rows(&mut self, keys: &[MatrixKey], reads: usize) {
        self.keyboard.hold_keys_exact_rows(keys, reads);
    }

    pub(crate) fn clear_keyboard_transients(&mut self) {
        self.keyboard.clear_transients();
    }

    pub(crate) fn drain_mmio_accesses(&mut self) -> Vec<String> {
        std::mem::take(&mut self.mmio_accesses)
    }

    pub(crate) fn lcd_snapshot(&self) -> LcdSnapshot {
        self.lcd.snapshot()
    }

    pub(crate) fn applet_memory_validation(&self) -> AppletMemoryValidation {
        let mut cursor = STOCK_APPLET_BASE;
        let start = cursor;
        let mut count = 0usize;
        let mut alpha_usb_native = None;
        let mut forth_mini = None;
        let mut basic_writer = None;
        let mut valid = true;

        while cursor + 0x84 <= self.bytes.len() {
            if self.bytes.get(cursor..cursor + 4) != Some([0xc0, 0xff, 0xee, 0xad].as_slice()) {
                break;
            }
            let Some(length) = read_be32(&self.bytes, cursor + 4).map(|value| value as usize)
            else {
                valid = false;
                break;
            };
            if length < 0x94 || cursor + length > self.bytes.len() {
                valid = false;
                break;
            }
            let name = applet_name(&self.bytes[cursor + 0x18..cursor + 0x40]);
            match name.as_deref() {
                Some("Alpha USB") => alpha_usb_native = Some(cursor as u32),
                Some("Forth Mini") => forth_mini = Some(cursor as u32),
                Some("Basic Writer") => basic_writer = Some(cursor as u32),
                _ => {}
            }
            count = count.saturating_add(1);
            cursor += length;
        }

        valid &= count > 0
            && self
                .applet_storage_end
                .is_some_and(|expected_end| expected_end as usize == cursor)
            && alpha_usb_native.is_some()
            && forth_mini.is_some()
            && basic_writer.is_some();
        AppletMemoryValidation {
            count,
            start: start as u32,
            end: cursor as u32,
            alpha_usb_native,
            forth_mini,
            basic_writer,
            valid,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn find_applet_entry(&self, wanted_name: &str) -> Option<u32> {
        self.find_applet_launch_info(wanted_name).map(|info| info.entry)
    }

    pub(crate) fn find_applet_launch_info(&self, wanted_name: &str) -> Option<AppletLaunchInfo> {
        let mut cursor = STOCK_APPLET_BASE;
        let mut slot = 1u32;
        while cursor + 0x94 <= self.bytes.len()
            && self.bytes.get(cursor..cursor + 4) == Some([0xc0, 0xff, 0xee, 0xad].as_slice())
        {
            let length = read_be32(&self.bytes, cursor + 4)? as usize;
            if length < 0x94 || cursor + length > self.bytes.len() {
                return None;
            }
            let name = applet_name(&self.bytes[cursor + 0x18..cursor + 0x40])?;
            if name == wanted_name {
                let entry_offset = read_be32(&self.bytes, cursor + 0x84)?;
                let id = u16::from_be_bytes([self.bytes[cursor + 0x14], self.bytes[cursor + 0x15]]);
                return Some(AppletLaunchInfo {
                    slot,
                    base: cursor as u32,
                    entry: cursor as u32 + entry_offset,
                    entry_offset,
                    id,
                });
            }
            cursor += length;
            slot = slot.saturating_add(1);
        }
        None
    }

    pub(crate) fn set_mmio_logging(&mut self, enabled: bool) -> bool {
        let previous = self.mmio_logging;
        self.mmio_logging = enabled;
        previous
    }

    pub(crate) fn service_deferred_timers(&mut self) {
        // Timer queue at 0x5d3c: 5 records × 14 bytes each.
        // Record layout: byte 0 = pending flag (1=armed), bytes 2-3 = deadline (u16),
        //                bytes 6-9 = completion callback address.
        const TIMER_QUEUE_BASE: usize = 0x0000_5d3c;
        const TIMER_QUEUE_RECORD_LEN: usize = 14;
        const TIMER_QUEUE_RECORDS: usize = 5;

        for index in 0..TIMER_QUEUE_RECORDS {
            let record = TIMER_QUEUE_BASE + index * TIMER_QUEUE_RECORD_LEN;
            if self.bytes.get(record).copied() != Some(1) {
                continue;
            }
            let Some(deadline) = self.peek_word((record + 2) as u32) else {
                continue;
            };
            if !timer_due(self.timer_counter as u16, deadline) {
                continue;
            }

            if let Some(state) = self.bytes.get_mut(record) {
                *state = 0;
            }
            let Some(completion_ptr) = self.peek_long((record + 6) as u32) else {
                continue;
            };
            let completion_ptr = completion_ptr as usize;
            if let Some(completion) = self.bytes.get_mut(completion_ptr) {
                *completion = 0xff;
                self.record_mmio(format!(
                    "deferred timer {index} completed byte 0x{completion_ptr:08x}=0xff"
                ));
            }
        }
    }

    pub(crate) fn advance_cpu_cycles(&mut self, cycles: usize) {
        self.keyboard.advance_cycles(cycles as u64);
        self.pll_clk32_cycles = self.pll_clk32_cycles.saturating_add(cycles as u64);
        while self.pll_clk32_cycles >= PLL_CLK32_CYCLES_PER_EDGE {
            self.pll_clk32_cycles -= PLL_CLK32_CYCLES_PER_EDGE;
            let next = self.mmio_bytes.get(&0xf202).copied().unwrap_or(0) ^ 0x80;
            self.mmio_bytes.insert(0xf202, next);
        }

        self.timer_counter_phase = self
            .timer_counter_phase
            .saturating_add(cycles as u64 * PLL_CLK32_EDGE_HZ);
        while self.timer_counter_phase >= TIMER_COUNTER_DENOMINATOR {
            self.timer_counter_phase -= TIMER_COUNTER_DENOMINATOR;
            self.advance_timer_counter(1);
        }
    }

    pub(crate) fn peek_word(&self, addr: u32) -> Option<u16> {
        let addr = addr as usize;
        Some(u16::from_be_bytes([
            *self.bytes.get(addr)?,
            *self.bytes.get(addr + 1)?,
        ]))
    }

    pub(crate) fn peek_byte(&self, addr: u32) -> Option<u8> {
        self.bytes.get(addr as usize).copied()
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

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn overlay_bytes(&mut self, overlay: &[u8]) {
        let len = self.bytes.len().min(overlay.len());
        self.bytes[..len].copy_from_slice(&overlay[..len]);
    }

    pub(crate) fn overlay_range(&mut self, start: u32, overlay: &[u8]) {
        let start = start as usize;
        let end = start.saturating_add(overlay.len()).min(self.bytes.len());
        if start >= end {
            return;
        }
        self.bytes[start..end].copy_from_slice(&overlay[..end - start]);
    }

    pub(crate) fn refresh_applet_storage_bounds(&mut self) {
        if let Some(end) = find_applet_storage_end(&self.bytes) {
            self.applet_storage_end = Some(end);
            write_be32(&mut self.bytes, 0x0000_0e8a, STOCK_APPLET_BASE as u32);
            write_be32(&mut self.bytes, 0x0000_0e8e, end);
        }
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
            return self.mmio_bytes.get(&addr).copied().unwrap_or(0);
        }
        if addr == 0xf449 {
            return self.mmio_bytes.get(&addr).copied().unwrap_or(0) | 0x20;
        }
        match addr {
            0x0100_8000 => return self.lcd.read_status(0),
            0x0100_8001 => return self.lcd.read_data(0),
            0x0100_0000 => return self.lcd.read_status(1),
            0x0100_0001 => return self.lcd.read_data(1),
            _ => {}
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
                return self.timer_counter as u16;
            }
            _ => {}
        }
        u16::from_be_bytes([self.read_mmio_byte(addr), self.read_mmio_byte(addr + 1)])
    }

    fn write_mmio_byte(&mut self, addr: u32, value: u8) {
        let normalized = normalize_mmio(addr);
        if normalized == 0xf411 && self.f411_row_select_enabled {
            self.keyboard.select_row(value);
        } else if let Some(rows) = gpio_keyboard_row_select(normalized, value) {
            let mask = gpio_keyboard_row_mask(normalized);
            self.gpio_keyboard_rows = (self.gpio_keyboard_rows & !mask) | rows;
            self.keyboard.select_rows(self.gpio_keyboard_rows);
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

    fn advance_timer_counter(&mut self, ticks: u32) {
        let previous = self.timer_counter as u16;
        self.timer_counter = self.timer_counter.wrapping_add(ticks);
        let current = self.timer_counter as u16;
        if current < previous {
            // 0x0000_5d94: timer overflow high-word base (incremented when timer_counter wraps)
            const TIMER_HIGH_BASE: usize = 0x0000_5d94;
            let base = read_be32(&self.bytes, TIMER_HIGH_BASE)
                .unwrap_or(0)
                .wrapping_add(0x0001_0000);
            write_be32(&mut self.bytes, TIMER_HIGH_BASE, base);
            self.record_mmio(format!("timer overflow high base -> 0x{base:08x}"));
        }
    }
}

fn timer_due(now: u16, deadline: u16) -> bool {
    now.wrapping_sub(deadline) < 0x8000
}

fn find_last_system_package(image: &[u8]) -> Option<usize> {
    image
        .windows(4)
        .enumerate()
        .filter_map(|(offset, window)| (window == [0xc0, 0xff, 0xee, 0xad]).then_some(offset))
        .next_back()
}

fn applet_name(bytes: &[u8]) -> Option<String> {
    let len = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let name = std::str::from_utf8(&bytes[..len]).ok()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

fn load_stock_applets(bytes: &mut [u8]) {
    let applet_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../analysis/device-dumps/applets");
    let extra_applets = EXTRA_APPLET_PATHS
        .iter()
        .map(|path| Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .collect::<Vec<_>>();
    let extra_names = extra_applets
        .iter()
        .filter_map(|path| std::fs::read(path).ok())
        .filter_map(|image| applet_name(image.get(0x18..0x40)?))
        .collect::<BTreeSet<_>>();
    let mut applets = std::fs::read_dir(applet_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "os3kapp")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with('A') && !name.starts_with("AF"))
        })
        .filter(|path| {
            std::fs::read(path)
                .ok()
                .and_then(|image| applet_name(image.get(0x18..0x40)?))
                .is_none_or(|name| !extra_names.contains(&name))
        })
        .collect::<Vec<_>>();
    applets.sort();
    let stock_limit = 31usize.saturating_sub(extra_applets.len());

    let mut cursor = STOCK_APPLET_BASE;
    let applet_start = cursor;
    for path in applets.into_iter().take(stock_limit).chain(extra_applets) {
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
        || (0x0000_0f00..=0x0000_1120).contains(&addr)
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

fn gpio_keyboard_row_select(addr: u32, value: u8) -> Option<u16> {
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
            Some(rows_from_bits(value, ROWS))
        }
        0xf408 => Some(if value & 0x20 != 0 { 1 << 0x09 } else { 0 }),
        0xf440 => {
            const ROWS: &[(u8, u8)] = &[
                (0x80, 0x0a),
                (0x40, 0x0b),
                (0x20, 0x0c),
                (0x10, 0x0d),
                (0x08, 0x0e),
                (0x04, 0x0f),
            ];
            Some(rows_from_bits(value, ROWS))
        }
        _ => None,
    }
}

fn gpio_keyboard_row_mask(addr: u32) -> u16 {
    match addr {
        0xf410 => 0x00ff,
        0xf408 => 1 << 0x09,
        0xf440 => 0xfc00,
        _ => 0,
    }
}

fn rows_from_bits(value: u8, rows: &[(u8, u8)]) -> u16 {
    rows.iter().fold(0, |selected, (bit, row)| {
        if value & bit != 0 {
            selected | (1 << row)
        } else {
            selected
        }
    })
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

        assert_eq!(memory.get_byte(0x0100_8000), Some(0x00));
        assert_eq!(memory.get_byte(0x0100_8001), Some(0x00));
        assert_eq!(memory.get_byte(0x0100_0000), Some(0x00));
        assert_eq!(memory.get_byte(0x0100_0001), Some(0x00));
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

    #[test]
    fn timer_counter_follows_prescaled_32768_hz_clock() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom_default()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;

        memory.advance_cpu_cycles(super::CPU_HZ as usize);

        assert_eq!(memory.get_word(0xffff_f608), Some(992));
        assert_eq!(memory.get_word(0xffff_f608), Some(992));
        Ok(())
    }

    #[test]
    fn full_system_loads_exported_custom_applets() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom("../analysis/cab/os3kneorom.os3kos")?;
        let memory = EmuMemory::load_small_rom(&firmware)?;
        let applet_start = u32::from_be_bytes(memory.bytes[0x0e8a..0x0e8e].try_into()?) as usize;
        let applet_end = u32::from_be_bytes(memory.bytes[0x0e8e..0x0e92].try_into()?) as usize;

        assert!(applet_start >= super::STOCK_APPLET_BASE);
        assert!(applet_end > applet_start);
        let applet_storage = &memory.bytes[applet_start..applet_end];
        assert!(contains_bytes(applet_storage, b"Alpha USB"));
        assert!(contains_bytes(applet_storage, b"Forth Mini"));
        assert!(contains_bytes(applet_storage, b"Basic Writer"));

        let validation = memory.applet_memory_validation();
        assert!(validation.valid);
        assert_eq!(validation.start as usize, applet_start);
        assert_eq!(validation.end as usize, applet_end);
        assert!(validation.alpha_usb_native.is_some());
        assert!(validation.forth_mini.is_some());
        assert!(validation.basic_writer.is_some());
        Ok(())
    }

    #[test]
    fn full_system_keeps_exported_custom_applets_after_recovery_seed()
    -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom("../analysis/cab/os3kneorom.os3kos")?;
        let mut session = crate::firmware_session::FirmwareSession::boot_small_rom(firmware)?;

        crate::recovery_seed::apply_seed_file_if_present(
            &mut session,
            crate::recovery_seed::default_seed_path(),
        )?;

        let validation = session.snapshot().debug_words;
        assert!(
            validation
                .iter()
                .any(|(addr, value)| *addr == 0x0000_0e8a
                    && *value == super::STOCK_APPLET_BASE as u32)
        );
        assert!(
            session
                .start_applet_message_for_validation("Forth Mini", 0x19)
                .is_ok()
        );
        Ok(())
    }

    fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }
}
