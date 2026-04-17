use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use tracing::{error, info};

use crate::{
    backup,
    protocol::FileEntry,
    usb::{self, NeoClient, NeoMode},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    Waiting,
    MainMenu,
    Files,
}

pub struct App {
    pub screen: Screen,
    pub status: String,
    pub error: Option<String>,
    pub main_selected: usize,
    pub file_selected: usize,
    pub files: Vec<FileEntry>,
    client: Option<NeoClient>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Waiting,
            status: "Connect the AlphaSmart NEO by USB. Waiting for HID keyboard mode..."
                .to_owned(),
            error: None,
            main_selected: 0,
            file_selected: 0,
            files: Vec::new(),
            client: None,
        }
    }

    pub fn poll_connection(&mut self) -> anyhow::Result<()> {
        match usb::detect_mode()? {
            NeoMode::Missing => {
                self.status =
                    "Connect the NEO with USB. It should appear as HID keyboard mode first."
                        .to_owned();
            }
            NeoMode::Hid => {
                self.status = "NEO HID mode found. Switching to direct USB mode...".to_owned();
                usb::switch_hid_to_direct().context("switch HID device to direct USB")?;
                if !usb::wait_for_mode(NeoMode::Direct, 40, Duration::from_millis(125))? {
                    anyhow::bail!("NEO did not re-enumerate in direct USB mode");
                }
                self.init_direct()?;
            }
            NeoMode::Direct => {
                self.status = "NEO already in direct USB mode. Initializing...".to_owned();
                self.init_direct()?;
            }
        }
        Ok(())
    }

    pub fn open_files(&mut self) -> anyhow::Result<()> {
        let client = self
            .client
            .as_mut()
            .context("NEO client is not initialized")?;
        self.status = "Reading AlphaWord file list...".to_owned();
        self.files = client.list_files().context("list AlphaWord files")?;
        self.file_selected = 0;
        self.screen = Screen::Files;
        self.status = "Select a file to back it up, or choose All files.".to_owned();
        Ok(())
    }

    pub fn backup_selected(&mut self) -> anyhow::Result<PathBuf> {
        let dir = backup::create_backup_dir()?;
        let selected_all = self.file_selected >= self.files.len();
        let targets = if selected_all {
            self.files.clone()
        } else {
            vec![self.files[self.file_selected].clone()]
        };
        for entry in targets {
            let client = self
                .client
                .as_mut()
                .context("NEO client is not initialized")?;
            self.status = format!("Downloading {}...", entry.name);
            let payload = client
                .download_file(entry.slot)
                .with_context(|| format!("download slot {}", entry.slot))?;
            let saved = backup::save_file(&dir, &entry, &payload)?;
            info!(
                slot = entry.slot,
                bytes = saved.bytes,
                raw = %saved.raw_path.display(),
                txt = %saved.txt_path.display(),
                "saved AlphaWord backup"
            );
        }
        self.status = format!("Saved backup to {}", dir.display());
        Ok(dir)
    }

    pub fn set_error(&mut self, error_value: anyhow::Error) {
        let message = format!("{error_value:#}");
        error!(error = ?error_value, "operation failed");
        self.error = Some(message.clone());
        self.status = message;
    }

    fn init_direct(&mut self) -> anyhow::Result<()> {
        self.client = Some(NeoClient::open_and_init().context("initialize direct USB NEO")?);
        self.screen = Screen::MainMenu;
        self.status = "NEO initialized. Choose an action.".to_owned();
        Ok(())
    }
}

pub fn human_bytes(bytes: u32) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", f64::from(bytes) / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", f64::from(bytes) / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

pub fn approximate_words_from_bytes(bytes: u32) -> usize {
    (bytes as usize / 6).max(1)
}
