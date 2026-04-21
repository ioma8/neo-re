use std::path::{Path, PathBuf};

use anyhow::Result;
use eframe::egui;

use crate::firmware::FirmwareRuntime;
use crate::firmware_session::{FirmwareSession, FirmwareSnapshot};
use crate::keyboard::{matrix_cells, matrix_key_is_alphanumeric, matrix_key_label};
use crate::lcd::LcdSnapshot;

/// Runs the desktop Small ROM emulator UI.
///
/// # Errors
///
/// Returns an error if the native GUI runtime fails to start.
pub fn run(path: PathBuf) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Alpha Emulator")
            .with_inner_size([900.0, 640.0])
            .with_min_inner_size([760.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Alpha Emulator",
        options,
        Box::new(move |cc| {
            install_style(&cc.egui_ctx);
            Ok(Box::new(AlphaEmuApp::load(&path)))
        }),
    )
    .map_err(|error| anyhow::anyhow!("failed to run GUI: {error}"))
}

struct AlphaEmuApp {
    firmware_path: PathBuf,
    session: Option<FirmwareSession>,
    load_error: Option<String>,
}

impl AlphaEmuApp {
    fn load(path: &Path) -> Self {
        let mut app = Self {
            firmware_path: path.to_path_buf(),
            session: None,
            load_error: None,
        };
        app.boot_path(path);
        app
    }

    fn boot_path(&mut self, path: &Path) {
        self.firmware_path = path.to_path_buf();
        let loaded = FirmwareRuntime::load_small_rom(path)
            .map_err(|error| error.to_string())
            .and_then(|rom| {
                FirmwareSession::boot_small_rom(rom).map_err(|error| error.to_string())
            });
        match loaded {
            Ok(mut session) => {
                session.run_steps(300_000);
                self.session = Some(session);
                self.load_error = None;
            }
            Err(error) => {
                self.session = None;
                self.load_error = Some(format!("{}: {error}", path.display()));
            }
        }
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("AlphaSmart firmware", &["os3kos"])
            .pick_file()
        {
            self.boot_path(&path);
        }
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(22.0);
                ui.vertical_centered(|ui| {
                    ui.set_max_width(840.0);
                    render_header(ui, self);
                    ui.add_space(18.0);
                    match self.session.as_mut() {
                        Some(session) => {
                            let snapshot = session.snapshot();
                            render_lcd(ui, &snapshot.lcd);
                            ui.add_space(14.0);
                            render_summary(ui, &self.firmware_path, &snapshot);
                            ui.add_space(14.0);
                            render_controls(ui, session);
                            ui.add_space(14.0);
                            render_trace(ui, &session.snapshot());
                        }
                        None => render_empty_state(ui, self.load_error.as_deref()),
                    }
                });
            });
    }
}

impl eframe::App for AlphaEmuApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ui.ctx());
        egui::Frame::new()
            .fill(app_bg())
            .show(ui, |ui| self.render(ui));
    }
}

impl AlphaEmuApp {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let mut handled = false;
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed,
                    repeat: false,
                    ..
                } = event
                    && let Some(value) = char_for_key(*key)
                {
                    if *pressed {
                        session.press_char(value);
                    } else {
                        session.release_char(value);
                    }
                    handled = true;
                }
            }
        });
        if handled {
            session.run_steps(10_000);
        }
    }
}

fn char_for_key(key: egui::Key) -> Option<char> {
    match key {
        egui::Key::E => Some('e'),
        egui::Key::R => Some('r'),
        egui::Key::N => Some('n'),
        egui::Key::I => Some('i'),
        _ => None,
    }
}

fn install_style(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::light());
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    style.visuals.selection.bg_fill = accent_blue();
    style.visuals.window_fill = app_bg();
    ctx.set_global_style(style);
}

fn app_bg() -> egui::Color32 {
    egui::Color32::from_rgb(246, 247, 249)
}

fn card_bg() -> egui::Color32 {
    egui::Color32::from_rgb(255, 255, 255)
}

fn text_primary() -> egui::Color32 {
    egui::Color32::from_rgb(32, 38, 46)
}

fn text_secondary() -> egui::Color32 {
    egui::Color32::from_rgb(102, 112, 127)
}

fn border() -> egui::Stroke {
    egui::Stroke::new(1.0, egui::Color32::from_rgb(219, 224, 231))
}

fn accent_blue() -> egui::Color32 {
    egui::Color32::from_rgb(0, 122, 255)
}

fn render_header(ui: &mut egui::Ui, app: &mut AlphaEmuApp) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("AlphaSmart NEO firmware emulator")
                    .size(24.0)
                    .strong()
                    .color(text_primary()),
            );
            ui.label(
                egui::RichText::new("Small ROM boot, CPU trace, and MMIO logging.")
                    .size(14.0)
                    .color(text_secondary()),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if primary_button(ui, "Open firmware").clicked() {
                app.open_file_dialog();
            }
        });
    });
}

fn render_empty_state(ui: &mut egui::Ui, error: Option<&str>) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new("No firmware loaded")
                .size(18.0)
                .strong()
                .color(text_primary()),
        );
        if let Some(error) = error {
            ui.add_space(6.0);
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
    });
}

fn render_summary(ui: &mut egui::Ui, path: &Path, snapshot: &FirmwareSnapshot) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new("Small ROM session")
                .size(18.0)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            metadata_pill(ui, "File", path.display());
            metadata_pill(ui, "PC", format!("0x{:08x}", snapshot.pc));
            metadata_pill(ui, "SSP", format!("0x{:08x}", snapshot.ssp));
            metadata_pill(ui, "Steps", snapshot.steps);
            metadata_pill(
                ui,
                "State",
                snapshot
                    .last_exception
                    .as_deref()
                    .unwrap_or(if snapshot.stopped {
                        "stopped"
                    } else {
                        "running"
                    }),
            );
        });
    });
}

fn render_controls(ui: &mut egui::Ui, session: &mut FirmwareSession) {
    card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            if primary_button(ui, "Run 200 steps").clicked() {
                session.run_steps(200);
            }
            if secondary_button(ui, "Run 5,000 steps").clicked() {
                session.run_steps(5_000);
            }
            if secondary_button(ui, "Run 250,000 steps").clicked() {
                session.run_steps(250_000);
            }
        });
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("Special matrix keys")
                .size(14.0)
                .strong()
                .color(text_primary()),
        );
        ui.label(
            egui::RichText::new(
                "Mapped from firmware matrix entries without typed alphanumeric mapping.",
            )
            .size(12.0)
            .color(text_secondary()),
        );
        ui.add_space(8.0);
        render_special_matrix_buttons(ui, session);
    });
}

fn render_special_matrix_buttons(ui: &mut egui::Ui, session: &mut FirmwareSession) {
    let cells = matrix_cells();
    let mut any_pressed = false;
    for row in 0..16u8 {
        ui.horizontal_wrapped(|ui| {
            for cell in cells.iter().filter(|cell| cell.row == row) {
                let raw = cell.raw.code();
                if matrix_key_is_alphanumeric(raw) {
                    continue;
                }
                let label = format!("{} (0x{:02x})", matrix_key_label(raw), cell.logical);
                let response = button_for_matrix_key(
                    ui,
                    &label,
                    format!("row {:02x}, col {:x}", cell.row, cell.col),
                );
                if response.clicked() {
                    session.press_matrix_code(raw);
                    session.release_matrix_code(raw);
                    any_pressed = true;
                }
            }
        });
        ui.add_space(6.0);
    }
    if any_pressed {
        session.run_steps(10_000);
    }
}

fn button_for_matrix_key(ui: &mut egui::Ui, label: &str, hover: String) -> egui::Response {
    let response = ui.add(
        egui::Button::new(egui::RichText::new(label).size(11.0).color(text_primary()))
            .fill(egui::Color32::from_rgb(238, 241, 245))
            .stroke(border())
            .corner_radius(8.0)
            .min_size(egui::vec2(52.0, 28.0)),
    );
    response.on_hover_text(hover)
}

fn render_lcd(ui: &mut egui::Ui, lcd: &LcdSnapshot) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new("Emulated LCD")
                .size(15.0)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(10.0);
        let width = ui.available_width().min(800.0);
        let height = width * lcd.height as f32 / lcd.width as f32;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let lcd_bg = egui::Color32::from_rgb(196, 208, 174);
        let pixel = egui::Color32::from_rgb(47, 58, 48);
        painter.rect_filled(rect, 8.0, lcd_bg);
        painter.rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(1.5, egui::Color32::from_rgb(93, 105, 81)),
            egui::StrokeKind::Inside,
        );
        let scale_x = rect.width() / lcd.width as f32;
        let scale_y = rect.height() / lcd.height as f32;
        for y in 0..lcd.height {
            for x in 0..lcd.width {
                if lcd.pixels[y * lcd.width + x] {
                    let min = egui::pos2(
                        rect.left() + x as f32 * scale_x,
                        rect.top() + y as f32 * scale_y,
                    );
                    let max = egui::pos2(min.x + scale_x.max(1.0), min.y + scale_y.max(1.0));
                    painter.rect_filled(egui::Rect::from_min_max(min, max), 0.0, pixel);
                }
            }
        }
    });
}

fn render_trace(ui: &mut egui::Ui, snapshot: &FirmwareSnapshot) {
    ui.columns(2, |columns| {
        trace_panel(&mut columns[0], "Recent CPU trace", &snapshot.trace);
        trace_panel(&mut columns[1], "MMIO accesses", &snapshot.mmio_accesses);
    });
}

fn trace_panel(ui: &mut egui::Ui, title: &str, lines: &[String]) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new(title)
                .size(15.0)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .max_height(320.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if lines.is_empty() {
                    ui.label(egui::RichText::new("none").color(text_secondary()));
                }
                for line in lines {
                    ui.label(
                        egui::RichText::new(line)
                            .monospace()
                            .size(12.0)
                            .color(text_secondary()),
                    );
                }
            });
    });
}

fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(card_bg())
        .stroke(border())
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(16))
        .show(ui, add_contents);
}

fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE))
            .fill(accent_blue())
            .stroke(egui::Stroke::NONE)
            .corner_radius(8.0),
    )
}

fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(text_primary()))
            .fill(egui::Color32::from_rgb(238, 241, 245))
            .stroke(border())
            .corner_radius(8.0),
    )
}

fn metadata_pill(ui: &mut egui::Ui, label: &str, value: impl ToString) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(239, 243, 248))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(12.0)
                        .color(text_secondary()),
                );
                ui.label(
                    egui::RichText::new(value.to_string())
                        .size(12.0)
                        .strong()
                        .color(text_primary()),
                );
            });
        });
}
