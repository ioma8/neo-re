use std::path::Path;

use thiserror::Error;

use crate::domain::AppletMetadata;

const HEADER_SIZE: usize = 0x84;

#[derive(Clone, Debug)]
pub struct Os3kApp {
    pub metadata: AppletMetadata,
    pub entry_offset: u32,
    pub image: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum Os3kAppError {
    #[error("OS3KApp image is too short")]
    TooShort,
    #[error("OS3KApp magic mismatch")]
    BadMagic,
    #[error("declared OS3KApp size {declared} does not match actual size {actual}")]
    SizeMismatch { declared: usize, actual: usize },
    #[error("OS3KApp entry offset is outside image: 0x{0:08x}")]
    BadEntry(u32),
    #[error("failed to read OS3KApp file")]
    Read(#[from] std::io::Error),
}

impl Os3kApp {
    /// Reads and parses an `OS3KApp` package from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the package shape is
    /// invalid for the subset supported by this emulator.
    pub fn read(path: impl AsRef<Path>) -> Result<Self, Os3kAppError> {
        Self::parse(std::fs::read(path)?)
    }

    /// Parses an `OS3KApp` package image.
    ///
    /// # Errors
    ///
    /// Returns an error if required header fields are missing, inconsistent, or
    /// point outside the image.
    pub fn parse(image: Vec<u8>) -> Result<Self, Os3kAppError> {
        if image.len() < HEADER_SIZE + 16 {
            return Err(Os3kAppError::TooShort);
        }
        if read_be32(&image, 0x00) != 0xC0FF_EEAD {
            return Err(Os3kAppError::BadMagic);
        }

        let declared = read_be32(&image, 0x04) as usize;
        if declared != image.len() {
            return Err(Os3kAppError::SizeMismatch {
                declared,
                actual: image.len(),
            });
        }

        let entry_offset = read_be32(&image, HEADER_SIZE);
        if entry_offset as usize >= image.len() {
            return Err(Os3kAppError::BadEntry(entry_offset));
        }

        let metadata = AppletMetadata {
            applet_id: read_be16(&image, 0x14),
            name: read_ascii(&image[0x18..0x40]),
            version_major: image[0x3C],
            version_minor_bcd: image[0x3D],
        };

        Ok(Self {
            metadata,
            entry_offset,
            image,
        })
    }
}

fn read_be16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_be32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_ascii(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::Os3kApp;

    #[test]
    fn loads_alpha_usb_package() -> Result<(), Box<dyn std::error::Error>> {
        let app = Os3kApp::read("../exports/applets/alpha-usb-native.os3kapp")?;
        assert_eq!(app.metadata.applet_id, 0xA130);
        assert_eq!(app.metadata.name, "Alpha USB");
        assert_eq!(app.entry_offset, 0x94);
        assert!(app.image.len() > app.entry_offset as usize);
        Ok(())
    }
}
