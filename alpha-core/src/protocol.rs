use anyhow::{Context, bail};

pub const ALPHAWORD_APPLET_ID: u16 = 0xA000;
pub const SMARTAPPLET_HEADER_SIZE: usize = 0x84;

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub slot: u8,
    pub name: String,
    pub attribute_bytes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmartAppletRecord {
    pub applet_id: u16,
    pub version: String,
    pub name: String,
    pub file_size: u32,
    pub applet_class: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Response {
    pub status: u8,
    pub argument: u32,
    pub trailing: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmartAppletHeader {
    pub magic: u32,
    pub file_size: u32,
    pub base_memory_size: u32,
    pub payload_or_code_size: u32,
    pub flags_and_version: u32,
    pub applet_id: u16,
    pub header_version: u8,
    pub file_count: u8,
    pub name: String,
    pub version_major: u8,
    pub version_minor: u8,
    pub applet_class: u8,
    pub copyright: String,
    pub extra_memory_size: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OsFlashSegment {
    pub address: u32,
    pub length: u32,
}

pub fn reset_packet() -> [u8; 8] {
    *b"?\xff\0reset"
}

pub fn switch_packet() -> [u8; 8] {
    *b"?Swtch\0\0"
}

pub fn command(code: u8, argument: u32, trailing: u16) -> [u8; 8] {
    let mut packet = [0_u8; 8];
    packet[0] = code;
    packet[1..5].copy_from_slice(&argument.to_be_bytes());
    packet[5..7].copy_from_slice(&trailing.to_be_bytes());
    packet[7] = packet[..7]
        .iter()
        .fold(0_u8, |sum, byte| sum.wrapping_add(*byte));
    packet
}

pub fn list_applets_command(page_offset: u32, page_size: u16) -> [u8; 8] {
    command(0x04, page_offset, page_size)
}

pub fn retrieve_applet_command(applet_id: u16) -> [u8; 8] {
    command(0x0F, 0, applet_id)
}

pub fn retrieve_chunk_command() -> [u8; 8] {
    command(0x10, 0, 0)
}

pub fn add_applet_begin_command(argument: u32, trailing: u16) -> [u8; 8] {
    command(0x06, argument, trailing)
}

pub fn program_applet_command() -> [u8; 8] {
    command(0x0B, 0, 0)
}

pub fn finalize_applet_update_command() -> [u8; 8] {
    command(0x07, 0, 0)
}

pub fn remove_applet_by_index_command(index: u16) -> [u8; 8] {
    command(0x05, 5, index)
}

pub fn clear_applet_area_command() -> [u8; 8] {
    command(0x11, 0, 0)
}

pub fn enter_small_rom_command() -> [u8; 8] {
    command(0x18, 0, 0)
}

pub fn clear_os_segment_map_command() -> [u8; 8] {
    command(0x16, 0, 0)
}

pub fn erase_os_segment_command(address: u32, erase_kb: u16) -> [u8; 8] {
    command(0x17, address, erase_kb)
}

pub fn restart_device_command() -> [u8; 8] {
    command(0x08, 0, 0)
}

pub fn parse_smartapplet_header(packet: &[u8]) -> anyhow::Result<SmartAppletHeader> {
    if packet.len() != SMARTAPPLET_HEADER_SIZE {
        bail!(
            "SmartApplet header must be {SMARTAPPLET_HEADER_SIZE} bytes, got {}",
            packet.len()
        );
    }
    Ok(SmartAppletHeader {
        magic: u32::from_be_bytes(packet[0x00..0x04].try_into().context("applet magic")?),
        file_size: u32::from_be_bytes(packet[0x04..0x08].try_into().context("applet file size")?),
        base_memory_size: u32::from_be_bytes(
            packet[0x08..0x0C]
                .try_into()
                .context("applet base memory size")?,
        ),
        payload_or_code_size: u32::from_be_bytes(
            packet[0x0C..0x10]
                .try_into()
                .context("applet payload/code size")?,
        ),
        flags_and_version: u32::from_be_bytes(
            packet[0x10..0x14]
                .try_into()
                .context("applet flags/version")?,
        ),
        applet_id: u16::from_be_bytes(packet[0x14..0x16].try_into().context("applet id")?),
        header_version: packet[0x16],
        file_count: packet[0x17],
        name: read_c_string(packet, 0x18, 0x28),
        version_major: decode_bcd(packet[0x3C]),
        version_minor: decode_bcd(packet[0x3D]),
        applet_class: packet[0x3F],
        copyright: read_c_string(packet, 0x40, 0x40),
        extra_memory_size: u32::from_be_bytes(
            packet[0x80..0x84]
                .try_into()
                .context("applet extra memory size")?,
        ),
    })
}

pub fn parse_smartapplet_record(packet: &[u8]) -> anyhow::Result<SmartAppletRecord> {
    let header = parse_smartapplet_header(packet)?;
    Ok(SmartAppletRecord {
        applet_id: header.applet_id,
        version: format!("{}.{}", header.version_major, header.version_minor),
        name: header.name,
        file_size: header.file_size,
        applet_class: header.applet_class,
    })
}

pub fn derive_add_applet_start_fields(header: &SmartAppletHeader) -> (u32, u16) {
    let combined_memory_size = header
        .base_memory_size
        .wrapping_add(header.extra_memory_size);
    let argument = header.file_size | ((combined_memory_size & 0xFFFF_0000) << 8);
    let trailing = (combined_memory_size & 0xFFFF) as u16;
    (argument, trailing)
}

pub fn parse_neo_os_segments(image: &[u8]) -> anyhow::Result<Vec<OsFlashSegment>> {
    if image.len() < 0x70 {
        bail!("OS image is too short");
    }
    if &image[6..24] != b"System 3 Neo      " {
        bail!("OS image is not a NEO System 3 image");
    }
    if !(image.len() - 0x50).is_multiple_of(8) {
        bail!("truncated OS segment table");
    }
    let mut segments = Vec::new();
    for offset in (0x50..image.len()).step_by(8) {
        let address = u32::from_be_bytes(
            image[offset..offset + 4]
                .try_into()
                .context("OS segment address")?,
        );
        let length = u32::from_be_bytes(
            image[offset + 4..offset + 8]
                .try_into()
                .context("OS segment length")?,
        );
        if address == 0 && length == 0 {
            break;
        }
        if length == 0 {
            bail!("invalid OS segment length at offset 0x{offset:x}");
        }
        segments.push(OsFlashSegment { address, length });
    }
    if segments.is_empty() {
        bail!("OS image contains no flash segments");
    }
    Ok(segments)
}

pub fn parse_response(packet: &[u8]) -> anyhow::Result<Response> {
    if packet.len() != 8 {
        bail!("updater response must be 8 bytes, got {}", packet.len());
    }
    let expected = packet[..7]
        .iter()
        .fold(0_u8, |sum, byte| sum.wrapping_add(*byte));
    if packet[7] != expected {
        bail!(
            "updater response checksum mismatch: got 0x{:02x}, expected 0x{:02x}",
            packet[7],
            expected
        );
    }
    Ok(Response {
        status: packet[0],
        argument: u32::from_be_bytes(packet[1..5].try_into().context("response argument")?),
        trailing: u16::from_be_bytes(packet[5..7].try_into().context("response trailing")?),
    })
}

pub fn parse_file_entry(slot: u8, payload: &[u8]) -> anyhow::Result<FileEntry> {
    if payload.len() != 0x28 {
        bail!(
            "file attribute record must be 40 bytes, got {}",
            payload.len()
        );
    }
    let name = String::from_utf8_lossy(
        payload[..0x18]
            .split(|byte| *byte == 0)
            .next()
            .unwrap_or_default(),
    )
    .trim()
    .to_owned();
    let attribute_bytes =
        u32::from_be_bytes(payload[0x1c..0x20].try_into().context("file length")?);
    Ok(FileEntry {
        slot,
        name,
        attribute_bytes,
    })
}

fn decode_bcd(value: u8) -> u8 {
    (value & 0x0F) + ((value >> 4) & 0x0F) * 10
}

fn read_c_string(raw: &[u8], offset: usize, size: usize) -> String {
    String::from_utf8_lossy(
        raw[offset..offset + size]
            .split(|byte| *byte == 0)
            .next()
            .unwrap_or_default(),
    )
    .trim()
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_smartapplet_header_metadata() {
        let mut header = [0_u8; SMARTAPPLET_HEADER_SIZE];
        header[0x00..0x04].copy_from_slice(&0xC0FF_EEAD_u32.to_be_bytes());
        header[0x04..0x08].copy_from_slice(&0x1234_u32.to_be_bytes());
        header[0x08..0x0C].copy_from_slice(&0x0100_u32.to_be_bytes());
        header[0x0C..0x10].copy_from_slice(&0x0094_u32.to_be_bytes());
        header[0x10..0x14].copy_from_slice(&0xFF00_00CE_u32.to_be_bytes());
        header[0x14..0x16].copy_from_slice(&0xA130_u16.to_be_bytes());
        header[0x16] = 1;
        header[0x17] = 1;
        header[0x18..0x18 + b"Alpha USB".len()].copy_from_slice(b"Alpha USB");
        header[0x3C] = 0x01;
        header[0x3D] = 0x20;
        header[0x3F] = 0x01;
        header[0x40..0x40 + b"neo-re".len()].copy_from_slice(b"neo-re");
        header[0x80..0x84].copy_from_slice(&0x2000_u32.to_be_bytes());

        let parsed = parse_smartapplet_header(&header).unwrap();

        assert_eq!(parsed.applet_id, 0xA130);
        assert_eq!(parsed.name, "Alpha USB");
        assert_eq!(parsed.version_major, 1);
        assert_eq!(parsed.version_minor, 20);
        assert_eq!(parsed.extra_memory_size, 0x2000);
    }

    #[test]
    fn derives_add_applet_start_fields() {
        let header = SmartAppletHeader {
            magic: 0,
            file_size: 0x1234,
            base_memory_size: 0x0100,
            payload_or_code_size: 0,
            flags_and_version: 0,
            applet_id: 0,
            header_version: 1,
            file_count: 1,
            name: String::new(),
            version_major: 0,
            version_minor: 0,
            applet_class: 1,
            copyright: String::new(),
            extra_memory_size: 0x2000,
        };

        assert_eq!(derive_add_applet_start_fields(&header), (0x1234, 0x2100));
    }

    #[test]
    fn builds_smartapplet_commands() {
        assert_eq!(list_applets_command(3, 7), command(0x04, 3, 7));
        assert_eq!(retrieve_applet_command(0xA130), command(0x0F, 0, 0xA130));
        assert_eq!(
            add_applet_begin_command(0x1234, 0x2100),
            command(0x06, 0x1234, 0x2100)
        );
        assert_eq!(program_applet_command(), command(0x0B, 0, 0));
        assert_eq!(finalize_applet_update_command(), command(0x07, 0, 0));
        assert_eq!(clear_applet_area_command(), command(0x11, 0, 0));
    }

    #[test]
    fn parses_neo_os_segments() {
        let mut image = vec![0_u8; 0x70];
        image[6..24].copy_from_slice(b"System 3 Neo      ");
        image[0x50..0x54].copy_from_slice(&0x0041_0000_u32.to_be_bytes());
        image[0x54..0x58].copy_from_slice(&0x0006_0000_u32.to_be_bytes());
        image[0x58..0x5C].copy_from_slice(&0x005F_FC00_u32.to_be_bytes());
        image[0x5C..0x60].copy_from_slice(&0x0000_0400_u32.to_be_bytes());

        let segments = parse_neo_os_segments(&image).unwrap();

        assert_eq!(
            segments,
            vec![
                OsFlashSegment {
                    address: 0x0041_0000,
                    length: 0x0006_0000,
                },
                OsFlashSegment {
                    address: 0x005F_FC00,
                    length: 0x0000_0400,
                },
            ]
        );
    }

    #[test]
    fn rejects_truncated_neo_os_segment_table() {
        let mut image = vec![0_u8; 0x75];
        image[6..24].copy_from_slice(b"System 3 Neo      ");
        image[0x50..0x54].copy_from_slice(&0x0041_0000_u32.to_be_bytes());
        image[0x54..0x58].copy_from_slice(&0x0006_0000_u32.to_be_bytes());
        image[0x58..0x5C].copy_from_slice(&0x005F_FC00_u32.to_be_bytes());
        image[0x5C..0x60].copy_from_slice(&0x0000_0400_u32.to_be_bytes());

        let error = parse_neo_os_segments(&image).unwrap_err().to_string();

        assert!(error.contains("truncated OS segment table"));
    }
}
