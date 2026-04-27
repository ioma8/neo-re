# Alpha GUI Tauri React Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the broken egui Alpha GUI frontend with a polished Tauri + React + Tailwind desktop app while preserving the proven Rust USB/protocol/backend logic.

**Architecture:** Extract reusable device-management logic from `alpha-cli` into a Rust library crate, then expose it through Tauri commands consumed by a React frontend. The existing egui app remains temporarily as a debug fallback until Tauri reaches parity, but new user-facing UI work goes into `alpha-gui-tauri`.

**Tech Stack:** Rust 2024, Tauri 2, React, TypeScript, Vite, Tailwind CSS, existing `alpha-cli` protocol/USB modules, `serde`, `tokio`, `specta` or `tauri-specta` for typed command bindings if practical.

---

## References

- Existing product spec: `docs/superpowers/specs/2026-04-25-alpha-gui-tabbed-manager-design.md`
- Saved visual reference snippets: `docs/superpowers/specs/assets/2026-04-25-alpha-gui-html-reference.md`
- Current Rust backend candidates:
  - `alpha-cli/src/protocol.rs`
  - `alpha-cli/src/neo_client.rs`
  - `alpha-cli/src/usb.rs`
  - `alpha-cli/src/usb_android.rs`
  - `alpha-cli/src/backup.rs`
  - `alpha-cli/src/bundled_assets.rs`
  - `alpha-cli/src/applet_workflow.rs`
  - `alpha-cli/src/operation_progress.rs`
- Current egui frontend to deprecate later: `alpha-cli/src/gui.rs`

## Non-Goals

- Do not polish egui further except to keep it compiling.
- Do not use Python helpers from the GUI or Tauri backend.
- Do not remove the existing egui app until Tauri reaches feature parity.
- Do not implement unvalidated Small ROM operations.
- Do not support non-NEO devices in the first Tauri release.

## Target File Structure

- Create: `alpha-core/Cargo.toml`
  - Reusable Rust library for protocol, USB, bundled assets, backup, progress, and workflows.
- Create: `alpha-core/src/lib.rs`
  - Public module exports and stable backend API boundary.
- Create: `alpha-core/src/protocol.rs`
  - Move from `alpha-cli/src/protocol.rs`.
- Create: `alpha-core/src/neo_client.rs`
  - Move from `alpha-cli/src/neo_client.rs`.
- Create: `alpha-core/src/usb.rs`
  - Move desktop USB transport from `alpha-cli/src/usb.rs`.
- Create: `alpha-core/src/usb_support.rs`
  - Move from `alpha-cli/src/usb_support.rs`.
- Create: `alpha-core/src/backup.rs`
  - Move from `alpha-cli/src/backup.rs`, preserving Android cfg only if still needed by egui.
- Create: `alpha-core/src/bundled_assets.rs`
  - Move from `alpha-cli/src/bundled_assets.rs`.
- Create: `alpha-core/src/applet_workflow.rs`
  - Move from `alpha-cli/src/applet_workflow.rs`.
- Create: `alpha-core/src/operation_progress.rs`
  - Move from `alpha-cli/src/operation_progress.rs`.
- Modify: `alpha-cli/Cargo.toml`
  - Depend on `alpha-core = { path = "../alpha-core" }`.
- Modify: `alpha-cli/src/lib.rs`
  - Re-export or import `alpha-core` modules so current egui app keeps compiling during migration.
- Create: `alpha-gui-tauri/package.json`
  - Frontend scripts and dependencies.
- Create: `alpha-gui-tauri/src-tauri/Cargo.toml`
  - Tauri Rust crate depending on `alpha-core`.
- Create: `alpha-gui-tauri/src-tauri/src/main.rs`
  - Tauri command registration and application entrypoint.
- Create: `alpha-gui-tauri/src-tauri/src/commands.rs`
  - Tauri command functions.
- Create: `alpha-gui-tauri/src-tauri/src/state.rs`
  - Shared operation state, progress channel registry, and device service ownership.
- Create: `alpha-gui-tauri/src-tauri/src/types.rs`
  - Serializable DTOs for files, applets, device mode, progress, command responses.
- Create: `alpha-gui-tauri/src/index.html`
  - Vite entry.
- Create: `alpha-gui-tauri/src/main.tsx`
  - React root.
- Create: `alpha-gui-tauri/src/App.tsx`
  - Connection gate and top-level app routing.
- Create: `alpha-gui-tauri/src/api/commands.ts`
  - Typed frontend wrappers around Tauri `invoke`.
- Create: `alpha-gui-tauri/src/api/progress.ts`
  - Event subscription for operation progress.
- Create: `alpha-gui-tauri/src/state/deviceStore.ts`
  - React state for connection, inventory, selected tab, progress, confirmations.
- Create: `alpha-gui-tauri/src/components/layout/AppShell.tsx`
  - Desktop sidebar and mobile bottom navigation.
- Create: `alpha-gui-tauri/src/components/layout/ConnectionScreen.tsx`
  - Connection-first screen with debug bypass in dev only.
- Create: `alpha-gui-tauri/src/components/tabs/Dashboard.tsx`
  - File list and backup actions.
- Create: `alpha-gui-tauri/src/components/tabs/SmartApplets.tsx`
  - Applet checklist and applet flashing.
- Create: `alpha-gui-tauri/src/components/tabs/OsOperations.tsx`
  - Backup everything, system flash, validated operations.
- Create: `alpha-gui-tauri/src/components/tabs/About.tsx`
  - Project info and warning.
- Create: `alpha-gui-tauri/src/components/ui/Button.tsx`
  - Shared button styles.
- Create: `alpha-gui-tauri/src/components/ui/Card.tsx`
  - Shared card container.
- Create: `alpha-gui-tauri/src/components/ui/ConfirmDialog.tsx`
  - Confirmation modal.
- Create: `alpha-gui-tauri/src/components/ui/ProgressDialog.tsx`
  - Operation progress modal.
- Create: `alpha-gui-tauri/src/styles/tailwind.css`
  - Tailwind imports and CSS variables based on HTML snippets.
- Create: `alpha-gui-tauri/tailwind.config.ts`
  - Theme tokens from saved HTML reference.
- Create: `alpha-gui-tauri/vite.config.ts`
  - Vite config for Tauri.
- Create: `alpha-gui-tauri/tsconfig.json`
  - TypeScript config.
- Modify: `README.md`
  - Add Tauri run/build commands after implementation.
- Modify: `alpha-cli/README.md`
  - Mark egui GUI as temporary/debug legacy once Tauri is usable.

## Backend Command Surface

Expose only these commands initially:

```rust
#[tauri::command]
async fn detect_device() -> Result<DeviceModeDto, String>;

#[tauri::command]
async fn switch_hid_to_direct() -> Result<(), String>;

#[tauri::command]
async fn get_inventory() -> Result<InventoryDto, String>;

#[tauri::command]
async fn backup_file(slot: u8) -> Result<BackupResultDto, String>;

#[tauri::command]
async fn backup_everything() -> Result<BackupResultDto, String>;

#[tauri::command]
async fn list_bundled_applets() -> Result<Vec<BundledAppletDto>, String>;

#[tauri::command]
async fn install_alpha_usb() -> Result<InventoryDto, String>;

#[tauri::command]
async fn flash_applets(selection: AppletSelectionDto) -> Result<InventoryDto, String>;

#[tauri::command]
async fn flash_system_image(reformat_rest_of_rom: bool) -> Result<(), String>;
```

Progress is emitted through Tauri events:

```rust
#[derive(Clone, serde::Serialize)]
struct ProgressEventDto {
    operation_id: String,
    title: String,
    phase: String,
    item: Option<String>,
    completed: Option<usize>,
    total: Option<usize>,
    indeterminate: bool,
    log: Option<String>,
}
```

## Frontend Flow

- The app starts on `ConnectionScreen`.
- `detect_device` runs on launch and on a timer.
- Desktop `HID` mode offers auto-switch and manual switch.
- Mobile support is out of scope for the first Tauri desktop build, but UI copy must preserve the Alpha USB instruction path.
- Debug builds show `Debug: Open UI Without Device`.
- Once direct mode or debug bypass is active, render `AppShell`.
- Tabs are `Dashboard`, `SmartApplets`, `OS Operations`, `About`.
- All destructive operations require `ConfirmDialog`.
- Long operations show `ProgressDialog` driven by Tauri progress events.

## Task 1: Create `alpha-core` Library Crate

**Files:**
- Create: `alpha-core/Cargo.toml`
- Create: `alpha-core/src/lib.rs`
- Move/copy initially: `alpha-cli/src/protocol.rs` to `alpha-core/src/protocol.rs`
- Move/copy initially: `alpha-cli/src/neo_client.rs` to `alpha-core/src/neo_client.rs`
- Move/copy initially: `alpha-cli/src/usb.rs` to `alpha-core/src/usb.rs`
- Move/copy initially: `alpha-cli/src/usb_support.rs` to `alpha-core/src/usb_support.rs`
- Test: `alpha-core/src/neo_client.rs`

- [ ] **Step 1: Write failing crate-level check**

Run:

```bash
cargo check --manifest-path alpha-core/Cargo.toml
```

Expected: FAIL because `alpha-core/Cargo.toml` does not exist.

- [ ] **Step 2: Create minimal crate**

Create `alpha-core/Cargo.toml`:

```toml
[package]
name = "alpha-core"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1.0.102"
chrono = "0.4.44"
directories = "6.0.0"
serde = { version = "1", features = ["derive"] }
tracing = "0.1.44"

[target.'cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))'.dependencies]
rusb = "0.9.4"
```

Create `alpha-core/src/lib.rs`:

```rust
pub mod neo_client;
pub mod protocol;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub mod usb;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub mod usb_support;
```

- [ ] **Step 3: Copy backend modules**

Copy the current contents:

```bash
cp alpha-cli/src/protocol.rs alpha-core/src/protocol.rs
cp alpha-cli/src/neo_client.rs alpha-core/src/neo_client.rs
cp alpha-cli/src/usb.rs alpha-core/src/usb.rs
cp alpha-cli/src/usb_support.rs alpha-core/src/usb_support.rs
```

- [ ] **Step 4: Fix crate paths**

In `alpha-core/src/*`, keep imports as `crate::...`. They should already match.

If `usb.rs` references `crate::usb_support`, it remains valid.

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test --manifest-path alpha-core/Cargo.toml
```

Expected: PASS with copied protocol/client tests.

- [ ] **Step 6: Commit**

```bash
git add alpha-core
git commit -m "feat: extract alpha core backend crate"
```

## Task 2: Move Workflow Modules Into `alpha-core`

**Files:**
- Create: `alpha-core/src/backup.rs`
- Create: `alpha-core/src/bundled_assets.rs`
- Create: `alpha-core/src/applet_workflow.rs`
- Create: `alpha-core/src/operation_progress.rs`
- Modify: `alpha-core/src/lib.rs`
- Test: copied module tests

- [ ] **Step 1: Copy modules**

Run:

```bash
cp alpha-cli/src/backup.rs alpha-core/src/backup.rs
cp alpha-cli/src/bundled_assets.rs alpha-core/src/bundled_assets.rs
cp alpha-cli/src/applet_workflow.rs alpha-core/src/applet_workflow.rs
cp alpha-cli/src/operation_progress.rs alpha-core/src/operation_progress.rs
```

- [ ] **Step 2: Export modules**

Modify `alpha-core/src/lib.rs`:

```rust
pub mod applet_workflow;
pub mod backup;
pub mod bundled_assets;
pub mod neo_client;
pub mod operation_progress;
pub mod protocol;
```

- [ ] **Step 3: Fix Android-only backup dependency**

If `backup.rs` references `crate::android_storage`, guard the Android branch out of `alpha-core` for now:

```rust
#[cfg(target_os = "android")]
{
    anyhow::bail!("Android public document storage is not implemented in alpha-core yet")
}
```

Rationale: first Tauri target is desktop. Do not block the desktop rewrite on mobile storage.

- [ ] **Step 4: Run tests**

Run:

```bash
cargo test --manifest-path alpha-core/Cargo.toml
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-core
git commit -m "feat: move alpha gui workflows into core"
```

## Task 3: Make `alpha-cli` Depend On `alpha-core`

**Files:**
- Modify: `alpha-cli/Cargo.toml`
- Modify: `alpha-cli/src/lib.rs`
- Modify imports in `alpha-cli/src/gui.rs`, `alpha-cli/src/app.rs`, and related files only as needed

- [ ] **Step 1: Add dependency**

Modify `alpha-cli/Cargo.toml`:

```toml
alpha-core = { path = "../alpha-core" }
```

- [ ] **Step 2: Re-export core modules in `alpha-cli/src/lib.rs`**

Prefer temporary compatibility re-exports:

```rust
pub use alpha_core::{
    applet_workflow, backup, bundled_assets, neo_client, operation_progress, protocol,
};
```

For desktop:

```rust
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub use alpha_core::usb;
```

Keep Android-specific modules in `alpha-cli` until a later mobile/Tauri decision.

- [ ] **Step 3: Remove duplicated source modules only after compile passes**

Do not delete old files immediately. First make imports resolve through re-exports.

- [ ] **Step 4: Run validation**

Run:

```bash
cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui
cargo test --manifest-path alpha-cli/Cargo.toml
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-cli/Cargo.toml alpha-cli/src/lib.rs
git commit -m "refactor: use alpha core from alpha cli"
```

## Task 4: Scaffold Tauri + React App

**Files:**
- Create: `alpha-gui-tauri/package.json`
- Create: `alpha-gui-tauri/index.html`
- Create: `alpha-gui-tauri/src/main.tsx`
- Create: `alpha-gui-tauri/src/App.tsx`
- Create: `alpha-gui-tauri/src-tauri/Cargo.toml`
- Create: `alpha-gui-tauri/src-tauri/tauri.conf.json`
- Create: `alpha-gui-tauri/src-tauri/src/main.rs`
- Create: `alpha-gui-tauri/vite.config.ts`
- Create: `alpha-gui-tauri/tsconfig.json`

- [ ] **Step 1: Write failing frontend check**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
```

Expected: FAIL because app is not scaffolded.

- [ ] **Step 2: Create minimal package**

Create `alpha-gui-tauri/package.json`:

```json
{
  "name": "alpha-gui-tauri",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "typecheck": "tsc --noEmit",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "clsx": "^2.1.1",
    "lucide-react": "^0.468.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.6.0",
    "vite": "^5.4.0"
  }
}
```

- [ ] **Step 3: Create Tauri Rust crate**

Create `alpha-gui-tauri/src-tauri/Cargo.toml`:

```toml
[package]
name = "alpha-gui-tauri"
version = "0.1.0"
edition = "2024"

[dependencies]
alpha-core = { path = "../../alpha-core" }
anyhow = "1.0.102"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tauri = { version = "2", features = [] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync"] }
tracing = "0.1.44"
tracing-subscriber = "0.3.23"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **Step 4: Create minimal React root**

Create `alpha-gui-tauri/src/App.tsx`:

```tsx
export function App() {
  return <div className="min-h-screen bg-background text-on-background">AlphaGUI</div>;
}
```

Create `alpha-gui-tauri/src/main.tsx`:

```tsx
import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles/tailwind.css";

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

- [ ] **Step 5: Run validation**

Run:

```bash
npm --prefix alpha-gui-tauri install
npm --prefix alpha-gui-tauri run typecheck
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-gui-tauri package-lock.json
git commit -m "feat: scaffold tauri react alpha gui"
```

## Task 5: Tailwind Theme From HTML References

**Files:**
- Create: `alpha-gui-tauri/tailwind.config.ts`
- Create: `alpha-gui-tauri/postcss.config.js`
- Create: `alpha-gui-tauri/src/styles/tailwind.css`
- Modify: `alpha-gui-tauri/package.json`

- [ ] **Step 1: Install Tailwind dependencies**

Run:

```bash
npm --prefix alpha-gui-tauri install -D tailwindcss postcss autoprefixer
```

- [ ] **Step 2: Add Tailwind config**

Create `alpha-gui-tauri/tailwind.config.ts` using the saved HTML palette:

```ts
import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "#faf8ff",
        surface: "#faf8ff",
        "surface-container": "#ecedfa",
        "surface-container-lowest": "#ffffff",
        "surface-container-low": "#f2f3ff",
        "surface-container-high": "#e6e7f4",
        "on-surface": "#191b24",
        "on-surface-variant": "#424656",
        primary: "#0050cb",
        "primary-container": "#0066ff",
        "on-primary": "#ffffff",
        outline: "#727687",
        "outline-variant": "#c2c6d8",
        error: "#ba1a1a",
        "error-container": "#ffdad6",
        "on-error-container": "#93000a"
      },
      fontFamily: {
        sans: ["Inter", "ui-sans-serif", "system-ui"],
        mono: ["ui-monospace", "SFMono-Regular", "Menlo", "monospace"]
      },
      borderRadius: {
        xl: "0.75rem"
      }
    }
  },
  plugins: []
} satisfies Config;
```

- [ ] **Step 3: Add base CSS**

Create `alpha-gui-tauri/src/styles/tailwind.css`:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  font-family: Inter, ui-sans-serif, system-ui, sans-serif;
  color: #191b24;
  background: #faf8ff;
}

body {
  margin: 0;
  min-width: 320px;
  min-height: 100vh;
}

button, input {
  font: inherit;
}
```

- [ ] **Step 4: Run validation**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-gui-tauri
git commit -m "style: add alpha gui tailwind theme"
```

## Task 6: Tauri DTOs And Basic Commands

**Files:**
- Create: `alpha-gui-tauri/src-tauri/src/types.rs`
- Create: `alpha-gui-tauri/src-tauri/src/commands.rs`
- Modify: `alpha-gui-tauri/src-tauri/src/main.rs`
- Test: `alpha-gui-tauri/src-tauri/src/types.rs`

- [ ] **Step 1: Write failing DTO serialization test**

Add:

```rust
#[test]
fn device_mode_serializes_as_string() {
    let json = serde_json::to_string(&DeviceModeDto::Missing).unwrap();
    assert_eq!(json, "\"missing\"");
}
```

Run:

```bash
cargo test --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml device_mode_serializes_as_string
```

Expected: FAIL because DTOs do not exist.

- [ ] **Step 2: Implement DTOs**

Create:

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntryDto {
    pub slot: u8,
    pub name: String,
    pub bytes: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartAppletDto {
    pub applet_id: u16,
    pub version: String,
    pub name: String,
    pub file_size: u32,
    pub applet_class: u8,
    pub installed: bool,
    pub bundled: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceModeDto {
    Missing,
    Hid,
    HidUnavailable,
    Direct,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryDto {
    pub files: Vec<FileEntryDto>,
    pub applets: Vec<SmartAppletDto>,
}
```

- [ ] **Step 3: Implement initial commands**

Create commands:

```rust
#[tauri::command]
pub async fn detect_device() -> Result<DeviceModeDto, String> {
    alpha_core::usb::detect_mode()
        .map(DeviceModeDto::from)
        .map_err(|error| format!("{error:#}"))
}

#[tauri::command]
pub async fn get_inventory() -> Result<InventoryDto, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut client = alpha_core::usb::NeoClient::open_and_init()
            .map_err(|error| format!("{error:#}"))?;
        let files = client.list_files().map_err(|error| format!("{error:#}"))?;
        let applets = client.list_smart_applets().map_err(|error| format!("{error:#}"))?;
        Ok(InventoryDto::from_core(files, applets))
    })
    .await
    .map_err(|error| format!("{error:#}"))?
}
```

- [ ] **Step 4: Register commands**

In `main.rs`:

```rust
mod commands;
mod types;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::detect_device,
            commands::get_inventory,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run AlphaGUI");
}
```

- [ ] **Step 5: Validate**

Run:

```bash
cargo test --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-gui-tauri/src-tauri
git commit -m "feat: add tauri backend device commands"
```

## Task 7: Frontend Command Wrappers

**Files:**
- Create: `alpha-gui-tauri/src/api/types.ts`
- Create: `alpha-gui-tauri/src/api/commands.ts`
- Test: `alpha-gui-tauri/src/api/commands.test.ts` if Vitest is introduced, otherwise rely on typecheck for this task

- [ ] **Step 1: Add TypeScript DTOs**

Create:

```ts
export type DeviceMode = "missing" | "hid" | "hid-unavailable" | "direct";

export type FileEntry = {
  slot: number;
  name: string;
  bytes: number;
};

export type SmartApplet = {
  appletId: number;
  version: string;
  name: string;
  fileSize: number;
  appletClass: number;
  installed: boolean;
  bundled: boolean;
};

export type Inventory = {
  files: FileEntry[];
  applets: SmartApplet[];
};
```

- [ ] **Step 2: Add wrappers**

Create:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { DeviceMode, Inventory } from "./types";

export function detectDevice(): Promise<DeviceMode> {
  return invoke<DeviceMode>("detect_device");
}

export function getInventory(): Promise<Inventory> {
  return invoke<Inventory>("get_inventory");
}
```

- [ ] **Step 3: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri/src/api
git commit -m "feat: add typed tauri command wrappers"
```

## Task 8: React State Store

**Files:**
- Create: `alpha-gui-tauri/src/state/deviceStore.ts`
- Modify: `alpha-gui-tauri/src/App.tsx`

- [ ] **Step 1: Write state shape**

Use simple React state first. Do not add Zustand/Redux unless the code becomes painful.

Create:

```ts
export type MainTab = "dashboard" | "applets" | "os" | "about";

export type ConnectionState = {
  mode: DeviceMode;
  debugBypass: boolean;
  inventory: Inventory | null;
  selectedTab: MainTab;
  busy: boolean;
  error: string | null;
};
```

- [ ] **Step 2: Implement hook**

Create:

```ts
export function useDeviceState() {
  const [state, setState] = useState<ConnectionState>({
    mode: "missing",
    debugBypass: false,
    inventory: null,
    selectedTab: "dashboard",
    busy: false,
    error: null,
  });

  return { state, setState };
}
```

- [ ] **Step 3: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri/src/state alpha-gui-tauri/src/App.tsx
git commit -m "feat: add alpha gui react state"
```

## Task 9: Connection Screen

**Files:**
- Create: `alpha-gui-tauri/src/components/layout/ConnectionScreen.tsx`
- Modify: `alpha-gui-tauri/src/App.tsx`

- [ ] **Step 1: Implement presentational component**

Create:

```tsx
type Props = {
  mode: DeviceMode;
  busy: boolean;
  error: string | null;
  onScan: () => void;
  onDebugBypass: () => void;
};

export function ConnectionScreen({ mode, busy, error, onScan, onDebugBypass }: Props) {
  return (
    <main className="min-h-screen bg-surface flex items-center justify-center p-6">
      <section className="w-full max-w-xl rounded-xl border border-outline-variant bg-surface-container-lowest p-8 text-center shadow-sm">
        <div className="mx-auto mb-6 flex h-24 w-24 items-center justify-center rounded-full border-4 border-dashed border-surface-container text-primary">
          <Cable className="h-14 w-14" />
        </div>
        <h1 className="text-3xl font-semibold tracking-tight text-on-surface">Awaiting Device</h1>
        <p className="mt-2 text-on-surface-variant">{messageForMode(mode)}</p>
        {error ? <p className="mt-4 rounded-lg bg-error-container p-3 text-on-error-container">{error}</p> : null}
        <button className="mt-6 w-full rounded-lg bg-primary px-5 py-3 font-semibold text-on-primary" onClick={onScan} disabled={busy}>
          {busy ? "Scanning..." : "Scan for Device"}
        </button>
        {import.meta.env.DEV ? (
          <button className="mt-3 w-full rounded-lg border border-outline-variant bg-surface-container px-5 py-3 text-on-surface" onClick={onDebugBypass}>
            Debug: Open UI Without Device
          </button>
        ) : null}
      </section>
    </main>
  );
}
```

- [ ] **Step 2: Wire into `App.tsx`**

Use `detectDevice()` on scan. If mode is `direct`, call `getInventory()`.

- [ ] **Step 3: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri/src
git commit -m "feat: add tauri connection screen"
```

## Task 10: App Shell And Navigation

**Files:**
- Create: `alpha-gui-tauri/src/components/layout/AppShell.tsx`
- Create: `alpha-gui-tauri/src/components/layout/NavItem.tsx`
- Modify: `alpha-gui-tauri/src/App.tsx`

- [ ] **Step 1: Implement desktop shell**

Use the saved HTML structure:

```tsx
export function AppShell({ selectedTab, onSelectTab, children }: PropsWithChildren<Props>) {
  return (
    <div className="min-h-screen bg-background text-on-background lg:flex">
      <aside className="hidden w-64 shrink-0 border-r border-outline-variant bg-surface-container-lowest p-6 lg:block">
        <div className="text-2xl font-semibold text-primary">AlphaGUI</div>
        <div className="mt-1 text-sm text-on-surface-variant">NEO Manager</div>
        <nav className="mt-8 space-y-2">
          {tabs.map((tab) => (
            <NavItem key={tab.id} tab={tab} selected={selectedTab === tab.id} onClick={() => onSelectTab(tab.id)} />
          ))}
        </nav>
      </aside>
      <div className="flex min-w-0 flex-1 flex-col">
        <main className="mx-auto w-full max-w-6xl flex-1 p-6 pb-24 lg:p-8">{children}</main>
        <nav className="fixed bottom-0 left-0 right-0 border-t border-outline-variant bg-surface-container-lowest lg:hidden">
          ...
        </nav>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Validate responsive layout**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add alpha-gui-tauri/src/components/layout alpha-gui-tauri/src/App.tsx
git commit -m "feat: add tauri app shell navigation"
```

## Task 11: Dashboard Tab

**Files:**
- Create: `alpha-gui-tauri/src/components/tabs/Dashboard.tsx`
- Create: `alpha-gui-tauri/src/components/ui/Card.tsx`
- Create: `alpha-gui-tauri/src/components/ui/Button.tsx`
- Modify: `alpha-gui-tauri/src/App.tsx`

- [ ] **Step 1: Implement static dashboard from inventory props**

```tsx
export function Dashboard({ files, onBackupAll, onBackupFile }: Props) {
  return (
    <div className="space-y-6">
      <header className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">Dashboard</h1>
          <p className="text-on-surface-variant">Manage files on the connected AlphaSmart NEO.</p>
        </div>
        <Button onClick={onBackupAll}>Backup All Files</Button>
      </header>
      <Card>
        <div className="border-b border-outline-variant px-5 py-3 text-xs font-semibold uppercase tracking-wide text-on-surface-variant">
          Device Files
        </div>
        {files.length === 0 ? <EmptyFiles /> : files.map((file) => <FileRow key={file.slot} file={file} onBackup={() => onBackupFile(file.slot)} />)}
      </Card>
    </div>
  );
}
```

- [ ] **Step 2: Add placeholder handlers**

Until backup commands are implemented, handlers set a visible `Not implemented yet` error.

- [ ] **Step 3: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri/src/components alpha-gui-tauri/src/App.tsx
git commit -m "feat: add dashboard tab"
```

## Task 12: SmartApplets Tab

**Files:**
- Create: `alpha-gui-tauri/src/components/tabs/SmartApplets.tsx`
- Create: `alpha-gui-tauri/src/components/ui/Checkbox.tsx`
- Modify: `alpha-gui-tauri/src/App.tsx`
- Modify: `alpha-gui-tauri/src/api/types.ts`

- [ ] **Step 1: Add applet checklist frontend model**

```ts
export type AppletRow = {
  key: string;
  name: string;
  version?: string;
  size?: number;
  installed: boolean;
  checked: boolean;
  bundled: boolean;
};
```

- [ ] **Step 2: Implement SmartApplets tab UI**

Follow the saved HTML reference:

- top blue Alpha USB action
- applet checklist in a white card
- checked installed rows
- `Flash to device` disabled until changes
- `Add new applet from file` visible but can remain disabled until file picker task

- [ ] **Step 3: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri/src
git commit -m "feat: add smartapplets tab"
```

## Task 13: OS Operations And About Tabs

**Files:**
- Create: `alpha-gui-tauri/src/components/tabs/OsOperations.tsx`
- Create: `alpha-gui-tauri/src/components/tabs/About.tsx`
- Modify: `alpha-gui-tauri/src/App.tsx`

- [ ] **Step 1: Implement OS Operations**

Use the saved HTML reference:

- `Backup Everything` primary safe action
- `Reflash System` destructive card
- firmware card hidden or disabled with copy explaining no validated bundled firmware image
- validated Small ROM section with only `Probe direct USB` or no mutation operations

- [ ] **Step 2: Implement About**

Include:

- AlphaGUI title
- version
- NEO-only validation note
- GitHub/documentation links
- use-at-own-risk warning

- [ ] **Step 3: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri/src/components/tabs alpha-gui-tauri/src/App.tsx
git commit -m "feat: add os and about tabs"
```

## Task 14: Progress Events And Dialogs

**Files:**
- Create: `alpha-gui-tauri/src/components/ui/ProgressDialog.tsx`
- Create: `alpha-gui-tauri/src/api/progress.ts`
- Modify: `alpha-gui-tauri/src-tauri/src/commands.rs`
- Modify: `alpha-gui-tauri/src-tauri/src/types.rs`

- [ ] **Step 1: Add progress DTO**

In Rust:

```rust
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEventDto {
    pub title: String,
    pub phase: String,
    pub item: Option<String>,
    pub completed: Option<usize>,
    pub total: Option<usize>,
    pub indeterminate: bool,
    pub log: Option<String>,
}
```

- [ ] **Step 2: Emit progress from commands**

Use `app.emit("operation-progress", event)` from long-running commands.

- [ ] **Step 3: Subscribe in frontend**

Create:

```ts
import { listen } from "@tauri-apps/api/event";
import type { ProgressEvent } from "./types";

export function listenToProgress(handler: (event: ProgressEvent) => void) {
  return listen<ProgressEvent>("operation-progress", (event) => handler(event.payload));
}
```

- [ ] **Step 4: Implement ProgressDialog**

Use determinate progress when `total` is non-null and non-zero. Otherwise show spinner text.

- [ ] **Step 5: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-gui-tauri
git commit -m "feat: add tauri progress events"
```

## Task 15: Backup Commands

**Files:**
- Modify: `alpha-gui-tauri/src-tauri/src/commands.rs`
- Modify: `alpha-gui-tauri/src/api/commands.ts`
- Modify: `alpha-gui-tauri/src/components/tabs/Dashboard.tsx`
- Modify: `alpha-gui-tauri/src/components/tabs/OsOperations.tsx`

- [ ] **Step 1: Implement `backup_file` command**

Use `alpha-core::backup` and `alpha-core::usb::NeoClient`.

- [ ] **Step 2: Implement `backup_everything` command**

Reuse current logic from egui task functions, moved into a core service if duplication appears.

- [ ] **Step 3: Wire Dashboard actions**

Call `backup_file(slot)` and `backup_everything()`.

- [ ] **Step 4: Wire OS `Backup Everything`**

Use the same `backup_everything()` wrapper.

- [ ] **Step 5: Validate**

Run:

```bash
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-gui-tauri alpha-core
git commit -m "feat: add tauri backup operations"
```

## Task 16: SmartApplet Flash Commands

**Files:**
- Modify: `alpha-gui-tauri/src-tauri/src/commands.rs`
- Modify: `alpha-gui-tauri/src-tauri/src/types.rs`
- Modify: `alpha-gui-tauri/src/api/commands.ts`
- Modify: `alpha-gui-tauri/src/components/tabs/SmartApplets.tsx`

- [ ] **Step 1: Implement `list_bundled_applets`**

Return catalog from `alpha-core::bundled_assets::BundledCatalog::dev_defaults()`.

- [ ] **Step 2: Implement `install_alpha_usb`**

Resolve bundled Alpha USB bytes/path and call `client.install_smart_applet_with_progress`.

- [ ] **Step 3: Implement `flash_applets`**

Accept selected applet IDs and custom paths. Classify add-only vs clear/reinstall using `alpha-core::applet_workflow`.

- [ ] **Step 4: Wire frontend**

Make `Flash Alpha USB...` and `Flash to device` call real commands.

- [ ] **Step 5: Validate**

Run:

```bash
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
cargo test --manifest-path alpha-core/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-gui-tauri alpha-core
git commit -m "feat: add tauri smartapplet flashing"
```

## Task 17: OS Flash Command

**Files:**
- Modify: `alpha-gui-tauri/src-tauri/src/commands.rs`
- Modify: `alpha-gui-tauri/src/api/commands.ts`
- Modify: `alpha-gui-tauri/src/components/tabs/OsOperations.tsx`
- Test: `alpha-core/src/protocol.rs` and `alpha-core/src/neo_client.rs` existing tests

- [ ] **Step 1: Implement `flash_system_image`**

Resolve bundled system image from `alpha-core::bundled_assets`.

Call:

```rust
client.install_neo_os_image_with_progress(&image, reformat_rest_of_rom, |event| {
    emit_progress(app, event);
})?;
```

- [ ] **Step 2: Add confirmation in frontend**

Require explicit confirmation before calling command.

- [ ] **Step 3: Validate**

Run:

```bash
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
cargo test --manifest-path alpha-core/Cargo.toml
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri alpha-core
git commit -m "feat: add tauri system flashing"
```

## Task 18: File Picker For Custom Applets

**Files:**
- Modify: `alpha-gui-tauri/package.json`
- Modify: `alpha-gui-tauri/src/components/tabs/SmartApplets.tsx`
- Modify: `alpha-gui-tauri/src/api/types.ts`

- [ ] **Step 1: Add dialog plugin if needed**

Use Tauri dialog plugin:

```bash
npm --prefix alpha-gui-tauri install @tauri-apps/plugin-dialog
```

Add Rust plugin dependency if required by current Tauri version.

- [ ] **Step 2: Implement file select**

Use:

```ts
import { open } from "@tauri-apps/plugin-dialog";

const selected = await open({
  multiple: true,
  filters: [{ name: "SmartApplet", extensions: ["os3kapp"] }],
});
```

- [ ] **Step 3: Add selected custom applets to checklist**

Custom applets default checked and are included in `flash_applets`.

- [ ] **Step 4: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-gui-tauri
git commit -m "feat: add custom applet picker"
```

## Task 19: Visual Polish Pass Against HTML Reference

**Files:**
- Modify: `alpha-gui-tauri/src/components/**/*.tsx`
- Modify: `alpha-gui-tauri/src/styles/tailwind.css`

- [ ] **Step 1: Compare screens to reference**

Open:

- `docs/superpowers/specs/assets/2026-04-25-alpha-gui-html-reference.md`

Check:

- connection screen spacing and card
- desktop nav width and selected state
- mobile bottom nav
- dashboard file rows
- SmartApplet checklist
- OS warning cards
- About warning block

- [ ] **Step 2: Make only CSS/component polish changes**

Do not alter backend behavior in this task.

- [ ] **Step 3: Validate**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-gui-tauri/src
git commit -m "style: polish tauri alpha gui screens"
```

## Task 20: Desktop Runtime Smoke Test

**Files:**
- Modify only if bugs are found.

- [ ] **Step 1: Run Tauri dev app**

Run:

```bash
npm --prefix alpha-gui-tauri run tauri dev
```

Expected:

- window opens
- no black/unpainted regions
- connection screen renders full window
- debug bypass opens tabs in dev
- all tabs render without broken wrapping

- [ ] **Step 2: Test no-device behavior**

With no NEO connected:

- `Scan for Device` reports missing
- debug bypass opens UI
- backup/flash actions fail gracefully with a clear error

- [ ] **Step 3: Test direct-device behavior if hardware is available**

With NEO in direct mode:

- inventory loads
- files list appears
- applets list appears
- backup one file works

Do not test destructive flashing unless explicitly approved for that run.

- [ ] **Step 4: Commit fixes**

If fixes were needed:

```bash
git add alpha-gui-tauri alpha-core
git commit -m "fix: complete tauri desktop smoke test"
```

## Task 21: Documentation And Deprecation Notes

**Files:**
- Modify: `README.md`
- Modify: `alpha-cli/README.md`
- Create: `alpha-gui-tauri/README.md`

- [ ] **Step 1: Document commands**

Add:

```bash
npm --prefix alpha-gui-tauri install
npm --prefix alpha-gui-tauri run tauri dev
npm --prefix alpha-gui-tauri run tauri build
```

- [ ] **Step 2: Document architecture**

State:

- `alpha-core` owns protocol and USB backend.
- `alpha-gui-tauri` owns the user-facing desktop GUI.
- `alpha-cli` egui is temporary/debug legacy.

- [ ] **Step 3: Document risks**

Include:

- validated only on AlphaSmart NEO
- OS/app flashing can brick device
- keep USB stable during flash
- debug bypass only bypasses UI connection gate, not device operations

- [ ] **Step 4: Validate docs links**

Run:

```bash
rg -n "alpha-gui-tauri|alpha-core|tauri dev|alpha-gui" README.md alpha-cli/README.md alpha-gui-tauri/README.md
```

Expected: relevant docs entries found.

- [ ] **Step 5: Commit**

```bash
git add README.md alpha-cli/README.md alpha-gui-tauri/README.md
git commit -m "docs: document tauri alpha gui"
```

## Task 22: Final Quality Gates

**Files:**
- Modify only if failures are found.

- [ ] **Step 1: Rust checks**

Run:

```bash
cargo test --manifest-path alpha-core/Cargo.toml
cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui
cargo test --manifest-path alpha-cli/Cargo.toml
cargo check --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml
cargo clippy --manifest-path alpha-gui-tauri/src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: all PASS.

- [ ] **Step 2: Frontend checks**

Run:

```bash
npm --prefix alpha-gui-tauri run typecheck
npm --prefix alpha-gui-tauri run build
```

Expected: all PASS.

- [ ] **Step 3: Helper scan**

Run:

```bash
rg -n "real-check|uv|desktop_tools|std::process::Command|process::Command|Command::new" alpha-core alpha-gui-tauri/src-tauri alpha-gui-tauri/src
```

Expected: no matches, except documentation comments if any are intentional.

- [ ] **Step 4: Commit final fixes**

```bash
git add alpha-core alpha-gui-tauri alpha-cli README.md docs
git commit -m "chore: finalize tauri alpha gui rewrite"
```

## Risk Notes

- Tauri 2 plugin versions may need minor dependency adjustment based on the local lockfile and platform.
- Moving `include_bytes!` bundled assets into `alpha-core` changes relative paths; validate paths immediately after extraction.
- macOS USB permissions may still require user action outside the app.
- Tauri mobile is not part of this plan. The React UI can be reused later, but mobile USB/JNI bridging needs a separate plan.
- The first Tauri implementation should preserve egui as a fallback until real-device backup and non-destructive inventory are tested.

## Completion Criteria

- `alpha-gui-tauri` launches and renders a polished full-window React UI.
- Debug bypass opens all tabs without device.
- With a direct-mode NEO, inventory loads through Rust backend.
- Backup operations work from Tauri.
- Alpha USB install and SmartApplet flows call Rust backend with progress.
- System flash command is present, confirmed, and wired to bundled image.
- No Tauri command shells out to Python, `uv`, or `real-check`.
- All final quality gates pass.
