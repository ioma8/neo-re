# NeoManager 3.9.3 Initial Recon

Date: 2026-03-31

## Scope

This note captures the first-pass inspection of the `NEOManager3_9_3USPC` installer payload. The goal is to identify the binaries, transport layers, and driver artifacts that matter for reversing the NEO and AlphaHub communication path.

## Source Package

Installer contents in `/Users/jakubkolcar/customs/neo-re/NEOManager3_9_3USPC`:

- `setup.exe`
- `NEO Manager.msi`
- `NEO64bitDriver.msi`
- `Data1.cab`
- `ISSetup.dll`
- `Install_Inf.bat`
- `ReadMe.htm`
- `EULA.pdf`

The package appears to be an InstallShield-based Windows release from mid-2013.

## Extracted Artifacts

The main application payload extracted from `Data1.cab` contains the files most relevant to reversing:

- `neomanager.exe`
- `ashubcomm.dll`
- `asusbcomm.dll`
- `asusbdrv.inf`
- `asusbdrvxp.sys`
- `asusbdrv64.sys`
- `kwgateway.exe`
- `neo2forgoogledocs.exe`
- `sqlite.dll`
- `xerces_c_2_2_0.dll`

Observations:

- `neomanager.exe` is a 32-bit native Windows application.
- `ashubcomm.dll` is a 32-bit native DLL.
- `asusbcomm.dll` is a 32-bit native DLL.
- `asusbdrvxp.sys` is the 32-bit kernel driver.
- `asusbdrv64.sys` is the 64-bit kernel driver from `NEO64bitDriver.msi`.
- `kwgateway.exe` and `neo2forgoogledocs.exe` are .NET assemblies and are likely secondary integrations, not the core device transport path.

## Device IDs

The USB driver INF identifies two device IDs:

- `USB\\VID_081e&PID_bd01` labeled as `NEO Device`
- `USB\\VID_081e&PID_0100` labeled as `AlphaHub`

This already gives us the first hardware mapping:

- Direct NEO data connection: `VID_081e PID_bd01`
- Fresh direct physical attach can initially enumerate as `VID_081e PID_bd04`, a HID keyboard-mode device. NeoManager switches that mode to `PID_bd01` through HID output reports.
- Hub/cart connection: likely `VID_081e PID_0100`

## Transport Split

`neomanager.exe` imports both helper DLLs:

- `AsUsbComm.dll`
- `AsHubComm.dll`

Relevant imports from `neomanager.exe`:

- `AsUSBCommIsAlphaSmartPresent`
- `AsUSBCommReadData`
- `AsUSBCommWriteData`
- `AsUSBCommSwitchToApplet`
- `AsUSBCommResetConnection`
- `AsHubCommReadData`
- `AsHubCommWriteData`
- `AsHubCommGetHubStatus`
- `AsHubCommResetConnection`

This strongly suggests two transport modes:

1. Direct USB communication with a single NEO
2. AlphaHub-mediated communication for a cart or lab of devices

## Exported Helper APIs

### `asusbcomm.dll`

Exports:

- `AsUSBCommIsAlphaSmartPresent`
- `AsUSBCommReadData`
- `AsUSBCommResetConnection`
- `AsUSBCommSwitchToApplet`
- `AsUSBCommWriteData`
- `AsUSBUpdater_BuildCommand`
- `rlAsUSBUpdaterGetMACAddress`
- `rlAsUSBUpdaterSetMACAddress`

Key imported APIs:

- `CreateFileA`
- `ReadFile`
- `WriteFile`
- `DeviceIoControl`
- `HidD_GetHidGuid`
- `HidD_GetAttributes`
- `HidD_GetPreparsedData`
- `HidP_GetCaps`
- `SetupDiGetClassDevsA`
- `SetupDiEnumDeviceInterfaces`
- `SetupDiGetDeviceInterfaceDetailA`

Strings of interest:

- `\\\\.\\AsUSBDrv%d`
- `AsUSBCommSwitchToApplet`
- `rlAsUSBUpdaterGetMACAddress`
- `rlAsUSBUpdaterSetMACAddress`

Inference:

- The DLL enumerates HID interfaces through `SetupAPI` and `HID.DLL`.
- It also opens a driver-backed device path `\\\\.\\AsUSBDrv%d`.
- The direct USB path likely combines HID-level discovery with I/O through the custom kernel driver.

### `ashubcomm.dll`

Exports:

- `AsHubCommGetHubStatus`
- `AsHubCommReadData`
- `AsHubCommResetConnection`
- `AsHubCommWriteData`
- `AsHubResetConnection`

Key imported APIs:

- `CreateFileA`
- `ReadFile`
- `WriteFile`
- `DeviceIoControl`

Strings of interest:

- `\\\\.\\AsUSBDrv%d`

Inference:

- Hub communication also flows through the same device naming scheme.
- The DLL likely layers hub-port addressing or command multiplexing on top of the same kernel driver.

## Driver Notes

From `asusbdrv.inf`:

- Driver class: `USB`
- Provider: `Renaissance Learning, Inc.`
- Driver version: `06/12/2013,1.0`
- Service names:
  - `AsUsbDrvXP`
  - `AsUsbDrv64`

Driver binaries:

- `/Users/jakubkolcar/customs/neo-re/analysis/cab/asusbdrvxp.sys`
- `/Users/jakubkolcar/customs/neo-re/analysis/driver64_cab/asusbdrv64.sys`

Interesting string in the 32-bit driver:

- `C:\\AS\\Software\\OS3000\\HostSrc\\USB\\PC\\DDK_Drv\\objfre_wxp_x86\\i386\\AsUsbDrvXP.pdb`

Interesting string in the 64-bit driver:

- `c:\\as\\software\\os3000\\hostsrc\\usb\\pc\\ddk_drv64\\objfre_win7_amd64\\amd64\\AsUsbDrv64.pdb`

Inference:

- The vendor internally referred to this codebase as `OS3000`.
- There is a dedicated host-side USB driver project for Windows.
- Driver reversing should reveal the actual IOCTL surface and read/write model used by both helper DLLs.

## Application-Level Clues

Interesting strings in `neomanager.exe`:

- `Clearing ROM via USB`
- `Installing System via USB`
- `Clearing ROM via AlphaHub`
- `Installing System via AlphaHub`
- `UpdaterListenViaHubSingle`
- `UpdaterMakeDeafViaHubSingle`
- `Updater_BroadcastDataToHubAndGetResponses`
- `Updater_SendHubCommandsAndGetResponses`
- `AlphaSmart USB`
- `Error writing to hub port.`
- `Unexpected response from hub port.`
- `Trying to put a Neo OS on an AS3000`
- `Trying to put an AS3000 OS on a NEO`
- `OS 3KNeo Small ROM`

Inference:

- The updater path is present in the main application, not just a separate flashing tool.
- Hub mode probably supports broadcast commands plus per-port responses.
- There is explicit model differentiation between older AlphaSmart hardware and NEO-class devices.

## Initial Reverse-Engineering Priorities

Recommended order:

1. Reverse `asusbcomm.dll` to recover function signatures, packet framing, and how it talks to `AsUSBDrv`.
2. Reverse `ashubcomm.dll` to understand the extra layer for AlphaHub addressing and status polling.
3. Reverse `asusbdrvxp.sys` and `asusbdrv64.sys` to enumerate IOCTLs, device names, and USB endpoint handling.
4. Trace the main `neomanager.exe` call sites for operations like detect, read, write, reset, applet switch, and OS update.
5. Document packet formats and state machines in separate Markdown notes once recovered.

## Immediate Questions For Next Pass

These are the first high-value unknowns:

- What IOCTL codes does `AsUSBDrv` expose?
- Which USB endpoints are used for direct NEO communication?
  - Live confirmation: after the `081e:bd04` -> `081e:bd01` switch, the tested NEO exposes bulk OUT `0x01` and bulk IN `0x82`.
- Is HID only used for enumeration, or also for data transfer?
  - Live confirmation: HID is used to trigger direct USB mode from keyboard mode. It is not the AlphaWord data path after the device re-enumerates as `081e:bd01`.
  - The confirmed switch for the tested NEO is HID output report payloads `e0 e1 e2 e3 e4`.
- What are the parameter and buffer layouts for:
  - `AsUSBCommWriteData`
  - `AsUSBCommReadData`
  - `AsUSBCommSwitchToApplet`
  - `AsHubCommWriteData`
  - `AsHubCommReadData`
  - `AsHubCommGetHubStatus`
- What does hub status look like on the wire?
- How are updater commands framed for single-device and hub-broadcast modes?

## Requested Decompilation Targets

If you want to use Ghidra next, these should be the first decompilation targets:

- `analysis/cab/asusbcomm.dll`
- `analysis/cab/ashubcomm.dll`
- `analysis/cab/asusbdrvxp.sys`

Inside `asusbcomm.dll`, the first functions worth extracting are:

- `AsUSBCommWriteData`
- `AsUSBCommReadData`
- `AsUSBCommSwitchToApplet`
- `AsUSBUpdater_BuildCommand`

Inside `ashubcomm.dll`, the first functions worth extracting are:

- `AsHubCommGetHubStatus`
- `AsHubCommWriteData`
- `AsHubCommReadData`

Inside the driver, the high-value targets are:

- `DriverEntry`
- `IRP_MJ_CREATE`
- `IRP_MJ_CLOSE`
- `IRP_MJ_DEVICE_CONTROL`
- Any helper that builds URBs or maps read/write requests to USB pipes

## Current Workspace

Generated extraction directories:

- `/Users/jakubkolcar/customs/neo-re/analysis/extracted`
- `/Users/jakubkolcar/customs/neo-re/analysis/cab`
- `/Users/jakubkolcar/customs/neo-re/analysis/driver64`
- `/Users/jakubkolcar/customs/neo-re/analysis/driver64_cab`

These are derived artifacts and can be kept as working material for continued analysis.
