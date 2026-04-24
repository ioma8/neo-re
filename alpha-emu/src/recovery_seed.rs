use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::firmware::{FirmwareError, FirmwareRuntime};
use crate::firmware_session::{FirmwareSession, FirmwareSessionError};

const MAGIC: &[u8; 12] = b"NEOSEED1\0\0\0\0";
const RECOVERY_Y_STEP: usize = 9_000_000;
const RECOVERY_ENTER_STEP: usize = 18_000_000;
const RECOVERY_MAX_STEPS_AFTER_ENTER: usize = 250_000_000;
const RECOVERY_READY_PC: u32 = 0x0043_5a26;
const ENTER_CODE: u8 = 0x69;

const RANGES: &[RecoverySeedRange] = &[
    RecoverySeedRange {
        start: 0x0000_0400,
        len: 0x0000_0400,
    },
    RecoverySeedRange {
        start: 0x0000_0e00,
        len: 0x0000_0d00,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RecoverySeedRange {
    start: u32,
    len: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoverySeed {
    ranges: Vec<(RecoverySeedRange, Vec<u8>)>,
}

#[derive(Debug, Error)]
pub enum RecoverySeedError {
    #[error("firmware error")]
    Firmware(#[from] FirmwareError),
    #[error("firmware session error")]
    Session(#[from] FirmwareSessionError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("recovery seed can only be generated from the full NEO System image")]
    NotFullSystemImage,
    #[error("invalid recovery seed file")]
    InvalidSeed,
    #[error("recovery run ended with exception: {0}")]
    RecoveryException(String),
    #[error(
        "recovery run did not reach post-recovery boot point 0x{expected:08x}; pc=0x{actual:08x}"
    )]
    RecoveryDidNotComplete { expected: u32, actual: u32 },
}

pub fn default_seed_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("state")
        .join("full-system-recovery.seed")
}

pub fn generate_and_save_seed(
    firmware_path: impl AsRef<Path>,
    seed_path: impl AsRef<Path>,
) -> Result<PathBuf, RecoverySeedError> {
    generate_and_save_seed_with_progress(firmware_path, seed_path, |_| {})
}

pub fn generate_and_save_seed_with_progress(
    firmware_path: impl AsRef<Path>,
    seed_path: impl AsRef<Path>,
    progress: impl FnMut(&str),
) -> Result<PathBuf, RecoverySeedError> {
    let firmware = FirmwareRuntime::load_small_rom(firmware_path)?;
    let seed = generate_seed(firmware, progress)?;
    let seed_path = seed_path.as_ref();
    seed.write(seed_path)?;
    Ok(seed_path.to_path_buf())
}

pub fn apply_seed_file_if_present(
    session: &mut FirmwareSession,
    seed_path: impl AsRef<Path>,
) -> Result<bool, RecoverySeedError> {
    let seed_path = seed_path.as_ref();
    if !seed_path.exists() {
        return Ok(false);
    }
    let seed = RecoverySeed::read(seed_path)?;
    seed.apply(session);
    Ok(true)
}

fn generate_seed(
    firmware: FirmwareRuntime,
    mut progress: impl FnMut(&str),
) -> Result<RecoverySeed, RecoverySeedError> {
    if !firmware.is_neo_system_image() {
        return Err(RecoverySeedError::NotFullSystemImage);
    }
    progress("Booting full OS recovery path...");
    let mut session = FirmwareSession::boot_small_rom(firmware)?;
    session.run_realtime_steps(RECOVERY_Y_STEP);
    progress("Confirming firmware recovery prompt...");
    session.tap_char('Y');
    session.run_realtime_steps(RECOVERY_ENTER_STEP - RECOVERY_Y_STEP);
    progress("Confirming recovery restart...");
    session.tap_matrix_code_long(ENTER_CODE);
    progress("Waiting for post-recovery boot point...");
    let reached_ready_pc =
        session.run_until_pc_or_steps(RECOVERY_READY_PC, RECOVERY_MAX_STEPS_AFTER_ENTER);

    let snapshot = session.snapshot();
    if let Some(exception) = snapshot.last_exception {
        return Err(RecoverySeedError::RecoveryException(exception));
    }
    if !reached_ready_pc {
        return Err(RecoverySeedError::RecoveryDidNotComplete {
            expected: RECOVERY_READY_PC,
            actual: snapshot.pc,
        });
    }
    progress("Saving recovered low-memory seed...");
    Ok(RecoverySeed::from_memory(session.memory_bytes()))
}

impl RecoverySeed {
    fn from_memory(memory: &[u8]) -> Self {
        let ranges = RANGES
            .iter()
            .map(|range| {
                let start = range.start as usize;
                let end = start + range.len as usize;
                (*range, memory[start..end].to_vec())
            })
            .collect();
        Self { ranges }
    }

    fn apply(&self, session: &mut FirmwareSession) {
        for (range, bytes) in &self.ranges {
            session.overlay_memory_range(range.start, bytes);
        }
        session.refresh_applet_storage_bounds();
    }

    fn read(path: &Path) -> Result<Self, RecoverySeedError> {
        let bytes = fs::read(path)?;
        if bytes.len() < MAGIC.len() + 4 || &bytes[..MAGIC.len()] != MAGIC {
            return Err(RecoverySeedError::InvalidSeed);
        }
        let mut cursor = MAGIC.len();
        let count = read_u32(&bytes, &mut cursor)? as usize;
        let mut ranges = Vec::with_capacity(count);
        for _ in 0..count {
            let start = read_u32(&bytes, &mut cursor)?;
            let len = read_u32(&bytes, &mut cursor)?;
            let end = cursor.saturating_add(len as usize);
            if end > bytes.len() {
                return Err(RecoverySeedError::InvalidSeed);
            }
            ranges.push((
                RecoverySeedRange { start, len },
                bytes[cursor..end].to_vec(),
            ));
            cursor = end;
        }
        if cursor != bytes.len() {
            return Err(RecoverySeedError::InvalidSeed);
        }
        Ok(Self { ranges })
    }

    fn write(&self, path: &Path) -> Result<(), RecoverySeedError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        write_u32(&mut bytes, self.ranges.len() as u32);
        for (range, data) in &self.ranges {
            write_u32(&mut bytes, range.start);
            write_u32(&mut bytes, range.len);
            bytes.extend_from_slice(data);
        }
        fs::write(path, bytes)?;
        Ok(())
    }
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, RecoverySeedError> {
    let end = cursor.saturating_add(4);
    let Some(raw) = bytes.get(*cursor..end) else {
        return Err(RecoverySeedError::InvalidSeed);
    };
    *cursor = end;
    Ok(u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}
