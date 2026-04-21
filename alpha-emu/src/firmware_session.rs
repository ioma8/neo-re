use std::num::Wrapping;

use m68000::M68000;
use m68000::cpu_details::Mc68000;
use m68000::exception::Vector;
use thiserror::Error;

use crate::firmware::{FirmwareError, FirmwareRuntime};
use crate::keyboard::{matrix_key_for_char, matrix_key_for_code};
use crate::lcd::LcdSnapshot;
use crate::memory::{EmuMemory, MemoryError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmwareSnapshot {
    pub pc: u32,
    pub ssp: u32,
    pub steps: usize,
    pub stopped: bool,
    pub last_exception: Option<String>,
    pub trace: Vec<String>,
    pub mmio_accesses: Vec<String>,
    pub lcd: LcdSnapshot,
}

#[derive(Debug, Error)]
pub enum FirmwareSessionError {
    #[error("firmware error")]
    Firmware(#[from] FirmwareError),
    #[error("memory error")]
    Memory(#[from] MemoryError),
}

#[derive(Debug)]
pub struct FirmwareSession {
    cpu: M68000<Mc68000>,
    memory: EmuMemory,
    steps: usize,
    last_exception: Option<String>,
    trace: Vec<String>,
    mmio_accesses: Vec<String>,
}

impl FirmwareSession {
    pub fn boot_small_rom_default() -> Result<Self, FirmwareSessionError> {
        Self::boot_small_rom(FirmwareRuntime::load_small_rom_default()?)
    }

    pub fn boot_small_rom(firmware: FirmwareRuntime) -> Result<Self, FirmwareSessionError> {
        let (ssp, pc) = firmware.reset_vectors()?;
        let memory = EmuMemory::load_small_rom(&firmware)?;
        let mut cpu: M68000<Mc68000> = M68000::new_no_reset();
        cpu.regs.ssp = Wrapping(ssp);
        cpu.regs.pc = Wrapping(pc);
        Ok(Self {
            cpu,
            memory,
            steps: 0,
            last_exception: None,
            trace: vec![format!("Small ROM reset: ssp=0x{ssp:08x} pc=0x{pc:08x}")],
            mmio_accesses: Vec::new(),
        })
    }

    pub fn run_steps(&mut self, max_steps: usize) {
        for _ in 0..max_steps {
            if self.cpu.stop || self.last_exception.is_some() {
                break;
            }

            let (pc, disassembly, _, exception) = self
                .cpu
                .disassembler_interpreter_exception(&mut self.memory);
            self.steps = self.steps.saturating_add(1);
            if !disassembly.is_empty() {
                self.push_trace(format!("0x{pc:08x}: {disassembly}"));
            }
            for access in self.memory.drain_mmio_accesses() {
                self.push_mmio_access(access);
            }

            if let Some(vector) = exception {
                self.last_exception = Some(format_exception(vector, pc));
                break;
            }
        }
    }

    pub fn type_small_rom_password(&mut self) {
        self.memory.type_small_rom_password();
    }

    pub fn press_char(&mut self, value: char) {
        if let Some(key) = matrix_key_for_char(value) {
            self.memory.press_key(key);
        }
    }

    pub fn release_char(&mut self, value: char) {
        if let Some(key) = matrix_key_for_char(value) {
            self.memory.release_key(key);
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

    #[must_use]
    pub fn snapshot(&self) -> FirmwareSnapshot {
        FirmwareSnapshot {
            pc: self.cpu.regs.pc.0,
            ssp: self.cpu.regs.ssp.0,
            steps: self.steps,
            stopped: self.cpu.stop,
            last_exception: self.last_exception.clone(),
            trace: self.trace.clone(),
            mmio_accesses: self.mmio_accesses.clone(),
            lcd: self.memory.lcd_snapshot(),
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
        if self.mmio_accesses.len() > 256 {
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
        assert!(snapshot.trace[0].contains("Small ROM reset"));
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
}
