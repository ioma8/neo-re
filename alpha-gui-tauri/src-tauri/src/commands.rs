use std::{collections::BTreeSet, fs};

use alpha_core::{
    applet_workflow::{AppletChecklist, AppletSourceKind},
    backup,
    bundled_assets::{BundledAppletKind, BundledCatalog, BundledOsImageKind, BundledSource},
    neo_client::NeoClientProgress,
    protocol::{FileEntry, SmartAppletRecord},
    usb::{self, NeoClient, NeoMode},
};
use tauri::{AppHandle, Emitter};

use crate::types::{
    AddedAppletSelectionDto, AppletChecklistRowDto, AppletSelectionDto, BackupResultDto,
    BundledAppletDto, DeviceModeDto, FileDto, InventoryDto, ProgressEventDto, SmartAppletDto,
};

const PROGRESS_EVENT: &str = "alpha-progress";

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
pub async fn backup_file(slot: u8, app: AppHandle) -> Result<BackupResultDto, String> {
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
        let dir = backup::create_backup_dir()?;
        let saved = backup::save_file(&dir, &entry, &payload)?;
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
            directory: dir.display().to_string(),
            saved_files: 1,
            saved_applets: 0,
            bytes: saved.bytes,
        })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(error_string)
}

#[tauri::command]
pub async fn backup_all_files(app: AppHandle) -> Result<BackupResultDto, String> {
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
        let dir = backup::create_backup_dir()?;
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
            let saved = backup::save_file(&dir, entry, &payload)?;
            bytes += saved.bytes;
            saved_files += 1;
        }
        Ok(BackupResultDto {
            directory: dir.display().to_string(),
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
pub async fn backup_everything(app: AppHandle) -> Result<BackupResultDto, String> {
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
        let dir = backup::create_device_backup_dir()?;
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
            let saved = backup::save_file(&dir, entry, &payload)?;
            bytes += saved.bytes;
            saved_files += 1;
        }

        let applet_dir = dir.join("applets");
        fs::create_dir_all(&applet_dir)?;
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
            let saved = backup::save_raw_payload(
                &applet_dir,
                &format!("{:04x}-{}", applet.applet_id, safe_name(&applet.name)),
                "os3kapp",
                &payload,
            )?;
            bytes += saved.bytes;
            saved_applets += 1;
        }

        Ok(BackupResultDto {
            directory: dir.display().to_string(),
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
                    bytes: fs::read(path)?,
                });
            }
        }
    }
    Ok(resolved)
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
