use anyhow::bail;
use rusb::{Device, GlobalContext};

use crate::usb::{PID_DIRECT, VID};

pub fn find_direct_device() -> anyhow::Result<Option<Device<GlobalContext>>> {
    for device in rusb::devices()?.iter() {
        let desc = device.device_descriptor()?;
        if desc.vendor_id() == VID && desc.product_id() == PID_DIRECT {
            return Ok(Some(device));
        }
    }
    Ok(None)
}

pub fn select_bulk_endpoints(device: &Device<GlobalContext>) -> anyhow::Result<(u8, u8, u8)> {
    let config = device.active_config_descriptor()?;
    for interface in config.interfaces() {
        for descriptor in interface.descriptors() {
            let mut in_ep = None;
            let mut out_ep = None;
            for endpoint in descriptor.endpoint_descriptors() {
                if endpoint.transfer_type() == rusb::TransferType::Bulk {
                    match endpoint.direction() {
                        rusb::Direction::In => in_ep = Some(endpoint.address()),
                        rusb::Direction::Out => out_ep = Some(endpoint.address()),
                    }
                }
            }
            if let (Some(input), Some(output)) = (in_ep, out_ep) {
                return Ok((descriptor.interface_number(), input, output));
            }
        }
    }
    bail!("direct USB device has no bulk IN/OUT endpoint pair")
}
