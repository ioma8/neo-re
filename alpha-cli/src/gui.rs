use std::time::{Duration, Instant};

use anyhow::Context;
use eframe::egui::{self, Align, Color32, Layout, RichText, Stroke, Vec2};
use tracing_subscriber::{fmt, prelude::*};

use crate::{
    app::{self, App, Screen},
    backup,
};

pub fn run(options: eframe::NativeOptions) -> eframe::Result {
    if let Err(error) = init_logging() {
        eprintln!("failed to initialize logging: {error:#}");
    }

    eframe::run_native(
        "Alpha GUI",
        options,
        Box::new(|cc| {
            configure_style(&cc.egui_ctx);
            Ok(Box::new(AlphaGui::default()))
        }),
    )
}

pub fn options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Alpha GUI")
            .with_inner_size(Vec2::new(880.0, 620.0))
            .with_min_inner_size(Vec2::new(720.0, 480.0)),
        ..Default::default()
    }
}

fn init_logging() -> anyhow::Result<()> {
    let log_dir = backup::app_dir()?.join("logs");
    std::fs::create_dir_all(&log_dir).context("create log directory")?;
    let log_file =
        std::fs::File::create(log_dir.join("alpha-gui.log")).context("create GUI log file")?;
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(log_file).with_ansi(false))
        .try_init()
        .context("initialize GUI tracing subscriber")?;
    Ok(())
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = Vec2::new(12.0, 10.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(33, 132, 214);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(230, 241, 252);
    ctx.set_global_style(style);
}

struct AlphaGui {
    app: App,
    last_poll: Instant,
    selected_row: usize,
}

impl Default for AlphaGui {
    fn default() -> Self {
        Self {
            app: App::new(),
            last_poll: Instant::now() - Duration::from_secs(5),
            selected_row: 0,
        }
    }
}

impl eframe::App for AlphaGui {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app.tick();
        if self.app.screen == Screen::Waiting
            && self.last_poll.elapsed() >= Duration::from_millis(650)
        {
            self.last_poll = Instant::now();
            if let Err(error) = self.app.poll_connection() {
                self.app.set_error(error);
            }
        }

        ctx.request_repaint_after(Duration::from_millis(120));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            ui.add_space(4.0);
            header(ui);
            ui.add_space(12.0);
            match self.app.screen {
                Screen::Waiting => self.waiting_view(ui),
                Screen::MainMenu => self.main_menu(ui),
                Screen::Files => self.files_view(ui),
            }
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                status_bar(ui, &self.app.status);
            });
        });

        if self.app.error.is_some() {
            self.error_window(ui.ctx());
        } else if self.app.download.is_some() {
            self.progress_window(ui.ctx());
        }
    }
}

impl AlphaGui {
    fn waiting_view(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(24.0)
            .show(ui, |ui| {
                ui.heading("Connect your AlphaSmart NEO");
                ui.add_space(8.0);
                ui.label("1. Plug the NEO into this computer with USB.");
                ui.label("2. Leave it in normal USB keyboard mode.");
                ui.label("3. Alpha GUI will switch it to direct USB mode and initialize it.");
                ui.add_space(12.0);
                ui.spinner();
                ui.label("Waiting for the device...");
            });
    }

    fn main_menu(&mut self, ui: &mut egui::Ui) {
        ui.heading("Ready");
        ui.label("The NEO is initialized. Choose what to inspect.");
        ui.add_space(16.0);
        if ui
            .add_sized([240.0, 44.0], egui::Button::new("Files on device"))
            .clicked()
            && let Err(error) = self.app.open_files()
        {
            self.app.set_error(error);
        }
    }

    fn files_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Files on device");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Back").clicked() && !self.app.is_downloading() {
                    self.app.screen = Screen::MainMenu;
                }
            });
        });
        ui.label(
            "Select a file, or choose All files. Backups are saved as length-validated .txt files.",
        );
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let files = self.app.files.clone();
            for (index, entry) in files.iter().enumerate() {
                let selected = self.selected_row == index;
                let label = format!(
                    "Slot {:>2}   {:<24}   {:>9}   ~{} words",
                    entry.slot,
                    entry.name,
                    app::human_bytes(entry.attribute_bytes),
                    app::approximate_words_from_bytes(entry.attribute_bytes)
                );
                if selectable_row(ui, selected, &label).clicked() {
                    self.selected_row = index;
                }
            }
            let all_index = self.app.files.len();
            if selectable_row(ui, self.selected_row == all_index, "All files").clicked() {
                self.selected_row = all_index;
            }
        });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let disabled = self.app.is_downloading();
            if ui
                .add_enabled(!disabled, egui::Button::new("Download selected"))
                .clicked()
            {
                self.app.file_selected = self.selected_row;
                if let Err(error) = self.app.start_backup_selected() {
                    self.app.set_error(error);
                }
            }
            ui.label("Destination: ~/alpha-cli/backups/{date-time}/");
        });
    }

    fn progress_window(&self, ctx: &egui::Context) {
        let Some(progress) = &self.app.download else {
            return;
        };
        egui::Window::new("Downloading")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(360.0);
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.heading(&progress.message);
                    ui.label(format!("File {} of {}", progress.current, progress.total));
                });
            });
    }

    fn error_window(&mut self, ctx: &egui::Context) {
        let Some(message) = self.app.error.clone() else {
            return;
        };
        egui::Window::new("Problem")
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(460.0);
                ui.colored_label(Color32::from_rgb(180, 30, 30), message);
                ui.add_space(8.0);
                ui.label("Details were written to ~/alpha-cli/logs/alpha-gui.log");
                if ui.button("Dismiss").clicked() {
                    self.app.error = None;
                    self.app.status = "Ready.".to_owned();
                }
            });
    }
}

fn header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Alpha GUI").size(28.0).strong());
        ui.label(
            RichText::new("AlphaSmart NEO backup")
                .size(16.0)
                .color(Color32::DARK_GRAY),
        );
    });
    ui.separator();
}

fn selectable_row(ui: &mut egui::Ui, selected: bool, text: &str) -> egui::Response {
    let frame = egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 10))
        .stroke(if selected {
            Stroke::new(1.5, Color32::from_rgb(33, 132, 214))
        } else {
            Stroke::new(1.0, Color32::from_gray(220))
        })
        .fill(if selected {
            Color32::from_rgb(232, 244, 255)
        } else {
            Color32::from_rgb(250, 250, 250)
        })
        .corner_radius(8.0);
    frame
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(text);
        })
        .response
        .interact(egui::Sense::click())
}

fn status_bar(ui: &mut egui::Ui, status: &str) {
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Status").strong());
        ui.label(status);
    });
}
