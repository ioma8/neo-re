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
            .with_inner_size(Vec2::new(900.0, 640.0))
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
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 9.0);
    style.visuals.window_corner_radius = 6.0.into();
    style.visuals.widgets.active.bg_fill = ACCENT;
    style.visuals.widgets.hovered.bg_fill = SOFT_BLUE;
    ctx.set_global_style(style);
}

const ACCENT: Color32 = Color32::from_rgb(18, 105, 185);
const ACCENT_DARK: Color32 = Color32::from_rgb(23, 73, 118);
const INK: Color32 = Color32::from_rgb(28, 36, 46);
const MUTED: Color32 = Color32::from_rgb(96, 106, 119);
const LINE: Color32 = Color32::from_rgb(216, 222, 230);
const SOFT_BLUE: Color32 = Color32::from_rgb(232, 242, 252);
const ROW_SELECTED: Color32 = Color32::from_rgb(228, 240, 251);
const SURFACE: Color32 = Color32::from_rgb(250, 251, 253);
const ROW_ALT: Color32 = Color32::from_rgb(247, 249, 251);

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
            .fill(SURFACE)
            .show(ui, |ui| {
                let height = ui.available_height();
                ui.set_min_height(height);
                header(ui, compact);
                ui.add_space(if compact { 10.0 } else { 14.0 });

                let footer_height = if self.app.screen == Screen::Files {
                    if compact { 126.0 } else { 78.0 }
                } else if compact {
                    58.0
                } else {
                    42.0
                };
                let body_height = (ui.available_height() - footer_height).max(180.0);
                ui.allocate_ui(
                    Vec2::new(ui.available_width(), body_height),
                    |ui| match self.app.screen {
                        Screen::Waiting => self.waiting_view(ui, compact),
                        Screen::MainMenu => self.main_menu(ui, compact),
                        Screen::Files => self.files_view(ui, compact),
                    },
                );

                ui.add_space(10.0);
                footer(ui, compact, &self.app.status, self.app.screen);
                self.footer_actions(ui, compact);
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
        ui.vertical_centered(|ui| {
            ui.add_space(if compact { 28.0 } else { 60.0 });
            ui.spinner();
            ui.add_space(16.0);
            ui.heading(RichText::new("Connect your AlphaSmart NEO").color(INK));
            ui.label(RichText::new("Leave it in normal USB keyboard mode.").color(MUTED));
            ui.add_space(18.0);
            compact_steps(ui);
        });
    }

    fn main_menu(&mut self, ui: &mut egui::Ui, compact: bool) {
        let top_space = if compact { 42.0 } else { 96.0 };
        ui.add_space(top_space);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Ready").size(30.0).strong().color(INK));
            ui.label(RichText::new("NEO initialized").size(15.0).color(MUTED));
            ui.add_space(24.0);
            let button_width = if compact {
                ui.available_width().min(360.0)
            } else {
                340.0
            };
            let response = ui.add_sized(
                [button_width, 48.0],
                egui::Button::new(RichText::new("Files on device").strong()).fill(ACCENT),
            );
            if response.clicked()
                && let Err(error) = self.app.open_files()
            {
                self.app.set_error(error);
            }
        });
    }

    fn files_view(&mut self, ui: &mut egui::Ui, compact: bool) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Files").color(INK));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_enabled(!self.app.is_downloading(), egui::Button::new("Back"))
                    .clicked()
                {
                    self.app.screen = Screen::MainMenu;
                }
            });
        });

        table_header(ui, compact);
        let list_height = (ui.available_height() - 10.0).max(160.0);
        egui::Frame::new()
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(6.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(list_height)
                    .show(ui, |ui| {
                        let files = self.app.files.clone();
                        for (index, entry) in files.iter().enumerate() {
                            let selected = self.selected_row == index;
                            if file_row(ui, selected, index, entry, compact).clicked() {
                                self.selected_row = index;
                            }
                        }
                        let all_index = self.app.files.len();
                        if all_files_row(ui, self.selected_row == all_index, compact).clicked() {
                            self.selected_row = all_index;
                        }
                    });
            });
    }

    fn footer_actions(&mut self, ui: &mut egui::Ui, compact: bool) {
        if self.app.screen != Screen::Files {
            return;
        }
        let disabled = self.app.is_downloading();
        let selection = selected_label(&self.app.files, self.selected_row);
        if compact {
            ui.label(RichText::new(selection).color(MUTED));
            if ui
                .add_enabled_ui(!disabled, |ui| {
                    ui.add_sized(
                        [ui.available_width(), 48.0],
                        egui::Button::new(RichText::new("Download selected").strong()).fill(ACCENT),
                    )
                })
                .inner
                .clicked()
            {
                self.start_selected_backup();
            }
        } else {
            ui.horizontal(|ui| {
                ui.label(RichText::new(selection).color(MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_enabled(
                            !disabled,
                            egui::Button::new(RichText::new("Download selected").strong())
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.start_selected_backup();
                    }
                });
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
            ui.label(RichText::new("Alpha GUI").size(26.0).strong().color(INK));
            ui.label(
                RichText::new("AlphaSmart NEO backup")
                    .size(14.0)
                    .color(MUTED),
            );
        });
    } else {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Alpha GUI").size(28.0).strong().color(INK));
            ui.label(
                RichText::new("AlphaSmart NEO backup")
                    .size(15.0)
                    .color(MUTED),
            );
        });
    }
    ui.add_space(8.0);
    ui.separator();
}

fn compact_steps(ui: &mut egui::Ui) {
    for text in [
        "Plug in the NEO by USB.",
        "Keep it in keyboard mode.",
        "Alpha GUI switches it to direct mode.",
    ] {
        ui.label(RichText::new(text).color(MUTED));
    }
}

fn file_row(
    ui: &mut egui::Ui,
    selected: bool,
    index: usize,
    entry: &crate::protocol::FileEntry,
    compact: bool,
) -> egui::Response {
    let slot = format!("{}", entry.slot);
    let size = app::human_bytes(entry.attribute_bytes);
    let words = app::approximate_words_from_bytes(entry.attribute_bytes);
    row_frame(selected, index)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if compact {
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(RichText::new(&entry.name).strong().color(INK)).wrap());
                    ui.label(RichText::new(format!("{size}  ~{words} words")).color(MUTED));
                });
            } else {
                table_row(
                    ui,
                    RichText::new(slot).color(MUTED),
                    RichText::new(&entry.name).strong().color(INK),
                    RichText::new(size).color(MUTED),
                    RichText::new(format!("~{words}")).color(MUTED),
                );
            }
        })
        .response
        .interact(egui::Sense::click())
}

fn all_files_row(ui: &mut egui::Ui, selected: bool, compact: bool) -> egui::Response {
    row_frame(selected, 99)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if compact {
                ui.vertical(|ui| {
                    ui.label(RichText::new("All files").strong().color(INK));
                    ui.label(RichText::new("Download every AlphaWord slot").color(MUTED));
                });
            } else {
                table_row(
                    ui,
                    RichText::new("").color(MUTED),
                    RichText::new("All files").strong().color(INK),
                    RichText::new("").color(MUTED),
                    RichText::new("all slots").color(MUTED),
                );
            }
        })
        .response
        .interact(egui::Sense::click())
}

fn table_header(ui: &mut egui::Ui, compact: bool) {
    if compact {
        return;
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_sized(
            [52.0, 20.0],
            egui::Label::new(RichText::new("Slot").color(MUTED)),
        );
        let file_width = (ui.available_width() - 216.0).max(220.0);
        ui.add_sized(
            [file_width, 20.0],
            egui::Label::new(RichText::new("File").color(MUTED)),
        );
        ui.add_sized(
            [96.0, 20.0],
            egui::Label::new(RichText::new("Size").color(MUTED)),
        );
        ui.add_sized(
            [96.0, 20.0],
            egui::Label::new(RichText::new("Words").color(MUTED)),
        );
    });
    ui.add_space(2.0);
}

fn table_row(ui: &mut egui::Ui, slot: RichText, name: RichText, size: RichText, words: RichText) {
    ui.horizontal(|ui| {
        ui.add_sized([52.0, 20.0], egui::Label::new(slot));
        let file_width = (ui.available_width() - 216.0).max(220.0);
        ui.add_sized([file_width, 20.0], egui::Label::new(name).truncate());
        ui.add_sized([96.0, 20.0], egui::Label::new(size));
        ui.add_sized([96.0, 20.0], egui::Label::new(words));
    });
}

fn row_frame(selected: bool, index: usize) -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 7))
        .stroke(if selected {
            Stroke::new(1.0, ACCENT)
        } else {
            Stroke::NONE
        })
        .fill(match (selected, index.is_multiple_of(2)) {
            (true, _) => ROW_SELECTED,
            (false, true) => Color32::WHITE,
            (false, false) => ROW_ALT,
        })
}

fn footer(ui: &mut egui::Ui, compact: bool, status: &str, screen: Screen) {
    ui.separator();
    if screen == Screen::Files {
        ui.label(RichText::new("Select a file to back it up, or choose All files.").color(MUTED));
    } else if compact {
        ui.vertical(|ui| {
            ui.label(RichText::new(status).color(MUTED));
        });
    } else {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Status").strong().color(ACCENT_DARK));
            ui.label(RichText::new(status).color(MUTED));
        });
    }
}

fn dialog_width(ctx: &egui::Context, preferred: f32) -> f32 {
    (ctx.content_rect().width() - 32.0).clamp(260.0, preferred)
}

fn selected_label(files: &[crate::protocol::FileEntry], selected_row: usize) -> String {
    if selected_row >= files.len() {
        return "Selected: all files".to_owned();
    }
    let entry = &files[selected_row];
    format!(
        "Selected: {} · {} · ~{} words",
        entry.name,
        app::human_bytes(entry.attribute_bytes),
        app::approximate_words_from_bytes(entry.attribute_bytes)
    )
}
