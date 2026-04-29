# AlphaGUI OS Recovery Operations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add proven OS and recovery operations to AlphaGUI: restart device, recovery diagnostics, original stock applet restore, and Small ROM OS-flash guidance.

**Architecture:** Keep all device/protocol work in Rust. Add small, testable `alpha-core` helpers for stock restore ordering and diagnostics, expose them through Tauri commands, then wire React UI cards and dialogs into the existing OS Operations tab. Do not call `real-check` or any Python helper from the app.

**Tech Stack:** Rust 2024, `alpha-core`, Tauri 2 commands, React + TypeScript + Tailwind, existing progress event model, existing confirmation/progress dialogs.

---

## Source Documents

- Spec: `docs/superpowers/specs/2026-04-29-alpha-gui-os-recovery-operations-design.md`
- Recovery runbook: `docs/2026-04-18-neo-recovery-runbook.md`
- Real-check CLI reference: `real-check/src/real_check/__init__.py`
- Real-check client behavior: `real-check/src/real_check/client.py`
- Rust core client: `alpha-core/src/neo_client.rs`
- Current Tauri commands: `alpha-gui-tauri/src-tauri/src/commands.rs`
- Current OS tab UI: `alpha-gui-tauri/src/components/tabs/OsOperations.tsx`

## File Structure

- Modify `alpha-core/src/bundled_assets.rs`
  - Add original stock applet restore ordering helpers.
  - Keep `Alpha USB` excluded from stock restore.
- Modify `alpha-core/src/neo_client.rs`
  - Add read-only diagnostic collection methods equivalent to `debug-applets` and `debug-attributes`.
  - Add tests for diagnostics and restore verification behavior using fake transports.
- Modify `alpha-core/src/protocol.rs`
  - Add small helpers only if diagnostics need reusable formatting/parsing access.
- Modify `alpha-gui-tauri/src-tauri/src/types.rs`
  - Add DTOs for recovery diagnostics result if needed.
- Modify `alpha-gui-tauri/src-tauri/src/commands.rs`
  - Add Tauri commands:
    - `restart_device`
    - `read_recovery_diagnostics`
    - `restore_original_stock_applets`
  - Keep all implementation in Rust.
- Modify `alpha-gui-tauri/src-tauri/src/lib.rs`
  - Register new commands.
- Modify `alpha-gui-tauri/src/api/types.ts`
  - Add frontend diagnostic result type if the command returns structured data.
- Modify `alpha-gui-tauri/src/api/commands.ts`
  - Add wrappers for new commands.
- Modify `alpha-gui-tauri/src/App.tsx`
  - Add handlers, confirmations, progress handling, diagnostic log state, reconnect behavior after restart.
- Modify `alpha-gui-tauri/src/components/tabs/OsOperations.tsx`
  - Replace disabled/fake operation content with normal actions and advanced recovery section.
- Create `alpha-gui-tauri/src/components/ui/DiagnosticLogDialog.tsx`
  - Copyable monospace recovery diagnostic output.

## Task 1: Stock Restore Plan In Core

**Files:**
- Modify: `alpha-core/src/bundled_assets.rs`

- [ ] **Step 1: Write failing tests for original stock restore order**

Add tests inside `#[cfg(test)] mod tests` in `alpha-core/src/bundled_assets.rs`:

```rust
#[test]
fn original_stock_restore_order_excludes_alpha_usb_and_system() {
    let catalog = BundledCatalog::dev_defaults();
    let ordered = catalog.original_stock_restore_applets();
    let ids = ordered
        .iter()
        .filter_map(|item| item.applet_id)
        .collect::<Vec<_>>();

    assert!(!ids.contains(&0xa130), "Alpha USB must not be restored here");
    assert!(!ids.contains(&0x0000), "System applet must not be restored here");
    assert_eq!(
        ids,
        vec![
            0xa000, 0xaf00, 0xaf75, 0xaf02, 0xaf73, 0xaf03, 0xa004,
            0xa007, 0xa006, 0xa001, 0xa002, 0xa027, 0xa005,
        ]
    );
}

#[test]
fn original_stock_restore_order_is_resolvable_without_picker() {
    let catalog = BundledCatalog::dev_defaults();

    assert!(catalog
        .original_stock_restore_applets()
        .iter()
        .all(|item| item.kind == BundledAppletKind::Stock
            && item.source.is_resolvable_without_picker()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path alpha-core/Cargo.toml original_stock_restore_order
```

Expected: FAIL because `original_stock_restore_applets` does not exist.

- [ ] **Step 3: Implement stock restore helper**

Add to `impl BundledCatalog`:

```rust
pub fn original_stock_restore_applets(&self) -> Vec<&BundledApplet> {
    const RESTORE_ORDER: &[u16] = &[
        0xa000, 0xaf00, 0xaf75, 0xaf02, 0xaf73, 0xaf03, 0xa004,
        0xa007, 0xa006, 0xa001, 0xa002, 0xa027, 0xa005,
    ];

    RESTORE_ORDER
        .iter()
        .filter_map(|id| {
            self.applets
                .iter()
                .find(|applet| applet.kind == BundledAppletKind::Stock && applet.applet_id == Some(*id))
        })
        .collect()
}
```

Do not include `0x0000` or `0xa130`.

- [ ] **Step 4: Run tests**

Run:

```bash
cargo test --manifest-path alpha-core/Cargo.toml original_stock_restore_order
cargo test --manifest-path alpha-core/Cargo.toml bundled_assets
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-core/src/bundled_assets.rs
git commit -m "feat: add original stock applet restore plan"
```

## Task 2: Recovery Diagnostics In Core

**Files:**
- Modify: `alpha-core/src/neo_client.rs`

- [ ] **Step 1: Write failing diagnostics tests**

Add tests to `alpha-core/src/neo_client.rs` test module. Use the existing fake transport helpers in that file.

Test applet diagnostics:

```rust
#[test]
fn recovery_diagnostics_include_raw_applet_records() {
    let record = smartapplet_record(0xa000, "AlphaWord Plus", 0x200);
    let payload_sum = checksum(&record);
    let transport = FakeTransport::new(vec![
        b"Switched".to_vec(),
        response(0x44, record.len() as u32, payload_sum),
        record,
        response(0x90, 0, 0),
        response(0x90, 0, 0),
    ]);
    let mut client = SharedNeoClient::new(transport).unwrap();

    let report = client.read_recovery_diagnostics().unwrap();

    assert!(report.contains("SmartApplet records"));
    assert!(report.contains("page_offset=0 status=0x44"));
    assert!(report.contains("applet_id=0xa000"));
    assert!(report.contains("AlphaWord Plus"));
    assert!(report.contains("AlphaWord file attributes"));
}
```

Test file attributes diagnostics:

```rust
#[test]
fn recovery_diagnostics_include_file_attribute_statuses() {
    let attributes = alpha_word_attributes_record("File 1", 512, 512);
    let transport = FakeTransport::new(vec![
        b"Switched".to_vec(),
        response(0x90, 0, 0),
        response(0x5a, attributes.len() as u32, checksum(&attributes)),
        attributes,
        response(0x90, 0, 0),
        response(0x90, 0, 0),
        response(0x90, 0, 0),
        response(0x90, 0, 0),
        response(0x90, 0, 0),
        response(0x90, 0, 0),
        response(0x90, 0, 0),
    ]);
    let mut client = SharedNeoClient::new(transport).unwrap();

    let report = client.read_recovery_diagnostics().unwrap();

    assert!(report.contains("slot 1 status=0x5a"));
    assert!(report.contains("name=File 1"));
    assert!(report.contains("file_length=512"));
}
```

If helper names differ in the existing test module, adapt to existing names rather than duplicating large fake-transport code.

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --manifest-path alpha-core/Cargo.toml recovery_diagnostics
```

Expected: FAIL because `read_recovery_diagnostics` does not exist.

- [ ] **Step 3: Implement `read_recovery_diagnostics`**

Add method to `SharedNeoClient<T>`:

```rust
pub fn read_recovery_diagnostics(&mut self) -> anyhow::Result<String> {
    let mut lines = Vec::new();
    self.append_smartapplet_diagnostics(&mut lines)?;
    self.append_alpha_word_attribute_diagnostics(&mut lines)?;
    Ok(lines.join("\n"))
}
```

Implementation requirements:

- Do not write any destructive command.
- SmartApplet diagnostics should page through `list_applets_command(page_offset, 7)`.
- Include status, argument, trailing, payload checksum, parsed row metadata, and raw record hex.
- Stop on status `0x90`, zero argument, parse error, or short final page.
- File diagnostics should request slots `1..=8` using the same opcode path as `list_files`.
- Include status for every slot.
- If status is `0x5a`, read payload, validate checksum, parse `FileEntry`, and include name/length.
- If parsing fails, include the error text in the report and continue where safe.

Return `anyhow::Result<String>`; tests should call `.unwrap()`.

- [ ] **Step 4: Run tests**

Run:

```bash
cargo test --manifest-path alpha-core/Cargo.toml recovery_diagnostics
cargo test --manifest-path alpha-core/Cargo.toml
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-core/src/neo_client.rs
git commit -m "feat: add recovery diagnostics reader"
```

## Task 3: Restore Original Stock Applets Backend

**Files:**
- Modify: `alpha-gui-tauri/src-tauri/src/commands.rs`
- Modify: `alpha-gui-tauri/src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing backend unit tests**

Add tests in `alpha-gui-tauri/src-tauri/src/commands.rs` test module or create one if needed:

```rust
#[test]
fn restore_stock_ids_exclude_alpha_usb_and_system() {
    let ids = original_stock_restore_plan_ids();
    assert_eq!(
        ids,
        vec![
            0xa000, 0xaf00, 0xaf75, 0xaf02, 0xaf73, 0xaf03, 0xa004,
            0xa007, 0xa006, 0xa001, 0xa002, 0xa027, 0xa005,
        ]
    );
    assert!(!ids.contains(&0xa130));
    assert!(!ids.contains(&0x0000));
}
```

Add a pure helper test for verification logic:

```rust
#[test]
fn stock_restore_verification_requires_new_id_in_inventory() {
    let installed = vec![SmartAppletRecord {
        applet_id: 0xa000,
        version: "3.4".to_owned(),
        name: "AlphaWord Plus".to_owned(),
        file_size: 1,
        applet_class: 1,
    }];

    assert!(verify_installed_applet_id(&installed, 0xa000).is_ok());
    assert!(verify_installed_applet_id(&installed, 0xa001).is_err());
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml restore_stock
```

Expected: FAIL because helpers do not exist.

- [ ] **Step 3: Implement helper functions**

Add private helpers in `commands.rs`:

```rust
fn original_stock_restore_plan_ids() -> Vec<u16> {
    BundledCatalog::dev_defaults()
        .original_stock_restore_applets()
        .into_iter()
        .filter_map(|applet| applet.applet_id)
        .collect()
}

fn verify_installed_applet_id(installed: &[SmartAppletRecord], applet_id: u16) -> anyhow::Result<()> {
    if installed.iter().any(|record| record.applet_id == applet_id) {
        Ok(())
    } else {
        anyhow::bail!("applet 0x{applet_id:04x} did not appear after install")
    }
}
```

- [ ] **Step 4: Add Tauri command implementation**

Add:

```rust
#[tauri::command]
pub async fn restore_original_stock_applets(app: AppHandle) -> Result<InventoryDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let catalog = BundledCatalog::dev_defaults();
        let applets = catalog.original_stock_restore_applets();
        if applets.is_empty() {
            anyhow::bail!("no bundled stock applets available for restore");
        }
        let mut client = NeoClient::open_and_init()?;
        emit_progress(&app, "restore-stock-applets", "Restore stock applets", "Clearing SmartApplet area", None, Some(0), Some(applets.len()));
        client.clear_smart_applet_area()?;
        for (index, applet) in applets.iter().enumerate() {
            let bytes = resolve_source(&applet.source)?;
            let applet_id = applet.applet_id.ok_or_else(|| anyhow::anyhow!("stock applet {} has no id", applet.name))?;
            emit_progress(&app, "restore-stock-applets", "Restore stock applets", "Installing", Some(applet.name.clone()), Some(index + 1), Some(applets.len()));
            client.install_smart_applet_with_progress(&bytes, |event| {
                emit_client_progress(&app, "restore-stock-applets", "Restore stock applets", event);
            })?;
            let installed = client.list_smart_applets()?;
            verify_installed_applet_id(&installed, applet_id)?;
        }
        inventory_from_client(&mut client)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}
```

Adjust borrow/lifetime details as needed. Do not install `Alpha USB`.

- [ ] **Step 5: Register command**

Add `commands::restore_original_stock_applets` to `tauri::generate_handler!` in `alpha-gui-tauri/src-tauri/src/lib.rs`.

- [ ] **Step 6: Run validation**

Run:

```bash
cargo test --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml restore_stock
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add alpha-gui-tauri/src-tauri/src/commands.rs alpha-gui-tauri/src-tauri/src/lib.rs
git commit -m "feat: add stock applet recovery command"
```

## Task 4: Restart And Diagnostics Tauri Commands

**Files:**
- Modify: `alpha-gui-tauri/src-tauri/src/types.rs`
- Modify: `alpha-gui-tauri/src-tauri/src/commands.rs`
- Modify: `alpha-gui-tauri/src-tauri/src/lib.rs`

- [ ] **Step 1: Add DTO type**

Add to `types.rs`:

```rust
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryDiagnosticsDto {
    pub log: String,
}
```

- [ ] **Step 2: Add commands**

Add to `commands.rs`:

```rust
#[tauri::command]
pub async fn restart_device(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        emit_progress(&app, "restart-device", "Restart device", "Sending restart command", None, None, None);
        let mut client = NeoClient::open_and_init()?;
        client.restart_device()
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn read_recovery_diagnostics(app: AppHandle) -> Result<RecoveryDiagnosticsDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        emit_progress(&app, "recovery-diagnostics", "Read diagnostics", "Opening device", None, None, None);
        let mut client = NeoClient::open_and_init()?;
        emit_progress(&app, "recovery-diagnostics", "Read diagnostics", "Reading diagnostic records", None, None, None);
        let log = client.read_recovery_diagnostics()?;
        Ok(RecoveryDiagnosticsDto { log })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}
```

Import `RecoveryDiagnosticsDto` in the `use crate::types::{...}` list.

- [ ] **Step 3: Register commands**

Add to `tauri::generate_handler!`:

```rust
commands::restart_device,
commands::read_recovery_diagnostics,
```

- [ ] **Step 4: Run validation**

Run:

```bash
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-gui-tauri/src-tauri/src/types.rs alpha-gui-tauri/src-tauri/src/commands.rs alpha-gui-tauri/src-tauri/src/lib.rs
git commit -m "feat: expose recovery diagnostics and restart commands"
```

## Task 5: TypeScript Command Wrappers

**Files:**
- Modify: `alpha-gui-tauri/src/api/types.ts`
- Modify: `alpha-gui-tauri/src/api/commands.ts`

- [ ] **Step 1: Add frontend type**

In `types.ts`:

```ts
export interface RecoveryDiagnostics {
  log: string;
}
```

- [ ] **Step 2: Add wrappers**

In `commands.ts`:

```ts
import type { ..., RecoveryDiagnostics } from "./types";

export function restartDevice(): Promise<void> {
  return invoke("restart_device");
}

export function readRecoveryDiagnostics(): Promise<RecoveryDiagnostics> {
  return invoke("read_recovery_diagnostics");
}

export function restoreOriginalStockApplets(): Promise<Inventory> {
  return invoke("restore_original_stock_applets");
}
```

- [ ] **Step 3: Run validation**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri/src/api/types.ts alpha-gui-tauri/src/api/commands.ts
git commit -m "feat: add recovery operation api wrappers"
```

## Task 6: Diagnostic Log Dialog

**Files:**
- Create: `alpha-gui-tauri/src/components/ui/DiagnosticLogDialog.tsx`

- [ ] **Step 1: Create dialog component**

Create:

```tsx
import { Button } from "./Button";

interface Props {
  open: boolean;
  log: string;
  onClose: () => void;
}

export function DiagnosticLogDialog({ open, log, onClose }: Props) {
  if (!open) return null;

  async function copyLog() {
    await navigator.clipboard.writeText(log);
  }

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/35 p-4">
      <div className="flex max-h-[85vh] w-full max-w-4xl flex-col rounded-xl border border-outline-variant bg-surface-container-lowest shadow-2xl">
        <div className="flex items-center justify-between border-b border-outline-variant p-5">
          <div>
            <p className="text-xs font-bold uppercase tracking-[0.18em] text-primary">Recovery Diagnostics</p>
            <h3 className="mt-1 text-2xl font-semibold text-on-surface">Diagnostic Log</h3>
          </div>
          <button className="text-on-surface-variant hover:text-on-surface" onClick={onClose}>
            Close
          </button>
        </div>
        <pre className="m-0 flex-1 overflow-auto whitespace-pre-wrap break-words bg-surface-container-low p-5 font-mono text-xs leading-5 text-on-surface">
          {log}
        </pre>
        <div className="flex justify-end gap-3 border-t border-outline-variant p-4">
          <Button variant="secondary" onClick={() => void copyLog()}>Copy Log</Button>
          <Button onClick={onClose}>Done</Button>
        </div>
      </div>
    </div>
  );
}
```

If Clipboard API type or permission handling fails in typecheck, catch errors and show no toast for this pass.

- [ ] **Step 2: Run validation**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add alpha-gui-tauri/src/components/ui/DiagnosticLogDialog.tsx
git commit -m "feat: add recovery diagnostic log dialog"
```

## Task 7: OS Operations UI

**Files:**
- Modify: `alpha-gui-tauri/src/components/tabs/OsOperations.tsx`

- [ ] **Step 1: Replace props**

Change props to:

```ts
interface Props {
  onBackupEverything: () => void;
  onFlashSystem: () => void;
  onFlashSystemFromSmallRom: () => void;
  onRestartDevice: () => void;
  onReadDiagnostics: () => void;
  onRestoreStockApplets: () => void;
}
```

- [ ] **Step 2: Update normal operation cards**

Render:

- `System Backup` card with `Backup Everything`
- `Reflash Bundled OS` dangerous card
- `Restart Device` secondary card

Remove disabled fake `Reflash Firmware`.

- [ ] **Step 3: Add Advanced Recovery section**

Add a clearly separated section:

```tsx
<section className="rounded-xl border border-error/30 bg-error-container/20 p-6">
  <div className="mb-5">
    <p className="text-xs font-bold uppercase tracking-[0.18em] text-error">Advanced Recovery</p>
    <h3 className="mt-1 text-2xl font-semibold">Validated repair tools</h3>
    <p className="mt-2 text-on-surface-variant">
      Use these only with a current backup. They are for damaged applet/file catalogs or diagnostics before repair.
    </p>
  </div>
  ...
</section>
```

Cards inside:

- `Read Diagnostics`
- `Restore Original Stock Applets`
- `Small ROM Recovery`

Small ROM copy must include:

```text
Hold Right Shift + comma + period + slash while powering on, then enter the password "ernie" when prompted. Connect USB after the Small ROM Updater appears. SmartApplet operations are not available in Small ROM; use this only to reflash the bundled OS.
```

- [ ] **Step 4: Run validation**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
```

Expected: PASS after App wiring is completed in the next task; if this task temporarily breaks props, combine with Task 8 before validating.

- [ ] **Step 5: Commit after Task 8**

Do not commit this task alone if props are not yet wired.

## Task 8: App Wiring And Confirmations

**Files:**
- Modify: `alpha-gui-tauri/src/App.tsx`
- Modify: `alpha-gui-tauri/src/components/tabs/OsOperations.tsx`
- Modify: `alpha-gui-tauri/src/components/ui/DiagnosticLogDialog.tsx`

- [ ] **Step 1: Import wrappers and dialog**

In `App.tsx`, import:

```ts
readRecoveryDiagnostics,
restartDevice,
restoreOriginalStockApplets,
```

Import `DiagnosticLogDialog`.

- [ ] **Step 2: Add diagnostic state**

```ts
const [diagnosticLog, setDiagnosticLog] = useState<string | null>(null);
```

- [ ] **Step 3: Wire OS props**

Pass:

```tsx
onRestartDevice={() => setConfirmRequest({
  title: "Restart device?",
  message: "This restarts the connected device. USB will disconnect temporarily while it reboots.",
  confirmLabel: "Restart",
  destructive: false,
  onConfirm: () => void runOperationWithoutRefresh(async () => {
    await restartDevice();
    setConnected(false);
    setMode("missing");
  }),
})}
onReadDiagnostics={() => void runOperationWithoutRefresh(async () => {
  const result = await readRecoveryDiagnostics();
  setDiagnosticLog(result.log);
})}
onRestoreStockApplets={() => setConfirmRequest({
  title: "Restore original stock applets?",
  message: "This clears the SmartApplet area and reinstalls only bundled original stock applets. Alpha USB and user applets will not be installed by this recovery action. Back up everything first.",
  confirmLabel: "Restore Stock Applets",
  destructive: true,
  onConfirm: () => void runOperation(async () => {
    setInventory(await restoreOriginalStockApplets());
    setAddedApplets([]);
  }),
})}
onFlashSystemFromSmallRom={() => setConfirmRequest({
  title: "Reflash bundled OS from Small ROM?",
  message: "Use this only after entering Small ROM/updater mode and connecting USB. This flashes the bundled stock OS image and can brick the device if interrupted.",
  confirmLabel: "Reflash OS",
  destructive: true,
  onConfirm: () => void runOperationWithoutRefresh(async () => {
    await flashSystemImage(false);
    setConnected(false);
    setMode("missing");
  }),
})}
```

Keep existing `onFlashSystem` behavior but update title if needed.

- [ ] **Step 4: Render diagnostic dialog**

Near existing dialogs:

```tsx
<DiagnosticLogDialog
  open={diagnosticLog !== null}
  log={diagnosticLog ?? ""}
  onClose={() => setDiagnosticLog(null)}
/>
```

- [ ] **Step 5: Run validation**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-gui-tauri/src/App.tsx alpha-gui-tauri/src/components/tabs/OsOperations.tsx alpha-gui-tauri/src/components/ui/DiagnosticLogDialog.tsx
git commit -m "feat: wire OS recovery operations UI"
```

## Task 9: End-To-End Validation And Android Build

**Files:**
- No source edits unless validation reveals issues.

- [ ] **Step 1: Run full validation**

Run:

```bash
cargo test --manifest-path alpha-core/Cargo.toml
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 2: Run Android debug build**

Run:

```bash
npm --prefix alpha-gui-tauri run tauri -- android build --debug
```

Expected: PASS and writes:

```text
alpha-gui-tauri/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

- [ ] **Step 3: If emulator is available, install and launch**

Run:

```bash
adb devices
adb install -r alpha-gui-tauri/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
adb shell am force-stop cz.jakubkolcar.alphagui
adb shell monkey -p cz.jakubkolcar.alphagui -c android.intent.category.LAUNCHER 1
```

Expected: app launches. Manual visual check: OS Operations shows normal actions, Advanced Recovery, Small ROM instructions, no disabled fake firmware operation.

- [ ] **Step 4: Check git status**

Run:

```bash
git status -sb
```

Expected: only intended files changed, with any unrelated pre-existing files left unstaged.

- [ ] **Step 5: Commit validation fixes if any**

If changes were required:

```bash
git add <changed alpha-core/alpha-gui-tauri files>
git commit -m "fix: polish OS recovery operation validation"
```

Do not commit unrelated `alpha-emu`, `aplha-rust-native`, or `hypotheses.tsv` changes unless explicitly requested.

## Completion Criteria

- Normal OS Operations exposes Backup Everything, Reflash Bundled OS, and Restart Device.
- Advanced Recovery exposes Read Diagnostics, Restore Original Stock Applets, and Small ROM bundled OS flash guidance.
- Restore Original Stock Applets installs only original stock applets, not Alpha USB.
- Individual applet removal is not exposed.
- Validator-disabled OS is not exposed.
- Fake firmware flash is removed or remains clearly unavailable without pretending to be validated.
- All app/device operations are implemented in Rust, not Python.
- TypeScript build, Rust checks/tests, and Android debug build pass.
