use thiserror::Error;

use crate::cpu68k::{self, CpuError};
use crate::domain::EmulatorSnapshot;
use crate::neo_os::NeoOs;
use crate::os3kapp::{Os3kApp, Os3kAppError};

#[derive(Debug)]
pub struct AppletHost {
    app: Os3kApp,
    os: NeoOs,
    last_status: Option<u32>,
    last_trace: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Error)]
pub enum HostError {
    #[error("OS3KApp error")]
    Os3k(#[from] Os3kAppError),
    #[error("CPU error")]
    Cpu(#[from] CpuError),
}

impl AppletHost {
    /// Loads an `OS3KApp` package into a fresh emulator host.
    ///
    /// # Errors
    ///
    /// Returns an error when the package cannot be read or parsed.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, HostError> {
        Ok(Self {
            app: Os3kApp::read(path)?,
            os: NeoOs::default(),
            last_status: None,
            last_trace: Vec::new(),
            error: None,
        })
    }

    pub fn open_applet(&mut self) {
        self.run_message(0x19);
    }

    pub fn simulate_usb_attach(&mut self) {
        self.run_message(0x3_0001);
    }

    pub fn reset(&mut self) {
        self.os = NeoOs::default();
        self.last_status = None;
        self.last_trace.clear();
        self.error = None;
    }

    #[must_use]
    pub fn snapshot(&self) -> EmulatorSnapshot {
        EmulatorSnapshot {
            metadata: self.app.metadata.clone(),
            lcd: self.os.lcd.clone(),
            usb_mode: self.os.usb_mode,
            last_status: self.last_status,
            last_trace: self.last_trace.clone(),
            error: self.error.clone(),
        }
    }

    fn run_message(&mut self, message: u32) {
        match cpu68k::run_message(&self.app, &mut self.os, message) {
            Ok(result) => {
                self.last_status = Some(result.status);
                self.last_trace = result.trace;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppletHost;
    use crate::domain::UsbMode;

    #[test]
    fn alpha_usb_focus_draws_instructions() -> Result<(), Box<dyn std::error::Error>> {
        let mut host = AppletHost::load("../exports/applets/alpha-usb-native.os3kapp")?;
        host.open_applet();
        let snapshot = host.snapshot();

        assert_eq!(snapshot.error, None);
        assert_eq!(snapshot.lcd.rows()[1], "Now connect the NEO");
        assert_eq!(snapshot.lcd.rows()[2], "to your computer or");
        assert_eq!(snapshot.lcd.rows()[3], "smartphone via USB.");
        Ok(())
    }

    #[test]
    fn alpha_usb_attach_switches_to_direct() -> Result<(), Box<dyn std::error::Error>> {
        let mut host = AppletHost::load("../exports/applets/alpha-usb-native.os3kapp")?;
        host.simulate_usb_attach();
        let snapshot = host.snapshot();

        assert_eq!(snapshot.error, None);
        assert_eq!(snapshot.usb_mode, UsbMode::Direct);
        assert_eq!(snapshot.last_status, Some(0x11));
        Ok(())
    }
}
