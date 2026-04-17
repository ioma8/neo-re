use anyhow::{Context, bail};

pub const ALPHAWORD_APPLET_ID: u16 = 0xA000;

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub slot: u8,
    pub name: String,
    pub attribute_bytes: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Response {
    pub status: u8,
    pub argument: u32,
    pub trailing: u16,
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

pub fn normalize_text(raw: &[u8]) -> String {
    let cleaned = raw
        .iter()
        .map(|byte| if *byte == 0 { b' ' } else { *byte })
        .collect::<Vec<_>>();
    String::from_utf8_lossy(&cleaned)
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}
