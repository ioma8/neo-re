#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainTab {
    Dashboard,
    SmartApplets,
    OsOperations,
    About,
}

impl MainTab {
    pub const ALL: [Self; 4] = [
        Self::Dashboard,
        Self::SmartApplets,
        Self::OsOperations,
        Self::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::SmartApplets => "SmartApplets",
            Self::OsOperations => "OS Operations",
            Self::About => "About",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "▣",
            Self::SmartApplets => "▤",
            Self::OsOperations => "◆",
            Self::About => "?",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceMode {
    Missing,
    Hid,
    HidUnavailable,
    Direct,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformCapabilities {
    pub can_auto_switch_hid: bool,
    pub can_pick_custom_applet_file: bool,
    pub can_run_manager_operations: bool,
    pub mobile: bool,
}

pub fn debug_connection_bypass_available() -> bool {
    cfg!(debug_assertions)
}

impl PlatformCapabilities {
    pub fn current() -> Self {
        #[cfg(target_os = "android")]
        {
            Self::mobile()
        }

        #[cfg(not(target_os = "android"))]
        {
            Self::desktop()
        }
    }

    pub fn desktop() -> Self {
        Self {
            can_auto_switch_hid: true,
            can_pick_custom_applet_file: true,
            can_run_manager_operations: true,
            mobile: false,
        }
    }

    pub fn mobile() -> Self {
        Self {
            can_auto_switch_hid: false,
            can_pick_custom_applet_file: false,
            can_run_manager_operations: true,
            mobile: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionAction {
    WaitForDevice,
    AutoSwitchToDirect,
    InstallAlphaUsbFromDesktop,
    EnterApp,
    Retry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionGate {
    pub action: ConnectionAction,
    pub message: String,
}

impl ConnectionGate {
    pub fn from_mode(mode: DeviceMode, capabilities: PlatformCapabilities) -> Self {
        match mode {
            DeviceMode::Direct if capabilities.can_run_manager_operations => Self {
                action: ConnectionAction::EnterApp,
                message: "Direct USB connection ready.".to_owned(),
            },
            DeviceMode::Direct => Self {
                action: ConnectionAction::Retry,
                message: "Direct USB connection detected, but this build does not include a native manager backend for this platform.".to_owned(),
            },
            DeviceMode::Hid if capabilities.can_auto_switch_hid => Self {
                action: ConnectionAction::AutoSwitchToDirect,
                message: "Keyboard-mode device detected. Switching to direct USB.".to_owned(),
            },
            DeviceMode::Hid => Self {
                action: ConnectionAction::InstallAlphaUsbFromDesktop,
                message: "Keyboard-mode device detected. Install and run Alpha USB from the desktop app before using mobile.".to_owned(),
            },
            DeviceMode::HidUnavailable => Self {
                action: ConnectionAction::Retry,
                message: "AlphaSmart is present, but HID access is unavailable on this platform.".to_owned(),
            },
            DeviceMode::Missing => Self {
                action: ConnectionAction::WaitForDevice,
                message: "Connect your AlphaSmart NEO over USB.".to_owned(),
            },
            DeviceMode::Unknown => Self {
                action: ConnectionAction::Retry,
                message: "Checking AlphaSmart USB connection.".to_owned(),
            },
        }
    }

    pub fn tabs_visible(&self) -> bool {
        self.action == ConnectionAction::EnterApp
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfirmationKind {
    AppletReflash,
    FirmwareFlash,
    SystemFlash,
    SmallRomOperation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmationRequest {
    pub kind: ConfirmationKind,
    pub title: String,
    pub message: String,
    pub destructive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_tabs_until_direct_usb() {
        assert!(
            !ConnectionGate::from_mode(DeviceMode::Missing, PlatformCapabilities::desktop())
                .tabs_visible()
        );
        assert!(
            ConnectionGate::from_mode(DeviceMode::Direct, PlatformCapabilities::desktop())
                .tabs_visible()
        );
    }

    #[test]
    fn desktop_hid_requests_auto_switch() {
        assert_eq!(
            ConnectionGate::from_mode(DeviceMode::Hid, PlatformCapabilities::desktop()).action,
            ConnectionAction::AutoSwitchToDirect
        );
    }

    #[test]
    fn mobile_hid_shows_alpha_usb_instruction() {
        assert_eq!(
            ConnectionGate::from_mode(DeviceMode::Hid, PlatformCapabilities::mobile()).action,
            ConnectionAction::InstallAlphaUsbFromDesktop
        );
    }

    #[test]
    fn mobile_direct_enters_native_manager_ui() {
        let gate = ConnectionGate::from_mode(DeviceMode::Direct, PlatformCapabilities::mobile());

        assert!(gate.tabs_visible());
    }

    #[test]
    fn debug_connection_bypass_is_debug_only() {
        assert_eq!(debug_connection_bypass_available(), cfg!(debug_assertions));
    }
}
