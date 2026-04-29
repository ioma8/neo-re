import { invoke } from "@tauri-apps/api/core";
import type {
  AddedAppletFile,
  BackupResult,
  BundledApplet,
  DeviceMode,
  Inventory,
  RecoveryDiagnostics,
} from "./types";

export function detectDevice(): Promise<DeviceMode> {
  return invoke("detect_device");
}

export function switchHidToDirect(): Promise<void> {
  return invoke("switch_hid_to_direct");
}

export function getInventory(): Promise<Inventory> {
  return invoke("get_inventory");
}

export function defaultBackupRoot(): Promise<string> {
  return invoke("default_backup_root");
}

export function runtimePlatform(): Promise<"android" | "desktop"> {
  return invoke("runtime_platform");
}

export function debugBypassEnabled(): Promise<boolean> {
  return invoke("debug_bypass_enabled");
}

export function pickBackupFolder(): Promise<string | null> {
  return invoke("pick_backup_folder");
}

export function backupFile(slot: number, backupRoot: string | null): Promise<BackupResult> {
  return invoke("backup_file", { slot, target: { backupRoot } });
}

export function backupAllFiles(backupRoot: string | null): Promise<BackupResult> {
  return invoke("backup_all_files", { target: { backupRoot } });
}

export function backupEverything(backupRoot: string | null): Promise<BackupResult> {
  return invoke("backup_everything", { target: { backupRoot } });
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

export function restoreOriginalStockApplets(): Promise<Inventory> {
  return invoke("restore_original_stock_applets");
}

export function restartDevice(): Promise<void> {
  return invoke("restart_device");
}

export function readRecoveryDiagnostics(): Promise<RecoveryDiagnostics> {
  return invoke("read_recovery_diagnostics");
}

export function flashSystemImage(reformatRestOfRom: boolean): Promise<void> {
  return invoke("flash_system_image", { reformatRestOfRom });
}
