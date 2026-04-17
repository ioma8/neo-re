use std::{thread, time::Duration};

use anyhow::{Context, bail};
use rusb::{DeviceHandle, GlobalContext};

use crate::protocol::{self, FileEntry};
use crate::usb_support::{find_direct_device, select_bulk_endpoints, validate_payload_sum};

pub(crate) const VID: u16 = 0x081E;
pub(crate) const PID_DIRECT: u16 = 0xBD01;
pub(crate) const PID_HID: u16 = 0xBD04;
const HID_SWITCH_REPORTS: [u8; 5] = [0xE0, 0xE1, 0xE2, 0xE3, 0xE4];
const TIMEOUT: Duration = Duration::from_millis(1_000);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NeoMode {
    Missing,
    Hid,
    Direct,
}

pub struct NeoClient {
    handle: DeviceHandle<GlobalContext>,
    in_ep: u8,
    out_ep: u8,
}

pub fn detect_mode() -> anyhow::Result<NeoMode> {
    let devices = rusb::devices().context("enumerate USB devices")?;
    let mut saw_hid = false;
    for device in devices.iter() {
        let desc = device.device_descriptor().context("read USB descriptor")?;
        if desc.vendor_id() == VID && desc.product_id() == PID_DIRECT {
            return Ok(NeoMode::Direct);
        }
        if desc.vendor_id() == VID && desc.product_id() == PID_HID {
            saw_hid = true;
        }
    }
    Ok(if saw_hid {
        NeoMode::Hid
    } else {
        NeoMode::Missing
    })
}

pub fn switch_hid_to_direct() -> anyhow::Result<()> {
    let handle = rusb::open_device_with_vid_pid(VID, PID_HID)
        .context("open AlphaSmart HID keyboard mode device")?;
    for report in HID_SWITCH_REPORTS {
        let written = handle.write_control(0x21, 0x09, 0x0200, 0, &[report], TIMEOUT)?;
        if written != 1 {
            bail!("short HID switch report write: {written}");
        }
        thread::sleep(Duration::from_millis(60));
    }
    Ok(())
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
        let device = find_direct_device()?.context("open direct USB device")?;
        let handle = device.open().context("open direct USB handle")?;
        let (interface, in_ep, out_ep) = select_bulk_endpoints(&device)?;
        if matches!(handle.kernel_driver_active(interface), Ok(true)) {
            let _ = handle.detach_kernel_driver(interface);
        }
        handle
            .claim_interface(interface)
            .context("claim direct USB interface")?;
        let mut client = Self {
            handle,
            in_ep,
            out_ep,
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
        self.handle.write_bulk(self.out_ep, payload, TIMEOUT)?;
        Ok(())
    }

    fn read_exact(&mut self, len: usize) -> anyhow::Result<Vec<u8>> {
        let mut result = Vec::with_capacity(len);
        while result.len() < len {
            let mut buffer = vec![0_u8; len - result.len()];
            let read = self.handle.read_bulk(self.in_ep, &mut buffer, TIMEOUT)?;
            result.extend_from_slice(&buffer[..read]);
        }
        Ok(result)
    }
}
