use std::path::Path;

use thiserror::Error;

const DEFAULT_SMALL_ROM_PATH: &str = "../analysis/cab/smallos3kneorom.os3kos";

#[derive(Clone, Debug)]
pub struct FirmwareRuntime {
    image: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum FirmwareError {
    #[error("failed to read firmware file")]
    Read(#[from] std::io::Error),
    #[error("firmware image is too short to contain reset vectors")]
    MissingResetVectors,
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
}
