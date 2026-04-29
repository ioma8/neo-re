use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use chrono::Local;

use alpha_core::{
    applet_workflow::{AppletChecklist, AppletSourceKind},
    backup,
    bundled_assets::{BundledAppletKind, BundledCatalog, BundledOsImageKind, BundledSource},
    neo_client::NeoClientProgress,
    protocol::{FileEntry, SmartAppletRecord},
    usb::{self, NeoClient, NeoMode},
};
#[cfg(target_os = "android")]
use tauri::Manager;
use tauri::{AppHandle, Emitter};

use crate::types::{
    AddedAppletSelectionDto, AppletChecklistRowDto, AppletSelectionDto, BackupResultDto,
    BackupTargetDto, BundledAppletDto, DeviceModeDto, FileDto, InventoryDto, ProgressEventDto,
    RecoveryDiagnosticsDto, SmartAppletDto,
};

const PROGRESS_EVENT: &str = "alpha-progress";
static BACKUP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
pub async fn detect_device() -> Result<DeviceModeDto, String> {
    tauri::async_runtime::spawn_blocking(|| usb::detect_mode().map(device_mode_dto))
        .await
        .map_err(|error| error.to_string())?
        .map_err(error_string)
}

#[tauri::command]
pub async fn switch_hid_to_direct() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        usb::switch_hid_to_direct()?;
        if !usb::wait_for_mode(NeoMode::Direct, 20, std::time::Duration::from_millis(250))? {
            anyhow::bail!("device did not enter direct USB mode after HID switch");
        }
        Ok::<_, anyhow::Error>(())
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn get_inventory() -> Result<InventoryDto, String> {
    tauri::async_runtime::spawn_blocking(load_inventory)
        .await
        .map_err(|error| error.to_string())?
        .map_err(error_string)
}

#[tauri::command]
pub async fn list_bundled_applets() -> Result<Vec<BundledAppletDto>, String> {
    Ok(BundledCatalog::dev_defaults()
        .applets
        .iter()
        .map(bundled_applet_dto)
        .collect())
}

#[tauri::command]
pub async fn default_backup_root(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        default_backup_root_for_app(&app).map(|path| path.display().to_string())
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub fn runtime_platform() -> &'static str {
    #[cfg(target_os = "android")]
    {
        "android"
    }

    #[cfg(not(target_os = "android"))]
    {
        "desktop"
    }
}

#[tauri::command]
pub fn debug_bypass_enabled() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
pub async fn pick_backup_folder() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(crate::android_saf::pick_backup_folder)
        .await
        .map_err(|error| error.to_string())?
        .map_err(error_string)
}

#[tauri::command]
pub async fn backup_file(
    slot: u8,
    target: Option<BackupTargetDto>,
    app: AppHandle,
) -> Result<BackupResultDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        emit_progress(
            &app,
            "backup-file",
            "Backup file",
            "Starting",
            None,
            None,
            None,
        );
        let mut client = NeoClient::open_and_init()?;
        let files = client.list_files()?;
        let entry = files
            .iter()
            .find(|entry| entry.slot == slot)
            .ok_or_else(|| anyhow::anyhow!("slot {slot} is empty or not listed"))?
            .clone();
        let payload = client.download_file(slot)?;
        let target = create_backup_target(&app, target, "backups")?;
        save_device_file(&target, &entry, &payload)?;
        emit_progress(
            &app,
            "backup-file",
            "Backup file",
            "Saved",
            Some(entry.name),
            Some(1),
            Some(1),
        );
        Ok(BackupResultDto {
            directory: target.display_path(),
            saved_files: 1,
            saved_applets: 0,
            bytes: payload.len(),
        })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn backup_all_files(
    target: Option<BackupTargetDto>,
    app: AppHandle,
) -> Result<BackupResultDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        emit_progress(
            &app,
            "backup-all-files",
            "Backup all files",
            "Opening device",
            None,
            None,
            None,
        );
        let mut client = NeoClient::open_and_init()?;
        let target = create_backup_target(&app, target, "backups")?;
        let files = client.list_files()?;
        let mut bytes = 0;
        let mut saved_files = 0;
        for (index, entry) in files.iter().enumerate() {
            emit_progress(
                &app,
                "backup-all-files",
                "Backup all files",
                "Downloading",
                Some(entry.name.clone()),
                Some(index + 1),
                Some(files.len()),
            );
            let payload = client.download_file(entry.slot)?;
            save_device_file(&target, entry, &payload)?;
            bytes += payload.len();
            saved_files += 1;
        }
        Ok(BackupResultDto {
            directory: target.display_path(),
            saved_files,
            saved_applets: 0,
            bytes,
        })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn backup_everything(
    target: Option<BackupTargetDto>,
    app: AppHandle,
) -> Result<BackupResultDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        emit_progress(
            &app,
            "backup-everything",
            "Backup everything",
            "Opening device",
            None,
            None,
            None,
        );
        let mut client = NeoClient::open_and_init()?;
        let target = create_backup_target(&app, target, "device-backups")?;
        let files = client.list_files()?;
        let mut bytes = 0;
        let mut saved_files = 0;
        for (index, entry) in files.iter().enumerate() {
            emit_progress(
                &app,
                "backup-everything",
                "Backup files",
                "Downloading",
                Some(entry.name.clone()),
                Some(index + 1),
                Some(files.len()),
            );
            let payload = client.download_file(entry.slot)?;
            save_device_file(&target, entry, &payload)?;
            bytes += payload.len();
            saved_files += 1;
        }

        let applets = client.list_smart_applets()?;
        let mut saved_applets = 0;
        for (index, applet) in applets.iter().enumerate() {
            emit_progress(
                &app,
                "backup-everything",
                "Backup SmartApplets",
                "Downloading",
                Some(applet.name.clone()),
                Some(index + 1),
                Some(applets.len()),
            );
            let payload = client.download_smart_applet(applet.applet_id)?;
            save_raw_backup_payload(
                &target,
                Some("applets"),
                &format!("{:04x}-{}", applet.applet_id, safe_name(&applet.name)),
                "os3kapp",
                &payload,
            )?;
            bytes += payload.len();
            saved_applets += 1;
        }

        Ok(BackupResultDto {
            directory: target.display_path(),
            saved_files,
            saved_applets,
            bytes,
        })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn install_alpha_usb(app: AppHandle) -> Result<InventoryDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let catalog = BundledCatalog::dev_defaults();
        let alpha_usb = catalog
            .applets
            .iter()
            .find(|applet| applet.kind == BundledAppletKind::AlphaUsb)
            .ok_or_else(|| anyhow::anyhow!("bundled Alpha USB applet is missing"))?;
        let bytes = resolve_source(&alpha_usb.source)?;
        let mut client = NeoClient::open_and_init()?;
        client.install_smart_applet_with_progress(&bytes, |event| {
            emit_client_progress(&app, "install-alpha-usb", "Install Alpha USB", event);
        })?;
        inventory_from_client(&mut client)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn flash_applets(
    selection: AppletSelectionDto,
    app: AppHandle,
) -> Result<InventoryDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let catalog = BundledCatalog::dev_defaults();
        let mut client = NeoClient::open_and_init()?;
        let installed = client.list_smart_applets()?;
        let mut checklist = AppletChecklist::from_installed_and_bundled(&installed, &catalog.applets);
        let checked = selection.checked_keys.into_iter().collect::<BTreeSet<_>>();
        for added in &selection.added_files {
            checklist
                .rows
                .push(added_file_row(added, checked.contains(&added.key)));
        }
        for row in &mut checklist.rows {
            row.checked = checked.contains(&row.key);
        }
        let requires_clear = checklist.rows.iter().any(|row| row.installed && !row.checked);
        let resolved = resolve_applet_flash_plan(&checklist, &catalog, requires_clear)?;
        if requires_clear {
            emit_progress(
                &app,
                "flash-applets",
                "Flash SmartApplets",
                "Clearing applet area",
                None,
                None,
                None,
            );
            client.clear_smart_applet_area()?;
        }
        for row in checklist.rows.iter().filter(|row| row.checked) {
            match &row.source {
                AppletSourceKind::Bundled { id: _ } => {
                    if !requires_clear && row.installed {
                        continue;
                    }
                    let bytes = resolved
                        .iter()
                        .find(|applet| applet.key == row.key)
                        .ok_or_else(|| anyhow::anyhow!("resolved applet {} is missing", row.key))?
                        .bytes
                        .as_slice();
                    client.install_smart_applet_with_progress(bytes, |event| {
                        emit_client_progress(&app, "flash-applets", "Flash SmartApplets", event);
                    })?;
                }
                AppletSourceKind::InstalledOnly { applet_id } => {
                    if requires_clear {
                        anyhow::bail!(
                            "cannot reinstall installed-only applet 0x{applet_id:04x}; back it up or use bundled stock applets"
                        );
                    }
                }
                AppletSourceKind::AddedFromFile { path } => {
                    let bytes = resolved
                        .iter()
                        .find(|applet| applet.key == row.key)
                        .ok_or_else(|| anyhow::anyhow!("resolved applet file {path} is missing"))?
                        .bytes
                        .as_slice();
                    client.install_smart_applet_with_progress(bytes, |event| {
                        emit_client_progress(&app, "flash-applets", "Flash SmartApplets", event);
                    })?;
                }
            }
        }
        inventory_from_client(&mut client)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn restore_original_stock_applets(app: AppHandle) -> Result<InventoryDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let catalog = BundledCatalog::dev_defaults();
        let applets = catalog.original_stock_restore_applets();
        if applets.is_empty() {
            anyhow::bail!("no bundled stock applets available for restore");
        }

        let mut client = NeoClient::open_and_init()?;
        emit_progress(
            &app,
            "restore-stock-applets",
            "Restore stock applets",
            "Clearing SmartApplet area",
            None,
            Some(0),
            Some(applets.len()),
        );
        client.clear_smart_applet_area()?;

        for (index, applet) in applets.iter().enumerate() {
            let bytes = resolve_source(&applet.source)?;
            let applet_id = applet
                .applet_id
                .ok_or_else(|| anyhow::anyhow!("stock applet {} has no id", applet.name))?;
            emit_progress(
                &app,
                "restore-stock-applets",
                "Restore stock applets",
                "Installing",
                Some(applet.name.clone()),
                Some(index + 1),
                Some(applets.len()),
            );
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

#[tauri::command]
pub async fn restart_device(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        emit_progress(
            &app,
            "restart-device",
            "Restart device",
            "Sending restart command",
            None,
            None,
            None,
        );
        let mut client = NeoClient::open_and_init()?;
        client.restart_device()
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn read_recovery_diagnostics(
    app: AppHandle,
) -> Result<RecoveryDiagnosticsDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        emit_progress(
            &app,
            "recovery-diagnostics",
            "Read diagnostics",
            "Opening device",
            None,
            None,
            None,
        );
        let mut client = NeoClient::open_and_init()?;
        emit_progress(
            &app,
            "recovery-diagnostics",
            "Read diagnostics",
            "Reading diagnostic records",
            None,
            None,
            None,
        );
        let log = client.read_recovery_diagnostics()?;
        Ok(RecoveryDiagnosticsDto { log })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

struct ResolvedAppletImage {
    key: String,
    bytes: Vec<u8>,
}

fn resolve_applet_flash_plan(
    checklist: &AppletChecklist,
    catalog: &BundledCatalog,
    requires_clear: bool,
) -> anyhow::Result<Vec<ResolvedAppletImage>> {
    let mut resolved = Vec::new();
    for row in checklist.rows.iter().filter(|row| row.checked) {
        match &row.source {
            AppletSourceKind::Bundled { id } => {
                let applet = catalog
                    .applets
                    .iter()
                    .find(|applet| &applet.id == id)
                    .ok_or_else(|| anyhow::anyhow!("bundled applet {id} is missing"))?;
                resolved.push(ResolvedAppletImage {
                    key: row.key.clone(),
                    bytes: resolve_source(&applet.source)?,
                });
            }
            AppletSourceKind::InstalledOnly { applet_id } => {
                if requires_clear {
                    anyhow::bail!(
                        "cannot clear applet area while preserving installed-only applet 0x{applet_id:04x}; back it up or provide the applet file first"
                    );
                }
            }
            AppletSourceKind::AddedFromFile { path } => {
                resolved.push(ResolvedAppletImage {
                    key: row.key.clone(),
                    bytes: read_applet_file(path)?,
                });
            }
        }
    }
    Ok(resolved)
}

fn read_applet_file(path: &str) -> anyhow::Result<Vec<u8>> {
    if crate::android_saf::is_android_content_uri(path) {
        crate::android_saf::read_content_uri(path)
    } else {
        fs::read(path).map_err(Into::into)
    }
}

fn added_file_row(
    added: &AddedAppletSelectionDto,
    checked: bool,
) -> alpha_core::applet_workflow::AppletChecklistRow {
    alpha_core::applet_workflow::AppletChecklistRow {
        key: added.key.clone(),
        display_name: added.display_name.clone(),
        version: None,
        size: fs::metadata(&added.path)
            .ok()
            .map(|metadata| metadata.len()),
        installed: false,
        checked,
        source: AppletSourceKind::AddedFromFile {
            path: added.path.clone(),
        },
    }
}

#[tauri::command]
pub async fn flash_system_image(reformat_rest_of_rom: bool, app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let catalog = BundledCatalog::dev_defaults();
        let image = catalog
            .os_image_by_kind(BundledOsImageKind::System)
            .ok_or_else(|| anyhow::anyhow!("bundled NEO OS image is missing"))?;
        let bytes = resolve_source(&image.source)?;
        let mut client = NeoClient::open_and_init()?;
        client.install_neo_os_image_with_progress(&bytes, reformat_rest_of_rom, |event| {
            emit_client_progress(&app, "flash-system", "Flash system image", event);
        })?;
        client.restart_device()
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

fn load_inventory() -> anyhow::Result<InventoryDto> {
    let mut client = NeoClient::open_and_init()?;
    inventory_from_client(&mut client)
}

fn inventory_from_client(client: &mut NeoClient) -> anyhow::Result<InventoryDto> {
    let files = client.list_files()?;
    let installed_applets = client.list_smart_applets()?;
    Ok(inventory_dto(files, installed_applets))
}

fn inventory_dto(files: Vec<FileEntry>, installed: Vec<SmartAppletRecord>) -> InventoryDto {
    let catalog = BundledCatalog::dev_defaults();
    let checklist = AppletChecklist::from_installed_and_bundled(&installed, &catalog.applets);
    InventoryDto {
        files: files.into_iter().map(file_dto).collect(),
        installed_applets: installed.into_iter().map(smart_applet_dto).collect(),
        bundled_applets: catalog.applets.iter().map(bundled_applet_dto).collect(),
        applet_rows: checklist.rows.into_iter().map(checklist_row_dto).collect(),
    }
}

fn file_dto(entry: FileEntry) -> FileDto {
    FileDto {
        slot: entry.slot,
        name: entry.name,
        attribute_bytes: entry.attribute_bytes,
    }
}

fn smart_applet_dto(record: SmartAppletRecord) -> SmartAppletDto {
    SmartAppletDto {
        applet_id: record.applet_id,
        version: record.version,
        name: record.name,
        file_size: record.file_size,
        applet_class: record.applet_class,
    }
}

fn bundled_applet_dto(applet: &alpha_core::bundled_assets::BundledApplet) -> BundledAppletDto {
    BundledAppletDto {
        id: applet.id.clone(),
        applet_id: applet.applet_id,
        name: applet.name.clone(),
        version: applet.version.clone(),
        size: applet.size,
        kind: match applet.kind {
            BundledAppletKind::Stock => "stock",
            BundledAppletKind::AlphaUsb => "alphaUsb",
        }
        .to_owned(),
    }
}

fn checklist_row_dto(
    row: alpha_core::applet_workflow::AppletChecklistRow,
) -> AppletChecklistRowDto {
    let source_kind = match row.source {
        AppletSourceKind::InstalledOnly { .. } => "installedOnly",
        AppletSourceKind::Bundled { .. } => "bundled",
        AppletSourceKind::AddedFromFile { .. } => "addedFromFile",
    };
    AppletChecklistRowDto {
        key: row.key,
        display_name: row.display_name,
        version: row.version,
        size: row.size,
        installed: row.installed,
        checked: row.checked,
        source_kind: source_kind.to_owned(),
    }
}

fn resolve_source(source: &BundledSource) -> anyhow::Result<Vec<u8>> {
    match source {
        BundledSource::Embedded { bytes, .. } => Ok(bytes.to_vec()),
        BundledSource::DevPath(path) => Ok(fs::read(path)?),
    }
}

#[cfg(test)]
fn original_stock_restore_plan_ids() -> Vec<u16> {
    BundledCatalog::dev_defaults()
        .original_stock_restore_applets()
        .into_iter()
        .filter_map(|applet| applet.applet_id)
        .collect()
}

fn verify_installed_applet_id(
    installed: &[SmartAppletRecord],
    applet_id: u16,
) -> anyhow::Result<()> {
    if installed.iter().any(|record| record.applet_id == applet_id) {
        Ok(())
    } else {
        anyhow::bail!("applet 0x{applet_id:04x} did not appear after install")
    }
}

enum BackupTarget {
    Filesystem {
        dir: PathBuf,
    },
    AndroidTree {
        root_uri: String,
        kind: String,
        timestamp: String,
    },
}

impl BackupTarget {
    fn display_path(&self) -> String {
        match self {
            Self::Filesystem { dir } => dir.display().to_string(),
            Self::AndroidTree {
                root_uri,
                kind,
                timestamp,
            } => format!("{root_uri}/{kind}/{timestamp}"),
        }
    }
}

fn create_backup_target(
    app: &AppHandle,
    target: Option<BackupTargetDto>,
    kind: &str,
) -> anyhow::Result<BackupTarget> {
    match target.and_then(|target| target.backup_root) {
        Some(root) if is_android_tree_uri(&root) => Ok(BackupTarget::AndroidTree {
            root_uri: root,
            kind: kind.to_owned(),
            timestamp: backup_timestamp(),
        }),
        Some(root) if !root.trim().is_empty() => {
            let root = PathBuf::from(root);
            ensure_backup_root(&root)?;
            Ok(BackupTarget::Filesystem {
                dir: backup::create_timestamped_dir_in(root, kind)?,
            })
        }
        _ => Ok(BackupTarget::Filesystem {
            dir: backup::create_timestamped_dir_in(default_backup_root_for_app(app)?, kind)?,
        }),
    }
}

fn save_device_file(
    target: &BackupTarget,
    entry: &FileEntry,
    payload: &[u8],
) -> anyhow::Result<()> {
    let base = format!("slot-{:02}-{}", entry.slot, safe_name(&entry.name));
    let text = backup::text_export_bytes(payload)?;
    save_backup_bytes(target, None, &format!("{base}.txt"), &text)
}

fn save_raw_backup_payload(
    target: &BackupTarget,
    subdir: Option<&str>,
    base_name: &str,
    extension: &str,
    payload: &[u8],
) -> anyhow::Result<()> {
    save_backup_bytes(target, subdir, &format!("{base_name}.{extension}"), payload)
}

fn save_backup_bytes(
    target: &BackupTarget,
    subdir: Option<&str>,
    file_name: &str,
    bytes: &[u8],
) -> anyhow::Result<()> {
    match target {
        BackupTarget::Filesystem { dir } => {
            let dir = match subdir {
                Some(subdir) => {
                    let dir = dir.join(subdir);
                    fs::create_dir_all(&dir)?;
                    dir
                }
                None => dir.clone(),
            };
            fs::write(dir.join(file_name), bytes)?;
            Ok(())
        }
        BackupTarget::AndroidTree {
            root_uri,
            kind,
            timestamp,
        } => {
            let relative_path = match subdir {
                Some(subdir) => backup_relative_path(
                    kind,
                    timestamp,
                    &format!("{}/{}", subdir.trim_matches('/'), file_name),
                ),
                None => backup_relative_path(kind, timestamp, file_name),
            };
            crate::android_saf::write_backup_file(root_uri, &relative_path, bytes)
        }
    }
}

fn backup_timestamp() -> String {
    let sequence = BACKUP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{}-{sequence:04}",
        Local::now().format("%Y-%m-%d_%H-%M-%S-%9f")
    )
}

fn is_android_tree_uri(value: &str) -> bool {
    value.starts_with("content://") && value.contains("/tree/")
}

fn backup_relative_path(kind: &str, timestamp: &str, file_name: &str) -> String {
    format!(
        "{}/{}/{}",
        kind.trim_matches('/'),
        timestamp.trim_matches('/'),
        file_name.trim_matches('/')
    )
}

fn default_backup_root_for_app(app: &AppHandle) -> anyhow::Result<PathBuf> {
    #[cfg(target_os = "android")]
    {
        let root = app.path().document_dir()?.join("AlphaGUI");
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        backup::default_backup_root_dir()
    }
}

fn ensure_backup_root(root: &Path) -> anyhow::Result<()> {
    if root.exists() && root.is_dir() {
        Ok(())
    } else if root.exists() {
        anyhow::bail!("backup path is not a directory: {}", root.display())
    } else {
        fs::create_dir_all(root)
            .map_err(Into::into)
            .map_err(|error: anyhow::Error| {
                error.context(format!("create backup root {}", root.display()))
            })
    }
}

fn device_mode_dto(mode: NeoMode) -> DeviceModeDto {
    match mode {
        NeoMode::Missing => DeviceModeDto::Missing,
        NeoMode::Hid => DeviceModeDto::Hid,
        NeoMode::HidUnavailable => DeviceModeDto::HidUnavailable,
        NeoMode::Direct => DeviceModeDto::Direct,
    }
}

fn emit_client_progress(
    app: &AppHandle,
    operation_id: &'static str,
    title: &'static str,
    event: NeoClientProgress,
) {
    match event {
        NeoClientProgress::OsSegmentErased {
            completed,
            total,
            address,
        } => emit_progress(
            app,
            operation_id,
            title,
            "Erased OS segment",
            Some(format!("0x{address:08x}")),
            Some(completed),
            Some(total),
        ),
        NeoClientProgress::ChunkProgrammed {
            label,
            completed,
            total,
        } => emit_progress(
            app,
            operation_id,
            title,
            label,
            None,
            Some(completed),
            Some(total),
        ),
    }
}

fn emit_progress(
    app: &AppHandle,
    operation_id: &'static str,
    title: &'static str,
    phase: &'static str,
    item: Option<String>,
    completed: Option<usize>,
    total: Option<usize>,
) {
    let _ = app.emit(
        PROGRESS_EVENT,
        ProgressEventDto {
            operation_id: operation_id.to_owned(),
            title: title.to_owned(),
            phase: phase.to_owned(),
            item,
            completed,
            total,
            indeterminate: completed.is_none() || total.is_none(),
            log: None,
        },
    );
}

fn safe_name(name: &str) -> String {
    let clean = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned();
    if clean.is_empty() {
        "untitled".to_owned()
    } else {
        clean
    }
}

fn error_string(error: anyhow::Error) -> String {
    format!("{error:#}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_tree_uri_targets_are_detected() {
        assert!(is_android_tree_uri(
            "content://com.android.externalstorage.documents/tree/primary%3ADocuments"
        ));
        assert!(!is_android_tree_uri("/tmp/alpha-gui"));
    }

    #[test]
    fn backup_relative_paths_use_forward_slashes() {
        let path = backup_relative_path("backups", "2026-04-27_12-30-00", "slot-01-Notes.txt");
        assert_eq!(path, "backups/2026-04-27_12-30-00/slot-01-Notes.txt");
    }

    #[test]
    fn backup_timestamps_are_distinct_for_same_second_operations() {
        let first = backup_timestamp();
        let second = backup_timestamp();
        assert_ne!(first, second);
    }

    #[test]
    fn restore_stock_ids_exclude_alpha_usb_and_system() {
        let ids = original_stock_restore_plan_ids();
        assert_eq!(
            ids,
            vec![
                0xa000, 0xaf00, 0xaf75, 0xaf02, 0xaf73, 0xaf03, 0xa004, 0xa007, 0xa006,
                0xa001, 0xa002, 0xa027, 0xa005,
            ]
        );
        assert!(!ids.contains(&0xa130));
        assert!(!ids.contains(&0x0000));
    }

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
}
