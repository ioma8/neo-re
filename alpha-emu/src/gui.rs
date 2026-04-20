use std::path::{Path, PathBuf};

use anyhow::Result;
use eframe::egui;

use crate::applet_host::AppletHost;
use crate::domain::{EmulatorSnapshot, Lcd, UsbMode};

/// Runs the desktop emulator UI.
///
/// # Errors
///
/// Returns an error if the native GUI runtime fails to start.
pub fn run(path: PathBuf) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Alpha Emulator")
            .with_inner_size([780.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Alpha Emulator",
        options,
        Box::new(move |_cc| Ok(Box::new(AlphaEmuApp::load(&path)))),
    )
    .map_err(|error| anyhow::anyhow!("failed to run GUI: {error}"))
}

struct AlphaEmuApp {
    host: Option<AppletHost>,
    load_error: Option<String>,
}

impl AlphaEmuApp {
    fn load(path: &Path) -> Self {
        match AppletHost::load(path) {
            Ok(host) => Self {
                host: Some(host),
                load_error: None,
            },
            Err(error) => Self {
                host: None,
                load_error: Some(format!("{}: {error}", path.display())),
            },
        }
    }
}

impl eframe::App for AlphaEmuApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.add_space(16.0);
        ui.heading("AlphaSmart NEO emulator");
        ui.label("First slice: run the real Alpha USB SmartApplet package.");
        ui.add_space(12.0);

        if let Some(error) = &self.load_error {
            ui.colored_label(ui.visuals().error_fg_color, error);
            return;
        }

        let Some(host) = &mut self.host else {
            ui.label("No applet loaded.");
            return;
        };

        let snapshot = host.snapshot();
        render_metadata(ui, &snapshot);
        ui.add_space(12.0);
        render_lcd(ui, &snapshot.lcd);
        ui.add_space(12.0);
        render_controls(ui, host);
        ui.add_space(12.0);
        render_status(ui, &host.snapshot());
    }
}

fn render_metadata(ui: &mut egui::Ui, snapshot: &EmulatorSnapshot) {
    ui.horizontal(|ui| {
        ui.label(format!("Applet: {}", snapshot.metadata.name));
        ui.separator();
        ui.label(format!("ID: 0x{:04x}", snapshot.metadata.applet_id));
        ui.separator();
        ui.label(format!(
            "USB: {}",
            match snapshot.usb_mode {
                UsbMode::HidKeyboard => "HID keyboard",
                UsbMode::Direct => "Direct USB",
            }
        ));
    });
}

fn render_lcd(ui: &mut egui::Ui, lcd: &Lcd) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(188, 199, 164))
        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(78, 86, 68)))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(18))
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(ui.available_width(), 180.0));
            for row in lcd.rows() {
                let text = if row.is_empty() { " " } else { row };
                ui.monospace(text);
            }
        });
}

fn render_controls(ui: &mut egui::Ui, host: &mut AppletHost) {
    ui.horizontal(|ui| {
        if ui.button("Open applet").clicked() {
            host.open_applet();
        }
        if ui.button("Simulate USB attach").clicked() {
            host.simulate_usb_attach();
        }
        if ui.button("Reset emulator").clicked() {
            host.reset();
        }
    });
}

fn render_status(ui: &mut egui::Ui, snapshot: &EmulatorSnapshot) {
    if let Some(status) = snapshot.last_status {
        ui.label(format!("Last status: 0x{status:08x}"));
    } else {
        ui.label("Last status: none");
    }

    if let Some(error) = &snapshot.error {
        ui.colored_label(ui.visuals().error_fg_color, error);
    }

    egui::CollapsingHeader::new("Recent m68k trace")
        .default_open(false)
        .show(ui, |ui| {
            for line in &snapshot.last_trace {
                ui.monospace(line);
            }
        });
}
