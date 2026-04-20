use std::num::Wrapping;

use m68000::cpu_details::Mc68000;
use m68000::exception::Vector;
use m68000::{M68000, MemoryAccess};
use thiserror::Error;

use crate::memory::{APPLET_RAM_BASE, EmuMemory};
use crate::os_shims::NeoOsShims;
use crate::os3kapp::Os3kApp;

const STACK_START: u32 = 0x0070_0000;
const STATUS_ADDR: u32 = 0x0002_0000;
const RETURN_SENTINEL: u32 = 0x007f_fff0;
const MAX_STEPS: usize = 20_000;

const OS_SET_USB_STAGE_A: u32 = 0x0041_f9a0;
const OS_DELAY: u32 = 0x0042_4780;
const OS_SET_USB_STAGE_B: u32 = 0x0044_044e;
const OS_SET_USB_STAGE_C: u32 = 0x0044_047c;
const OS_MARK_DIRECT: u32 = 0x0041_0b26;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunResult {
    pub status: u32,
    pub trace: Vec<String>,
    pub yielded: bool,
}

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("applet image does not fit emulator memory")]
    ImageTooLarge,
    #[error(
        "unsupported exception vector {vector} at dis_pc=0x{dis_pc:08x} cpu_pc=0x{cpu_pc:08x} opcodes={opcodes}"
    )]
    UnsupportedException {
        vector: u8,
        dis_pc: u32,
        cpu_pc: u32,
        opcodes: String,
    },
    #[error("message did not finish after {MAX_STEPS} m68k steps")]
    StepLimit,
    #[error("failed stack operation at 0x{0:08x}")]
    StackAccess(u32),
}

/// Runs one NEO process message against the loaded applet.
///
/// This is the machine-code boundary: applet bytes are interpreted by
/// `m68000`; this module only seeds the NEO process-message stack frame and
/// handles calls that leave the applet for firmware services.
///
/// # Errors
///
/// Returns an error if the applet image cannot be mapped, the interpreted code
/// hits an unsupported CPU exception, or the bounded step limit is reached.
pub fn run_process_message(
    app: &Os3kApp,
    os: &mut NeoOsShims,
    message: u32,
) -> Result<RunResult, RunnerError> {
    let mut session = AppletSession::start(app, message)?;
    session.run_until_yield_or_return(os)
}

#[derive(Debug)]
pub struct AppletSession {
    memory: EmuMemory,
    cpu: M68000<Mc68000>,
    trace: Vec<String>,
}

impl AppletSession {
    pub fn start(app: &Os3kApp, message: u32) -> Result<Self, RunnerError> {
        let mut memory = EmuMemory::load(app)?;
        let mut cpu: M68000<Mc68000> = M68000::new_no_reset();
        cpu.regs.pc = Wrapping(app.entry_offset);
        cpu.regs.a[5] = Wrapping(APPLET_RAM_BASE);
        cpu.regs.ssp = Wrapping(STACK_START);
        seed_entry_stack(&mut memory, message)?;
        Ok(Self {
            memory,
            cpu,
            trace: Vec::new(),
        })
    }

    pub fn run_until_yield_or_return(
        &mut self,
        os: &mut NeoOsShims,
    ) -> Result<RunResult, RunnerError> {
        for _ in 0..MAX_STEPS {
            if self.cpu.regs.pc.0 == RETURN_SENTINEL {
                return Ok(RunResult {
                    status: self.memory.get_long(STATUS_ADDR).unwrap_or(0),
                    trace: self.trace.clone(),
                    yielded: false,
                });
            }

            if handle_os_call(&mut self.cpu, &mut self.memory, os)? {
                continue;
            }
            if handle_long_branch(&mut self.cpu, &mut self.memory) {
                continue;
            }

            let (pc, dis, _, exception) = self
                .cpu
                .disassembler_interpreter_exception(&mut self.memory);
            if !dis.is_empty() {
                self.push_trace(format!("0x{pc:08x}: {dis}"));
            }

            if let Some(vector) = exception {
                if vector == Vector::LineAEmulator as u8 {
                    if let Some(opcode) = self.memory.get_word(pc) {
                        self.push_trace(format!("0x{pc:08x}: trap 0x{opcode:04x}"));
                    }
                    if handle_line_a(pc, &mut self.cpu, &mut self.memory, os)? {
                        return Ok(RunResult {
                            status: self.memory.get_long(STATUS_ADDR).unwrap_or(0),
                            trace: self.trace.clone(),
                            yielded: true,
                        });
                    }
                } else {
                    return Err(unsupported_exception(
                        vector,
                        pc,
                        &self.cpu,
                        &mut self.memory,
                    ));
                }
            }
        }

        Err(RunnerError::StepLimit)
    }

    fn push_trace(&mut self, line: String) {
        self.trace.push(line);
        if self.trace.len() > 80 {
            self.trace.remove(0);
        }
    }
}

fn handle_long_branch(cpu: &mut M68000<Mc68000>, memory: &mut EmuMemory) -> bool {
    let pc = cpu.regs.pc.0;
    if memory.get_word(pc) != Some(0x60FF) {
        return false;
    }
    let Some(displacement) = memory.get_long(pc + 2) else {
        return false;
    };
    cpu.regs.pc = Wrapping((pc + 2).wrapping_add(displacement));
    true
}

fn seed_entry_stack(memory: &mut EmuMemory, message: u32) -> Result<(), RunnerError> {
    memory
        .set_long(STACK_START, RETURN_SENTINEL)
        .ok_or(RunnerError::StackAccess(STACK_START))?;
    memory
        .set_long(STACK_START + 4, message)
        .ok_or(RunnerError::StackAccess(STACK_START + 4))?;
    memory
        .set_long(STACK_START + 8, 0)
        .ok_or(RunnerError::StackAccess(STACK_START + 8))?;
    memory
        .set_long(STACK_START + 12, STATUS_ADDR)
        .ok_or(RunnerError::StackAccess(STACK_START + 12))?;
    Ok(())
}

fn handle_os_call(
    cpu: &mut M68000<Mc68000>,
    memory: &mut EmuMemory,
    os: &mut NeoOsShims,
) -> Result<bool, RunnerError> {
    match cpu.regs.pc.0 {
        OS_SET_USB_STAGE_A | OS_DELAY | OS_SET_USB_STAGE_B | OS_SET_USB_STAGE_C => {
            return_from_subroutine(cpu, memory)?;
            Ok(true)
        }
        OS_MARK_DIRECT => {
            os.switch_to_direct();
            return_from_subroutine(cpu, memory)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn handle_line_a(
    pc: u32,
    cpu: &mut M68000<Mc68000>,
    memory: &mut EmuMemory,
    os: &mut NeoOsShims,
) -> Result<bool, RunnerError> {
    let opcode = memory.get_word(pc).unwrap_or(0);
    match opcode {
        0xA000 => os.clear_screen(),
        0xA004 => {
            let sp = cpu.regs.sp();
            let row = memory.get_long(sp + 4).unwrap_or(1);
            let col = memory.get_long(sp + 8).unwrap_or(1);
            let width = memory.get_long(sp + 12).unwrap_or(1);
            os.set_text_region(row, col, width);
        }
        0xA008 => {
            let sp = cpu.regs.sp();
            let row_ptr = memory.get_long(sp + 4).unwrap_or(0);
            let col_ptr = memory.get_long(sp + 8).unwrap_or(0);
            let _ = memory.set_byte(row_ptr, 1);
            let _ = memory.set_byte(col_ptr, 1);
        }
        0xA00C => {}
        0xA010 => {
            let sp = cpu.regs.sp();
            let raw = memory.get_long(sp + 4).unwrap_or(u32::from(b' '));
            let ch = raw.to_be_bytes()[3];
            os.append_char(ch);
        }
        0xA014 => {
            let sp = cpu.regs.sp();
            let ptr = memory.get_long(sp + 4).unwrap_or(0);
            let text = read_c_string(memory, ptr, 256);
            os.append_text(&text);
        }
        0xA018..=0xA03C => {
            if opcode & 0x0003 == 0 {
                cpu.regs.d[0] = Wrapping(0);
            } else {
                return Err(unsupported_exception(
                    Vector::LineAEmulator as u8,
                    pc,
                    cpu,
                    memory,
                ));
            }
        }
        0xA094 => {
            cpu.regs.d[0] = Wrapping(u32::from(os.read_key()));
        }
        0xA09C => {
            cpu.regs.d[0] = Wrapping(u32::from(os.is_key_ready()));
        }
        0xA098 => {}
        0xA0A4 => {}
        0xA25C => {
            return_from_subroutine(cpu, memory)?;
            return Ok(true);
        }
        0xA040..=0xA308 => {
            if opcode & 0x0003 == 0 {
                cpu.regs.d[0] = Wrapping(0);
            } else {
                return Err(unsupported_exception(
                    Vector::LineAEmulator as u8,
                    pc,
                    cpu,
                    memory,
                ));
            }
        }
        0xA364..=0xA3B0 => {
            if opcode & 0x0003 == 0 {
                cpu.regs.d[0] = Wrapping(0);
            } else {
                return Err(unsupported_exception(
                    Vector::LineAEmulator as u8,
                    pc,
                    cpu,
                    memory,
                ));
            }
        }
        _ => {
            return Err(unsupported_exception(
                Vector::LineAEmulator as u8,
                pc,
                cpu,
                memory,
            ));
        }
    }
    return_from_subroutine(cpu, memory)?;
    Ok(false)
}

fn read_c_string(memory: &mut EmuMemory, ptr: u32, limit: usize) -> String {
    let mut bytes = Vec::new();
    for offset in 0..limit as u32 {
        let Some(byte) = memory.get_byte(ptr + offset) else {
            break;
        };
        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn unsupported_exception(
    vector: u8,
    dis_pc: u32,
    cpu: &M68000<Mc68000>,
    memory: &mut EmuMemory,
) -> RunnerError {
    let cpu_pc = cpu.regs.pc.0;
    let opcodes = [dis_pc, dis_pc + 2, cpu_pc, cpu_pc + 2]
        .into_iter()
        .map(|addr| match memory.get_word(addr) {
            Some(word) => format!("0x{addr:08x}:0x{word:04x}"),
            None => format!("0x{addr:08x}:<out>"),
        })
        .collect::<Vec<_>>()
        .join(" ");
    RunnerError::UnsupportedException {
        vector,
        dis_pc,
        cpu_pc,
        opcodes,
    }
}

fn return_from_subroutine(
    cpu: &mut M68000<Mc68000>,
    memory: &mut EmuMemory,
) -> Result<(), RunnerError> {
    let sp = cpu.regs.sp();
    let ret = memory.get_long(sp).ok_or(RunnerError::StackAccess(sp))?;
    cpu.regs.pc = Wrapping(ret);
    cpu.regs.ssp += 4;
    Ok(())
}
