use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDto {
    pub slot: u8,
    pub name: String,
    pub attribute_bytes: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartAppletDto {
    pub applet_id: u16,
    pub version: String,
    pub name: String,
    pub file_size: u32,
    pub applet_class: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundledAppletDto {
    pub id: String,
    pub applet_id: Option<u16>,
    pub name: String,
    pub version: Option<String>,
    pub size: Option<u64>,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppletChecklistRowDto {
    pub key: String,
    pub display_name: String,
    pub version: Option<String>,
    pub size: Option<u64>,
    pub installed: bool,
    pub checked: bool,
    pub source_kind: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryDto {
    pub files: Vec<FileDto>,
    pub installed_applets: Vec<SmartAppletDto>,
    pub bundled_applets: Vec<BundledAppletDto>,
    pub applet_rows: Vec<AppletChecklistRowDto>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceModeDto {
    Missing,
    Hid,
    HidUnavailable,
    Direct,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResultDto {
    pub directory: String,
    pub saved_files: usize,
    pub saved_applets: usize,
    pub bytes: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupTargetDto {
    pub backup_root: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppletSelectionDto {
    pub checked_keys: Vec<String>,
    pub added_files: Vec<AddedAppletSelectionDto>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddedAppletSelectionDto {
    pub key: String,
    pub path: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEventDto {
    pub operation_id: String,
    pub title: String,
    pub phase: String,
    pub item: Option<String>,
    pub completed: Option<usize>,
    pub total: Option<usize>,
    pub indeterminate: bool,
    pub log: Option<String>,
}
