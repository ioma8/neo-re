use std::hint::black_box;
use std::num::Wrapping;
use std::time::Instant;

use m68000::M68000;
use m68000::MemoryAccess;
use m68000::cpu_details::Mc68000;

const STEPS: usize = 8_000_000;
const MEMORY_BYTES: usize = 32 * 1024 * 1024;

fn main() {
    run_workload("nop", |memory| {
        fill_repeating(memory, &[0x4e, 0x71]);
    });
    run_workload("branch", |memory| {
        memory[0..2].copy_from_slice(&[0x60, 0xfe]);
    });
    run_workload("ram_rw", |memory| {
        // move.w (a0),d0; move.w d0,(a0); bra.s -6
        memory[0..6].copy_from_slice(&[0x30, 0x10, 0x30, 0x80, 0x60, 0xfa]);
    });
}

fn run_workload(name: &str, program: impl Fn(&mut [u8])) {
    run_slice_memory(name, &program);
    run_simple_memory(name, &program);
}

fn run_slice_memory(name: &str, program: &impl Fn(&mut [u8])) {
    let mut memory = vec![0; MEMORY_BYTES];
    program(&mut memory);
    let mut cpu = boot_cpu();
    cpu.regs.a[0] = Wrapping(0x1000);
    let (cycles, elapsed_ms, achieved_hz) = run_cpu(&mut cpu, memory.as_mut_slice());
    println!(
        "workload={name} memory=slice steps={STEPS} cycles={cycles} elapsed_ms={elapsed_ms} achieved_hz={achieved_hz:.0}"
    );
}

fn run_simple_memory(name: &str, program: &impl Fn(&mut [u8])) {
    let mut bytes = vec![0; MEMORY_BYTES];
    program(&mut bytes);
    let mut memory = SimpleMemory { bytes };
    let mut cpu = boot_cpu();
    cpu.regs.a[0] = Wrapping(0x1000);
    let (cycles, elapsed_ms, achieved_hz) = run_cpu(&mut cpu, &mut memory);
    println!(
        "workload={name} memory=simple steps={STEPS} cycles={cycles} elapsed_ms={elapsed_ms} achieved_hz={achieved_hz:.0}"
    );
}

fn boot_cpu() -> M68000<Mc68000> {
    let mut cpu = M68000::new_no_reset();
    cpu.regs.ssp = Wrapping(0x0007_fffc);
    cpu.regs.pc = Wrapping(0);
    cpu
}

fn run_cpu<M: MemoryAccess + ?Sized>(
    cpu: &mut M68000<Mc68000>,
    memory: &mut M,
) -> (u64, u128, f64) {
    let started_at = Instant::now();
    let mut cycles = 0_u64;
    for _ in 0..STEPS {
        let (step_cycles, exception) = cpu.interpreter_exception(memory);
        if exception.is_some() {
            break;
        }
        cycles = cycles.saturating_add(step_cycles as u64);
    }
    black_box(cpu.regs.pc);
    let elapsed = started_at.elapsed();
    let achieved_hz = if elapsed.is_zero() {
        0.0
    } else {
        cycles as f64 / elapsed.as_secs_f64()
    };
    (cycles, elapsed.as_millis(), achieved_hz)
}

fn fill_repeating(memory: &mut [u8], pattern: &[u8]) {
    for chunk in memory.chunks_exact_mut(pattern.len()) {
        chunk.copy_from_slice(pattern);
    }
}

struct SimpleMemory {
    bytes: Vec<u8>,
}

impl MemoryAccess for SimpleMemory {
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
