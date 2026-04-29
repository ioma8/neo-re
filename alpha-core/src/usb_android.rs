use std::{thread, time::Duration};

use anyhow::bail;

use crate::neo_client::{DirectTransport, SharedNeoClient};

mod jni_bridge;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NeoMode {
    Missing,
    Hid,
    HidUnavailable,
    Direct,
}

pub type NeoClient = SharedNeoClient<AndroidTransport>;

pub struct AndroidTransport {
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

impl SharedNeoClient<AndroidTransport> {
    pub fn open_and_init() -> anyhow::Result<Self> {
        Self::new(AndroidTransport {
            usb: jni_bridge::AndroidUsb::open_direct()?,
        })
    }
}

impl DirectTransport for AndroidTransport {
    fn write(&mut self, payload: &[u8]) -> anyhow::Result<()> {
        self.usb.bulk_write(payload)
    }

    fn read_exact(&mut self, len: usize, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        let timeout_ms = duration_to_millis(timeout);
        let mut result = Vec::with_capacity(len);
        while result.len() < len {
            let want = len - result.len();
            let chunk = self.usb.bulk_read_timeout(want, timeout_ms)?;
            if chunk.is_empty() {
                bail!("Android bulk read returned zero bytes");
            }
            result.extend(chunk);
        }
        Ok(result)
    }
}

fn duration_to_millis(timeout: Duration) -> i32 {
    timeout.as_millis().try_into().unwrap_or(i32::MAX)
}
