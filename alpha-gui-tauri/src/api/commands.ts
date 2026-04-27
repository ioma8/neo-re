import { invoke } from "@tauri-apps/api/core";
import type { AddedAppletFile, BackupResult, BundledApplet, DeviceMode, Inventory } from "./types";

export function detectDevice(): Promise<DeviceMode> {
  return invoke("detect_device");
}

export function switchHidToDirect(): Promise<void> {
  return invoke("switch_hid_to_direct");
}

export function getInventory(): Promise<Inventory> {
  return invoke("get_inventory");
}

export function backupFile(slot: number): Promise<BackupResult> {
  return invoke("backup_file", { slot });
}

export function backupAllFiles(): Promise<BackupResult> {
  return invoke("backup_all_files");
}

export function backupEverything(): Promise<BackupResult> {
  return invoke("backup_everything");
}

export function listBundledApplets(): Promise<BundledApplet[]> {
  return invoke("list_bundled_applets");
}

export function installAlphaUsb(): Promise<Inventory> {
  return invoke("install_alpha_usb");
}

export function flashApplets(
  checkedKeys: string[],
  addedFiles: AddedAppletFile[],
): Promise<Inventory> {
  return invoke("flash_applets", { selection: { checkedKeys, addedFiles } });
}

export function flashSystemImage(reformatRestOfRom: boolean): Promise<void> {
  return invoke("flash_system_image", { reformatRestOfRom });
}
