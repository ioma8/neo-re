import { useEffect, useMemo, useRef, useState } from "react";
import {
  backupAllFiles,
  backupEverything,
  backupFile,
  debugBypassEnabled,
  defaultBackupRoot,
  detectDevice,
  flashApplets,
  flashSystemImage,
  getInventory,
  installAlphaUsb,
  readRecoveryDiagnostics,
  restartDevice,
  restoreOriginalStockApplets,
  switchHidToDirect,
} from "./api/commands";
import { selectAppletFile } from "./api/applets";
import {
  loadBackupRoot,
  saveBackupRoot,
  selectBackupDirectory,
} from "./api/backupSettings";
import { listenForProgress } from "./api/progress";
import type {
  AddedAppletFile,
  AppletChecklistRow,
  DeviceMode,
  Inventory,
  ProgressEvent,
} from "./api/types";
import { AppShell, type TabKey } from "./components/layout/AppShell";
import { ConnectionScreen } from "./components/layout/ConnectionScreen";
import { Dashboard } from "./components/tabs/Dashboard";
import { SmartApplets } from "./components/tabs/SmartApplets";
import { OsOperations } from "./components/tabs/OsOperations";
import { About } from "./components/tabs/About";
import { ProgressDialog } from "./components/ui/ProgressDialog";
import { ConfirmDialog, type ConfirmRequest } from "./components/ui/ConfirmDialog";
import { DiagnosticLogDialog } from "./components/ui/DiagnosticLogDialog";

const emptyInventory: Inventory = {
  files: [],
  installedApplets: [],
  bundledApplets: [],
  appletRows: [],
};

const debugInventory: Inventory = {
  files: [
    { slot: 1, name: "Chapter_1_Draft", attributeBytes: 14336 },
    { slot: 2, name: "Meeting_Notes", attributeBytes: 4096 },
    { slot: 3, name: "Journal_Entry_42", attributeBytes: 28672 },
  ],
  installedApplets: [],
  bundledApplets: [],
  appletRows: [
    {
      key: "a000",
      displayName: "AlphaWord",
      version: "3.2",
      size: 126976,
      installed: true,
      checked: true,
      sourceKind: "bundled",
    },
    {
      key: "alpha-usb",
      displayName: "Alpha USB",
      version: "0.1",
      size: 46080,
      installed: false,
      checked: false,
      sourceKind: "bundled",
    },
  ],
};

export default function App() {
  const [mode, setMode] = useState<DeviceMode>("missing");
  const [connected, setConnected] = useState(false);
  const [debugBypass, setDebugBypass] = useState(false);
  const [showDebugBypass, setShowDebugBypass] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [connectionMessage, setConnectionMessage] = useState("Scanning for Alpha writing device...");
  const [tab, setTab] = useState<TabKey>("dashboard");
  const [inventory, setInventory] = useState<Inventory>(emptyInventory);
  const [checkedKeys, setCheckedKeys] = useState<Set<string>>(new Set());
  const [baselineKeys, setBaselineKeys] = useState<Set<string>>(new Set());
  const [addedApplets, setAddedApplets] = useState<AddedAppletFile[]>([]);
  const [backupRoot, setBackupRoot] = useState<string | null>(() => loadBackupRoot());
  const [defaultBackupRootPath, setDefaultBackupRootPath] = useState<string | null>(null);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirmRequest, setConfirmRequest] = useState<ConfirmRequest | null>(null);
  const [diagnosticLog, setDiagnosticLog] = useState<string | null>(null);
  const scanInFlight = useRef(false);

  useEffect(() => {
    void scan();
    const id = window.setInterval(() => {
      if (!connected && !debugBypass) void scan(false);
    }, 2500);
    return () => window.clearInterval(id);
  }, [connected, debugBypass]);

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    void listenForProgress(setProgress).then((unlisten) => {
      cleanup = unlisten;
    });
    return () => cleanup?.();
  }, []);

  useEffect(() => {
    void defaultBackupRoot()
      .then(setDefaultBackupRootPath)
      .catch(() => setDefaultBackupRootPath(null));
  }, []);

  useEffect(() => {
    void debugBypassEnabled()
      .then(setShowDebugBypass)
      .catch(() => setShowDebugBypass(false));
  }, []);

  useEffect(() => {
    const next = new Set(inventory.appletRows.filter((row) => row.checked).map((row) => row.key));
    setCheckedKeys(next);
    setBaselineKeys(next);
  }, [inventory.appletRows]);

  const connectedLabel = debugBypass ? "Debug UI" : mode === "direct" ? "Direct USB" : "No device";
  const appletRows = useMemo(
    () => [...inventory.appletRows, ...addedApplets.map(addedAppletRow)],
    [inventory.appletRows, addedApplets],
  );
  const dirty = useMemo(
    () => addedApplets.length > 0 || !sameSet(checkedKeys, baselineKeys),
    [addedApplets.length, checkedKeys, baselineKeys],
  );

  async function scan(showSpinner = true) {
    if (scanInFlight.current) return;
    scanInFlight.current = true;
    if (showSpinner) setScanning(true);
    setConnectionMessage("Scanning for Alpha writing device...");
    try {
      let nextMode = await detectDevice();
      setMode(nextMode);
      if (nextMode === "hid") {
        setConnectionMessage("HID keyboard mode detected. Switching to direct USB...");
        await switchHidToDirect();
        setConnectionMessage("Waiting for direct USB mode...");
        nextMode = await detectDevice();
        setMode(nextMode);
      }
      if (nextMode === "direct") {
        setConnectionMessage("Direct USB detected. Loading device inventory...");
        setConnected(true);
        await refreshInventory();
      } else {
        setConnectionMessage(modeLabel(nextMode));
      }
    } catch (err) {
      setError(String(err));
      setConnectionMessage("Connection failed. Retrying automatically...");
    } finally {
      if (showSpinner) setScanning(false);
      scanInFlight.current = false;
    }
  }

  async function refreshInventory() {
    if (debugBypass) {
      setInventory(debugInventory);
      return;
    }
    setInventory(await getInventory());
  }

  async function runOperation(operation: () => Promise<void>) {
    setError(null);
    try {
      await operation();
      await refreshInventory();
    } catch (err) {
      setError(String(err));
    }
  }

  async function runOperationWithoutRefresh(operation: () => Promise<void>) {
    setError(null);
    try {
      await operation();
    } catch (err) {
      setError(String(err));
    }
  }

  if (!connected && !debugBypass) {
    return (
      <>
        <ConnectionScreen
          mode={mode}
          scanning={scanning}
          message={connectionMessage}
          showDebugOpen={showDebugBypass}
          onDebugOpen={() => {
            setDebugBypass(true);
            setInventory(debugInventory);
          }}
        />
        <ProgressDialog progress={progress} error={error} onClose={() => setError(null)} />
      </>
    );
  }

  return (
    <>
      <AppShell activeTab={tab} onTabChange={setTab} connectedLabel={connectedLabel}>
        {tab === "dashboard" && (
          <Dashboard
            files={inventory.files}
            backupRoot={backupRoot}
            defaultBackupRoot={defaultBackupRootPath}
            onBackupAll={() => void runOperationWithoutRefresh(async () => void (await backupAllFiles(backupRoot)))}
            onBackupFile={(slot) =>
              void runOperationWithoutRefresh(async () => void (await backupFile(slot, backupRoot)))
            }
            onChooseBackupRoot={() =>
              void (async () => {
                setError(null);
                try {
                  const selected = await selectBackupDirectory();
                  if (!selected) return;
                  setBackupRoot(selected);
                  saveBackupRoot(selected);
                } catch (err) {
                  setError(String(err));
                }
              })()
            }
            onResetBackupRoot={() => {
              setBackupRoot(null);
              saveBackupRoot(null);
            }}
            onRefresh={() => void refreshInventory()}
          />
        )}
        {tab === "applets" && (
          <SmartApplets
            rows={appletRows}
            checkedKeys={checkedKeys}
            dirty={dirty}
            onInstallAlphaUsb={() =>
              setConfirmRequest({
                title: "Flash Alpha USB SmartApplet?",
                message:
                  "This installs the bundled Alpha USB SmartApplet on the device. Keep the USB cable connected until the operation completes.",
                confirmLabel: "Flash Alpha USB",
                destructive: true,
                onConfirm: () =>
                  void runOperation(async () => {
                    setInventory(await installAlphaUsb());
                  }),
              })
            }
            onAddFromFile={() =>
              void (async () => {
                setError(null);
                try {
                  const added = await selectAppletFile();
                  if (!added) return;
                  setAddedApplets((current) =>
                    current.some((item) => item.key === added.key) ? current : [...current, added],
                  );
                  setCheckedKeys((current) => new Set(current).add(added.key));
                } catch (err) {
                  setError(String(err));
                }
              })()
            }
            onToggle={(key) => {
              setCheckedKeys((current) => {
                const next = new Set(current);
                if (next.has(key)) next.delete(key);
                else next.add(key);
                return next;
              });
            }}
            onFlash={() =>
              setConfirmRequest({
                title: "Reflash SmartApplets?",
                message:
                  "AlphaGUI will apply the selected SmartApplet set. If installed applets were unchecked, the applet area may be cleared before reinstalling selected applets.",
                confirmLabel: "Flash to Device",
                destructive: true,
                onConfirm: () =>
                  void runOperation(async () => {
                    setInventory(await flashApplets([...checkedKeys], addedApplets));
                    setAddedApplets([]);
                  }),
              })
            }
          />
        )}
        {tab === "os" && (
          <OsOperations
            onBackupEverything={() =>
              void runOperationWithoutRefresh(async () => void (await backupEverything(backupRoot)))
            }
            onFlashSystem={() =>
              setConfirmRequest({
                title: "Reflash bundled NEO OS image?",
                message:
                  "This writes the bundled OS image to the device and can brick the Alpha writing device if interrupted. Back up everything first.",
                confirmLabel: "Reflash OS",
                destructive: true,
                onConfirm: () =>
                  void runOperationWithoutRefresh(async () => {
                    await flashSystemImage(false);
                    setProgress({
                      operationId: "flash-system",
                      title: "Flash system image",
                      phase: "Device restarted",
                      item: "Reconnect the Alpha writing device after it finishes rebooting.",
                      completed: null,
                      total: null,
                      indeterminate: true,
                      log: null,
                    });
                    setConnected(false);
                    setMode("missing");
                  }),
              })
            }
            onFlashSystemFromSmallRom={() =>
              setConfirmRequest({
                title: "Reflash bundled OS from Small ROM?",
                message:
                  "Use this only after entering Small ROM/updater mode and connecting USB. This flashes the bundled stock OS image and can brick the device if interrupted.",
                confirmLabel: "Reflash OS",
                destructive: true,
                onConfirm: () =>
                  void runOperationWithoutRefresh(async () => {
                    await flashSystemImage(false);
                    setProgress({
                      operationId: "flash-system",
                      title: "Flash system image",
                      phase: "Device restarted",
                      item: "Reconnect the Alpha writing device after it finishes rebooting.",
                      completed: null,
                      total: null,
                      indeterminate: true,
                      log: null,
                    });
                    setConnected(false);
                    setMode("missing");
                  }),
              })
            }
            onRestartDevice={() =>
              setConfirmRequest({
                title: "Restart device?",
                message:
                  "This restarts the connected device. USB will disconnect temporarily while it reboots.",
                confirmLabel: "Restart",
                destructive: false,
                onConfirm: () =>
                  void runOperationWithoutRefresh(async () => {
                    await restartDevice();
                    setConnected(false);
                    setMode("missing");
                  }),
              })
            }
            onReadDiagnostics={() =>
              void runOperationWithoutRefresh(async () => {
                const result = await readRecoveryDiagnostics();
                setDiagnosticLog(result.log);
              })
            }
            onRestoreStockApplets={() =>
              setConfirmRequest({
                title: "Restore original stock applets?",
                message:
                  "This clears the SmartApplet area and reinstalls only bundled original stock applets. Alpha USB and user applets will not be installed by this recovery action. Back up everything first.",
                confirmLabel: "Restore Stock Applets",
                destructive: true,
                onConfirm: () =>
                  void runOperation(async () => {
                    setInventory(await restoreOriginalStockApplets());
                    setAddedApplets([]);
                  }),
              })
            }
          />
        )}
        {tab === "about" && <About />}
      </AppShell>
      <ProgressDialog
        progress={progress}
        error={error}
        onClose={() => {
          setProgress(null);
          setError(null);
        }}
      />
      <DiagnosticLogDialog
        open={diagnosticLog !== null}
        log={diagnosticLog ?? ""}
        onClose={() => setDiagnosticLog(null)}
      />
      <ConfirmDialog request={confirmRequest} onCancel={() => setConfirmRequest(null)} />
    </>
  );
}

function addedAppletRow(applet: AddedAppletFile): AppletChecklistRow {
  return {
    key: applet.key,
    displayName: applet.displayName,
    version: null,
    size: applet.size,
    installed: false,
    checked: true,
    sourceKind: "addedFromFile",
  };
}

function modeLabel(mode: DeviceMode) {
  switch (mode) {
    case "direct":
      return "Direct USB";
    case "hid":
      return "HID keyboard mode";
    case "hidUnavailable":
      return "HID unavailable";
    case "missing":
      return "No device";
  }
}

function sameSet(left: Set<string>, right: Set<string>) {
  if (left.size !== right.size) return false;
  for (const item of left) {
    if (!right.has(item)) return false;
  }
  return true;
}
