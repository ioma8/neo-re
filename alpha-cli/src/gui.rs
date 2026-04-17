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

                let footer_height = if compact { 102.0 } else { 62.0 };
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
        let max_width = if compact { ui.available_width() } else { 420.0 };
        ui.add_space(if compact { 20.0 } else { 54.0 });
        ui.allocate_ui(Vec2::new(max_width, 150.0), |ui| {
            ui.heading(RichText::new("Ready").color(INK));
            ui.label(RichText::new("NEO initialized. Choose an action.").color(MUTED));
            ui.add_space(18.0);
            let button_width = if compact { ui.available_width() } else { 260.0 };
            if ui
                .add_sized(
                    [button_width, 46.0],
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
    }

    fn footer_actions(&mut self, ui: &mut egui::Ui, compact: bool) {
        if self.app.screen != Screen::Files {
            return;
        }
        let disabled = self.app.is_downloading();
        if compact {
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
                if ui
                    .add_enabled(
                        !disabled,
                        egui::Button::new(RichText::new("Download selected").strong()).fill(ACCENT),
                    )
                    .clicked()
                {
                    self.start_selected_backup();
                }
                ui.label(RichText::new("~/alpha-cli/backups/{date-time}/").color(MUTED));
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
    entry: &crate::protocol::FileEntry,
    compact: bool,
) -> egui::Response {
    let title = format!("Slot {}  {}", entry.slot, entry.name);
    let size = app::human_bytes(entry.attribute_bytes);
    let words = app::approximate_words_from_bytes(entry.attribute_bytes);
    row_frame(selected)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if compact {
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(RichText::new(title).strong().color(INK)).wrap());
                    ui.label(RichText::new(format!("{size}  ~{words} words")).color(MUTED));
                });
            } else {
                two_column_row(
                    ui,
                    RichText::new(title).strong().color(INK),
                    RichText::new(format!("{size}    ~{words} words")).color(MUTED),
                );
            }
        })
        .response
        .interact(egui::Sense::click())
}

fn all_files_row(ui: &mut egui::Ui, selected: bool, compact: bool) -> egui::Response {
    row_frame(selected)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if compact {
                ui.vertical(|ui| {
                    ui.label(RichText::new("All files").strong().color(INK));
                    ui.label(RichText::new("Download every AlphaWord slot").color(MUTED));
                });
            } else {
                two_column_row(
                    ui,
                    RichText::new("All files").strong().color(INK),
                    RichText::new("Download every AlphaWord slot").color(MUTED),
                );
            }
        })
        .response
        .interact(egui::Sense::click())
}

fn two_column_row(ui: &mut egui::Ui, left: RichText, right: RichText) {
    ui.horizontal(|ui| {
        let right_width = 250.0_f32.min(ui.available_width() * 0.45);
        let left_width = (ui.available_width() - right_width - 12.0).max(160.0);
        ui.add_sized([left_width, 22.0], egui::Label::new(left).truncate());
        ui.allocate_ui_with_layout(
            Vec2::new(right_width, 22.0),
            Layout::right_to_left(Align::Center),
            |ui| {
                ui.label(right);
            },
        );
    });
}

fn row_frame(selected: bool) -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 10))
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
}

fn footer(ui: &mut egui::Ui, compact: bool, status: &str, screen: Screen) {
    ui.separator();
    if compact || screen == Screen::Files {
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
