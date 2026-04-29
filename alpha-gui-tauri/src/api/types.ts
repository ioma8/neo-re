export type DeviceMode = "missing" | "hid" | "hidUnavailable" | "direct";

export interface DeviceFile {
  slot: number;
  name: string;
  attributeBytes: number;
}

export interface SmartApplet {
  appletId: number;
  version: string;
  name: string;
  fileSize: number;
  appletClass: number;
}

export interface BundledApplet {
  id: string;
  appletId: number | null;
  name: string;
  version: string | null;
  size: number | null;
  kind: "stock" | "alphaUsb" | string;
}

export interface AppletChecklistRow {
  key: string;
  displayName: string;
  version: string | null;
  size: number | null;
  installed: boolean;
  checked: boolean;
  sourceKind: "installedOnly" | "bundled" | "addedFromFile" | string;
}

export interface AddedAppletFile {
  key: string;
  path: string;
  displayName: string;
  size: number | null;
}

export interface Inventory {
  files: DeviceFile[];
  installedApplets: SmartApplet[];
  bundledApplets: BundledApplet[];
  appletRows: AppletChecklistRow[];
}

export interface BackupResult {
  directory: string;
  savedFiles: number;
  savedApplets: number;
  bytes: number;
}

export interface ProgressEvent {
  operationId: string;
  title: string;
  phase: string;
  item: string | null;
  completed: number | null;
  total: number | null;
  indeterminate: boolean;
  log: string | null;
}

export interface RecoveryDiagnostics {
  log: string;
}
