import { open } from "@tauri-apps/plugin-dialog";
import { pickBackupFolder, runtimePlatform } from "./commands";

const BACKUP_ROOT_KEY = "alpha-gui.backupRoot";

export function loadBackupRoot(): string | null {
  return window.localStorage.getItem(BACKUP_ROOT_KEY);
}

export function saveBackupRoot(path: string | null) {
  if (path && path.trim()) {
    window.localStorage.setItem(BACKUP_ROOT_KEY, path);
  } else {
    window.localStorage.removeItem(BACKUP_ROOT_KEY);
  }
}

export async function selectBackupDirectory(): Promise<string | null> {
  if ((await runtimePlatform()) === "android") {
    return await pickBackupFolder();
  }

  const selected = await open({
    directory: true,
    multiple: false,
    title: "Choose backup folder",
  });

  return typeof selected === "string" ? selected : null;
}
