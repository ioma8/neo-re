use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, anyhow, bail};
use eframe::egui::{self, Align, Color32, Layout, RichText, Stroke, Vec2};
#[cfg(not(target_os = "android"))]
use tracing_subscriber::fmt;
use tracing_subscriber::{filter::LevelFilter, prelude::*};

use crate::neo_client::NeoClientProgress;
use crate::{
    applet_workflow::{AppletChecklist, AppletFlashPlan, AppletSourceKind},
    backup,
    bundled_assets::{BundledAppletKind, BundledCatalog, BundledOsImageKind, BundledSource},
    gui_about, gui_applets, gui_connection, gui_dashboard,
    gui_model::{
        ConnectionGate, DeviceMode, MainTab, PlatformCapabilities,
        debug_connection_bypass_available,
    },
    gui_os,
    operation_progress::{OperationProgress, ProgressEvent},
    protocol::{FileEntry, SmartAppletRecord},
    usb::{self, NeoClient, NeoMode},
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
            .with_inner_size(Vec2::new(1320.0, 900.0))
            .with_min_inner_size(Vec2::new(1024.0, 720.0)),
        ..Default::default()
    }
}

fn init_logging() -> anyhow::Result<()> {
    #[cfg(target_os = "android")]
    {
        tracing_subscriber::registry()
            .with(LevelFilter::INFO)
            .with(tracing_android::layer("AlphaGUI").context("initialize logcat tracing")?)
            .try_init()
            .context("initialize Android GUI tracing subscriber")?;
        Ok(())
    }

    #[cfg(not(target_os = "android"))]
    {
        let log_dir = backup::app_dir()?.join("logs");
        std::fs::create_dir_all(&log_dir).context("create log directory")?;
        let log_file =
            std::fs::File::create(log_dir.join("alpha-gui.log")).context("create GUI log file")?;
        tracing_subscriber::registry()
            .with(LevelFilter::INFO)
            .with(fmt::layer().with_writer(log_file).with_ansi(false))
            .try_init()
            .context("initialize GUI tracing subscriber")?;
        Ok(())
    }
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = Vec2::new(12.0, 12.0);
    style.spacing.button_padding = Vec2::new(16.0, 10.0);
    style.spacing.interact_size = Vec2::new(40.0, 30.0);
    style.visuals.window_corner_radius = 16.0.into();
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.panel_fill = SURFACE;
    style.visuals.extreme_bg_color = SURFACE;
    style.visuals.widgets.inactive.bg_fill = CONTROL;
    style.visuals.widgets.inactive.weak_bg_fill = CONTROL;
    style.visuals.widgets.hovered.bg_fill = CONTROL_HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = CONTROL_HOVER;
    style.visuals.widgets.active.bg_fill = ACCENT;
    style.visuals.widgets.active.weak_bg_fill = ACCENT_SOFT;
    style.visuals.widgets.noninteractive.bg_fill = SURFACE;
    ctx.set_global_style(style);
}

const ACCENT: Color32 = Color32::from_rgb(0, 122, 255);
const ACCENT_DARK: Color32 = Color32::from_rgb(0, 87, 184);
const ACCENT_SOFT: Color32 = Color32::from_rgb(222, 239, 255);
const DANGER: Color32 = Color32::from_rgb(196, 56, 39);
const DANGER_SOFT: Color32 = Color32::from_rgb(251, 234, 231);
const SUCCESS: Color32 = Color32::from_rgb(20, 138, 74);
const SUCCESS_SOFT: Color32 = Color32::from_rgb(228, 246, 236);
const CONTROL: Color32 = Color32::from_rgb(238, 241, 246);
const CONTROL_HOVER: Color32 = Color32::from_rgb(228, 233, 240);
const INK: Color32 = Color32::from_rgb(24, 28, 36);
const MUTED: Color32 = Color32::from_rgb(95, 103, 116);
const LINE: Color32 = Color32::from_rgb(223, 228, 236);
const SURFACE: Color32 = Color32::from_rgb(245, 247, 250);
const SURFACE_STRONG: Color32 = Color32::from_rgb(255, 255, 255);
const ROW_ALT: Color32 = Color32::from_rgb(249, 250, 252);
const MOBILE_ACTION_HEIGHT: f32 = 48.0;
const NAV_WIDTH: f32 = 248.0;

struct AlphaGui {
    last_mode_poll: Instant,
    mode: DeviceModeState,
    files: Vec<FileEntry>,
    applets: Vec<SmartAppletRecord>,
    status: String,
    error: Option<String>,
    logs: Vec<String>,
    selected_slot: usize,
    selected_applet: usize,
    selected_tab: MainTab,
    catalog: BundledCatalog,
    applet_checks: BTreeMap<String, bool>,
    added_applets: Vec<PathBuf>,
    custom_applet_path: String,
    auto_switch_attempted: bool,
    #[cfg(debug_assertions)]
    debug_connection_bypass: bool,
    confirmation: Option<PendingConfirmation>,
    operation_progress: Option<OperationProgress>,
    last_operation_summary: Option<String>,
    task: Option<RunningTask>,
}

struct RunningTask {
    title: String,
    started_at: Instant,
    receiver: Receiver<TaskEvent>,
}

enum TaskEvent {
    Log(String),
    Status(String),
    Progress(ProgressEvent),
    Inventory {
        files: Vec<FileEntry>,
        applets: Vec<SmartAppletRecord>,
        note: String,
    },
    Finished {
        note: String,
    },
    Failed {
        message: String,
    },
}

#[derive(Clone, Debug)]
enum PendingConfirmation {
    AppletClearReinstall {
        applets: Vec<PathBuf>,
    },
    FlashOs {
        image: PathBuf,
        reformat_rest_of_rom: bool,
        title: String,
    },
}

impl From<DeviceModeState> for DeviceMode {
    fn from(value: DeviceModeState) -> Self {
        match value {
            DeviceModeState::Missing => Self::Missing,
            DeviceModeState::Hid => Self::Hid,
            DeviceModeState::HidUnavailable => Self::HidUnavailable,
            DeviceModeState::Direct => Self::Direct,
            DeviceModeState::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeviceModeState {
    Missing,
    Hid,
    HidUnavailable,
    Direct,
    Unknown,
}

impl DeviceModeState {
    fn badge(self) -> (&'static str, Color32) {
        match self {
            Self::Missing => ("No device", MUTED),
            Self::Hid => ("Keyboard mode", ACCENT_DARK),
            Self::HidUnavailable => ("HID unavailable", DANGER),
            Self::Direct => ("Direct USB", SUCCESS),
            Self::Unknown => ("Checking", MUTED),
        }
    }
}

impl Default for AlphaGui {
    fn default() -> Self {
        Self {
            last_mode_poll: Instant::now() - Duration::from_secs(5),
            mode: DeviceModeState::Unknown,
            files: Vec::new(),
            applets: Vec::new(),
            status: "Alpha GUI ready. Refresh inventory when the NEO is connected.".to_owned(),
            error: None,
            logs: vec!["Alpha GUI initialized.".to_owned()],
            selected_slot: 0,
            selected_applet: 0,
            selected_tab: MainTab::Dashboard,
            catalog: BundledCatalog::dev_defaults(),
            applet_checks: BTreeMap::new(),
            added_applets: Vec::new(),
            custom_applet_path: String::new(),
            auto_switch_attempted: false,
            #[cfg(debug_assertions)]
            debug_connection_bypass: false,
            confirmation: None,
            operation_progress: None,
            last_operation_summary: None,
            task: None,
        }
    }
}

impl eframe::App for AlphaGui {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_mode();
        self.drain_task_events();
        ctx.request_repaint_after(Duration::from_millis(120));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let compact = self.is_compact(ui);
        let margin = if compact { 14 } else { 24 };
        let top_margin = margin + android_status_bar_padding();

        let root_frame = egui::Frame::central_panel(ui.style())
            .inner_margin(egui::Margin {
                left: margin,
                right: margin,
                top: top_margin,
                bottom: margin,
            })
            .fill(SURFACE);

        egui::CentralPanel::default()
            .frame(root_frame)
            .show_inside(ui, |ui| {
                if self.connection_gate().tabs_visible() {
                    self.app_shell(ui, compact);
                } else {
                    self.connection_screen(ui, compact);
                }
            });

        if self.error.is_some() {
            self.error_window(ui.ctx());
        }
        if self.confirmation.is_some() {
            self.confirmation_window(ui.ctx());
        }
        if self.task.is_some() {
            self.progress_window(ui.ctx());
        }
    }
}

impl AlphaGui {
    fn is_compact(&self, ui: &egui::Ui) -> bool {
        ui.available_width() < 860.0 || PlatformCapabilities::current().mobile
    }

    fn connection_gate(&self) -> ConnectionGate {
        #[cfg(debug_assertions)]
        if self.debug_connection_bypass {
            return ConnectionGate::from_mode(DeviceMode::Direct, PlatformCapabilities::current());
        }

        ConnectionGate::from_mode(self.mode.into(), PlatformCapabilities::current())
    }

    fn app_shell(&mut self, ui: &mut egui::Ui, compact: bool) {
        if compact {
            self.mobile_app_shell(ui);
        } else {
            ui.horizontal(|ui| {
                self.desktop_nav(ui);
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width((ui.available_width() - 16.0).max(480.0));
                    self.tab_content(ui);
                });
            });
        }
    }

    fn mobile_app_shell(&mut self, ui: &mut egui::Ui) {
        self.mobile_top_bar(ui);
        ui.add_space(12.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.tab_content(ui);
            ui.add_space(72.0);
        });
        self.mobile_bottom_nav(ui);
    }

    fn tab_content(&mut self, ui: &mut egui::Ui) {
        if let Some(summary) = &self.last_operation_summary {
            info_strip(ui, SUCCESS_SOFT, SUCCESS, summary);
            ui.add_space(10.0);
        }
        match self.selected_tab {
            MainTab::Dashboard => self.dashboard_tab(ui),
            MainTab::SmartApplets => self.smartapplets_tab(ui),
            MainTab::OsOperations => self.os_operations_tab(ui),
            MainTab::About => self.about_tab(ui),
        }
    }

    fn connection_screen(&mut self, ui: &mut egui::Ui, compact: bool) {
        let gate = self.connection_gate();
        ui.vertical_centered(|ui| {
            ui.add_space(if compact { 28.0 } else { 80.0 });
            egui::Frame::new()
                .fill(SURFACE_STRONG)
                .stroke(Stroke::new(1.0, LINE))
                .corner_radius(8.0)
                .inner_margin(egui::Margin::same(if compact { 18 } else { 28 }))
                .show(ui, |ui| {
                    ui.set_max_width(if compact { ui.available_width() } else { 560.0 });
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("⌁").size(64.0).color(ACCENT));
                        ui.label(RichText::new("Awaiting Device").size(26.0).strong().color(INK));
                        ui.label(RichText::new(&gate.message).size(15.0).color(MUTED));
                        ui.add_space(12.0);
                        pill(ui, self.mode.badge().0, self.mode.badge().1, Color32::WHITE);
                    });
                    ui.add_space(16.0);
                    if PlatformCapabilities::current().mobile && self.mode == DeviceModeState::Hid {
                        info_strip(ui, ACCENT_SOFT, ACCENT_DARK, gui_connection::MOBILE_ALPHA_USB_HINT);
                    }
                    if self.mode == DeviceModeState::Hid && PlatformCapabilities::current().can_auto_switch_hid {
                        info_strip(ui, ACCENT_SOFT, ACCENT_DARK, "Desktop detected keyboard mode and will switch to direct USB automatically.");
                    }
                    if let Some(progress) = &self.operation_progress {
                        ui.add_space(12.0);
                        progress_block(ui, progress);
                    }
                    ui.add_space(16.0);
                    if full_width_action_button(ui, "Scan for Device", true).clicked() {
                        self.poll_mode_now();
                    }
                    if self.mode == DeviceModeState::Hid
                        && PlatformCapabilities::current().can_auto_switch_hid
                        && full_width_action_button(ui, "Switch to Direct USB", false).clicked()
                    {
                        self.spawn_task("Switching to direct USB", switch_to_direct_task);
                    }
                    if debug_connection_bypass_available() {
                        ui.add_space(8.0);
                        if full_width_action_button(ui, "Debug: Open UI Without Device", false)
                            .clicked()
                        {
                            self.debug_connection_bypass = true;
                            self.selected_tab = MainTab::Dashboard;
                            self.push_log("Debug connection bypass enabled.");
                        }
                    }
                });
        });
    }

    fn desktop_nav(&mut self, ui: &mut egui::Ui) {
        ui.set_width(NAV_WIDTH);
        ui.vertical(|ui| {
            ui.label(RichText::new("AlphaGUI").size(24.0).strong().color(ACCENT));
            ui.label(RichText::new("NEO Manager").size(13.0).color(MUTED));
            ui.add_space(22.0);
            for tab in MainTab::ALL {
                let selected = self.selected_tab == tab;
                let label = format!("{}  {}", tab.icon(), tab.label());
                let response = ui.add_sized(
                    [ui.available_width(), 42.0],
                    button_widget(
                        &label,
                        if selected { ACCENT_SOFT } else { SURFACE },
                        if selected { ACCENT_DARK } else { MUTED },
                        if selected {
                            Stroke::new(1.0, ACCENT)
                        } else {
                            Stroke::NONE
                        },
                    ),
                );
                if response.clicked() {
                    self.selected_tab = tab;
                }
            }
            ui.add_space(18.0);
            info_strip(
                ui,
                self.mode.badge().1.gamma_multiply(0.14),
                self.mode.badge().1,
                self.mode.badge().0,
            );
        });
    }

    fn mobile_top_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⌁").size(22.0).color(ACCENT));
            ui.label(RichText::new("AlphaGUI").size(20.0).strong().color(ACCENT));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                pill(ui, self.mode.badge().0, self.mode.badge().1, Color32::WHITE);
            });
        });
    }

    fn mobile_bottom_nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        egui::Frame::new()
            .fill(SURFACE_STRONG)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.columns(4, |columns| {
                    for (index, tab) in MainTab::ALL.into_iter().enumerate() {
                        let selected = self.selected_tab == tab;
                        if columns[index]
                            .add_sized(
                                [columns[index].available_width(), 44.0],
                                button_widget(
                                    tab.label(),
                                    if selected {
                                        ACCENT_SOFT
                                    } else {
                                        SURFACE_STRONG
                                    },
                                    if selected { ACCENT_DARK } else { MUTED },
                                    Stroke::NONE,
                                ),
                            )
                            .clicked()
                        {
                            self.selected_tab = tab;
                        }
                    }
                });
            });
    }

    fn dashboard_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Dashboard").size(30.0).strong().color(INK));
                ui.label(
                    RichText::new("Manage files on the connected AlphaSmart NEO.").color(MUTED),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if primary_button(ui, "↧ Backup All Files").clicked() {
                    self.spawn_task("Backing up all files", full_backup_task);
                }
            });
        });
        ui.add_space(18.0);
        egui::Frame::new()
            .fill(SURFACE_STRONG)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Device Files").strong().color(MUTED));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} listed", self.files.len())).color(MUTED),
                        );
                    });
                });
                ui.separator();
                if self.files.is_empty() {
                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("No AlphaWord files loaded.")
                                .strong()
                                .color(INK),
                        );
                        ui.label(RichText::new(gui_dashboard::EMPTY_FILES_HINT).color(MUTED));
                        if secondary_button(ui, "Refresh inventory").clicked() {
                            self.spawn_task("Refreshing inventory", refresh_inventory_task);
                        }
                    });
                    ui.add_space(16.0);
                } else {
                    for file in self.files.clone() {
                        file_row(ui, &file, |ui| {
                            if secondary_button(ui, "Backup").clicked() {
                                self.spawn_task(
                                    format!("Backing up slot {}", file.slot),
                                    move |tx| backup_one_slot_task(tx, file.slot),
                                );
                            }
                        });
                    }
                }
            });
    }

    fn smartapplets_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("SmartApplets").size(30.0).strong().color(INK));
        ui.label(RichText::new("Manage bundled and installed SmartApplets.").color(MUTED));
        ui.add_space(18.0);
        if ui
            .add_sized(
                [ui.available_width(), 54.0],
                button_widget(
                    gui_applets::ALPHA_USB_ACTION,
                    ACCENT,
                    Color32::WHITE,
                    Stroke::NONE,
                ),
            )
            .clicked()
        {
            match self.materialize_alpha_usb_applet() {
                Ok(path) => self.spawn_task("Installing Alpha USB", move |tx| {
                    install_applet_task(tx, path, false)
                }),
                Err(error) => self.set_error(error),
            }
        }
        ui.add_space(16.0);

        let mut checklist = self.current_applet_checklist();
        let plan = checklist.plan();
        egui::Frame::new()
            .fill(SURFACE_STRONG)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Available Applets").strong().color(INK));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} installed", self.applets.len())).color(MUTED),
                        );
                    });
                });
                ui.separator();
                for row in &mut checklist.rows {
                    applet_check_row(ui, row, &mut self.applet_checks);
                    ui.separator();
                }
            });
        ui.add_space(14.0);
        let can_add_custom_applet = PlatformCapabilities::current().can_pick_custom_applet_file;
        if can_add_custom_applet {
            ui.label(RichText::new("Custom applet path").size(12.0).color(MUTED));
            ui.text_edit_singleline(&mut self.custom_applet_path);
        } else {
            info_strip(
                ui,
                ACCENT_SOFT,
                ACCENT_DARK,
                "Custom applet file picking is desktop-only in this build. Bundled applets remain available.",
            );
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    can_add_custom_applet,
                    button_widget("Add new applet from file", CONTROL, INK, Stroke::new(1.0, LINE)),
                )
                .clicked()
            {
                let path = PathBuf::from(self.custom_applet_path.trim());
                if path.extension().and_then(|ext| ext.to_str()) != Some("os3kapp") {
                    self.set_error(anyhow!("custom applet must be a .os3kapp file"));
                } else if !path.exists() {
                    self.set_error(anyhow!("custom applet does not exist: {}", path.display()));
                } else {
                    self.added_applets.push(path.clone());
                    self.applet_checks.insert(format!("file-{}", path.display()), true);
                    self.custom_applet_path.clear();
                }
            }
            let enabled = !matches!(plan, AppletFlashPlan::NoChanges);
            if ui
                .add_enabled(enabled, button_widget("Flash to device", ACCENT, Color32::WHITE, Stroke::NONE))
                .clicked()
            {
                match plan {
                    AppletFlashPlan::NoChanges => {}
                    AppletFlashPlan::InstallOnly { .. } => {
                        let paths = self.selected_new_applet_paths(&checklist);
                        self.spawn_task("Installing selected applets", move |tx| install_applets_task(tx, paths));
                    }
                    AppletFlashPlan::ClearAndReinstall { .. } => {
                        let missing = self.checked_rows_without_source(&checklist);
                        if missing > 0 {
                            self.set_error(anyhow!(
                                "{missing} checked applet(s) have no bundled/custom image and cannot be reinstalled; uncheck them or add matching .os3kapp files first"
                            ));
                        } else {
                            let paths = self.selected_bundled_applet_paths(&checklist);
                            self.confirmation = Some(PendingConfirmation::AppletClearReinstall { applets: paths });
                        }
                    }
                }
            }
        });
    }

    fn os_operations_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("OS Operations")
                .size(30.0)
                .strong()
                .color(INK),
        );
        ui.label(
            RichText::new("Back up the device before firmware or system changes.").color(MUTED),
        );
        ui.add_space(18.0);
        egui::Frame::new()
            .fill(SURFACE_STRONG)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::same(18))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("System Backup")
                                .size(20.0)
                                .strong()
                                .color(INK),
                        );
                        ui.label(
                            RichText::new("Create a complete backup of files and SmartApplets.")
                                .color(MUTED),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if primary_button(ui, "↧ Backup Everything").clicked() {
                            self.spawn_task("Backing up everything", full_backup_task);
                        }
                    });
                });
            });
        ui.add_space(16.0);
        self.os_flash_panel(ui, "Reflash System", BundledOsImageKind::System, false);
        info_strip(
            ui,
            ACCENT_SOFT,
            ACCENT_DARK,
            "Firmware flashing is hidden because this build only bundles a validated NEO system image.",
        );
        ui.add_space(16.0);
        egui::Frame::new()
            .fill(SURFACE_STRONG)
            .stroke(Stroke::new(1.0, LINE))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new("Validated Small ROM Operations").size(18.0).strong().color(INK));
                ui.label(RichText::new("No additional Small ROM mutation commands are exposed until they are backed by proven helpers.").color(MUTED));
                ui.add_space(8.0);
                if secondary_button(ui, "Probe direct USB").clicked() {
                    self.spawn_task("Probing direct USB", probe_task);
                }
                ui.label(RichText::new(gui_os::FLASH_WARNING).color(DANGER));
            });
    }

    fn about_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            egui::Frame::new()
                .fill(SURFACE_STRONG)
                .stroke(Stroke::new(1.0, LINE))
                .corner_radius(8.0)
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⌁").size(42.0).color(ACCENT));
                        ui.vertical(|ui| {
                            ui.label(RichText::new("AlphaGUI").size(32.0).strong().color(INK));
                            ui.label(RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).monospace().color(MUTED));
                        });
                    });
                    ui.add_space(12.0);
                    ui.label(RichText::new(gui_about::PROJECT_SUMMARY).color(MUTED));
                    ui.label(RichText::new("Validated only on AlphaSmart NEO. Neo 2 and other AlphaSmart models are not validated.").color(MUTED));
                    ui.add_space(16.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.hyperlink_to("GitHub Repository", "https://github.com/");
                        ui.hyperlink_to("Documentation", "https://github.com/");
                    });
                });
            ui.add_space(16.0);
            info_strip(
                ui,
                DANGER_SOFT,
                DANGER,
                "Use at your own risk. Flashing firmware, system images, or applet areas can brick the device if interrupted or used incorrectly.",
            );
        });
    }

    fn os_flash_panel(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        image_kind: BundledOsImageKind,
        reformat_rest_of_rom: bool,
    ) {
        egui::Frame::new()
            .fill(SURFACE_STRONG)
            .stroke(Stroke::new(1.0, DANGER_SOFT))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new(title).size(20.0).strong().color(INK));
                ui.label(RichText::new("Uses the bundled validated NEO OS image. Keep USB stable until completion.").color(MUTED));
                ui.add_space(12.0);
                let image = self.catalog.os_image_by_kind(image_kind);
                let enabled = image.is_some_and(|image| bundled_source_available(&image.source));
                if ui
                    .add_enabled(
                        enabled,
                        button_widget(title, DANGER_SOFT, DANGER, Stroke::new(1.0, DANGER)),
                    )
                    .clicked()
                    && let Some(image) = image
                {
                    match materialize_bundled_source(&image.source) {
                        Ok(image) => {
                            self.confirmation = Some(PendingConfirmation::FlashOs {
                                image,
                                reformat_rest_of_rom,
                                title: title.to_owned(),
                            });
                        }
                        Err(error) => self.set_error(error),
                    }
                }
                if !enabled {
                    ui.label(RichText::new("Bundled image is missing in this checkout.").color(DANGER));
                }
            });
    }

    fn current_applet_checklist(&self) -> AppletChecklist {
        let mut checklist =
            AppletChecklist::from_installed_and_bundled(&self.applets, &self.catalog.applets);
        for path in &self.added_applets {
            let key = format!("file-{}", path.display());
            checklist
                .rows
                .push(crate::applet_workflow::AppletChecklistRow {
                    key: key.clone(),
                    display_name: path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Custom Applet")
                        .to_owned(),
                    version: None,
                    size: path.metadata().ok().map(|metadata| metadata.len()),
                    installed: false,
                    checked: true,
                    source: AppletSourceKind::AddedFromFile {
                        path: path.display().to_string(),
                    },
                });
        }
        for row in &mut checklist.rows {
            if let Some(checked) = self.applet_checks.get(&row.key) {
                row.checked = *checked;
            }
        }
        checklist
    }

    fn selected_new_applet_paths(&self, checklist: &AppletChecklist) -> Vec<PathBuf> {
        checklist
            .rows
            .iter()
            .filter(|row| row.checked && !row.installed)
            .filter_map(|row| self.path_for_applet_row(row))
            .collect()
    }

    fn selected_bundled_applet_paths(&self, checklist: &AppletChecklist) -> Vec<PathBuf> {
        checklist
            .rows
            .iter()
            .filter(|row| row.checked)
            .filter_map(|row| self.path_for_applet_row(row))
            .collect()
    }

    fn materialize_alpha_usb_applet(&self) -> anyhow::Result<PathBuf> {
        let applet = self
            .catalog
            .applets
            .iter()
            .find(|applet| applet.kind == BundledAppletKind::AlphaUsb)
            .context("bundled Alpha USB applet is not available")?;
        materialize_bundled_source(&applet.source)
    }

    fn checked_rows_without_source(&self, checklist: &AppletChecklist) -> usize {
        checklist
            .rows
            .iter()
            .filter(|row| row.checked && !self.applet_row_has_available_source(row))
            .count()
    }

    fn applet_row_has_available_source(
        &self,
        row: &crate::applet_workflow::AppletChecklistRow,
    ) -> bool {
        match &row.source {
            AppletSourceKind::Bundled { id } => self
                .catalog
                .applets
                .iter()
                .find(|applet| &applet.id == id)
                .is_some_and(|applet| bundled_source_available(&applet.source)),
            AppletSourceKind::AddedFromFile { path } => PathBuf::from(path).exists(),
            AppletSourceKind::InstalledOnly { .. } => false,
        }
    }

    fn path_for_applet_row(
        &self,
        row: &crate::applet_workflow::AppletChecklistRow,
    ) -> Option<PathBuf> {
        match &row.source {
            AppletSourceKind::Bundled { id } => self
                .catalog
                .applets
                .iter()
                .find(|applet| &applet.id == id)
                .and_then(|applet| materialize_bundled_source(&applet.source).ok()),
            AppletSourceKind::AddedFromFile { path } => Some(PathBuf::from(path)),
            AppletSourceKind::InstalledOnly { .. } => None,
        }
    }

    fn progress_window(&self, ctx: &egui::Context) {
        let Some(task) = &self.task else {
            return;
        };
        let compact = is_phone_width(ctx.content_rect().width());
        egui::Window::new(&task.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(dialog_width(ctx, if compact { 340.0 } else { 420.0 }));
                if let Some(progress) = &self.operation_progress {
                    progress_block(ui, progress);
                } else {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&task.title).strong().color(INK));
                            ui.label(
                                RichText::new(format!(
                                    "Running for {:.1}s",
                                    task.started_at.elapsed().as_secs_f32()
                                ))
                                .color(MUTED),
                            );
                        });
                    });
                }
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!(
                        "Running for {:.1}s",
                        task.started_at.elapsed().as_secs_f32()
                    ))
                    .color(MUTED),
                );
                if let Some(last) = self.logs.last() {
                    ui.label(RichText::new(last).color(MUTED));
                }
            });
    }

    fn error_window(&mut self, ctx: &egui::Context) {
        let Some(message) = self.error.clone() else {
            return;
        };
        let compact = is_phone_width(ctx.content_rect().width());
        egui::Window::new("Problem")
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(dialog_width(ctx, if compact { 360.0 } else { 560.0 }));
                ui.colored_label(DANGER, message);
                ui.add_space(10.0);
                ui.label("The full error was also written to ~/alpha-cli/logs/alpha-gui.log");
                if ui
                    .add_sized(
                        [if compact { ui.available_width() } else { 120.0 }, 40.0],
                        button_widget("Dismiss", CONTROL, INK, Stroke::new(1.0, LINE)),
                    )
                    .clicked()
                {
                    self.error = None;
                }
            });
    }

    fn confirmation_window(&mut self, ctx: &egui::Context) {
        let Some(request) = self.confirmation.clone() else {
            return;
        };
        let compact = is_phone_width(ctx.content_rect().width());
        let (title, message, confirm_label) = match &request {
            PendingConfirmation::AppletClearReinstall { applets } => (
                "Reflash SmartApplets".to_owned(),
                format!(
                    "This will clear the SmartApplet area and reinstall {} checked bundled applets.",
                    applets.len()
                ),
                "Clear and Flash".to_owned(),
            ),
            PendingConfirmation::FlashOs { title, image, .. } => (
                title.clone(),
                format!(
                    "This will flash the bundled image {}. Interrupting this can brick the device.",
                    image.display()
                ),
                "Confirm Flash".to_owned(),
            ),
        };
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(dialog_width(ctx, if compact { 360.0 } else { 520.0 }));
                ui.label(RichText::new(message).color(INK));
                ui.add_space(12.0);
                ui.horizontal_wrapped(|ui| {
                    if danger_button(ui, &confirm_label).clicked() {
                        let action = self.confirmation.take();
                        if let Some(action) = action {
                            self.run_confirmation(action);
                        }
                    }
                    if secondary_button(ui, "Cancel").clicked() {
                        self.confirmation = None;
                    }
                });
            });
    }

    fn run_confirmation(&mut self, action: PendingConfirmation) {
        match action {
            PendingConfirmation::AppletClearReinstall { applets } => {
                self.spawn_task("Reflashing SmartApplets", move |tx| {
                    clear_and_install_applets_task(tx, applets)
                });
            }
            PendingConfirmation::FlashOs {
                image,
                reformat_rest_of_rom,
                ..
            } => {
                self.spawn_task("Flashing OS image", move |tx| {
                    flash_os_task(tx, image, reformat_rest_of_rom)
                });
            }
        }
    }

    fn poll_mode(&mut self) {
        if self.last_mode_poll.elapsed() < Duration::from_millis(850) {
            return;
        }
        self.poll_mode_now();
    }

    fn poll_mode_now(&mut self) {
        self.last_mode_poll = Instant::now();
        self.mode = match usb::detect_mode() {
            Ok(NeoMode::Missing) => DeviceModeState::Missing,
            Ok(NeoMode::Hid) => DeviceModeState::Hid,
            Ok(NeoMode::HidUnavailable) => DeviceModeState::HidUnavailable,
            Ok(NeoMode::Direct) => DeviceModeState::Direct,
            Err(error) => {
                self.push_log(format!("mode poll error: {error:#}"));
                DeviceModeState::Unknown
            }
        };
        if self.mode == DeviceModeState::Hid
            && PlatformCapabilities::current().can_auto_switch_hid
            && !self.auto_switch_attempted
            && self.task.is_none()
        {
            self.auto_switch_attempted = true;
            self.spawn_task("Switching to direct USB", switch_to_direct_task);
        }
        if self.mode == DeviceModeState::Direct {
            self.auto_switch_attempted = false;
        }
    }

    fn spawn_task<F>(&mut self, title: impl Into<String>, task_fn: F)
    where
        F: FnOnce(Sender<TaskEvent>) -> anyhow::Result<()> + Send + 'static,
    {
        if self.task.is_some() {
            self.set_error(anyhow!("another operation is already running"));
            return;
        }
        self.error = None;
        let title = title.into();
        self.push_log(format!("{title} started."));
        self.operation_progress = Some(OperationProgress::new(title.clone()));
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            if let Err(error) = task_fn(tx.clone()) {
                let _ = tx.send(TaskEvent::Failed {
                    message: format!("{error:#}"),
                });
            }
        });
        self.task = Some(RunningTask {
            title,
            started_at: Instant::now(),
            receiver: rx,
        });
    }

    fn drain_task_events(&mut self) {
        let Some(task) = self.task.take() else {
            return;
        };
        let mut keep_task = true;
        loop {
            match task.receiver.try_recv() {
                Ok(TaskEvent::Log(line)) => self.push_log(line),
                Ok(TaskEvent::Status(note)) => self.status = note,
                Ok(TaskEvent::Progress(event)) => {
                    if let Some(progress) = &mut self.operation_progress {
                        progress.apply(event);
                    }
                }
                Ok(TaskEvent::Inventory {
                    files,
                    applets,
                    note,
                }) => {
                    self.files = files;
                    self.applets = applets;
                    self.status = note.clone();
                    self.push_log(note);
                    self.last_operation_summary = Some(self.status.clone());
                    self.selected_slot = self.selected_slot.min(self.files.len().saturating_sub(1));
                    self.selected_applet = self
                        .selected_applet
                        .min(self.applets.len().saturating_sub(1));
                    keep_task = false;
                }
                Ok(TaskEvent::Finished { note }) => {
                    self.status = note.clone();
                    self.push_log(note);
                    self.last_operation_summary = Some(self.status.clone());
                    keep_task = false;
                }
                Ok(TaskEvent::Failed { message }) => {
                    self.set_error(anyhow!(message));
                    keep_task = false;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    keep_task = false;
                    break;
                }
            }
        }
        if keep_task {
            self.task = Some(task);
        } else {
            self.operation_progress = None;
        }
    }

    fn set_error(&mut self, error: anyhow::Error) {
        let message = format!("{error:#}");
        self.status = "Last operation failed.".to_owned();
        self.push_log(format!("ERROR: {message}"));
        self.error = Some(message);
    }

    fn push_log(&mut self, line: impl Into<String>) {
        self.logs.push(line.into());
        if self.logs.len() > 200 {
            let overflow = self.logs.len() - 200;
            self.logs.drain(0..overflow);
        }
    }
}

fn android_status_bar_padding() -> i8 {
    #[cfg(target_os = "android")]
    {
        28
    }

    #[cfg(not(target_os = "android"))]
    {
        0
    }
}

fn is_phone_width(width: f32) -> bool {
    width < 560.0
}

fn info_strip(ui: &mut egui::Ui, fill: Color32, ink: Color32, text: &str) {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(12.0)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(ink));
        });
}

fn pill(ui: &mut egui::Ui, text: &str, fill: Color32, ink: Color32) {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(999.0)
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.label(RichText::new(text).strong().color(ink));
        });
}

fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(button_widget(label, ACCENT, Color32::WHITE, Stroke::NONE))
}

fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(button_widget(label, CONTROL, INK, Stroke::new(1.0, LINE)))
}

fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(button_widget(
        label,
        DANGER_SOFT,
        DANGER,
        Stroke::new(1.0, DANGER),
    ))
}

fn full_width_action_button(ui: &mut egui::Ui, label: &str, primary: bool) -> egui::Response {
    let button = if primary {
        button_widget(label, ACCENT, Color32::WHITE, Stroke::NONE)
    } else {
        button_widget(label, CONTROL, INK, Stroke::new(1.0, LINE))
    };
    ui.add_sized([ui.available_width(), MOBILE_ACTION_HEIGHT], button)
}

fn button_widget(label: &str, fill: Color32, text: Color32, stroke: Stroke) -> egui::Button<'_> {
    egui::Button::new(RichText::new(label).strong().color(text))
        .fill(fill)
        .stroke(stroke)
        .corner_radius(12.0)
}

fn file_row(ui: &mut egui::Ui, file: &FileEntry, actions: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(ROW_ALT)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("□").size(22.0).color(MUTED));
                ui.vertical(|ui| {
                    ui.label(RichText::new(&file.name).strong().color(INK));
                    ui.label(
                        RichText::new(format!(
                            "Slot {} · {} bytes",
                            file.slot, file.attribute_bytes
                        ))
                        .size(12.0)
                        .color(MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), actions);
            });
        });
    ui.separator();
}

fn applet_check_row(
    ui: &mut egui::Ui,
    row: &mut crate::applet_workflow::AppletChecklistRow,
    overrides: &mut BTreeMap<String, bool>,
) {
    egui::Frame::new()
        .fill(if row.checked { SURFACE_STRONG } else { ROW_ALT })
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut checked = row.checked;
                if ui.checkbox(&mut checked, "").changed() {
                    row.checked = checked;
                    overrides.insert(row.key.clone(), checked);
                }
                ui.vertical(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&row.display_name).strong().color(INK));
                        if let Some(version) = &row.version {
                            pill(ui, version, CONTROL, MUTED);
                        }
                        if row.installed {
                            pill(ui, "installed", SUCCESS_SOFT, SUCCESS);
                        }
                    });
                    let size = row
                        .size
                        .map(|size| format!("{size} bytes"))
                        .unwrap_or_else(|| "size unknown".to_owned());
                    ui.label(RichText::new(size).size(12.0).color(MUTED));
                });
            });
        });
}

fn progress_block(ui: &mut egui::Ui, progress: &OperationProgress) {
    egui::Frame::new()
        .fill(CONTROL)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.label(RichText::new(&progress.title).strong().color(INK));
            ui.label(RichText::new(&progress.phase).color(MUTED));
            if let Some(item) = &progress.item {
                ui.label(RichText::new(item).monospace().color(INK));
            }
            if let Some(percent) = progress.bar_fraction() {
                ui.add(egui::ProgressBar::new(percent.clamp(0.0, 1.0)).show_percentage());
            } else {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Working...").color(MUTED));
                });
            }
            for line in &progress.logs {
                ui.label(RichText::new(line).size(12.0).color(MUTED));
            }
        });
}

fn materialize_bundled_source(source: &BundledSource) -> anyhow::Result<PathBuf> {
    match source {
        BundledSource::DevPath(path) => Ok(path.clone()),
        BundledSource::Embedded { name, bytes } => {
            let dir = std::env::temp_dir().join("alpha-gui-bundled-assets");
            fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
            let path = dir.join(name);
            fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
            Ok(path)
        }
    }
}

fn bundled_source_available(source: &BundledSource) -> bool {
    match source {
        BundledSource::DevPath(path) => path.exists(),
        BundledSource::Embedded { .. } => true,
    }
}

fn dialog_width(ctx: &egui::Context, preferred: f32) -> f32 {
    (ctx.content_rect().width() - 32.0).clamp(300.0, preferred)
}

fn emit_log(tx: &Sender<TaskEvent>, line: impl Into<String>) {
    let _ = tx.send(TaskEvent::Log(line.into()));
}

fn emit_status(tx: &Sender<TaskEvent>, line: impl Into<String>) {
    let _ = tx.send(TaskEvent::Status(line.into()));
}

fn emit_progress(tx: &Sender<TaskEvent>, event: ProgressEvent) {
    let _ = tx.send(TaskEvent::Progress(event));
}

fn emit_neo_progress(tx: &Sender<TaskEvent>, event: NeoClientProgress) {
    match event {
        NeoClientProgress::OsSegmentErased {
            completed,
            total,
            address,
        } => emit_progress(
            tx,
            ProgressEvent::phase_item(
                "Erasing OS flash segment",
                format!("0x{address:08x}"),
                completed,
                total,
            ),
        ),
        NeoClientProgress::ChunkProgrammed {
            label,
            completed,
            total,
        } => emit_progress(
            tx,
            ProgressEvent::phase_item(
                format!("Programming {label}"),
                format!("chunk {completed} of {total}"),
                completed,
                total,
            ),
        ),
    }
}

fn ensure_direct_mode(tx: &Sender<TaskEvent>) -> anyhow::Result<()> {
    match usb::detect_mode()? {
        NeoMode::Direct => {
            emit_progress(tx, ProgressEvent::phase("Direct USB ready"));
            emit_log(tx, "Device already in direct USB mode.");
            Ok(())
        }
        NeoMode::Hid => {
            emit_progress(tx, ProgressEvent::phase("Switching from keyboard mode"));
            emit_log(
                tx,
                "Device in keyboard mode. Sending direct-mode switch sequence.",
            );
            usb::switch_hid_to_direct()?;
            emit_progress(tx, ProgressEvent::phase("Waiting for direct USB"));
            if !usb::wait_for_mode(NeoMode::Direct, 40, Duration::from_millis(125))? {
                bail!("NEO did not re-enumerate in direct USB mode");
            }
            emit_progress(tx, ProgressEvent::phase("Direct USB ready"));
            emit_log(tx, "Device re-enumerated in direct USB mode.");
            Ok(())
        }
        NeoMode::Missing => bail!("no AlphaSmart NEO detected"),
        NeoMode::HidUnavailable => {
            bail!("AlphaSmart is present but HID access is unavailable on this platform")
        }
    }
}

fn open_client(tx: &Sender<TaskEvent>) -> anyhow::Result<NeoClient> {
    ensure_direct_mode(tx)?;
    emit_progress(tx, ProgressEvent::phase("Initializing direct USB protocol"));
    NeoClient::open_and_init().context("initialize direct USB NEO")
}

fn refresh_inventory_task(tx: Sender<TaskEvent>) -> anyhow::Result<()> {
    let mut client = open_client(&tx)?;
    refresh_inventory_with_client(&tx, &mut client)
}

fn refresh_inventory_with_client(
    tx: &Sender<TaskEvent>,
    client: &mut NeoClient,
) -> anyhow::Result<()> {
    emit_progress(tx, ProgressEvent::phase("Reading inventory"));
    emit_status(tx, "Reading AlphaWord and SmartApplet inventory...");
    let files = client.list_files()?;
    let applets = client.list_smart_applets()?;
    let note = format!(
        "Inventory refreshed: {} slots, {} applets.",
        files.len(),
        applets.len()
    );
    tx.send(TaskEvent::Inventory {
        files,
        applets,
        note,
    })
    .ok();
    Ok(())
}

fn switch_to_direct_task(tx: Sender<TaskEvent>) -> anyhow::Result<()> {
    ensure_direct_mode(&tx)?;
    tx.send(TaskEvent::Finished {
        note: "Device is in direct USB mode.".to_owned(),
    })
    .ok();
    Ok(())
}

fn install_applet_task(
    tx: Sender<TaskEvent>,
    path: PathBuf,
    _assume_updater: bool,
) -> anyhow::Result<()> {
    let mut client = open_client(&tx)?;
    if !path.exists() {
        bail!("applet image does not exist: {}", path.display());
    }
    emit_progress(
        &tx,
        ProgressEvent::phase_item("Installing applet", path.display().to_string(), 1, 1),
    );
    emit_status(&tx, format!("Installing applet {}", path.display()));
    let image = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    let installed =
        client.install_smart_applet_with_progress(&image, |event| emit_neo_progress(&tx, event))?;
    emit_log(
        &tx,
        format!(
            "Installed applet 0x{:04x} {}",
            installed.applet_id, installed.name
        ),
    );
    refresh_inventory_with_client(&tx, &mut client)
}

fn install_applets_task(tx: Sender<TaskEvent>, paths: Vec<PathBuf>) -> anyhow::Result<()> {
    let mut client = open_client(&tx)?;
    install_applets_with_client(&tx, &mut client, paths)?;
    refresh_inventory_with_client(&tx, &mut client)
}

fn install_applets_with_client(
    tx: &Sender<TaskEvent>,
    client: &mut NeoClient,
    paths: Vec<PathBuf>,
) -> anyhow::Result<()> {
    let total = paths.len();
    for (index, path) in paths.into_iter().enumerate() {
        if !path.exists() {
            bail!("applet image does not exist: {}", path.display());
        }
        emit_progress(
            tx,
            ProgressEvent::phase_item(
                "Installing applet",
                path.display().to_string(),
                index + 1,
                total,
            ),
        );
        let image = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let installed = client
            .install_smart_applet_with_progress(&image, |event| emit_neo_progress(tx, event))?;
        emit_log(
            tx,
            format!(
                "Installed applet 0x{:04x} {}",
                installed.applet_id, installed.name
            ),
        );
    }
    Ok(())
}

fn clear_and_install_applets_task(
    tx: Sender<TaskEvent>,
    paths: Vec<PathBuf>,
) -> anyhow::Result<()> {
    let mut client = open_client(&tx)?;
    emit_progress(&tx, ProgressEvent::phase("Clearing SmartApplet area"));
    client.clear_smart_applet_area()?;
    emit_log(&tx, "SmartApplet area cleared.");
    install_applets_with_client(&tx, &mut client, paths)?;
    refresh_inventory_with_client(&tx, &mut client)
}

fn flash_os_task(
    tx: Sender<TaskEvent>,
    path: PathBuf,
    reformat_rest_of_rom: bool,
) -> anyhow::Result<()> {
    let mut client = open_client(&tx)?;
    if !path.exists() {
        bail!("OS image does not exist: {}", path.display());
    }
    emit_progress(&tx, ProgressEvent::phase("Validating bundled OS image"));
    emit_status(&tx, format!("Flashing OS image {}", path.display()));
    let image = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    emit_progress(&tx, ProgressEvent::phase("Flashing OS image"));
    let chunks =
        client.install_neo_os_image_with_progress(&image, reformat_rest_of_rom, |event| {
            emit_neo_progress(&tx, event)
        })?;
    emit_log(&tx, format!("Programmed {chunks} OS chunks."));
    tx.send(TaskEvent::Finished {
        note: format!("OS flash complete from {}", path.display()),
    })
    .ok();
    Ok(())
}

fn probe_task(tx: Sender<TaskEvent>) -> anyhow::Result<()> {
    let mut client = open_client(&tx)?;
    let files = client.list_files()?;
    let applets = client.list_smart_applets()?;
    tx.send(TaskEvent::Finished {
        note: format!(
            "Probe complete: {} AlphaWord slots, {} SmartApplets.",
            files.len(),
            applets.len()
        ),
    })
    .ok();
    Ok(())
}

fn backup_one_slot_task(tx: Sender<TaskEvent>, slot: u8) -> anyhow::Result<()> {
    let mut client = open_client(&tx)?;
    let dir = backup::create_backup_dir()?;
    let raw_path = dir.join(format!("slot-{slot:02}.raw"));
    emit_progress(
        &tx,
        ProgressEvent::phase_item("Downloading AlphaWord slot", format!("slot {slot}"), 1, 1),
    );
    emit_status(&tx, format!("Backing up AlphaWord slot {slot}"));
    let raw = client.download_file(slot)?;
    fs::write(&raw_path, &raw).with_context(|| format!("write {}", raw_path.display()))?;
    let entry = FileEntry {
        slot,
        name: format!("slot-{slot:02}"),
        attribute_bytes: raw.len() as u32,
    };
    let saved = backup::save_file(&dir, &entry, &raw)?;
    emit_log(
        &tx,
        format!(
            "Saved slot {slot} raw={} text={}",
            raw_path.display(),
            saved.txt_path.display()
        ),
    );
    tx.send(TaskEvent::Finished {
        note: format!("Backed up slot {slot} into {}", dir.display()),
    })
    .ok();
    Ok(())
}

fn full_backup_task(tx: Sender<TaskEvent>) -> anyhow::Result<()> {
    let mut client = open_client(&tx)?;
    emit_progress(&tx, ProgressEvent::phase("Reading live device inventory"));
    emit_status(&tx, "Reading live device inventory...");
    let files = client.list_files()?;
    let applets = client.list_smart_applets()?;
    let root = backup::create_device_backup_dir()?;
    let alphaword_dir = root.join("alphaword");
    let applets_dir = root.join("applets");
    fs::create_dir_all(&alphaword_dir)?;
    fs::create_dir_all(&applets_dir)?;
    emit_log(
        &tx,
        format!("Created device backup directory {}", root.display()),
    );

    for (index, file) in files.iter().enumerate() {
        emit_progress(
            &tx,
            ProgressEvent::phase_item(
                "Backing up AlphaWord files",
                file.name.clone(),
                index + 1,
                files.len(),
            ),
        );
        emit_status(
            &tx,
            format!("Backing up AlphaWord slot {} ({})", file.slot, file.name),
        );
        let base = format!("slot-{:02}-{}", file.slot, sanitize_name(&file.name));
        let payload = client.download_file(file.slot)?;
        let raw_saved = backup::save_raw_payload(&alphaword_dir, &base, "raw", &payload)?;
        let payload = fs::read(&raw_saved.path)?;
        let entry = FileEntry {
            slot: file.slot,
            name: file.name.clone(),
            attribute_bytes: file.attribute_bytes,
        };
        let saved_txt = backup::save_file(&alphaword_dir, &entry, &payload)?;
        emit_log(
            &tx,
            format!(
                "Backed up slot {} raw={} text={}",
                file.slot,
                raw_saved.path.display(),
                saved_txt.txt_path.display()
            ),
        );
    }

    for (index, applet) in applets.iter().enumerate() {
        emit_progress(
            &tx,
            ProgressEvent::phase_item(
                "Backing up SmartApplets",
                applet.name.clone(),
                index + 1,
                applets.len(),
            ),
        );
        emit_status(
            &tx,
            format!(
                "Dumping SmartApplet 0x{:04x} ({})",
                applet.applet_id, applet.name
            ),
        );
        let path = applets_dir.join(format!("{:04X}.os3kapp", applet.applet_id));
        let image = client.download_smart_applet(applet.applet_id)?;
        fs::write(&path, image).with_context(|| format!("write {}", path.display()))?;
        emit_log(
            &tx,
            format!(
                "Dumped applet 0x{:04x} to {}",
                applet.applet_id,
                path.display()
            ),
        );
    }

    let manifest = build_backup_manifest(&files, &applets);
    let manifest_path = root.join("manifest.txt");
    fs::write(&manifest_path, manifest)
        .with_context(|| format!("write {}", manifest_path.display()))?;
    emit_log(&tx, format!("Wrote manifest {}", manifest_path.display()));
    tx.send(TaskEvent::Inventory {
        files,
        applets,
        note: format!("Full device backup saved to {}", root.display()),
    })
    .ok();
    Ok(())
}

fn build_backup_manifest(files: &[FileEntry], applets: &[SmartAppletRecord]) -> String {
    let mut lines = vec![
        "AlphaSmart NEO full backup".to_owned(),
        format!("files={}", files.len()),
        format!("applets={}", applets.len()),
        String::new(),
        "AlphaWord slots:".to_owned(),
    ];
    for file in files {
        lines.push(format!(
            "- slot={} name={} file_length={}",
            file.slot, file.name, file.attribute_bytes
        ));
    }
    lines.push(String::new());
    lines.push("SmartApplets:".to_owned());
    for applet in applets {
        lines.push(format!(
            "- applet_id=0x{:04x} version={} name={} file_size={} class=0x{:02x}",
            applet.applet_id, applet.version, applet.name, applet.file_size, applet.applet_class
        ));
    }
    lines.join("\n")
}

fn sanitize_name(name: &str) -> String {
    let clean = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned();
    if clean.is_empty() {
        "untitled".to_owned()
    } else {
        clean
    }
}
