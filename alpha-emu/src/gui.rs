use std::path::{Path, PathBuf};

use anyhow::Result;
use eframe::egui;

use crate::domain::{EmulatorSnapshot, Lcd, Screen, UsbMode};
use crate::neo_system::NeoSystem;

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
    system: Option<NeoSystem>,
    load_error: Option<String>,
}

impl AlphaEmuApp {
    fn load(path: &Path) -> Self {
        let mut app = Self {
            system: None,
            load_error: None,
        };
        app.load_applet(path);
        app
    }

    fn load_applet(&mut self, path: &Path) {
        match NeoSystem::load(path) {
            Ok(system) => {
                self.system = Some(system);
                self.load_error = None;
            }
            Err(error) => {
                self.system = None;
                self.load_error = Some(format!("{}: {error}", path.display()));
            }
        }
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("AlphaSmart applet", &["os3kapp"])
            .pick_file()
        {
            self.load_applet(&path);
        }
    }
}

impl eframe::App for AlphaEmuApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ui);

        ui.add_space(16.0);
        ui.heading("AlphaSmart NEO emulator");
        ui.label("Emulated SmartApplets menu and Alpha USB runtime.");
        ui.add_space(12.0);

        render_top_controls(ui, self);
        ui.add_space(12.0);

        let Some(system) = &mut self.system else {
            if let Some(error) = &self.load_error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            ui.label("No applet loaded.");
            return;
        };

        let snapshot = system.snapshot();
        render_metadata(ui, &snapshot);
        ui.add_space(12.0);
        render_lcd(ui, &snapshot.lcd);
        ui.add_space(12.0);
        render_runtime_controls(ui, system);
        ui.add_space(12.0);
        render_status(ui, &system.snapshot());
    }
}

impl AlphaEmuApp {
    fn handle_keyboard(&mut self, ui: &egui::Ui) {
        let Some(system) = &mut self.system else {
            return;
        };
        ui.input(|input| {
            if input.key_pressed(egui::Key::ArrowUp) {
                system.menu_up();
            }
            if input.key_pressed(egui::Key::ArrowDown) {
                system.menu_down();
            }
            if input.key_pressed(egui::Key::Enter) {
                if system.snapshot().screen == crate::domain::Screen::AppletRunning {
                    system.applet_key(0x0d);
                } else {
                    system.open_selected();
                }
            }
            if input.key_pressed(egui::Key::Escape) {
                system.applet_key(0x1b);
            }
            if input.key_pressed(egui::Key::Backspace) {
                system.applet_key(0x08);
            }
            for event in &input.events {
                if let egui::Event::Text(text) = event {
                    for byte in text.bytes() {
                        system.applet_key(byte);
                    }
                }
            }
        });
    }
}

fn render_top_controls(ui: &mut egui::Ui, app: &mut AlphaEmuApp) {
    ui.horizontal(|ui| {
        if ui.button("Open applet").clicked() {
            app.open_file_dialog();
        }
        ui.label("Menu: Up/Down arrows select, Enter opens.");
    });
}

fn render_metadata(ui: &mut egui::Ui, snapshot: &EmulatorSnapshot) {
    ui.horizontal(|ui| {
        let Some(metadata) = &snapshot.metadata else {
            ui.label("Applet: none");
            return;
        };
        ui.label(format!("Applet: {}", metadata.name));
        ui.separator();
        ui.label(format!("ID: 0x{:04x}", metadata.applet_id));
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

fn render_runtime_controls(ui: &mut egui::Ui, system: &mut NeoSystem) {
    ui.horizontal(|ui| {
        if ui.button("Simulate USB attach").clicked() {
            system.simulate_usb_attach();
        }
        if ui.button("Reset emulator").clicked() {
            system.reset();
        }
    });
}

fn render_status(ui: &mut egui::Ui, snapshot: &EmulatorSnapshot) {
    if let Some(status) = snapshot.last_status {
        ui.label(format!("Last status: 0x{status:08x}"));
    } else {
        ui.label("Last status: none");
    }
    ui.label(format!(
        "Screen: {}",
        match snapshot.screen {
            Screen::AppletsMenu => "SmartApplets menu",
            Screen::AppletRunning => "Applet running",
            Screen::UsbAttach => "USB attach",
        }
    ));

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
