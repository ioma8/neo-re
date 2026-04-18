use std::{thread, time::Duration};

use anyhow::bail;

use crate::protocol::{self, FileEntry};

mod jni_bridge;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NeoMode {
    Missing,
    Hid,
    HidUnavailable,
    Direct,
}

pub struct NeoClient {
    usb: jni_bridge::AndroidUsb,
}

pub fn detect_mode() -> anyhow::Result<NeoMode> {
    Ok(match jni_bridge::detect_mode()? {
        jni_bridge::UsbMode::Missing => NeoMode::Missing,
        jni_bridge::UsbMode::Hid => NeoMode::Hid,
        jni_bridge::UsbMode::HidUnavailable => NeoMode::HidUnavailable,
        jni_bridge::UsbMode::Direct => NeoMode::Direct,
    })
}

pub fn switch_hid_to_direct() -> anyhow::Result<()> {
    jni_bridge::switch_hid_to_direct()
}

pub fn wait_for_mode(target: NeoMode, attempts: usize, delay: Duration) -> anyhow::Result<bool> {
    for _ in 0..attempts {
        if detect_mode()? == target {
            return Ok(true);
        }
        thread::sleep(delay);
    }
    Ok(false)
}

impl NeoClient {
    pub fn open_and_init() -> anyhow::Result<Self> {
        let mut client = Self {
            usb: jni_bridge::AndroidUsb::open_direct()?,
        };
        client.enter_updater_mode()?;
        Ok(client)
    }

    pub fn list_files(&mut self) -> anyhow::Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        for slot in 1..=8 {
            self.write(&protocol::command(
                0x13,
                slot,
                protocol::ALPHAWORD_APPLET_ID,
            ))?;
            let response = self.read_response()?;
            if response.status == 0x90 {
                continue;
            }
            if response.status != 0x5A {
                bail!(
                    "slot {slot} attributes returned status 0x{:02x}",
                    response.status
                );
            }
            let payload = self.read_exact(response.argument as usize)?;
            validate_payload_sum(&payload, response.trailing, "attribute payload")?;
            entries.push(protocol::parse_file_entry(slot as u8, &payload)?);
        }
        Ok(entries)
    }

    pub fn download_file(&mut self, slot: u8) -> anyhow::Result<Vec<u8>> {
        let argument = (0x80000_u32 << 8) | u32::from(slot);
        self.write(&protocol::command(
            0x12,
            argument,
            protocol::ALPHAWORD_APPLET_ID,
        ))?;
        let start = self.read_response()?;
        if start.status != 0x53 {
            bail!(
                "slot {slot} retrieve start returned status 0x{:02x}",
                start.status
            );
        }

        let mut remaining = start.argument as usize;
        let mut payload = Vec::with_capacity(remaining);
        while remaining > 0 {
            self.write(&protocol::command(0x10, 0, 0))?;
            let chunk = self.read_response()?;
            if chunk.status != 0x4D {
                bail!("slot {slot} chunk returned status 0x{:02x}", chunk.status);
            }
            let bytes = self.read_exact(chunk.argument as usize)?;
            validate_payload_sum(&bytes, chunk.trailing, "chunk payload")?;
            remaining = remaining.saturating_sub(bytes.len());
            payload.extend(bytes);
        }
        Ok(payload)
    }

    fn enter_updater_mode(&mut self) -> anyhow::Result<()> {
        self.write(&protocol::reset_packet())?;
        self.write(&protocol::switch_packet())?;
        let response = self.read_exact(8)?;
        if response.as_slice() != b"Switched" {
            bail!(
                "unexpected switch response: {}",
                String::from_utf8_lossy(&response)
            );
        }
        Ok(())
    }

    fn read_response(&mut self) -> anyhow::Result<protocol::Response> {
        protocol::parse_response(&self.read_exact(8)?)
    }

    fn write(&mut self, payload: &[u8]) -> anyhow::Result<()> {
        self.usb.bulk_write(payload)
    }

    fn read_exact(&mut self, len: usize) -> anyhow::Result<Vec<u8>> {
        let mut result = Vec::with_capacity(len);
        while result.len() < len {
            let want = len - result.len();
            let chunk = self.usb.bulk_read(want)?;
            if chunk.is_empty() {
                bail!("Android bulk read returned zero bytes");
            }
            result.extend(chunk);
        }
        Ok(result)
    }
}

fn validate_payload_sum(payload: &[u8], expected: u16, label: &str) -> anyhow::Result<()> {
    let actual = payload
        .iter()
        .fold(0_u16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
    if actual != expected {
        bail!("{label} checksum mismatch: got 0x{actual:04x}, expected 0x{expected:04x}");
    }
    Ok(())
}
