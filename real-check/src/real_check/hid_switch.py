from dataclasses import dataclass
import ctypes
import ctypes.util
import sys
import time
from typing import Protocol

import usb.core


ALPHASMART_VENDOR_ID = 0x081E
ALPHASMART_DIRECT_PRODUCT_ID = 0xBD01
ALPHASMART_KEYBOARD_PRODUCT_ID = 0xBD04
MANAGER_SWITCH_SEQUENCE = (0xE0, 0xE1, 0xE2, 0xE3, 0xE4)


class HidBackend(Protocol):
    def open_alphasmart_keyboard(self): ...

    def write_output_report(self, handle, report: bytes) -> int: ...

    def close(self, handle) -> None: ...


@dataclass(frozen=True)
class ManagerSwitchResult:
    reports_sent: int


class AlreadyDirectMode(RuntimeError):
    pass


class HidApiBackend:
    def __init__(self) -> None:
        lib_path = ctypes.util.find_library("hidapi")
        if lib_path is None:
            raise RuntimeError("hidapi library not found")
        self._hid = ctypes.CDLL(lib_path)
        self._hid.hid_init.restype = ctypes.c_int
        self._hid.hid_exit.restype = ctypes.c_int
        self._hid.hid_open.argtypes = [ctypes.c_ushort, ctypes.c_ushort, ctypes.c_wchar_p]
        self._hid.hid_open.restype = ctypes.c_void_p
        self._hid.hid_close.argtypes = [ctypes.c_void_p]
        self._hid.hid_write.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t]
        self._hid.hid_write.restype = ctypes.c_int
        self._hid.hid_error.argtypes = [ctypes.c_void_p]
        self._hid.hid_error.restype = ctypes.c_wchar_p
        if self._hid.hid_init() != 0:
            raise RuntimeError("hidapi initialization failed")

    def open_alphasmart_keyboard(self):
        handle = self._hid.hid_open(ALPHASMART_VENDOR_ID, ALPHASMART_KEYBOARD_PRODUCT_ID, None)
        if not handle:
            raise RuntimeError(
                "could not open AlphaSmart HID keyboard; on macOS grant this terminal Input Monitoring permission"
            )
        return handle

    def write_output_report(self, handle, report: bytes) -> int:
        buffer = (ctypes.c_ubyte * len(report))(*report)
        written = self._hid.hid_write(handle, buffer, len(report))
        if written < 0:
            error = self._hid.hid_error(handle)
            raise RuntimeError(f"hid write failed: {error}")
        return int(written)

    def close(self, handle) -> None:
        self._hid.hid_close(handle)
        self._hid.hid_exit()


class MacIOHidBackend:
    REPORT_TYPE_OUTPUT = 1

    def __init__(self) -> None:
        self._cf = ctypes.CDLL("/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation")
        self._iokit = ctypes.CDLL("/System/Library/Frameworks/IOKit.framework/IOKit")
        self._configure_functions()
        self._manager = None

    def _configure_functions(self) -> None:
        self._cf.CFStringCreateWithCString.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_uint32]
        self._cf.CFStringCreateWithCString.restype = ctypes.c_void_p
        self._cf.CFNumberCreate.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p]
        self._cf.CFNumberCreate.restype = ctypes.c_void_p
        self._cf.CFDictionaryCreate.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_void_p),
            ctypes.POINTER(ctypes.c_void_p),
            ctypes.c_long,
            ctypes.c_void_p,
            ctypes.c_void_p,
        ]
        self._cf.CFDictionaryCreate.restype = ctypes.c_void_p
        self._cf.CFRelease.argtypes = [ctypes.c_void_p]
        self._cf.CFSetGetCount.argtypes = [ctypes.c_void_p]
        self._cf.CFSetGetCount.restype = ctypes.c_long
        self._cf.CFSetGetValues.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]

        self._iokit.IOHIDManagerCreate.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        self._iokit.IOHIDManagerCreate.restype = ctypes.c_void_p
        self._iokit.IOHIDManagerSetDeviceMatching.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
        self._iokit.IOHIDManagerOpen.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        self._iokit.IOHIDManagerOpen.restype = ctypes.c_int
        self._iokit.IOHIDManagerCopyDevices.argtypes = [ctypes.c_void_p]
        self._iokit.IOHIDManagerCopyDevices.restype = ctypes.c_void_p
        self._iokit.IOHIDDeviceOpen.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        self._iokit.IOHIDDeviceOpen.restype = ctypes.c_int
        self._iokit.IOHIDDeviceClose.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        self._iokit.IOHIDDeviceClose.restype = ctypes.c_int
        self._iokit.IOHIDDeviceSetReport.argtypes = [
            ctypes.c_void_p,
            ctypes.c_int,
            ctypes.c_long,
            ctypes.POINTER(ctypes.c_ubyte),
            ctypes.c_long,
        ]
        self._iokit.IOHIDDeviceSetReport.restype = ctypes.c_int

    @staticmethod
    def output_report_payload(report: bytes) -> bytes:
        if not report:
            raise ValueError("empty HID report")
        if report[0] != 0:
            raise ValueError(f"expected report ID 0, got {report[0]}")
        return report[1:]

    def open_alphasmart_keyboard(self):
        self._manager = self._iokit.IOHIDManagerCreate(None, 0)
        if not self._manager:
            raise RuntimeError("IOHIDManagerCreate failed")
        match = self._matching_dictionary()
        try:
            self._iokit.IOHIDManagerSetDeviceMatching(self._manager, match)
            result = self._iokit.IOHIDManagerOpen(self._manager, 0)
            if result != 0:
                raise RuntimeError(f"IOHIDManagerOpen failed: 0x{result & 0xffffffff:08x}")
            device = self._copy_first_matching_device()
            result = self._iokit.IOHIDDeviceOpen(device, 0)
            if result != 0:
                raise RuntimeError(f"IOHIDDeviceOpen failed: 0x{result & 0xffffffff:08x}")
            return device
        finally:
            self._cf.CFRelease(match)

    def _matching_dictionary(self):
        keys = (ctypes.c_void_p * 2)()
        values = (ctypes.c_void_p * 2)()
        vendor_id = ctypes.c_int32(ALPHASMART_VENDOR_ID)
        product_id = ctypes.c_int32(ALPHASMART_KEYBOARD_PRODUCT_ID)
        keys[0] = self._cf_string("VendorID")
        keys[1] = self._cf_string("ProductID")
        values[0] = self._cf_number(vendor_id)
        values[1] = self._cf_number(product_id)
        try:
            return self._cf.CFDictionaryCreate(None, keys, values, 2, None, None)
        finally:
            for pointer in (*keys, *values):
                self._cf.CFRelease(pointer)

    def _cf_string(self, value: str):
        return self._cf.CFStringCreateWithCString(None, value.encode("utf-8"), 0x08000100)

    def _cf_number(self, value: ctypes.c_int32):
        return self._cf.CFNumberCreate(None, 3, ctypes.byref(value))

    def _copy_first_matching_device(self):
        devices = self._iokit.IOHIDManagerCopyDevices(self._manager)
        if not devices:
            raise RuntimeError("AlphaSmart HID keyboard not found through IOHIDManager")
        try:
            count = self._cf.CFSetGetCount(devices)
            if count < 1:
                raise RuntimeError("AlphaSmart HID keyboard not found through IOHIDManager")
            device_refs = (ctypes.c_void_p * count)()
            self._cf.CFSetGetValues(devices, device_refs)
            return device_refs[0]
        finally:
            self._cf.CFRelease(devices)

    def write_output_report(self, handle, report: bytes) -> int:
        payload = self.output_report_payload(report)
        buffer = (ctypes.c_ubyte * len(payload))(*payload)
        result = self._iokit.IOHIDDeviceSetReport(handle, self.REPORT_TYPE_OUTPUT, 0, buffer, len(payload))
        if result != 0:
            raise RuntimeError(f"IOHIDDeviceSetReport failed: 0x{result & 0xffffffff:08x}")
        return len(report)

    def close(self, handle) -> None:
        self._iokit.IOHIDDeviceClose(handle, 0)
        if self._manager:
            self._cf.CFRelease(self._manager)
            self._manager = None


class LibUsbControlBackend:
    REQUEST_TYPE_SET_REPORT = 0x21
    SET_REPORT = 0x09
    OUTPUT_REPORT_VALUE = 0x0200
    INTERFACE_NUMBER = 0

    def __init__(self) -> None:
        lib_path = ctypes.util.find_library("usb-1.0")
        if lib_path is None:
            raise RuntimeError("libusb-1.0 library not found")
        self._libusb = ctypes.CDLL(lib_path)
        self._configure_functions()
        self._context = ctypes.c_void_p()
        result = self._libusb.libusb_init(ctypes.byref(self._context))
        if result != 0:
            raise RuntimeError(f"libusb_init failed: {result}")

    def _configure_functions(self) -> None:
        self._libusb.libusb_init.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
        self._libusb.libusb_init.restype = ctypes.c_int
        self._libusb.libusb_exit.argtypes = [ctypes.c_void_p]
        self._libusb.libusb_open_device_with_vid_pid.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint16,
            ctypes.c_uint16,
        ]
        self._libusb.libusb_open_device_with_vid_pid.restype = ctypes.c_void_p
        self._libusb.libusb_close.argtypes = [ctypes.c_void_p]
        self._libusb.libusb_control_transfer.argtypes = [
            ctypes.c_void_p,
            ctypes.c_uint8,
            ctypes.c_uint8,
            ctypes.c_uint16,
            ctypes.c_uint16,
            ctypes.POINTER(ctypes.c_ubyte),
            ctypes.c_uint16,
            ctypes.c_uint,
        ]
        self._libusb.libusb_control_transfer.restype = ctypes.c_int

    def open_alphasmart_keyboard(self):
        handle = self._libusb.libusb_open_device_with_vid_pid(
            self._context,
            ALPHASMART_VENDOR_ID,
            ALPHASMART_KEYBOARD_PRODUCT_ID,
        )
        if not handle:
            direct_handle = self._libusb.libusb_open_device_with_vid_pid(
                self._context,
                ALPHASMART_VENDOR_ID,
                ALPHASMART_DIRECT_PRODUCT_ID,
            )
            if direct_handle:
                self._libusb.libusb_close(direct_handle)
                raise AlreadyDirectMode("AlphaSmart is already in direct USB mode")
            raise RuntimeError("AlphaSmart HID keyboard not found")
        return handle

    def write_output_report(self, handle, report: bytes) -> int:
        if len(report) != 2 or report[0] != 0:
            raise ValueError("expected two-byte HID report with report ID 0")
        payload = (ctypes.c_ubyte * 1)(report[1])
        result = self._libusb.libusb_control_transfer(
            handle,
            self.REQUEST_TYPE_SET_REPORT,
            self.SET_REPORT,
            self.OUTPUT_REPORT_VALUE,
            self.INTERFACE_NUMBER,
            payload,
            1,
            1000,
        )
        if result < 0:
            raise RuntimeError(f"USB HID SET_REPORT failed: {result}")
        return len(report)

    def close(self, handle) -> None:
        if handle:
            self._libusb.libusb_close(handle)
        if self._context:
            self._libusb.libusb_exit(self._context)
            self._context = None


def _default_backend() -> HidBackend:
    if sys.platform == "darwin":
        return LibUsbControlBackend()
    return HidApiBackend()


def send_manager_switch_sequence(
    *,
    backend: HidBackend | None = None,
    delay_seconds: float = 2.0,
    wait_for_direct_seconds: float = 5.0,
) -> ManagerSwitchResult:
    return send_hid_output_report_sequence(
        MANAGER_SWITCH_SEQUENCE,
        backend=backend,
        delay_seconds=delay_seconds,
        wait_for_direct_seconds=wait_for_direct_seconds,
    )


def send_hid_output_report_sequence(
    sequence: tuple[int, ...],
    *,
    backend: HidBackend | None = None,
    delay_seconds: float = 2.0,
    wait_for_direct_seconds: float = 5.0,
) -> ManagerSwitchResult:
    if not sequence:
        raise ValueError("empty HID output-report sequence")
    for value in sequence:
        if value < 0 or value > 0xFF:
            raise ValueError(f"invalid HID report byte: 0x{value:x}")

    hid_backend = backend if backend is not None else _default_backend()
    try:
        handle = hid_backend.open_alphasmart_keyboard()
    except AlreadyDirectMode:
        return ManagerSwitchResult(reports_sent=0)
    reports_sent = 0
    try:
        for value in sequence:
            hid_backend.write_output_report(handle, bytes([0x00, value]))
            reports_sent += 1
        if delay_seconds > 0:
            time.sleep(delay_seconds)
    finally:
        hid_backend.close(handle)
    _wait_for_direct_device(wait_for_direct_seconds)
    return ManagerSwitchResult(reports_sent=reports_sent)


def _wait_for_direct_device(timeout_seconds: float) -> None:
    if timeout_seconds <= 0:
        return
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() <= deadline:
        if usb.core.find(idVendor=ALPHASMART_VENDOR_ID, idProduct=ALPHASMART_DIRECT_PRODUCT_ID) is not None:
            return
        time.sleep(0.1)
    raise RuntimeError("AlphaSmart did not re-enumerate as direct USB mode")
