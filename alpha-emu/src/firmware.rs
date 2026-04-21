use std::path::Path;

use thiserror::Error;

const DEFAULT_SMALL_ROM_PATH: &str = "../analysis/cab/smallos3kneorom.os3kos";

#[derive(Clone, Debug)]
pub struct FirmwareRuntime {
    image: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmwareSegment {
    pub address: u32,
    pub file_offset: usize,
    pub length: usize,
}

#[derive(Debug, Error)]
pub enum FirmwareError {
    #[error("failed to read firmware file")]
    Read(#[from] std::io::Error),
    #[error("firmware image is too short to contain reset vectors")]
    MissingResetVectors,
    #[error("firmware image contains an invalid segment table")]
    InvalidSegmentTable,
}

impl FirmwareRuntime {
    pub fn load_small_rom_default() -> Result<Self, FirmwareError> {
        Self::load_small_rom(DEFAULT_SMALL_ROM_PATH)
    }

    pub fn load_small_rom(path: impl AsRef<Path>) -> Result<Self, FirmwareError> {
        Ok(Self {
            image: std::fs::read(path)?,
        })
    }

    pub fn image(&self) -> &[u8] {
        &self.image
    }

    pub fn reset_vectors(&self) -> Result<(u32, u32), FirmwareError> {
        let ssp = read_be32(&self.image, 0).ok_or(FirmwareError::MissingResetVectors)?;
        let pc = read_be32(&self.image, 4).ok_or(FirmwareError::MissingResetVectors)?;
        Ok((ssp, pc))
    }

    pub fn boot_vectors(&self) -> Result<(u32, u32), FirmwareError> {
        if self.is_neo_system_image() {
            return Ok((0x0007_fff0, 0x0041_0082));
        }
        self.reset_vectors()
    }

    pub fn neo_system_segments(&self) -> Result<Vec<FirmwareSegment>, FirmwareError> {
        if !self.is_neo_system_image() {
            return Ok(Vec::new());
        }

        let mut segments = Vec::new();
        let mut data_offset: usize = 0x70;
        for offset in (0x50..self.image.len()).step_by(8) {
            let Some(address) = read_be32(&self.image, offset) else {
                return Err(FirmwareError::InvalidSegmentTable);
            };
            let Some(length) = read_be32(&self.image, offset + 4) else {
                return Err(FirmwareError::InvalidSegmentTable);
            };
            if address == 0 && length == 0 {
                break;
            }
            if length == 0 {
                return Err(FirmwareError::InvalidSegmentTable);
            }
            let length = length as usize;
            if data_offset.saturating_add(length) > self.image.len() {
                return Err(FirmwareError::InvalidSegmentTable);
            }
            segments.push(FirmwareSegment {
                address,
                file_offset: data_offset,
                length,
            });
            data_offset += length;
        }
        Ok(segments)
    }

    pub fn is_neo_system_image(&self) -> bool {
        self.image.len() >= 0x70 && self.image.get(6..24) == Some(b"System 3 Neo      ")
    }
}

fn read_be32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
        *bytes.get(offset + 3)?,
    ]))
}

#[cfg(test)]
mod tests {
    use super::FirmwareRuntime;

    #[test]
    fn loads_small_rom_and_reads_reset_vectors() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom_default()?;

        assert_eq!(firmware.reset_vectors()?, (0x0007_fff0, 0x0040_042a));
        Ok(())
    }

    #[test]
    fn reads_segmented_neo_system_boot_entry() -> Result<(), Box<dyn std::error::Error>> {
        let firmware = FirmwareRuntime::load_small_rom("../analysis/cab/os3kneorom.os3kos")?;

        assert_eq!(firmware.boot_vectors()?, (0x0007_fff0, 0x0041_0082));
        assert_eq!(firmware.neo_system_segments()?.len(), 3);
        Ok(())
    }
}
