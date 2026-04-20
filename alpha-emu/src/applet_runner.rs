use std::num::Wrapping;

use m68000::cpu_details::Mc68000;
use m68000::exception::Vector;
use m68000::{M68000, MemoryAccess};
use thiserror::Error;

use crate::memory::EmuMemory;
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
}

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("applet image does not fit emulator memory")]
    ImageTooLarge,
    #[error("unsupported exception vector {vector} at pc=0x{pc:08x}")]
    UnsupportedException { vector: u8, pc: u32 },
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
    let mut memory = EmuMemory::load(app)?;
    let mut cpu: M68000<Mc68000> = M68000::new_no_reset();
    cpu.regs.pc = Wrapping(app.entry_offset);
    cpu.regs.ssp = Wrapping(STACK_START);
    seed_entry_stack(&mut memory, message)?;

    let mut trace = Vec::new();
    for _ in 0..MAX_STEPS {
        if cpu.regs.pc.0 == RETURN_SENTINEL {
            return Ok(RunResult {
                status: memory.get_long(STATUS_ADDR).unwrap_or(0),
                trace,
            });
        }

        if handle_os_call(&mut cpu, &mut memory, os)? {
            continue;
        }
        if handle_long_branch(&mut cpu, &mut memory) {
            continue;
        }

        let (pc, dis, _, exception) = cpu.disassembler_interpreter_exception(&mut memory);
        if !dis.is_empty() {
            trace.push(format!("0x{pc:08x}: {dis}"));
            if trace.len() > 80 {
                trace.remove(0);
            }
        }

        if let Some(vector) = exception {
            if vector == Vector::LineAEmulator as u8 {
                if handle_line_a(pc, &mut cpu, &mut memory, os)? {
                    return Ok(RunResult {
                        status: memory.get_long(STATUS_ADDR).unwrap_or(0),
                        trace,
                    });
                }
            } else {
                return Err(RunnerError::UnsupportedException {
                    vector,
                    pc: cpu.regs.pc.0,
                });
            }
        }
    }

    Err(RunnerError::StepLimit)
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
            os.set_row_text(row, String::new());
        }
        0xA010 => {
            let sp = cpu.regs.sp();
            let raw = memory.get_long(sp + 4).unwrap_or(u32::from(b' '));
            let ch = raw.to_be_bytes()[3];
            os.append_char(ch);
        }
        0xA098 => {}
        0xA25C => return Ok(true),
        _ => {
            return Err(RunnerError::UnsupportedException {
                vector: Vector::LineAEmulator as u8,
                pc,
            });
        }
    }
    Ok(false)
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
