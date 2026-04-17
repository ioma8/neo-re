use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use anyhow::{Context, anyhow};
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
    pub download: Option<DownloadProgress>,
    client: Option<NeoClient>,
    backup_rx: Option<Receiver<BackupEvent>>,
}

pub struct DownloadProgress {
    pub spinner_index: usize,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

enum BackupEvent {
    Progress {
        current: usize,
        total: usize,
        name: String,
    },
    Saved {
        slot: u8,
        bytes: usize,
        txt_path: PathBuf,
    },
    Done {
        client: NeoClient,
        dir: PathBuf,
    },
    Failed {
        client: NeoClient,
        message: String,
    },
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
            download: None,
            client: None,
            backup_rx: None,
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

    pub fn start_backup_selected(&mut self) -> anyhow::Result<()> {
        let dir = backup::create_backup_dir()?;
        let selected_all = self.file_selected >= self.files.len();
        let targets = if selected_all {
            self.files.clone()
        } else {
            vec![self.files[self.file_selected].clone()]
        };
        let mut client = self
            .client
            .take()
            .context("NEO client is not initialized")?;
        let (tx, rx) = mpsc::channel();
        let total = targets.len();
        let dir_for_thread = dir.clone();
        thread::spawn(move || {
            for (index, entry) in targets.into_iter().enumerate() {
                if tx
                    .send(BackupEvent::Progress {
                        current: index + 1,
                        total,
                        name: entry.name.clone(),
                    })
                    .is_err()
                {
                    return;
                }
                let result = client
                    .download_file(entry.slot)
                    .with_context(|| format!("download slot {}", entry.slot))
                    .and_then(|payload| backup::save_file(&dir_for_thread, &entry, &payload));
                match result {
                    Ok(saved) => {
                        if tx
                            .send(BackupEvent::Saved {
                                slot: entry.slot,
                                bytes: saved.bytes,
                                txt_path: saved.txt_path,
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(error_value) => {
                        let _ = tx.send(BackupEvent::Failed {
                            client,
                            message: format!("{error_value:#}"),
                        });
                        return;
                    }
                }
            }
            let _ = tx.send(BackupEvent::Done {
                client,
                dir: dir_for_thread,
            });
        });
        self.backup_rx = Some(rx);
        self.download = Some(DownloadProgress {
            spinner_index: 0,
            current: 0,
            total,
            message: format!("Saving backup to {}", dir.display()),
        });
        self.status = "Downloading selected file(s)...".to_owned();
        Ok(())
    }

    pub fn tick(&mut self) {
        if let Some(progress) = &mut self.download {
            progress.spinner_index = progress.spinner_index.wrapping_add(1);
        }
        self.drain_backup_events();
    }

    pub fn is_downloading(&self) -> bool {
        self.download.is_some()
    }

    fn drain_backup_events(&mut self) {
        let Some(rx) = self.backup_rx.take() else {
            return;
        };
        let mut keep_rx = true;
        while let Ok(event) = rx.try_recv() {
            match event {
                BackupEvent::Progress {
                    current,
                    total,
                    name,
                } => {
                    if let Some(progress) = &mut self.download {
                        progress.current = current;
                        progress.total = total;
                        progress.message = format!("Downloading {name} ({current}/{total})");
                    }
                }
                BackupEvent::Saved {
                    slot,
                    bytes,
                    txt_path,
                } => {
                    info!(
                        slot = slot,
                        bytes = bytes,
                        txt = %txt_path.display(),
                        "saved AlphaWord text backup"
                    );
                    self.status = format!("Saved {}", txt_path.display());
                }
                BackupEvent::Done { client, dir } => {
                    self.client = Some(client);
                    self.download = None;
                    self.status = format!("Saved backup to {}", dir.display());
                    keep_rx = false;
                }
                BackupEvent::Failed { client, message } => {
                    self.client = Some(client);
                    self.download = None;
                    keep_rx = false;
                    self.set_error(anyhow!(message));
                }
            }
        }
        if keep_rx {
            self.backup_rx = Some(rx);
        }
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
