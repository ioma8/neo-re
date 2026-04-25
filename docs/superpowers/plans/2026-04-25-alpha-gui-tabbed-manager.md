# Alpha GUI Tabbed Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rework `alpha-cli` into a connection-gated, tabbed AlphaSmart NEO manager inspired by the saved HTML references, with bundled applets/OS images, clear progress, and shared desktop/mobile workflows.

**Architecture:** Keep Rust/egui. Extract pure workflow models first, then split the GUI into focused modules that share one app state. Platform differences are represented as capabilities: desktop can auto-switch HID to direct, mobile can only instruct the user until Alpha USB/direct mode is available.

**Tech Stack:** Rust 2024, egui/eframe 0.34, existing `real-check` desktop helpers, existing Android USB module, `anyhow`, `chrono`, `directories`.

---

## References

- Spec: `docs/superpowers/specs/2026-04-25-alpha-gui-tabbed-manager-design.md`
- HTML visual reference: `docs/superpowers/specs/assets/2026-04-25-alpha-gui-html-reference.md`
- Current GUI: `alpha-cli/src/gui.rs`
- Desktop command wrappers: `alpha-cli/src/desktop_tools.rs`
- Backup helpers: `alpha-cli/src/backup.rs`
- Protocol parsing: `alpha-cli/src/protocol.rs`

## File Structure

- Modify: `alpha-cli/src/lib.rs`
  - Add new GUI/workflow modules.
- Modify: `alpha-cli/src/gui.rs`
  - Keep `run`, `options`, global style, top-level `AlphaGui`, task dispatch, and shared shell.
  - Remove old card-stack layout as tabs replace it.
- Create: `alpha-cli/src/gui_model.rs`
  - Pure UI/workflow state: `MainTab`, `ConnectionGate`, `PlatformCapabilities`, confirmation types.
- Create: `alpha-cli/src/operation_progress.rs`
  - `OperationProgress`, `ProgressEvent`, helper constructors, progress merge/update logic.
- Create: `alpha-cli/src/bundled_assets.rs`
  - Bundled applet and OS image catalog, dev path resolution, packaged-resource placeholders.
- Create: `alpha-cli/src/applet_workflow.rs`
  - Applet checklist rows, installed/bundled merge, add-only vs clear/reinstall classification.
- Create: `alpha-cli/src/gui_connection.rs`
  - Connection-first screen and desktop/mobile connection messaging.
- Create: `alpha-cli/src/gui_dashboard.rs`
  - Dashboard tab: files list and backup actions.
- Create: `alpha-cli/src/gui_applets.rs`
  - SmartApplets tab: Alpha USB action, applet checklist, flash actions.
- Create: `alpha-cli/src/gui_os.rs`
  - OS Operations tab: backup everything, bundled firmware/system flash, validated Small ROM operations.
- Create: `alpha-cli/src/gui_about.rs`
  - About tab.
- Optional later split: `alpha-cli/src/gui_widgets.rs`
  - Shared egui widgets if `gui.rs` remains too large after extraction.

## Task 1: Pure Tab And Connection Model

**Files:**
- Create: `alpha-cli/src/gui_model.rs`
- Modify: `alpha-cli/src/lib.rs`
- Test: `alpha-cli/src/gui_model.rs`

- [ ] **Step 1: Write failing tests for tab gating and platform connection decisions**

Add tests like:

```rust
#[test]
fn hides_tabs_until_direct_usb() {
    assert!(!ConnectionGate::from_mode(DeviceModeState::Missing, PlatformCapabilities::desktop()).tabs_visible());
    assert!(ConnectionGate::from_mode(DeviceModeState::Direct, PlatformCapabilities::desktop()).tabs_visible());
}

#[test]
fn desktop_hid_requests_auto_switch() {
    assert_eq!(
        ConnectionGate::from_mode(DeviceModeState::Hid, PlatformCapabilities::desktop()).action,
        ConnectionAction::AutoSwitchToDirect
    );
}

#[test]
fn mobile_hid_shows_alpha_usb_instruction() {
    assert_eq!(
        ConnectionGate::from_mode(DeviceModeState::Hid, PlatformCapabilities::mobile()).action,
        ConnectionAction::InstallAlphaUsbFromDesktop
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml gui_model`

Expected: FAIL because `gui_model` types do not exist.

- [ ] **Step 3: Implement minimal model**

Create:

- `MainTab { Dashboard, SmartApplets, OsOperations, About }`
- `PlatformCapabilities { can_auto_switch_hid: bool, can_pick_custom_applet_file: bool }`
- `ConnectionAction { WaitForDevice, AutoSwitchToDirect, InstallAlphaUsbFromDesktop, EnterApp, Retry }`
- `ConnectionGate { action, message, tabs_visible }`

Use existing `DeviceModeState` or move it into `gui_model.rs` if doing so reduces coupling.

- [ ] **Step 4: Run validation**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml gui_model`

Expected: PASS.

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-cli/src/lib.rs alpha-cli/src/gui_model.rs
git commit -m "feat: add alpha gui connection model"
```

## Task 2: Progress Model

**Files:**
- Create: `alpha-cli/src/operation_progress.rs`
- Modify: `alpha-cli/src/lib.rs`
- Test: `alpha-cli/src/operation_progress.rs`

- [ ] **Step 1: Write failing tests for determinate and indeterminate progress**

```rust
#[test]
fn percent_is_known_when_total_is_nonzero() {
    let progress = OperationProgress::new("Backup").with_counts(2, 4);
    assert_eq!(progress.percent(), Some(0.5));
}

#[test]
fn progress_event_updates_phase_and_item() {
    let mut progress = OperationProgress::new("Applet flash");
    progress.apply(ProgressEvent::phase_item("installing", "AlphaWord", 1, 3));
    assert_eq!(progress.phase, "installing");
    assert_eq!(progress.item.as_deref(), Some("AlphaWord"));
    assert_eq!(progress.completed, Some(1));
    assert_eq!(progress.total, Some(3));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml operation_progress`

Expected: FAIL because module/types do not exist.

- [ ] **Step 3: Implement minimal progress model**

Include fields:

- `title: String`
- `phase: String`
- `item: Option<String>`
- `completed: Option<usize>`
- `total: Option<usize>`
- `indeterminate: bool`
- `logs: Vec<String>`

Keep logic pure and independent from egui.

- [ ] **Step 4: Run validation**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml operation_progress`

Expected: PASS.

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-cli/src/lib.rs alpha-cli/src/operation_progress.rs
git commit -m "feat: add alpha gui progress model"
```

## Task 3: Bundled Asset Catalog

**Files:**
- Create: `alpha-cli/src/bundled_assets.rs`
- Modify: `alpha-cli/src/lib.rs`
- Test: `alpha-cli/src/bundled_assets.rs`

- [ ] **Step 1: Discover actual repo asset paths**

Run:

```bash
rg --files exports analysis/cab | rg '(\.os3kapp|\.os3kos)$'
```

Record the stock applet source directory and validated OS image candidates in comments or constants.

- [ ] **Step 2: Write failing tests for bundled catalog**

```rust
#[test]
fn catalog_contains_alpha_usb() {
    let catalog = BundledCatalog::dev_defaults();
    assert!(catalog.applets.iter().any(|item| item.kind == BundledAppletKind::AlphaUsb));
}

#[test]
fn stock_workflows_do_not_require_user_paths() {
    let catalog = BundledCatalog::dev_defaults();
    assert!(catalog.applets.iter().all(|item| item.source.is_resolvable_without_picker()));
    assert!(catalog.os_images.iter().all(|item| item.source.is_resolvable_without_picker()));
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml bundled_assets`

Expected: FAIL because module/types do not exist.

- [ ] **Step 4: Implement dev catalog**

Implement:

- `BundledCatalog`
- `BundledApplet`
- `BundledAppletKind`
- `BundledOsImage`
- `BundledSource`

Use dev path resolution first. Leave packaged embedded bytes as a clear later extension point.

- [ ] **Step 5: Run validation**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml bundled_assets`

Expected: PASS.

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-cli/src/lib.rs alpha-cli/src/bundled_assets.rs
git commit -m "feat: add alpha gui bundled asset catalog"
```

## Task 4: Applet Checklist Workflow

**Files:**
- Create: `alpha-cli/src/applet_workflow.rs`
- Modify: `alpha-cli/src/lib.rs`
- Test: `alpha-cli/src/applet_workflow.rs`

- [ ] **Step 1: Write failing tests for applet merge and diff classification**

Cover:

- installed applets initialize checked
- bundled not installed applets initialize unchecked
- Alpha USB can be selected as a bundled applet
- add-only changes classify as `InstallOnly`
- unchecking installed applets classifies as `ClearAndReinstall`

Example:

```rust
#[test]
fn selecting_only_new_bundled_applet_is_install_only() {
    let state = AppletChecklist::from_installed_and_bundled(installed(), bundled());
    let changed = state.with_checked("alpha-usb", true);
    assert_eq!(changed.plan(), AppletFlashPlan::InstallOnly { count: 1 });
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml applet_workflow`

Expected: FAIL because module/types do not exist.

- [ ] **Step 3: Implement checklist model**

Implement:

- `AppletChecklist`
- `AppletChecklistRow`
- `AppletSourceKind`
- `AppletFlashPlan`
- `AppletFlashMode { NoChanges, InstallOnly, ClearAndReinstall }`

Keep this independent from egui and `real-check`.

- [ ] **Step 4: Run validation**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml applet_workflow`

Expected: PASS.

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-cli/src/lib.rs alpha-cli/src/applet_workflow.rs
git commit -m "feat: add alpha gui applet workflow model"
```

## Task 5: Connection-Gated Shell And Tabs

**Files:**
- Modify: `alpha-cli/src/gui.rs`
- Create: `alpha-cli/src/gui_connection.rs`
- Modify: `alpha-cli/src/lib.rs`

- [ ] **Step 1: Write a small failing unit test for initial tab state if feasible**

If `AlphaGui::default()` can be tested without egui context, add:

```rust
#[test]
fn starts_on_connection_without_visible_tabs() {
    let app = AlphaGui::default();
    assert_eq!(app.selected_tab, MainTab::Dashboard);
    assert!(!app.connection_gate().tabs_visible());
}
```

If the current type is not testable without too much exposure, skip this specific test and rely on `gui_model` tests from Task 1 plus `cargo check`.

- [ ] **Step 2: Run validation before edits**

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

- [ ] **Step 3: Add selected tab and connection gate rendering**

Modify `AlphaGui`:

- add `selected_tab: MainTab`
- add `catalog: BundledCatalog`
- keep inventories and task state

Update `ui()`:

- if not direct, render `gui_connection::connection_screen(...)`
- if direct, render the main app shell with nav and selected tab

- [ ] **Step 4: Implement native layout inspired by snippets**

Desktop:

- left rail width around 240-264
- brand at top
- icon+label tab buttons
- active blue accent

Mobile:

- compact top app bar
- bottom navigation
- same tab order

- [ ] **Step 5: Run validation**

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-linux-android --bin alpha-gui`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-cli/src/lib.rs alpha-cli/src/gui.rs alpha-cli/src/gui_connection.rs
git commit -m "feat: add alpha gui connection-gated tabs"
```

## Task 6: Dashboard Tab

**Files:**
- Create: `alpha-cli/src/gui_dashboard.rs`
- Modify: `alpha-cli/src/gui.rs`
- Modify: `alpha-cli/src/lib.rs`

- [ ] **Step 1: Run baseline check**

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

- [ ] **Step 2: Move file backup actions into dashboard tab**

Implement a dashboard matching the reference:

- title and short status
- prominent `Backup All Files`
- compact rows with document icon, slot/name/size
- per-row `Backup`
- empty state with refresh action

Reuse existing `full_backup_task` and `backup_one_slot_task`, but emit `ProgressEvent` around per-file phases if the functions are touched.

- [ ] **Step 3: Run validation**

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

Run: `cargo test --manifest-path alpha-cli/Cargo.toml`

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add alpha-cli/src/lib.rs alpha-cli/src/gui.rs alpha-cli/src/gui_dashboard.rs
git commit -m "feat: add alpha gui dashboard tab"
```

## Task 7: SmartApplets Tab

**Files:**
- Create: `alpha-cli/src/gui_applets.rs`
- Modify: `alpha-cli/src/gui.rs`
- Modify: `alpha-cli/src/lib.rs`
- Modify: `alpha-cli/src/desktop_tools.rs` if a batch install helper is needed

- [ ] **Step 1: Run workflow tests**

Run: `cargo test --manifest-path alpha-cli/Cargo.toml applet_workflow`

Expected: PASS.

- [ ] **Step 2: Implement checklist UI**

Use the HTML reference:

- top full-width/high-priority `Flash Alpha USB SmartApplet for smartphone connection`
- applet list with checkbox, name, version, source/status, size
- `Add new applet from file`
- `Flash to device` enabled only when `AppletChecklist::plan()` has changes

- [ ] **Step 3: Wire flash plans to existing helpers**

Initial implementation:

- `InstallOnly`: call `desktop_tools::install_applet` for selected new applets
- `ClearAndReinstall`: confirmation, then `desktop_tools::clear_applet_area`, then install checked applets

If stock restore can only use `restore-stock-applets` today, wrap that in a TODO-backed adapter, but keep the UI plan classification accurate.

- [ ] **Step 4: Add confirmation dialog**

Use shared confirmation state:

- title
- message
- destructive flag
- pending action

The clear/reinstall message must say applet area will be cleared.

- [ ] **Step 5: Run validation**

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

Run: `cargo test --manifest-path alpha-cli/Cargo.toml`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-cli/src/lib.rs alpha-cli/src/gui.rs alpha-cli/src/gui_applets.rs alpha-cli/src/desktop_tools.rs
git commit -m "feat: add alpha gui smartapplet tab"
```

## Task 8: OS Operations And About Tabs

**Files:**
- Create: `alpha-cli/src/gui_os.rs`
- Create: `alpha-cli/src/gui_about.rs`
- Modify: `alpha-cli/src/gui.rs`
- Modify: `alpha-cli/src/lib.rs`

- [ ] **Step 1: Write/extend bundled OS tests**

Ensure the catalog exposes firmware/system images without user file picking.

Run: `cargo test --manifest-path alpha-cli/Cargo.toml bundled_assets`

Expected before implementation: FAIL if OS image entries are missing.

- [ ] **Step 2: Implement OS Operations tab**

Use the reference structure:

- safe primary `Backup Everything`
- two dangerous operation panels: `Reflash Firmware`, `Reflash System`
- second confirmation for each reflash
- validated Small ROM operations list only if backed by real helper functions

Use existing `install_os_image` for OS image flashing. If firmware vs system cannot yet be separated by helper, expose only the validated operation and label unresolved operations disabled with honest text.

- [ ] **Step 3: Implement About tab**

Include:

- project identity
- app version from `env!("CARGO_PKG_VERSION")`
- GitHub link text
- docs link text
- NEO-only validation note
- brick-risk warning panel

- [ ] **Step 4: Run validation**

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

Run: `cargo test --manifest-path alpha-cli/Cargo.toml`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add alpha-cli/src/lib.rs alpha-cli/src/gui.rs alpha-cli/src/gui_os.rs alpha-cli/src/gui_about.rs alpha-cli/src/bundled_assets.rs
git commit -m "feat: add alpha gui os and about tabs"
```

## Task 9: Progress Overlay And Task Event Integration

**Files:**
- Modify: `alpha-cli/src/gui.rs`
- Modify: `alpha-cli/src/operation_progress.rs`
- Modify: tab modules that spawn tasks

- [ ] **Step 1: Write failing tests for progress event application if needed**

Add tests for log retention and completion summary.

Run: `cargo test --manifest-path alpha-cli/Cargo.toml operation_progress`

Expected: FAIL if new behavior is not implemented.

- [ ] **Step 2: Extend `TaskEvent`**

Add:

- `TaskEvent::Progress(ProgressEvent)`

Update `drain_task_events()` to apply progress to current operation.

- [ ] **Step 3: Replace spinner-only progress window**

Show:

- title
- phase
- item
- determinate egui progress bar when percent is known
- indeterminate bar/spinner when not known
- recent log lines

Keep the last operation summary visible after finish.

- [ ] **Step 4: Emit progress from backup/applet/OS tasks**

Emit phase-level progress around existing blocking helper calls. Do not fake byte-level OS progress unless the backend exposes it.

- [ ] **Step 5: Run validation**

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui`

Expected: PASS.

Run: `cargo test --manifest-path alpha-cli/Cargo.toml`

Expected: PASS.

Run: `cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-linux-android --bin alpha-gui`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add alpha-cli/src/gui.rs alpha-cli/src/operation_progress.rs alpha-cli/src/gui_dashboard.rs alpha-cli/src/gui_applets.rs alpha-cli/src/gui_os.rs
git commit -m "feat: add alpha gui structured progress"
```

## Task 10: Visual Polish And Final Review

**Files:**
- Modify: `alpha-cli/src/gui.rs`
- Modify: tab modules
- Modify: `alpha-cli/README.md`

- [ ] **Step 1: Review against HTML reference**

Open:

- `docs/superpowers/specs/assets/2026-04-25-alpha-gui-html-reference.md`
- native GUI modules

Check:

- connection screen structure
- left nav and bottom nav
- dashboard file rows
- SmartApplet checklist
- OS Operations danger panels
- About warning panel

- [ ] **Step 2: Tune style constants**

Adjust only shared style constants:

- primary blue toward `#0050cb`
- surface toward `#faf8ff`
- destructive red toward `#ba1a1a`
- small card radius
- tighter row spacing

- [ ] **Step 3: Update README**

Document:

- connection-gated flow
- desktop HID auto-switch
- mobile Alpha USB prerequisite
- bundled applets/OS images
- backup and flashing warnings

- [ ] **Step 4: Full validation**

Run:

```bash
cargo check --manifest-path alpha-cli/Cargo.toml --bin alpha-gui
cargo test --manifest-path alpha-cli/Cargo.toml
cargo check --manifest-path alpha-cli/Cargo.toml --target aarch64-linux-android --bin alpha-gui
```

Expected: all PASS.

- [ ] **Step 5: Code review**

Review changed files for:

- no raw stock paths exposed in normal UI
- no fake unvalidated operations
- no duplicated desktop/mobile workflow logic unless capability-specific
- progress shown for every long-running operation
- destructive confirmations present
- layout inspired by saved HTML snippets

- [ ] **Step 6: Commit**

```bash
git add alpha-cli docs/superpowers/plans/2026-04-25-alpha-gui-tabbed-manager.md
git commit -m "feat: redesign alpha gui tabbed manager"
```

## Notes For Execution

- Work only in `alpha-cli` plus this plan/doc if implementation notes need updates.
- Do not touch emulator code.
- Use `apply_patch` for manual edits.
- Keep commits small through Task 9; the final polish commit should not hide core workflow changes.
- The existing worktree may contain unrelated dirty changes. Do not revert them.
- If an existing helper cannot support a planned action exactly, disable that UI action or route it through the proven helper, then document the limitation in the module and README.
