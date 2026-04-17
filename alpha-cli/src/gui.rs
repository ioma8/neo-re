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
    style.spacing.item_spacing = Vec2::new(12.0, 12.0);
    style.spacing.button_padding = Vec2::new(18.0, 10.0);
    style.visuals.window_corner_radius = 8.0.into();
    style.visuals.widgets.active.bg_fill = ACCENT;
    style.visuals.widgets.hovered.bg_fill = SOFT_BLUE;
    ctx.set_global_style(style);
}

const ACCENT: Color32 = Color32::from_rgb(25, 112, 190);
const ACCENT_DARK: Color32 = Color32::from_rgb(20, 72, 116);
const INK: Color32 = Color32::from_rgb(25, 34, 45);
const MUTED: Color32 = Color32::from_rgb(88, 99, 112);
const LINE: Color32 = Color32::from_rgb(215, 222, 230);
const SOFT_BLUE: Color32 = Color32::from_rgb(231, 243, 253);
const PANEL: Color32 = Color32::from_rgb(249, 251, 253);
const ROW_SELECTED: Color32 = Color32::from_rgb(228, 241, 252);

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
        let compact = ui.available_width() < 640.0;
        let margin = if compact { 14 } else { 24 };
        egui::Frame::central_panel(ui.style())
            .inner_margin(egui::Margin::same(margin))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        header(ui, compact);
                        ui.add_space(if compact { 8.0 } else { 14.0 });
                        match self.app.screen {
                            Screen::Waiting => self.waiting_view(ui, compact),
                            Screen::MainMenu => self.main_menu(ui, compact),
                            Screen::Files => self.files_view(ui, compact),
                        }
                        ui.add_space(12.0);
                        status_bar(ui, &self.app.status, compact);
                    });
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
    fn waiting_view(&self, ui: &mut egui::Ui, compact: bool) {
        panel()
            .inner_margin(if compact { 18.0 } else { 28.0 })
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("Connect your AlphaSmart NEO").color(INK));
                    ui.add_space(6.0);
                    ui.label(RichText::new("Leave it in normal USB keyboard mode.").color(MUTED));
                    ui.add_space(if compact { 12.0 } else { 18.0 });
                    ui.spinner();
                    ui.add_space(10.0);
                    ui.label(RichText::new("Waiting for the device...").color(ACCENT_DARK));
                });
                ui.add_space(if compact { 12.0 } else { 18.0 });
                step_row(ui, "1", "Plug the NEO into this computer with USB.");
                step_row(ui, "2", "Keep the NEO in its normal USB keyboard mode.");
                step_row(
                    ui,
                    "3",
                    "Alpha GUI will switch it to direct USB mode and initialize it.",
                );
            });
    }

    fn main_menu(&mut self, ui: &mut egui::Ui, compact: bool) {
        panel()
            .inner_margin(if compact { 18.0 } else { 28.0 })
            .show(ui, |ui| {
                ui.heading(RichText::new("Ready").color(INK));
                ui.label(RichText::new("The NEO is initialized.").color(MUTED));
                ui.add_space(16.0);
                let button_width = if compact { ui.available_width() } else { 300.0 };
                if ui
                    .add_sized(
                        [button_width, 52.0],
                        egui::Button::new(RichText::new("Files on device").strong()).fill(ACCENT),
                    )
                    .clicked()
                    && let Err(error) = self.app.open_files()
                {
                    self.app.set_error(error);
                }
            });
    }

    fn files_view(&mut self, ui: &mut egui::Ui, compact: bool) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Files on device").color(INK));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_enabled(!self.app.is_downloading(), egui::Button::new("Back"))
                    .clicked()
                {
                    self.app.screen = Screen::MainMenu;
                }
            });
        });
        ui.label(RichText::new("Backups are saved as length-validated .txt files.").color(MUTED));
        ui.add_space(6.0);

        egui::Frame::new()
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(8.0)
            .show(ui, |ui| {
                let files = self.app.files.clone();
                egui::ScrollArea::vertical()
                    .max_height(if compact { 430.0 } else { 480.0 })
                    .show(ui, |ui| {
                        for (index, entry) in files.iter().enumerate() {
                            if file_row(ui, self.selected_row == index, entry, compact).clicked() {
                                self.selected_row = index;
                            }
                            ui.separator();
                        }
                        let all_index = self.app.files.len();
                        if all_files_row(ui, self.selected_row == all_index, compact).clicked() {
                            self.selected_row = all_index;
                        }
                    });
            });

        ui.add_space(12.0);
        if compact {
            let disabled = self.app.is_downloading();
            if ui
                .add_enabled_ui(!disabled, |ui| {
                    ui.add_sized(
                        [ui.available_width(), 50.0],
                        egui::Button::new(RichText::new("Download selected").strong()).fill(ACCENT),
                    )
                })
                .inner
                .clicked()
            {
                self.start_selected_backup();
            }
            ui.label(RichText::new("~/alpha-cli/backups/{date-time}/").color(MUTED));
        } else {
            ui.horizontal(|ui| {
                let disabled = self.app.is_downloading();
                if ui
                    .add_enabled(
                        !disabled,
                        egui::Button::new(RichText::new("Download selected").strong()).fill(ACCENT),
                    )
                    .clicked()
                {
                    self.start_selected_backup();
                }
                ui.label(
                    RichText::new("Destination: ~/alpha-cli/backups/{date-time}/").color(MUTED),
                );
            });
        }
    }

    fn progress_window(&self, ctx: &egui::Context) {
        let Some(progress) = &self.app.download else {
            return;
        };
        let width = dialog_width(ctx, 360.0);
        egui::Window::new("Downloading")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.heading(RichText::new(&progress.message).color(INK));
                    ui.label(
                        RichText::new(format!("File {} of {}", progress.current, progress.total))
                            .color(MUTED),
                    );
                });
            });
    }

    fn error_window(&mut self, ctx: &egui::Context) {
        let Some(message) = self.app.error.clone() else {
            return;
        };
        let width = dialog_width(ctx, 460.0);
        egui::Window::new("Problem")
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.colored_label(Color32::from_rgb(180, 30, 30), message);
                ui.add_space(8.0);
                ui.label("Details were written to ~/alpha-cli/logs/alpha-gui.log");
                if ui.button("Dismiss").clicked() {
                    self.app.error = None;
                    self.app.status = "Ready.".to_owned();
                }
            });
    }

    fn start_selected_backup(&mut self) {
        self.app.file_selected = self.selected_row;
        if let Err(error) = self.app.start_backup_selected() {
            self.app.set_error(error);
        }
    }
}

fn header(ui: &mut egui::Ui, compact: bool) {
    if compact {
        ui.vertical(|ui| {
            ui.label(RichText::new("Alpha GUI").size(28.0).strong().color(INK));
            ui.label(
                RichText::new("AlphaSmart NEO backup")
                    .size(15.0)
                    .color(MUTED),
            );
        });
    } else {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Alpha GUI").size(30.0).strong().color(INK));
            ui.label(
                RichText::new("AlphaSmart NEO backup")
                    .size(16.0)
                    .color(MUTED),
            );
        });
    }
    ui.add_space(8.0);
    ui.separator();
}

fn panel() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(8.0)
}

fn step_row(ui: &mut egui::Ui, number: &str, text: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(number)
                .strong()
                .color(Color32::WHITE)
                .background_color(ACCENT),
        );
        ui.add(egui::Label::new(RichText::new(text).color(INK)).wrap());
    });
}

fn file_row(
    ui: &mut egui::Ui,
    selected: bool,
    entry: &crate::protocol::FileEntry,
    compact: bool,
) -> egui::Response {
    let title = format!("Slot {}  {}", entry.slot, entry.name);
    let size = app::human_bytes(entry.attribute_bytes);
    let words = app::approximate_words_from_bytes(entry.attribute_bytes);
    flat_row_frame(selected)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if compact {
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(RichText::new(title).strong().color(INK)).wrap());
                    ui.label(RichText::new(format!("{size}  ~{words} words")).color(MUTED));
                });
            } else {
                ui.horizontal(|ui| {
                    let metadata_width = 210.0_f32.min(ui.available_width() * 0.42);
                    let title_width = (ui.available_width() - metadata_width - 12.0).max(180.0);
                    ui.add_sized(
                        [title_width, 22.0],
                        egui::Label::new(RichText::new(title).strong().color(INK)).truncate(),
                    );
                    ui.allocate_ui_with_layout(
                        Vec2::new(metadata_width, 22.0),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            ui.label(RichText::new(format!("~{words} words")).color(MUTED));
                            ui.label(RichText::new(size).color(ACCENT_DARK).strong());
                        },
                    );
                });
            }
        })
        .response
        .interact(egui::Sense::click())
}

fn all_files_row(ui: &mut egui::Ui, selected: bool, compact: bool) -> egui::Response {
    flat_row_frame(selected)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if compact {
                ui.vertical(|ui| {
                    ui.label(RichText::new("All files").strong().color(INK));
                    ui.label(RichText::new("Download every AlphaWord slot").color(MUTED));
                });
            } else {
                ui.horizontal(|ui| {
                    let metadata_width = 280.0_f32.min(ui.available_width() * 0.55);
                    let title_width = (ui.available_width() - metadata_width - 12.0).max(180.0);
                    ui.add_sized(
                        [title_width, 22.0],
                        egui::Label::new(RichText::new("All files").strong().color(INK)),
                    );
                    ui.allocate_ui_with_layout(
                        Vec2::new(metadata_width, 22.0),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            ui.label(RichText::new("Download every AlphaWord slot").color(MUTED));
                        },
                    );
                });
            }
        })
        .response
        .interact(egui::Sense::click())
}

fn flat_row_frame(selected: bool) -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 12))
        .stroke(if selected {
            Stroke::new(1.0, ACCENT)
        } else {
            Stroke::NONE
        })
        .fill(if selected {
            ROW_SELECTED
        } else {
            Color32::WHITE
        })
        .corner_radius(0.0)
}

fn status_bar(ui: &mut egui::Ui, status: &str, compact: bool) {
    panel()
        .inner_margin(if compact { 12.0 } else { 14.0 })
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Status").strong().color(ACCENT_DARK));
                ui.label(RichText::new(status).color(MUTED));
            });
        });
}

fn dialog_width(ctx: &egui::Context, preferred: f32) -> f32 {
    (ctx.content_rect().width() - 32.0).clamp(260.0, preferred)
}
