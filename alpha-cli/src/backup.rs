use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use chrono::Local;
use directories::BaseDirs;

use crate::protocol::FileEntry;

pub struct SavedFile {
    pub txt_path: PathBuf,
    pub bytes: usize,
}

pub fn app_dir() -> anyhow::Result<PathBuf> {
    let dirs = BaseDirs::new().context("resolve home directory")?;
    Ok(dirs.home_dir().join("alpha-cli"))
}

pub fn create_backup_dir() -> anyhow::Result<PathBuf> {
    let dir = backup_root_dir()?
        .join("backups")
        .join(Local::now().format("%Y-%m-%d_%H-%M-%S").to_string());
    fs::create_dir_all(&dir)
        .with_context(|| format!("create backup directory {}", dir.display()))?;
    Ok(dir)
}

fn backup_root_dir() -> anyhow::Result<PathBuf> {
    #[cfg(target_os = "android")]
    {
        crate::android_storage::public_documents_app_dir()
    }

    #[cfg(not(target_os = "android"))]
    {
        app_dir()
    }
}

pub fn save_file(dir: &Path, entry: &FileEntry, payload: &[u8]) -> anyhow::Result<SavedFile> {
    if !dir.is_dir() {
        bail!("backup target is not a directory: {}", dir.display());
    }
    let base = format!("slot-{:02}-{}", entry.slot, sanitize_name(&entry.name));
    let txt_path = dir.join(format!("{base}.txt"));
    let text = text_export_bytes(payload)?;
    fs::write(&txt_path, text).with_context(|| format!("write {}", txt_path.display()))?;
    Ok(SavedFile {
        txt_path,
        bytes: payload.len(),
    })
}

fn text_export_bytes(payload: &[u8]) -> anyhow::Result<Vec<u8>> {
    let text = payload
        .iter()
        .map(|byte| match *byte {
            0 => b' ',
            b'\r' => b'\n',
            byte => byte,
        })
        .collect::<Vec<_>>();
    if text.len() != payload.len() {
        bail!(
            "text export length mismatch: downloaded {} bytes but converted text is {} bytes",
            payload.len(),
            text.len()
        );
    }
    Ok(text)
}

fn sanitize_name(name: &str) -> String {
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
