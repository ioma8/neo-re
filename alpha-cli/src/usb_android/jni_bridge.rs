use std::{thread, time::Duration};

use anyhow::bail;
use jni::{
    Env, JValue, JavaVM, jni_sig, jni_str,
    objects::{Global, JObject, JObjectArray},
    refs::Reference,
    sys::{jbyte, jobjectArray},
};
use tracing::{info, warn};

const VID: i32 = 0x081E;
const PID_DIRECT: i32 = 0xBD01;
const PID_HID: i32 = 0xBD04;
const HID_SWITCH_REPORTS: [u8; 5] = [0xE0, 0xE1, 0xE2, 0xE3, 0xE4];
const TIMEOUT_MS: i32 = 1_000;
const USB_ENDPOINT_XFER_BULK: i32 = 2;
const USB_DIR_IN: i32 = 0x80;
const ACTION_USB_PERMISSION: &str = "cz.jakubkolcar.alpha_cli.USB_PERMISSION";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsbMode {
    Missing,
    Hid,
    HidUnavailable,
    Direct,
}

pub struct AndroidUsb {
    connection: Global<JObject<'static>>,
    in_endpoint: Global<JObject<'static>>,
    out_endpoint: Global<JObject<'static>>,
}

struct AndroidDevice<'local> {
    device: JObject<'local>,
    product_id: i32,
}

struct EndpointSelection<'local> {
    interface: JObject<'local>,
    in_endpoint: JObject<'local>,
    out_endpoint: JObject<'local>,
}

pub fn detect_mode() -> anyhow::Result<UsbMode> {
    with_env(|env| {
        let devices = alphasmart_devices(env)?;
        info!(
            count = devices.len(),
            "Android AlphaSmart USB scan complete"
        );
        if devices.iter().any(|device| device.product_id == PID_DIRECT) {
            return Ok(UsbMode::Direct);
        }
        if devices.iter().any(|device| device.product_id == PID_HID) {
            return Ok(UsbMode::Hid);
        }
        if alphasmart_input_keyboard_present(env)? {
            return Ok(UsbMode::HidUnavailable);
        }
        Ok(UsbMode::Missing)
    })
}

pub fn switch_hid_to_direct() -> anyhow::Result<()> {
    with_env(|env| {
        let manager = usb_manager(env)?;
        let Some(device) = alphasmart_devices(env)?
            .into_iter()
            .find(|device| device.product_id == PID_HID)
        else {
            warn!("AlphaSmart HID mode device not found during switch attempt");
            bail!("AlphaSmart HID keyboard mode device not found");
        };
        info!("requesting Android USB permission for HID switch");
        ensure_permission(env, &manager, &device.device)?;
        let connection = open_device(env, &manager, &device.device)?;
        for report in HID_SWITCH_REPORTS {
            let written = control_transfer(env, &connection, 0x21, 0x09, 0x0200, 0, &[report])?;
            info!(
                report = report,
                written = written,
                "sent Android HID switch report"
            );
            if written != 1 {
                bail!("short HID switch report write: {written}");
            }
            thread::sleep(Duration::from_millis(60));
        }
        Ok(())
    })
}

impl AndroidUsb {
    pub fn open_direct() -> anyhow::Result<Self> {
        with_env(|env| {
            let manager = usb_manager(env)?;
            let Some(device) = alphasmart_devices(env)?
                .into_iter()
                .find(|device| device.product_id == PID_DIRECT)
            else {
                warn!("AlphaSmart direct mode device not found during open");
                bail!("AlphaSmart direct USB device not found");
            };
            info!("requesting Android USB permission for direct mode");
            ensure_permission(env, &manager, &device.device)?;
            let selection = select_bulk_endpoints(env, &device.device)?;
            let connection = open_device(env, &manager, &device.device)?;
            let claimed = env
                .call_method(
                    &connection,
                    jni_str!("claimInterface"),
                    jni_sig!("(Landroid/hardware/usb/UsbInterface;Z)Z"),
                    &[JValue::Object(&selection.interface), JValue::Bool(true)],
                )?
                .z()?;
            if !claimed {
                bail!("Android UsbDeviceConnection.claimInterface returned false");
            }
            Ok(Self {
                connection: env.new_global_ref(&connection)?,
                in_endpoint: env.new_global_ref(&selection.in_endpoint)?,
                out_endpoint: env.new_global_ref(&selection.out_endpoint)?,
            })
        })
    }

    pub fn bulk_write(&self, payload: &[u8]) -> anyhow::Result<()> {
        let written =
            with_env(|env| bulk_transfer(env, &self.connection, &self.out_endpoint, payload))?;
        if written != payload.len() {
            bail!(
                "short Android bulk write: wrote {written} of {} bytes",
                payload.len()
            );
        }
        Ok(())
    }

    pub fn bulk_read_timeout(&self, length: usize, timeout_ms: i32) -> anyhow::Result<Vec<u8>> {
        with_env(|env| bulk_read(env, &self.connection, &self.in_endpoint, length, timeout_ms))
    }
}

fn with_env<T>(f: impl FnOnce(&mut Env<'_>) -> anyhow::Result<T>) -> anyhow::Result<T> {
    let context = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(context.vm().cast()) };
    vm.attach_current_thread(f)
}

fn usb_manager<'local>(env: &mut Env<'local>) -> anyhow::Result<JObject<'local>> {
    let service_name = env.new_string("usb")?;
    let context = android_context_object(env)?;
    let manager = env
        .call_method(
            &context,
            jni_str!("getSystemService"),
            jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
            &[JValue::Object(&service_name)],
        )?
        .l()?;
    if manager.is_null() {
        bail!("Android UsbManager service is not available");
    }
    Ok(manager)
}

fn android_context_object<'local>(env: &mut Env<'local>) -> anyhow::Result<JObject<'local>> {
    let context = ndk_context::android_context();
    let borrowed = unsafe { JObject::from_raw(env, context.context().cast()) };
    env.new_local_ref(&borrowed).map_err(Into::into)
}

fn alphasmart_devices<'local>(env: &mut Env<'local>) -> anyhow::Result<Vec<AndroidDevice<'local>>> {
    let manager = usb_manager(env)?;
    let map = env
        .call_method(
            &manager,
            jni_str!("getDeviceList"),
            jni_sig!("()Ljava/util/HashMap;"),
            &[],
        )?
        .l()?;
    let values = env
        .call_method(
            &map,
            jni_str!("values"),
            jni_sig!("()Ljava/util/Collection;"),
            &[],
        )?
        .l()?;
    let array_object = env
        .call_method(
            &values,
            jni_str!("toArray"),
            jni_sig!("()[Ljava/lang/Object;"),
            &[],
        )?
        .l()?;
    let array =
        unsafe { JObjectArray::<JObject>::from_raw(env, array_object.into_raw() as jobjectArray) };
    let len = array.len(env)?;
    let mut devices = Vec::new();
    for index in 0..len {
        let device = array.get_element(env, index)?;
        let vendor_id = int_method(env, &device, "getVendorId")?;
        let product_id = int_method(env, &device, "getProductId")?;
        if vendor_id == VID && matches!(product_id, PID_DIRECT | PID_HID) {
            devices.push(AndroidDevice { device, product_id });
        }
    }
    Ok(devices)
}

fn alphasmart_input_keyboard_present(env: &mut Env<'_>) -> anyhow::Result<bool> {
    let ids = env
        .call_static_method(
            jni_str!("android/view/InputDevice"),
            jni_str!("getDeviceIds"),
            jni_sig!("()[I"),
            &[],
        )?
        .l()?;
    let ids = unsafe { jni::objects::JIntArray::from_raw(env, ids.into_raw().cast()) };
    let len = ids.len(env)?;
    let mut values = vec![0_i32; len];
    ids.get_region(env, 0, &mut values)?;
    for id in values {
        let device = env
            .call_static_method(
                jni_str!("android/view/InputDevice"),
                jni_str!("getDevice"),
                jni_sig!("(I)Landroid/view/InputDevice;"),
                &[JValue::Int(id)],
            )?
            .l()?;
        if device.is_null() {
            continue;
        }
        let vendor_id = int_method(env, &device, "getVendorId")?;
        let product_id = int_method(env, &device, "getProductId")?;
        if vendor_id == VID && product_id == PID_HID {
            info!(
                "AlphaSmart HID keyboard is present through Android InputDevice but denied to UsbManager"
            );
            return Ok(true);
        }
    }
    Ok(false)
}

fn ensure_permission(
    env: &mut Env<'_>,
    manager: &JObject<'_>,
    device: &JObject<'_>,
) -> anyhow::Result<()> {
    let has_permission = env
        .call_method(
            manager,
            jni_str!("hasPermission"),
            jni_sig!("(Landroid/hardware/usb/UsbDevice;)Z"),
            &[JValue::Object(device)],
        )?
        .z()?;
    if has_permission {
        info!("Android USB permission already granted");
        return Ok(());
    }

    let permission_intent = permission_intent(env)?;
    info!("requesting Android USB permission dialog");
    env.call_method(
        manager,
        jni_str!("requestPermission"),
        jni_sig!("(Landroid/hardware/usb/UsbDevice;Landroid/app/PendingIntent;)V"),
        &[JValue::Object(device), JValue::Object(&permission_intent)],
    )?;
    bail!("Android USB permission requested; approve the dialog and try again")
}

fn permission_intent<'local>(env: &mut Env<'local>) -> anyhow::Result<JObject<'local>> {
    let action = env.new_string(ACTION_USB_PERMISSION)?;
    let intent = env.new_object(
        jni_str!("android/content/Intent"),
        jni_sig!("(Ljava/lang/String;)V"),
        &[JValue::Object(&action)],
    )?;
    let context = android_context_object(env)?;
    let flags = 0x0800_0000_i32 | 0x0400_0000_i32;
    Ok(env
        .call_static_method(
            jni_str!("android/app/PendingIntent"),
            jni_str!("getBroadcast"),
            jni_sig!(
                "(Landroid/content/Context;ILandroid/content/Intent;I)Landroid/app/PendingIntent;"
            ),
            &[
                JValue::Object(&context),
                JValue::Int(0),
                JValue::Object(&intent),
                JValue::Int(flags),
            ],
        )?
        .l()?)
}

fn open_device<'local>(
    env: &mut Env<'local>,
    manager: &JObject<'_>,
    device: &JObject<'_>,
) -> anyhow::Result<JObject<'local>> {
    let connection = env
        .call_method(
            manager,
            jni_str!("openDevice"),
            jni_sig!(
                "(Landroid/hardware/usb/UsbDevice;)Landroid/hardware/usb/UsbDeviceConnection;"
            ),
            &[JValue::Object(device)],
        )?
        .l()?;
    if connection.is_null() {
        bail!("Android UsbManager.openDevice returned null");
    }
    Ok(connection)
}

fn select_bulk_endpoints<'local>(
    env: &mut Env<'local>,
    device: &JObject<'local>,
) -> anyhow::Result<EndpointSelection<'local>> {
    let interface_count = int_method(env, device, "getInterfaceCount")?;
    for index in 0..interface_count {
        let interface = env
            .call_method(
                device,
                jni_str!("getInterface"),
                jni_sig!("(I)Landroid/hardware/usb/UsbInterface;"),
                &[JValue::Int(index)],
            )?
            .l()?;
        if let Some((in_endpoint, out_endpoint)) = bulk_endpoints(env, &interface)? {
            return Ok(EndpointSelection {
                interface,
                in_endpoint,
                out_endpoint,
            });
        }
    }
    bail!("Android direct USB device has no bulk IN/OUT endpoint pair")
}

fn bulk_endpoints<'local>(
    env: &mut Env<'local>,
    interface: &JObject<'local>,
) -> anyhow::Result<Option<(JObject<'local>, JObject<'local>)>> {
    let endpoint_count = int_method(env, interface, "getEndpointCount")?;
    let mut input = None;
    let mut output = None;
    for endpoint_index in 0..endpoint_count {
        let endpoint = env
            .call_method(
                interface,
                jni_str!("getEndpoint"),
                jni_sig!("(I)Landroid/hardware/usb/UsbEndpoint;"),
                &[JValue::Int(endpoint_index)],
            )?
            .l()?;
        if int_method(env, &endpoint, "getType")? == USB_ENDPOINT_XFER_BULK {
            if int_method(env, &endpoint, "getDirection")? == USB_DIR_IN {
                input = Some(endpoint);
            } else {
                output = Some(endpoint);
            }
        }
    }
    Ok(input.zip(output))
}

fn int_method(env: &mut Env<'_>, obj: &JObject<'_>, name: &'static str) -> anyhow::Result<i32> {
    let value = match name {
        "getVendorId" => env.call_method(obj, jni_str!("getVendorId"), jni_sig!("()I"), &[])?,
        "getProductId" => env.call_method(obj, jni_str!("getProductId"), jni_sig!("()I"), &[])?,
        "getInterfaceCount" => {
            env.call_method(obj, jni_str!("getInterfaceCount"), jni_sig!("()I"), &[])?
        }
        "getEndpointCount" => {
            env.call_method(obj, jni_str!("getEndpointCount"), jni_sig!("()I"), &[])?
        }
        "getType" => env.call_method(obj, jni_str!("getType"), jni_sig!("()I"), &[])?,
        "getDirection" => env.call_method(obj, jni_str!("getDirection"), jni_sig!("()I"), &[])?,
        _ => bail!("unsupported Android integer method {name}"),
    };
    Ok(value.i()?)
}

fn bulk_transfer(
    env: &mut Env<'_>,
    connection: &Global<JObject<'static>>,
    endpoint: &Global<JObject<'static>>,
    payload: &[u8],
) -> anyhow::Result<usize> {
    let array = env.new_byte_array(payload.len())?;
    let bytes = payload
        .iter()
        .map(|byte| *byte as jbyte)
        .collect::<Vec<_>>();
    array.set_region(env, 0, &bytes)?;
    let transferred = env
        .call_method(
            connection.as_obj(),
            jni_str!("bulkTransfer"),
            jni_sig!("(Landroid/hardware/usb/UsbEndpoint;[BII)I"),
            &[
                JValue::Object(endpoint.as_obj()),
                JValue::Object(array.as_ref()),
                JValue::Int(payload.len() as i32),
                JValue::Int(TIMEOUT_MS),
            ],
        )?
        .i()?;
    if transferred < 0 {
        bail!("Android bulkTransfer write failed with {transferred}");
    }
    Ok(transferred as usize)
}

fn bulk_read(
    env: &mut Env<'_>,
    connection: &Global<JObject<'static>>,
    endpoint: &Global<JObject<'static>>,
    length: usize,
    timeout_ms: i32,
) -> anyhow::Result<Vec<u8>> {
    let array = env.new_byte_array(length)?;
    let transferred = env
        .call_method(
            connection.as_obj(),
            jni_str!("bulkTransfer"),
            jni_sig!("(Landroid/hardware/usb/UsbEndpoint;[BII)I"),
            &[
                JValue::Object(endpoint.as_obj()),
                JValue::Object(array.as_ref()),
                JValue::Int(length as i32),
                JValue::Int(timeout_ms),
            ],
        )?
        .i()?;
    if transferred < 0 {
        bail!("Android bulkTransfer read failed with {transferred}");
    }
    let mut data = vec![0_i8; transferred as usize];
    array.get_region(env, 0, &mut data)?;
    Ok(data.into_iter().map(|byte| byte as u8).collect())
}

fn control_transfer(
    env: &mut Env<'_>,
    connection: &JObject<'_>,
    request_type: i32,
    request: i32,
    value: i32,
    index: i32,
    payload: &[u8],
) -> anyhow::Result<usize> {
    let array = env.new_byte_array(payload.len())?;
    let bytes = payload
        .iter()
        .map(|byte| *byte as jbyte)
        .collect::<Vec<_>>();
    array.set_region(env, 0, &bytes)?;
    let transferred = env
        .call_method(
            connection,
            jni_str!("controlTransfer"),
            jni_sig!("(IIII[BII)I"),
            &[
                JValue::Int(request_type),
                JValue::Int(request),
                JValue::Int(value),
                JValue::Int(index),
                JValue::Object(array.as_ref()),
                JValue::Int(payload.len() as i32),
                JValue::Int(TIMEOUT_MS),
            ],
        )?
        .i()?;
    if transferred < 0 {
        bail!("Android controlTransfer failed with {transferred}");
    }
    Ok(transferred as usize)
}
