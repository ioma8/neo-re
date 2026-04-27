import { open } from "@tauri-apps/plugin-dialog";
import type { AddedAppletFile } from "./types";

const APPLET_EXTENSIONS = ["os3kapp", "app", "bin"];

export async function selectAppletFile(): Promise<AddedAppletFile | null> {
  const selected = await open({
    multiple: false,
    filters: [{ name: "SmartApplet", extensions: APPLET_EXTENSIONS }],
  });

  if (typeof selected !== "string") return null;

  return appletFileFromPath(selected);
}

export function appletFileFromPath(path: string): AddedAppletFile {
  return {
    key: `file:${path}`,
    path,
    displayName: basename(path),
    size: null,
  };
}

function basename(path: string) {
  const normalized = path.replaceAll("\\", "/");
  return normalized.slice(normalized.lastIndexOf("/") + 1) || "Custom SmartApplet";
}
