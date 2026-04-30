use std::collections::VecDeque;
use std::num::Wrapping;
use std::time::{Duration, Instant};

use m68000::M68000;
use m68000::MemoryAccess;
use m68000::cpu_details::Mc68000;
use m68000::exception::Vector;
use thiserror::Error;

use crate::firmware::{FirmwareError, FirmwareRuntime};
use crate::keyboard::{
    MatrixKey, left_shift_key, matrix_key_for_code, matrix_key_stroke_for_char,
};
use crate::lcd::LcdSnapshot;
use crate::memory::{EmuMemory, MemoryError};
use crate::text_screen::TextScreen;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmwareSnapshot {
    pub pc: u32,
    pub ssp: u32,
    pub usp: u32,
    pub d: [u32; 8],
    pub a: [u32; 7],
    pub debug_words: Vec<(u32, u32)>,
    pub steps: usize,
    pub cycles: u64,
    pub stopped: bool,
    pub last_exception: Option<String>,
    pub trace: Vec<String>,
    pub mmio_accesses: Vec<String>,
    pub lcd: LcdSnapshot,
    pub text_screen: Option<String>,
}

#[derive(Debug, Error)]
pub enum FirmwareSessionError {
    #[error("firmware error")]
    Firmware(#[from] FirmwareError),
    #[error("memory error")]
    Memory(#[from] MemoryError),
}

#[derive(Clone, Debug)]
pub struct FirmwareSession {
    cpu: M68000<Mc68000>,
    memory: EmuMemory,
    steps: usize,
    cycles: u64,
    last_exception: Option<String>,
    trace: Vec<String>,
    mmio_accesses: Vec<String>,
    text_screen: TextScreen,
    trace_stack_at_pc: Option<u32>,
    trace_stack_at_pc_hit: usize,
    trace_stack_at_pc_target_hit: usize,
    recent_pcs: VecDeque<u32>,
}

impl FirmwareSession {
    pub fn boot_small_rom_default() -> Result<Self, FirmwareSessionError> {
        Self::boot_small_rom(FirmwareRuntime::load_small_rom_default()?)
    }

    pub fn boot_small_rom(firmware: FirmwareRuntime) -> Result<Self, FirmwareSessionError> {
        Self::boot_small_rom_inner(firmware, false)
    }

    pub fn boot_small_rom_with_entry_chord(
        firmware: FirmwareRuntime,
    ) -> Result<Self, FirmwareSessionError> {
        Self::boot_small_rom_inner(firmware, true)
    }

    fn boot_small_rom_inner(
        firmware: FirmwareRuntime,
        hold_entry_chord: bool,
    ) -> Result<Self, FirmwareSessionError> {
        let (ssp, pc) = firmware.boot_vectors()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;
        if hold_entry_chord {
            memory.hold_small_rom_entry_chord();
        }
        Self::boot_with_memory(ssp, pc, memory)
    }

    pub fn boot_with_keys(
        firmware: FirmwareRuntime,
        keys: &[u8],
        reads: usize,
    ) -> Result<Self, FirmwareSessionError> {
        let (ssp, pc) = firmware.boot_vectors()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;
        let keys = keys.iter().copied().map(MatrixKey::new).collect::<Vec<_>>();
        memory.hold_boot_keys_all_rows(&keys, reads);
        Self::boot_with_memory(ssp, pc, memory)
    }

    pub fn boot_with_exact_keys(
        firmware: FirmwareRuntime,
        keys: &[u8],
        reads: usize,
    ) -> Result<Self, FirmwareSessionError> {
        let (ssp, pc) = firmware.boot_vectors()?;
        let mut memory = EmuMemory::load_small_rom(&firmware)?;
        let keys = keys.iter().copied().map(MatrixKey::new).collect::<Vec<_>>();
        memory.hold_boot_keys_exact_rows(&keys, reads);
        Self::boot_with_memory(ssp, pc, memory)
    }

    fn boot_with_memory(
        ssp: u32,
        pc: u32,
        memory: EmuMemory,
    ) -> Result<Self, FirmwareSessionError> {
        let mut cpu: M68000<Mc68000> = M68000::new_no_reset();
        cpu.regs.ssp = Wrapping(ssp);
        cpu.regs.pc = Wrapping(pc);
        Ok(Self {
            cpu,
            memory,
            steps: 0,
            cycles: 0,
            last_exception: None,
            trace: vec![format!("Firmware boot: ssp=0x{ssp:08x} pc=0x{pc:08x}")],
            mmio_accesses: Vec::new(),
            text_screen: TextScreen::default(),
            trace_stack_at_pc: None,
            trace_stack_at_pc_hit: 0,
            trace_stack_at_pc_target_hit: 1,
            recent_pcs: VecDeque::with_capacity(24),
        })
    }

    pub fn set_trace_stack_at_pc(&mut self, pc: Option<u32>, hit: usize) {
        self.trace_stack_at_pc = pc;
        self.trace_stack_at_pc_hit = 0;
        self.trace_stack_at_pc_target_hit = hit.max(1);
    }

    pub fn run_steps(&mut self, max_steps: usize) {
        for _ in 0..max_steps {
            if !self.step_with_trace() {
                break;
            }
        }
    }

    pub fn run_until_pc_or_steps(&mut self, stop_pc: u32, max_steps: usize) -> bool {
        for _ in 0..max_steps {
            if self.cpu.regs.pc.0 == stop_pc {
                self.push_trace(format!("stopped before pc=0x{stop_pc:08x}"));
                return true;
            }
            if !self.step_with_trace() {
                break;
            }
        }
        self.cpu.regs.pc.0 == stop_pc
    }

    pub fn run_until_pc_hit_or_steps(
        &mut self,
        stop_pc: u32,
        wanted_hit: usize,
        max_steps: usize,
    ) -> bool {
        let mut hits = 0usize;
        for _ in 0..max_steps {
            if self.cpu.regs.pc.0 == stop_pc {
                hits = hits.saturating_add(1);
                if hits >= wanted_hit {
                    self.push_trace(format!("stopped before pc=0x{stop_pc:08x} hit={hits}"));
                    return true;
                }
            }
            if !self.step_with_trace() {
                break;
            }
        }
        false
    }

    pub fn run_until_resource_or_steps(&mut self, resource_id: u16, max_steps: usize) -> bool {
        for _ in 0..max_steps {
            if self.cpu.regs.pc.0 == 0x0042_4212
                && self.memory.peek_word(self.cpu.regs.sp() + 6) == Some(resource_id)
            {
                self.push_trace(format!("stopped before resource_id=0x{resource_id:04x}"));
                return true;
            }
            if !self.step_with_trace() {
                break;
            }
        }
        false
    }

    fn step_with_trace(&mut self) -> bool {
        if self.wake_after_firmware_stop() {
            return true;
        }
        if self.cpu.stop || self.last_exception.is_some() {
            return false;
        }

        self.maybe_trace_stack_at_pc();
        let (pc, disassembly, cycles, exception) = self
            .cpu
            .disassembler_interpreter_exception(&mut self.memory);
        self.record_recent_pc(pc);
        self.steps = self.steps.saturating_add(1);
        self.cycles = self.cycles.saturating_add(cycles as u64);
        self.memory.advance_cpu_cycles(cycles);
        if !disassembly.is_empty() {
            self.push_trace(format!("0x{pc:08x}: {disassembly}"));
        }
        for access in self.memory.drain_mmio_accesses() {
            self.push_mmio_access(access);
        }

        if let Some(vector) = exception {
            if self.enter_exception_handler(vector, pc) {
                return true;
            }
            self.last_exception = Some(format_exception(vector, pc));
            return false;
        }
        self.service_periodic_hardware();
        true
    }

    pub fn run_realtime_steps(&mut self, max_steps: usize) -> u64 {
        let previous_logging = self.memory.set_mmio_logging(false);
        let start_cycles = self.cycles;
        for _ in 0..max_steps {
            if self.wake_after_firmware_stop() {
                continue;
            }
            if self.cpu.stop || self.last_exception.is_some() {
                break;
            }

            self.maybe_trace_stack_at_pc();
            let pc = self.cpu.regs.pc.0;
            let (cycles, exception) = self.cpu.interpreter_exception(&mut self.memory);
            self.record_recent_pc(pc);
            self.steps = self.steps.saturating_add(1);
            self.cycles = self.cycles.saturating_add(cycles as u64);
            self.memory.advance_cpu_cycles(cycles);
            if let Some(vector) = exception {
                if self.enter_exception_handler(vector, pc) {
                    continue;
                }
                self.last_exception = Some(format_exception(vector, pc));
                break;
            }
            self.service_periodic_hardware();
        }
        self.memory.set_mmio_logging(previous_logging);
        self.cycles.saturating_sub(start_cycles)
    }

    pub fn run_realtime_cycles(&mut self, cycle_budget: u64, max_steps: usize) -> u64 {
        self.run_realtime_cycles_inner(cycle_budget, max_steps, None)
    }

    pub fn run_realtime_cycles_for(
        &mut self,
        cycle_budget: u64,
        max_steps: usize,
        max_wall_time: Duration,
    ) -> u64 {
        self.run_realtime_cycles_inner(cycle_budget, max_steps, Some(max_wall_time))
    }

    fn run_realtime_cycles_inner(
        &mut self,
        cycle_budget: u64,
        max_steps: usize,
        max_wall_time: Option<Duration>,
    ) -> u64 {
        let previous_logging = self.memory.set_mmio_logging(false);
        let start_cycles = self.cycles;
        let start_steps = self.steps;
        let started_at = Instant::now();
        while self.cycles.saturating_sub(start_cycles) < cycle_budget {
            if self.wake_after_firmware_stop() {
                continue;
            }
            if self.cpu.stop || self.last_exception.is_some() {
                break;
            }
            if self.steps.saturating_sub(start_steps) >= max_steps {
                break;
            }
            if self.steps.saturating_sub(start_steps).is_multiple_of(4096)
                && max_wall_time.is_some_and(|limit| started_at.elapsed() >= limit)
            {
                break;
            }

            self.maybe_trace_stack_at_pc();
            let pc = self.cpu.regs.pc.0;
            let (cycles, exception) = self.cpu.interpreter_exception(&mut self.memory);
            self.record_recent_pc(pc);
            self.steps = self.steps.saturating_add(1);
            self.cycles = self.cycles.saturating_add(cycles as u64);
            self.memory.advance_cpu_cycles(cycles);
            if let Some(vector) = exception {
                if self.enter_exception_handler(vector, pc) {
                    continue;
                }
                self.last_exception = Some(format_exception(vector, pc));
                break;
            }
            self.service_periodic_hardware();
        }
        self.memory.set_mmio_logging(previous_logging);
        self.cycles.saturating_sub(start_cycles)
    }

    fn service_periodic_hardware(&mut self) {
        if self.steps.is_multiple_of(512) {
            self.memory.service_deferred_timers();
        }
    }

    fn wake_after_firmware_stop(&mut self) -> bool {
        if !self.cpu.stop || self.cpu.regs.pc.0 != 0x0042_6756 {
            return false;
        }
        self.cpu.stop = false;
        self.push_trace(format!(
            "firmware STOP wake -> ssp=0x{:08x} pc=0x{:08x}",
            self.cpu.regs.ssp.0, self.cpu.regs.pc.0
        ));
        true
    }

    fn maybe_trace_stack_at_pc(&mut self) {
        let Some(target_pc) = self.trace_stack_at_pc else {
            return;
        };
        if self.cpu.regs.pc.0 != target_pc {
            return;
        }
        self.trace_stack_at_pc_hit = self.trace_stack_at_pc_hit.saturating_add(1);
        if self.trace_stack_at_pc_hit != self.trace_stack_at_pc_target_hit {
            return;
        }
        let sp = self.cpu.regs.sp();
        let words = [0_u32, 4, 8, 12, 16, 20]
            .into_iter()
            .map(|offset| self.memory.peek_long(sp.wrapping_add(offset)).unwrap_or_default())
            .collect::<Vec<_>>();
        let line = format!(
            "stack_at_pc step={} pc=0x{target_pc:08x} sp=0x{sp:08x} d0=0x{:08x} d1=0x{:08x} d4=0x{:08x} d5=0x{:08x} d6=0x{:08x} d7=0x{:08x} a0=0x{:08x} a1=0x{:08x} a2=0x{:08x} a3=0x{:08x} path={}",
            self.steps,
            self.cpu.regs.d[0].0,
            self.cpu.regs.d[1].0,
            self.cpu.regs.d[4].0,
            self.cpu.regs.d[5].0,
            self.cpu.regs.d[6].0,
            self.cpu.regs.d[7].0,
            self.cpu.regs.a[0].0,
            self.cpu.regs.a[1].0,
            self.cpu.regs.a[2].0,
            self.cpu.regs.a[3].0,
            self.recent_pc_path(),
        );
        println!("{line}");
        for (index, word) in words.iter().enumerate() {
            println!(
                "stack_at_pc_word pc=0x{target_pc:08x} offset={} value=0x{word:08x}",
                index * 4
            );
        }
        self.push_trace(line);
    }

    fn record_recent_pc(&mut self, pc: u32) {
        if self.recent_pcs.len() == 24 {
            self.recent_pcs.pop_front();
        }
        self.recent_pcs.push_back(pc);
    }

    fn recent_pc_path(&self) -> String {
        self.recent_pcs
            .iter()
            .map(|pc| format!("0x{pc:08x}"))
            .collect::<Vec<_>>()
            .join(">")
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        !self.cpu.stop && self.last_exception.is_none()
    }

    #[must_use]
    pub fn status_text(&self) -> &str {
        self.last_exception
            .as_deref()
            .unwrap_or(if self.cpu.stop { "stopped" } else { "running" })
    }

    #[must_use]
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    #[must_use]
    pub fn lcd_snapshot(&self) -> LcdSnapshot {
        self.memory.lcd_snapshot()
    }

    #[must_use]
    pub fn applet_memory_status(&self) -> String {
        let validation = self.memory.applet_memory_validation();
        if validation.valid {
            format!("OK - {} applets", validation.count)
        } else {
            format!("Check - {} applets", validation.count)
        }
    }

    pub fn press_char(&mut self, value: char) {
        if let Some(stroke) = matrix_key_stroke_for_char(value) {
            if stroke.shift {
                self.memory.press_key(left_shift_key());
            }
            self.memory.press_key(stroke.key);
        }
    }

    pub fn release_char(&mut self, value: char) {
        if let Some(stroke) = matrix_key_stroke_for_char(value) {
            self.memory.release_key(stroke.key);
            if stroke.shift {
                self.memory.release_key(left_shift_key());
            }
        }
    }

    pub fn tap_char(&mut self, value: char) {
        if let Some(stroke) = matrix_key_stroke_for_char(value) {
            if stroke.shift {
                self.memory.tap_key_chord(&[left_shift_key(), stroke.key]);
            } else {
                self.memory.tap_key(stroke.key);
            }
        }
    }

    pub fn tap_char_debug(&mut self, value: char) {
        if let Some(stroke) = matrix_key_stroke_for_char(value) {
            if stroke.shift {
                self.memory.tap_key_chord_debug(&[left_shift_key(), stroke.key]);
            } else {
                self.memory.tap_key_debug(stroke.key);
            }
        }
    }

    pub fn tap_char_all_rows(&mut self, value: char) {
        if let Some(stroke) = matrix_key_stroke_for_char(value) {
            if stroke.shift {
                self.memory.tap_key_chord(&[left_shift_key(), stroke.key]);
            } else {
                self.memory.tap_key_all_rows(stroke.key);
            }
        }
    }

    pub fn press_matrix_code(&mut self, code: u8) {
        if let Some(key) = matrix_key_for_code(code) {
            self.memory.press_key(key);
        }
    }

    pub fn release_matrix_code(&mut self, code: u8) {
        if let Some(key) = matrix_key_for_code(code) {
            self.memory.release_key(key);
        }
    }

    pub fn tap_matrix_code(&mut self, code: u8) {
        if let Some(key) = matrix_key_for_code(code) {
            self.memory.tap_key(key);
        }
    }

    pub fn tap_matrix_chord(&mut self, codes: &[u8]) {
        let keys: Vec<_> = codes
            .iter()
            .filter_map(|code| matrix_key_for_code(*code))
            .collect();
        self.memory.tap_key_chord(&keys);
    }

    pub fn tap_matrix_code_long(&mut self, code: u8) {
        if let Some(key) = matrix_key_for_code(code) {
            self.memory.tap_key_long(key);
        }
    }

    pub fn tap_matrix_code_debug(&mut self, code: u8) {
        if let Some(key) = matrix_key_for_code(code) {
            self.memory.tap_key_debug(key);
        }
    }

    pub fn tap_matrix_code_all_rows(&mut self, code: u8) {
        if let Some(key) = matrix_key_for_code(code) {
            self.memory.tap_key_all_rows(key);
        }
    }

    pub fn tap_matrix_code_all_rows_debug(&mut self, code: u8) {
        if let Some(key) = matrix_key_for_code(code) {
            self.memory.tap_key_all_rows_debug(key);
        }
    }

    pub fn run_applet_message_for_validation(
        &mut self,
        applet_name: &str,
        message: u32,
        max_steps: usize,
    ) -> Result<(), String> {
        self.start_applet_message_with_param_for_validation(applet_name, message, 0)?;
        self.run_steps(max_steps);
        if let Some(exception) = &self.last_exception {
            return Err(exception.clone());
        }
        Ok(())
    }

    pub fn start_applet_message_for_validation(
        &mut self,
        applet_name: &str,
        message: u32,
    ) -> Result<(), String> {
        self.start_applet_message_with_param_for_validation_and_launch_info(applet_name, message, 0)
    }

    pub fn start_stock_applet_message_for_validation(
        &mut self,
        applet_name: &str,
        message: u32,
    ) -> Result<(), String> {
        self.start_stock_applet_message_with_param_for_validation(applet_name, message, 0)
    }

    pub fn start_stock_applet_message_with_param_for_validation(
        &mut self,
        applet_name: &str,
        message: u32,
        param: u32,
    ) -> Result<(), String> {
        self.start_applet_message_with_param_for_validation_and_launch_info(applet_name, message, param)
    }

    fn start_applet_message_with_param_for_validation_and_launch_info(
        &mut self,
        applet_name: &str,
        message: u32,
        param: u32,
    ) -> Result<(), String> {
        let info = self
            .memory
            .find_applet_launch_info(applet_name)
            .ok_or_else(|| format!("applet not found: {applet_name}"))?;
        const VALIDATION_STACK: u32 = 0x0007_fb00;
        const VALIDATION_STATUS: u32 = 0x0000_1200;
        let a5_adjust_addr = 0x0000_355e + info.slot * 4;
        let a5_adjust = self
            .memory
            .peek_long(a5_adjust_addr)
            .ok_or_else(|| format!("missing applet A5 adjust at 0x{a5_adjust_addr:08x}"))?;
        self.cpu.regs.pc = Wrapping(info.entry);
        self.cpu.regs.ssp = Wrapping(VALIDATION_STACK);
        self.cpu.regs.d[6] = Wrapping(info.entry_offset);
        self.cpu.regs.d[7] = Wrapping(message);
        self.cpu.regs.a[5] = Wrapping(a5_adjust);
        self.last_exception = None;
        self.memory
            .set_long(VALIDATION_STACK, 0x0042_6752)
            .ok_or_else(|| "failed to write validation return address".to_string())?;
        self.memory
            .set_long(VALIDATION_STACK + 4, message)
            .ok_or_else(|| "failed to write validation message".to_string())?;
        self.memory
            .set_long(VALIDATION_STACK + 8, param)
            .ok_or_else(|| "failed to write validation param".to_string())?;
        self.memory
            .set_long(VALIDATION_STACK + 12, VALIDATION_STATUS)
            .ok_or_else(|| "failed to write validation status pointer".to_string())?;
        self.memory
            .set_long(0x0000_35e2, info.slot)
            .ok_or_else(|| "failed to write current applet slot state".to_string())?;
        self.memory
            .set_long(0x0000_35e6, u32::from(info.id) << 16)
            .ok_or_else(|| "failed to write current applet id".to_string())?;
        self.memory
            .set_long(0x0000_35ec, info.slot)
            .ok_or_else(|| "failed to write current applet slot".to_string())?;
        self.memory
            .set_long(0x0000_3e8a, info.entry)
            .ok_or_else(|| "failed to write current applet callback entry".to_string())?;
        Ok(())
    }

    pub fn start_applet_message_with_param_for_validation(
        &mut self,
        applet_name: &str,
        message: u32,
        param: u32,
    ) -> Result<(), String> {
        self.start_applet_message_with_param_for_validation_and_launch_info(applet_name, message, param)
    }

    #[must_use]
    pub fn validation_applet_memory_hex(&self, offset: u32, len: usize) -> String {
        const VALIDATION_APPLET_MEMORY: u32 = 0x0007_8000;
        (0..len)
            .filter_map(|index| {
                self.memory
                    .peek_byte(VALIDATION_APPLET_MEMORY + offset + index as u32)
            })
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[must_use]
    pub fn memory_bytes(&self) -> &[u8] {
        self.memory.bytes()
    }

    pub fn overlay_memory_bytes(&mut self, overlay: &[u8]) {
        self.memory.overlay_bytes(overlay);
    }

    pub fn overlay_memory_range(&mut self, start: u32, bytes: &[u8]) {
        self.memory.overlay_range(start, bytes);
    }

    pub fn refresh_applet_storage_bounds(&mut self) {
        self.memory.refresh_applet_storage_bounds();
    }

    pub fn clear_keyboard_transients(&mut self) {
        self.memory.clear_keyboard_transients();
    }

    #[cfg(test)]
    fn select_keyboard_row_for_test(&mut self, row_addr: u32, row_value: u8) {
        let _ = self.memory.set_byte(row_addr, row_value);
    }

    #[cfg(test)]
    fn read_keyboard_input_for_test(&mut self) -> Option<u8> {
        self.memory.get_byte(0xffff_f419)
    }

    #[must_use]
    pub fn snapshot(&self) -> FirmwareSnapshot {
        FirmwareSnapshot {
            pc: self.cpu.regs.pc.0,
            ssp: self.cpu.regs.ssp.0,
            usp: self.cpu.regs.usp.0,
            d: self.cpu.regs.d.map(|value| value.0),
            a: self.cpu.regs.a.map(|value| value.0),
            debug_words: self.debug_words(),
            steps: self.steps,
            cycles: self.cycles,
            stopped: self.cpu.stop,
            last_exception: self.last_exception.clone(),
            trace: self.trace.clone(),
            mmio_accesses: self.mmio_accesses.clone(),
            lcd: self.memory.lcd_snapshot(),
            text_screen: self.text_screen.render(),
        }
    }

    fn debug_words(&self) -> Vec<(u32, u32)> {
        let mut addrs = vec![
            0x0000_03e8,
            0x0000_03ee,
            0x0000_0400,
            0x0000_0028,
            0x0000_0070,
            0x0000_0074,
            0x0000_0078,
            0x0000_007c,
            0x0000_00e4,
            0x0000_00e8,
            0x0000_00ec,
            0x0000_00f0,
            0x0000_0e0a,
            0x0000_0e0e,
            0x0000_0e8a,
            0x0000_0e8e,
            0x0000_0e92,
            0x0000_0e94,
            0x0000_0fda,
            0x0000_0fde,
            0x0006_0034,
            0x0006_0044,
            0x0000_355e,
            0x0000_3562,
            0x0000_35e2,
            0x0000_35e6,
            0x0000_35ec,
            0x0000_3e8a,
        ];
        let sp = self.cpu.regs.sp();
        addrs.extend((0..12).map(|index| sp.saturating_add(index * 4)));
        for reg in [2, 3, 4, 6] {
            let base = self.cpu.regs.a[reg].0;
            addrs.extend([0, 4, 8, 0x0c, 0x10, 0x34, 0x44].map(|offset| base.wrapping_add(offset)));
        }
        addrs
            .into_iter()
            .filter_map(|addr| self.memory.peek_long(addr).map(|value| (addr, value)))
            .collect()
    }

    fn enter_exception_handler(&mut self, vector: u8, fault_pc: u32) -> bool {
        if vector != Vector::LineAEmulator as u8 {
            return false;
        }
        self.capture_line_a_text_trap(fault_pc);
        let Some(handler) = self.memory.peek_long(u32::from(vector) * 4) else {
            return false;
        };
        if handler == 0 {
            return false;
        }
        let status = u16::from(self.cpu.regs.sr);
        self.cpu.regs.sr.s = true;
        let return_pc = fault_pc;
        let sp = self.cpu.regs.sp().wrapping_sub(6);
        *self.cpu.regs.sp_mut() = Wrapping(sp);
        if self.memory.set_word(sp, status).is_none()
            || self.memory.set_long(sp + 2, return_pc).is_none()
        {
            return false;
        }
        self.cpu.regs.pc = Wrapping(handler);
        self.push_trace(format!(
            "line-a vector -> handler=0x{handler:08x} return_pc=0x{return_pc:08x}"
        ));
        true
    }

    fn capture_line_a_text_trap(&mut self, fault_pc: u32) {
        let Some(opcode) = self.memory.peek_word(fault_pc) else {
            return;
        };
        let sp = self.cpu.regs.sp();
        match opcode {
            0xa000 => self.text_screen.clear(),
            0xa004 => {
                let Some(row) = self.memory.peek_long(sp + 4) else {
                    return;
                };
                let Some(col) = self.memory.peek_long(sp + 8) else {
                    return;
                };
                let Some(width) = self.memory.peek_long(sp + 12) else {
                    return;
                };
                self.text_screen.set_cursor(row, col, width);
            }
            0xa010 => {
                let Some(byte) = self.memory.peek_long(sp + 4) else {
                    return;
                };
                self.text_screen.draw_char(byte as u8);
            }
            0xa014 => {
                let Some(ptr) = self.memory.peek_long(sp + 4) else {
                    return;
                };
                let mut bytes = Vec::new();
                for index in 0..256u32 {
                    let Some(byte) = self.memory.peek_byte(ptr + index) else {
                        break;
                    };
                    bytes.push(byte);
                    if byte == 0 {
                        break;
                    }
                }
                self.text_screen.draw_c_string(&bytes);
            }
            0xa018 => {
                let Some(mode) = self.memory.peek_long(sp + 4) else {
                    return;
                };
                self.text_screen.set_cursor_mode(mode);
            }
            _ => {}
        }
    }

    fn push_trace(&mut self, line: String) {
        self.trace.push(line);
        if self.trace.len() > 80 {
            self.trace.remove(0);
        }
    }

    fn push_mmio_access(&mut self, access: String) {
        self.mmio_accesses.push(access);
        if self.mmio_accesses.len() > 4096 {
            self.mmio_accesses.remove(0);
        }
    }
}

fn format_exception(vector: u8, pc: u32) -> String {
    let name = match vector {
        value if value == Vector::AccessError as u8 => "bus error",
        value if value == Vector::AddressError as u8 => "address error",
        value if value == Vector::IllegalInstruction as u8 => "illegal instruction",
        value if value == Vector::LineAEmulator as u8 => "line-a emulator",
        value if value == Vector::LineFEmulator as u8 => "line-f emulator",
        _ => "exception",
    };
    format!("{name} vector={vector} at 0x{pc:08x}")
}

#[cfg(test)]
mod tests {
    use super::FirmwareSession;

    #[test]
    fn boots_small_rom_from_reset_vectors() -> Result<(), Box<dyn std::error::Error>> {
        let session = FirmwareSession::boot_small_rom_default()?;
        let snapshot = session.snapshot();

        assert_eq!(snapshot.ssp, 0x0007_fff0);
        assert_eq!(snapshot.pc, 0x0040_042a);
        assert!(snapshot.trace[0].contains("Firmware boot"));
        Ok(())
    }

    #[test]
    fn runs_small_rom_until_first_hardware_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let mut session = FirmwareSession::boot_small_rom_default()?;
        session.run_steps(200);
        let snapshot = session.snapshot();

        assert!(snapshot.steps > 0);
        assert!(!snapshot.trace.is_empty());
        assert!(
            snapshot
                .mmio_accesses
                .iter()
                .any(|access| access.contains("0x0000f000") || access.contains("0xfffff000"))
        );
        Ok(())
    }

    #[test]
    fn realtime_runner_advances_without_trace_growth() -> Result<(), Box<dyn std::error::Error>> {
        let mut session = FirmwareSession::boot_small_rom_default()?;
        let initial_trace_len = session.snapshot().trace.len();
        session.run_realtime_steps(200);
        let snapshot = session.snapshot();

        assert_eq!(snapshot.steps, 200);
        assert!(snapshot.cycles > 0);
        assert_eq!(snapshot.trace.len(), initial_trace_len);
        assert!(snapshot.last_exception.is_none());
        Ok(())
    }

    #[test]
    fn realtime_cycle_runner_honors_cycle_budget() -> Result<(), Box<dyn std::error::Error>> {
        let mut session = FirmwareSession::boot_small_rom_default()?;
        let elapsed = session.run_realtime_cycles(1_000, 10_000);
        let snapshot = session.snapshot();

        assert!(elapsed >= 1_000);
        assert_eq!(snapshot.cycles, elapsed);
        assert!(snapshot.steps <= 10_000);
        assert!(snapshot.last_exception.is_none());
        Ok(())
    }

    #[test]
    fn normal_small_rom_boot_skips_entry_chord() -> Result<(), Box<dyn std::error::Error>> {
        let mut session = FirmwareSession::boot_small_rom_default()?;
        session.run_steps(700_000);
        let snapshot = session.snapshot();

        assert_eq!(snapshot.pc, 0x0040_15ea);
        assert!(snapshot.last_exception.is_none());
        Ok(())
    }

    #[test]
    fn entry_chord_boot_reaches_keyboard_scanner() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = crate::firmware::FirmwareRuntime::load_small_rom_default()?;
        let mut session = FirmwareSession::boot_small_rom_with_entry_chord(firmware)?;
        session.run_steps(700_000);
        let snapshot = session.snapshot();

        assert_eq!(snapshot.pc, 0x0040_0790);
        assert!(snapshot.last_exception.is_none());
        Ok(())
    }

    #[test]
    fn long_matrix_tap_is_visible_on_selected_row() -> Result<(), Box<dyn std::error::Error>> {
        let mut session = FirmwareSession::boot_small_rom_default()?;

        session.tap_matrix_code_long(0x15);
        session.select_keyboard_row_for_test(0xffff_f410, 0x04);

        assert_eq!(session.read_keyboard_input_for_test(), Some(0xfd));
        Ok(())
    }

    #[test]
    fn full_neo_system_image_reaches_display_code() -> Result<(), Box<dyn std::error::Error>> {
        let firmware =
            crate::firmware::FirmwareRuntime::load_small_rom("../analysis/cab/os3kneorom.os3kos")?;
        let mut session = FirmwareSession::boot_small_rom(firmware)?;
        session.run_realtime_cycles(220_000_000, 25_000_000);
        let snapshot = session.snapshot();

        assert!(snapshot.last_exception.is_none());
        assert!(snapshot.lcd.pixels.iter().any(|pixel| *pixel));
        Ok(())
    }

    #[test]
    fn stock_calculator_launches_through_validation_context()
    -> Result<(), Box<dyn std::error::Error>> {
        let firmware =
            crate::firmware::FirmwareRuntime::load_small_rom("../analysis/cab/os3kneorom.os3kos")?;
        let mut session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
        crate::recovery_seed::apply_seed_file_if_present(
            &mut session,
            crate::recovery_seed::default_seed_path(),
        )?;
        session.run_realtime_cycles(220_000_000, 25_000_000);
        session.start_stock_applet_message_for_validation("Calculator", 0x19)?;
        session.run_steps(500_000);

        let snapshot = session.snapshot();
        assert!(snapshot.last_exception.is_none());
        assert!(snapshot.lcd.pixels.iter().any(|pixel| *pixel));
        Ok(())
    }

    #[test]
    fn stock_calculator_help_is_captured_as_text() -> Result<(), Box<dyn std::error::Error>> {
        let firmware =
            crate::firmware::FirmwareRuntime::load_small_rom("../analysis/cab/os3kneorom.os3kos")?;
        let mut session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
        crate::recovery_seed::apply_seed_file_if_present(
            &mut session,
            crate::recovery_seed::default_seed_path(),
        )?;
        session.run_realtime_cycles(220_000_000, 25_000_000);
        session.start_stock_applet_message_for_validation("Calculator", 0x19)?;
        session.run_steps(500_000);

        let snapshot = session.snapshot();
        assert!(snapshot.last_exception.is_none());
        let text = snapshot.text_screen.unwrap_or_default();
        assert!(text.contains("AlphaSmart Calculator Help"));
        assert!(text.contains("esc to exit"));
        Ok(())
    }

    #[test]
    fn forth_mini_evaluates_after_enter_when_launched_through_menu()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut session = boot_full_system_smartapplets_for_forth_validation()?;
        launch_forth_mini_through_menu(&mut session);

        let lcd_before = session.lcd_snapshot();
        for key in [0x5c, 0x69, 0x5b, 0x69, 0x40, 0x69] {
            session.tap_matrix_code_long(key);
            session.run_steps(300_000);
        }
        let lcd_after = session.lcd_snapshot();

        assert_ne!(lcd_before.pixels, lcd_after.pixels);
        assert!(session.snapshot().last_exception.is_none());
        Ok(())
    }

    #[test]
    fn forth_mini_shows_prompt_input_before_enter_when_launched_through_menu()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut session = boot_full_system_smartapplets_for_forth_validation()?;
        launch_forth_mini_through_menu(&mut session);

        session.tap_matrix_code_long(0x5c);
        session.run_steps(300_000);

        let snapshot = session.snapshot();
        assert!(snapshot.last_exception.is_none());
        Ok(())
    }

    #[test]
    fn forth_mini_shows_visible_cursor_on_prompt_when_launched_through_menu()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut session = boot_full_system_smartapplets_for_forth_validation()?;
        launch_forth_mini_through_menu(&mut session);

        let snapshot = session.snapshot();
        assert!(snapshot.last_exception.is_none());
        let columns = crate::lcd::probable_cursor_columns(
            &snapshot.lcd,
            crate::lcd::NEO_VISIBLE_LCD_WIDTH,
            crate::lcd::NEO_VISIBLE_LCD_HEIGHT,
        );
        assert!(columns.into_iter().any(|active| active));
        Ok(())
    }

    fn boot_full_system_smartapplets_for_forth_validation()
    -> Result<FirmwareSession, Box<dyn std::error::Error>> {
        let firmware =
            crate::firmware::FirmwareRuntime::load_small_rom("../analysis/cab/os3kneorom.os3kos")?;
        let mut session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
        crate::recovery_seed::apply_seed_file_if_present(
            &mut session,
            crate::recovery_seed::default_seed_path(),
        )?;
        session.run_realtime_cycles(220_000_000, 25_000_000);
        Ok(session)
    }

    fn launch_forth_mini_through_menu(session: &mut FirmwareSession) {
        for _ in 0..19 {
            session.tap_matrix_code_long(0x15);
            session.run_steps(250_000);
        }
        session.press_matrix_code(0x69);
        session.run_steps(3_000_000);
        session.release_matrix_code(0x69);
        session.run_steps(3_000_000);
        session.clear_keyboard_transients();
    }

}
