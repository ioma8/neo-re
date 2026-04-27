pub mod app;
pub mod backup;
pub mod gui;
pub mod gui_about;
pub mod gui_applets;
pub mod gui_connection;
pub mod gui_dashboard;
pub mod gui_model;
pub mod gui_os;

pub use alpha_core::{applet_workflow, bundled_assets, operation_progress, protocol};

#[cfg(target_os = "android")]
pub mod neo_client;

#[cfg(not(target_os = "android"))]
pub use alpha_core::neo_client;

#[cfg(target_os = "android")]
pub mod android_storage;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub use alpha_core::usb;

#[cfg(target_os = "android")]
pub mod usb_android;

#[cfg(target_os = "android")]
pub use usb_android as usb;

#[cfg(not(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android"
)))]
pub mod usb {
    use std::time::Duration;

    use crate::protocol::FileEntry;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum NeoMode {
        Missing,
        Hid,
        HidUnavailable,
        Direct,
    }

    pub struct NeoClient;

    pub fn detect_mode() -> anyhow::Result<NeoMode> {
        Ok(NeoMode::Missing)
    }

    pub fn switch_hid_to_direct() -> anyhow::Result<()> {
        anyhow::bail!("USB access is not implemented for this target yet")
    }

    pub fn wait_for_mode(
        _target: NeoMode,
        _attempts: usize,
        _delay: Duration,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    impl NeoClient {
        pub fn open_and_init() -> anyhow::Result<Self> {
            anyhow::bail!("USB access is not implemented for this target yet")
        }

        pub fn list_files(&mut self) -> anyhow::Result<Vec<FileEntry>> {
            anyhow::bail!("USB access is not implemented for this target yet")
        }

        pub fn download_file(&mut self, _slot: u8) -> anyhow::Result<Vec<u8>> {
            anyhow::bail!("USB access is not implemented for this target yet")
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub use alpha_core::usb_support;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: android_activity::AndroidApp) {
    let mut options = gui::options();
    options.android_app = Some(app);
    if let Err(error) = gui::run(options) {
        tracing::error!(error = ?error, "Alpha GUI exited with an error");
    }
}
