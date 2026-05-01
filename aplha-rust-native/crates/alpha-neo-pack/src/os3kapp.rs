use std::error::Error;
use std::fmt::{Display, Formatter};

const HEADER_SIZE: usize = 0x84;
const ENTRY_OFFSET: u32 = 0x94;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppletManifest {
    pub id: u16,
    pub name: &'static str,
    pub version: Version,
    pub flags: u32,
    pub base_memory_size: u32,
    pub extra_memory_size: u32,
    pub copyright: &'static str,
    pub file_count: u8,
    pub alphaword_write_metadata: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version {
    pub major_bcd: u8,
    pub minor_bcd: u8,
}

impl Version {
    pub const fn decimal(major: u8, minor: u8) -> Self {
        Self {
            major_bcd: major,
            minor_bcd: ((minor / 10) << 4) | (minor % 10),
        }
    }
}

#[derive(Debug)]
pub enum Os3kAppError {
    EntryTooShort,
    FileTooLarge(usize),
}

impl Display for Os3kAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntryTooShort => f.write_str("compiled entry section is empty"),
            Self::FileTooLarge(size) => write!(f, "OS3KApp image is too large: {size} bytes"),
        }
    }
}

impl Error for Os3kAppError {}

pub fn build_image(manifest: &AppletManifest, entry_code: &[u8]) -> Result<Vec<u8>, Os3kAppError> {
    if entry_code.is_empty() {
        return Err(Os3kAppError::EntryTooShort);
    }

    let mut payload = Vec::with_capacity(16 + entry_code.len());
    payload.extend_from_slice(&ENTRY_OFFSET.to_be_bytes());
    payload.extend_from_slice(&0_u32.to_be_bytes());
    payload.extend_from_slice(&1_u32.to_be_bytes());
    payload.extend_from_slice(&2_u32.to_be_bytes());
    payload.extend_from_slice(entry_code);

    let info_table = build_info_table(manifest.file_count, manifest.alphaword_write_metadata);
    let info_table_len = info_table.len();
    let file_size = HEADER_SIZE + payload.len() + info_table_len;
    let file_size_u32 =
        u32::try_from(file_size).map_err(|_| Os3kAppError::FileTooLarge(file_size))?;
    let info_offset = HEADER_SIZE + payload.len();
    let info_offset_u32 =
        u32::try_from(info_offset).map_err(|_| Os3kAppError::FileTooLarge(file_size))?;

    let mut image = vec![0_u8; HEADER_SIZE];
    write_be32(&mut image, 0x00, 0xC0FF_EEAD);
    write_be32(&mut image, 0x04, file_size_u32);
    write_be32(&mut image, 0x08, manifest.base_memory_size);
    write_be32(&mut image, 0x0C, info_offset_u32);
    write_be32(&mut image, 0x10, manifest.flags);
    image[0x14..0x16].copy_from_slice(&manifest.id.to_be_bytes());
    image[0x16] = 1;
    image[0x17] = manifest.file_count;
    write_ascii_field(&mut image, 0x18, 0x28, manifest.name);
    image[0x3C] = manifest.version.major_bcd;
    image[0x3D] = manifest.version.minor_bcd;
    image[0x3F] = 1;
    write_ascii_field(&mut image, 0x40, 0x40, manifest.copyright);
    write_be32(&mut image, 0x80, manifest.extra_memory_size);

    image.extend_from_slice(&payload);
    image.extend_from_slice(&info_table);
    Ok(image)
}

pub fn validate_image(image: &[u8]) -> Result<(), Os3kAppError> {
    if image.len() < HEADER_SIZE + 16 {
        return Err(Os3kAppError::EntryTooShort);
    }

    let declared_size = u32::from_be_bytes([image[4], image[5], image[6], image[7]]);
    if declared_size as usize != image.len() {
        return Err(Os3kAppError::FileTooLarge(image.len()));
    }

    Ok(())
}

fn write_be32(target: &mut [u8], offset: usize, value: u32) {
    target[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn write_ascii_field(target: &mut [u8], offset: usize, size: usize, value: &str) {
    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(size);
    target[offset..offset + copy_len].copy_from_slice(&bytes[..copy_len]);
}

fn build_alphaword_write_info_table(file_count: u8) -> Vec<u8> {
    let mut records = Vec::new();
    if file_count != 0 {
        records.extend_from_slice(&build_info_record(0x0103, 0x8002, &[0x10, 0x02, 0x10, 0x01, 0x10, 0x02]));
    }
    records.extend_from_slice(&build_info_record(0x0105, 0x100B, b"write\0"));
    for key in 0x8011..=(0x8010 + u16::from(file_count)) {
        records.extend_from_slice(&build_info_record(0xC001, key, b"write\0"));
    }
    if file_count != 0 {
        records.extend_from_slice(&build_info_record(
            0x0102,
            0x1010,
            &[0x00, 0x00, 0xC8, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x01, 0x90, 0x00],
        ));
        records.extend_from_slice(&build_info_record(
            0x0102,
            0x1011,
            &[0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x64, 0x00],
        ));
        records.extend_from_slice(&build_info_record(
            0x0106,
            0x8004,
            b"(one page equals 2048 characters)\0",
        ));
        records.extend_from_slice(&build_info_record(0x0103, 0x100F, &[0x10, 0x02, 0x10, 0x01, 0x10, 0x02]));
        records.extend_from_slice(&build_info_record(0x0103, 0x8003, &[0x10, 0x02, 0x10, 0x01, 0x10, 0x02]));
    }
    records
}

fn build_info_table(file_count: u8, alphaword_write_metadata: bool) -> Vec<u8> {
    let mut records = if alphaword_write_metadata {
        build_alphaword_write_info_table(file_count)
    } else {
        Vec::new()
    };
    records.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0xCA, 0xFE, 0xFE, 0xED]);
    records
}

fn build_info_record(group: u16, key: u16, payload: &[u8]) -> Vec<u8> {
    let Ok(payload_len) = u16::try_from(payload.len()) else {
        return Vec::new();
    };
    let mut record = Vec::with_capacity(6 + payload.len() + (payload.len() & 1));
    record.extend_from_slice(&group.to_be_bytes());
    record.extend_from_slice(&key.to_be_bytes());
    record.extend_from_slice(&payload_len.to_be_bytes());
    record.extend_from_slice(payload);
    if payload.len() & 1 == 1 {
        record.push(0);
    }
    record
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packages_alpha_usb_shape() -> Result<(), Box<dyn Error>> {
        let manifest = AppletManifest {
            id: 0xA130,
            name: "Alpha USB",
            version: Version::decimal(1, 20),
            flags: 0xFF00_00CE,
            base_memory_size: 0x100,
            extra_memory_size: 0x2000,
            copyright: "neo-re benign SmartApplet probe",
            file_count: 0,
            alphaword_write_metadata: true,
        };
        let image = build_image(&manifest, &[0x4E, 0x75])?;

        assert_eq!(&image[0x00..0x04], &[0xC0, 0xFF, 0xEE, 0xAD]);
        assert_eq!(&image[0x14..0x18], &[0xA1, 0x30, 0x01, 0x00]);
        assert_eq!(&image[0x3C..0x40], &[0x01, 0x20, 0x00, 0x01]);
        assert_eq!(&image[0x94..0x96], &[0x4E, 0x75]);
        assert_eq!(&image[image.len() - 4..], &[0xCA, 0xFE, 0xFE, 0xED]);
        validate_image(&image)?;
        Ok(())
    }

    #[test]
    fn packages_forth_mini_shape() -> Result<(), Box<dyn Error>> {
        let manifest = AppletManifest {
            id: 0xA131,
            name: "Forth Mini",
            version: Version::decimal(0, 2),
            flags: 0xFF00_00CE,
            base_memory_size: 0x4000,
            extra_memory_size: 0x2000,
            copyright: "neo-re Betawise Forth SmartApplet",
            file_count: 1,
            alphaword_write_metadata: true,
        };
        let image = build_image(&manifest, &[0x4E, 0x75])?;
        let slot_1 = [
            0xC0, 0x01, 0x80, 0x11, 0x00, 0x06, b'w', b'r', b'i', b't', b'e', 0x00,
        ];

        assert_eq!(&image[0x00..0x04], &[0xC0, 0xFF, 0xEE, 0xAD]);
        assert_eq!(&image[0x14..0x16], &[0xA1, 0x31]);
        assert_eq!(image[0x17], 1);
        assert_eq!(&image[0x3C..0x40], &[0x00, 0x02, 0x00, 0x01]);
        assert_eq!(&image[0x94..0x96], &[0x4E, 0x75]);
        assert_eq!(&image[image.len() - 4..], &[0xCA, 0xFE, 0xFE, 0xED]);
        assert!(image.windows(slot_1.len()).any(|window| window == slot_1));
        validate_image(&image)?;
        Ok(())
    }

    #[test]
    fn packages_basic_writer_shape() -> Result<(), Box<dyn Error>> {
        let manifest = AppletManifest {
            id: 0xA132,
            name: "Basic Writer",
            version: Version::decimal(0, 1),
            flags: 0xFF00_00CE,
            base_memory_size: 0x4000,
            extra_memory_size: 0x2000,
            copyright: "neo-re Betawise Basic Writer SmartApplet",
            file_count: 8,
            alphaword_write_metadata: true,
        };
        let image = build_image(&manifest, &[0x4E, 0x75])?;

        assert_eq!(&image[0x00..0x04], &[0xC0, 0xFF, 0xEE, 0xAD]);
        assert_eq!(&image[0x14..0x18], &[0xA1, 0x32, 0x01, 0x08]);
        assert_eq!(image[0x17], 8);
        assert_eq!(&image[0x18..0x24], b"Basic Writer");
        assert_eq!(&image[0x94..0x96], &[0x4E, 0x75]);
        assert_eq!(&image[image.len() - 4..], &[0xCA, 0xFE, 0xFE, 0xED]);
        for key in 0x8011_u16..=0x8018 {
            let record = [
                0xC0,
                0x01,
                (key >> 8) as u8,
                key as u8,
                0x00,
                0x06,
                b'w',
                b'r',
                b'i',
                b't',
                b'e',
                0x00,
            ];
            assert!(image.windows(record.len()).any(|window| window == record));
        }
        validate_image(&image)?;
        Ok(())
    }

    #[test]
    fn packages_single_owned_file_shape() -> Result<(), Box<dyn Error>> {
        let manifest = AppletManifest {
            id: 0xA131,
            name: "Forth Mini",
            version: Version::decimal(0, 2),
            flags: 0xFF00_00CE,
            base_memory_size: 0x800,
            extra_memory_size: 0x2000,
            copyright: "neo-re native Rust SmartApplet",
            file_count: 1,
            alphaword_write_metadata: true,
        };
        let image = build_image(&manifest, &[0x4E, 0x75])?;
        let slot_1 = [
            0xC0, 0x01, 0x80, 0x11, 0x00, 0x06, b'w', b'r', b'i', b't', b'e', 0x00,
        ];
        let slot_2 = [
            0xC0, 0x01, 0x80, 0x12, 0x00, 0x06, b'w', b'r', b'i', b't', b'e', 0x00,
        ];

        assert_eq!(&image[0x14..0x18], &[0xA1, 0x31, 0x01, 0x01]);
        assert!(image.windows(slot_1.len()).any(|window| window == slot_1));
        assert!(!image.windows(slot_2.len()).any(|window| window == slot_2));
        validate_image(&image)?;
        Ok(())
    }
}
