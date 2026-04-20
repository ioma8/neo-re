use thiserror::Error;

use crate::cpu68k::{self, CpuError};
use crate::domain::{EmulatorSnapshot, Screen, UsbMode};
use crate::neo_os::NeoOs;
use crate::os3kapp::{Os3kApp, Os3kAppError};

#[derive(Debug)]
pub struct AppletHost {
    app: Os3kApp,
    os: NeoOs,
    screen: Screen,
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
        let app = Os3kApp::read(path)?;
        let mut os = NeoOs::default();
        os.draw_applets_menu(&app.metadata.name, true);
        Ok(Self {
            app,
            os,
            screen: Screen::AppletsMenu,
            last_status: None,
            last_trace: Vec::new(),
            error: None,
        })
    }

    pub fn open_applet(&mut self) {
        self.screen = Screen::AppletRunning;
        self.run_message(0x19);
    }

    pub fn simulate_usb_attach(&mut self) {
        self.screen = Screen::UsbAttach;
        self.os.draw_usb_attach_start();
        self.run_message(0x3_0001);
        if self.error.is_none() {
            if self.os.usb_mode == UsbMode::Direct {
                self.os.draw_direct_attached();
            } else {
                self.os.draw_usb_keyboard_attached();
            }
        }
    }

    pub fn reset(&mut self) {
        self.os = NeoOs::default();
        self.os.draw_applets_menu(&self.app.metadata.name, true);
        self.screen = Screen::AppletsMenu;
        self.last_status = None;
        self.last_trace.clear();
        self.error = None;
    }

    pub fn menu_up(&mut self) {}

    pub fn menu_down(&mut self) {}

    pub fn open_selected(&mut self) {
        if self.screen == Screen::AppletsMenu {
            self.open_applet();
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> EmulatorSnapshot {
        EmulatorSnapshot {
            metadata: Some(self.app.metadata.clone()),
            screen: self.screen,
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
    fn starts_in_applets_menu() -> Result<(), Box<dyn std::error::Error>> {
        let host = AppletHost::load("../exports/applets/alpha-usb-native.os3kapp")?;
        let snapshot = host.snapshot();

        assert_eq!(snapshot.screen, crate::domain::Screen::AppletsMenu);
        assert_eq!(snapshot.lcd.rows()[0], "SmartApplets");
        assert_eq!(snapshot.lcd.rows()[2], "> Alpha USB");
        Ok(())
    }

    #[test]
    fn alpha_usb_attach_switches_to_direct() -> Result<(), Box<dyn std::error::Error>> {
        let mut host = AppletHost::load("../exports/applets/alpha-usb-native.os3kapp")?;
        host.simulate_usb_attach();
        let snapshot = host.snapshot();

        assert_eq!(snapshot.error, None);
        assert_eq!(snapshot.usb_mode, UsbMode::Direct);
        assert_eq!(snapshot.lcd.rows()[1], "Connected to");
        assert_eq!(snapshot.lcd.rows()[2], "NEO Manager.");
        assert_eq!(snapshot.last_status, Some(0x11));
        Ok(())
    }
}
