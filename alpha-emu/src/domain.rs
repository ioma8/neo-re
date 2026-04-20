pub const LCD_ROWS: usize = 6;
pub const LCD_COLS: usize = 40;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lcd {
    rows: [String; LCD_ROWS],
}

impl Default for Lcd {
    fn default() -> Self {
        Self {
            rows: std::array::from_fn(|_| String::new()),
        }
    }
}

impl Lcd {
    #[must_use]
    pub fn rows(&self) -> &[String; LCD_ROWS] {
        &self.rows
    }

    pub fn clear(&mut self) {
        for row in &mut self.rows {
            row.clear();
        }
    }

    pub fn set_row(&mut self, index: usize, text: impl Into<String>) {
        if let Some(row) = self.rows.get_mut(index) {
            let mut text = text.into();
            text.truncate(LCD_COLS);
            *row = text;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsbMode {
    HidKeyboard,
    Direct,
}

impl UsbMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::HidKeyboard => "HID keyboard",
            Self::Direct => "Direct USB",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppletMetadata {
    pub applet_id: u16,
    pub name: String,
    pub version_major: u8,
    pub version_minor_bcd: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmulatorSnapshot {
    pub metadata: Option<AppletMetadata>,
    pub screen: Screen,
    pub lcd: Lcd,
    pub usb_mode: UsbMode,
    pub last_status: Option<u32>,
    pub last_trace: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    AppletsMenu,
    AppletRunning,
    UsbAttach,
}
