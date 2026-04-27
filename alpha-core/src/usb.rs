use std::{thread, time::Duration};

use anyhow::{Context, bail};
use rusb::{DeviceHandle, GlobalContext};

use crate::neo_client::{DirectTransport, SharedNeoClient};
use crate::usb_support::{find_direct_device, select_bulk_endpoints};

pub(crate) const VID: u16 = 0x081E;
pub(crate) const PID_DIRECT: u16 = 0xBD01;
pub(crate) const PID_HID: u16 = 0xBD04;
const HID_SWITCH_REPORTS: [u8; 5] = [0xE0, 0xE1, 0xE2, 0xE3, 0xE4];
const TIMEOUT: Duration = Duration::from_millis(1_000);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NeoMode {
    Missing,
    Hid,
    HidUnavailable,
    Direct,
}

pub type NeoClient = SharedNeoClient<DesktopTransport>;

pub struct DesktopTransport {
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

impl SharedNeoClient<DesktopTransport> {
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
        Self::new(DesktopTransport {
            handle,
            in_ep,
            out_ep,
        })
    }
}

impl DirectTransport for DesktopTransport {
    fn write(&mut self, payload: &[u8]) -> anyhow::Result<()> {
        let written = self.handle.write_bulk(self.out_ep, payload, TIMEOUT)?;
        if written != payload.len() {
            bail!(
                "short bulk write: wrote {written} of {} bytes",
                payload.len()
            );
        }
        Ok(())
    }

    fn read_exact(&mut self, len: usize, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        let mut result = Vec::with_capacity(len);
        while result.len() < len {
            let mut buffer = vec![0_u8; len - result.len()];
            let read = self.handle.read_bulk(self.in_ep, &mut buffer, timeout)?;
            if read == 0 {
                bail!("bulk read returned zero bytes");
            }
            result.extend_from_slice(&buffer[..read]);
        }
        Ok(result)
    }
}
