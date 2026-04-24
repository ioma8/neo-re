use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use eframe::egui;

use crate::firmware::FirmwareRuntime;
use crate::firmware_session::FirmwareSession;
use crate::keyboard::{matrix_key_for_char, matrix_key_label};
use crate::lcd::{LcdSnapshot, cursor_blink_snapshot};
use crate::recovery_seed;

const SHIFT_CODE: u8 = 0x6e;
const COMMAND_CODE: u8 = 0x14;
const OPTION_CODE: u8 = 0x41;
const CTRL_CODE: u8 = 0x7c;
const NEO_CPU_HZ: u64 = 33_000_000;
const REALTIME_FRAME_INTERVAL: Duration = Duration::from_millis(16);
const MAX_REALTIME_STEPS_PER_FRAME: usize = 1_000_000;
const MAX_REALTIME_CATCHUP: Duration = Duration::from_millis(16);
const MAX_REALTIME_WORK_PER_FRAME: Duration = Duration::from_millis(15);
const SPEED_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const NEO_VISIBLE_LCD_HEIGHT: usize = 64;
const NEO_VISIBLE_LCD_WIDTH: usize = 264;
const LCD_INNER_PADDING: f32 = 4.0;

/// Runs the desktop Small ROM emulator UI.
///
/// # Errors
///
/// Returns an error if the native GUI runtime fails to start.
pub fn run(path: PathBuf) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Alpha Emulator")
            .with_inner_size([920.0, 560.0])
            .with_min_inner_size([760.0, 500.0]),
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
    modifier_state: ModifierMatrixState,
    last_realtime_tick: Instant,
    last_speed_sample: Instant,
    last_speed_cycles: u64,
    realtime_cycle_remainder: f64,
    measured_hz: f64,
    last_target_cycles: u64,
    last_actual_cycles: u64,
    recovery_seed_path: PathBuf,
    recovery_status: Option<String>,
    recovery_task: Option<Receiver<RecoveryTaskMessage>>,
}

impl AlphaEmuApp {
    fn load(path: &Path) -> Self {
        let mut app = Self {
            firmware_path: path.to_path_buf(),
            session: None,
            load_error: None,
            modifier_state: ModifierMatrixState::default(),
            last_realtime_tick: Instant::now(),
            last_speed_sample: Instant::now(),
            last_speed_cycles: 0,
            realtime_cycle_remainder: 0.0,
            measured_hz: 0.0,
            last_target_cycles: 0,
            last_actual_cycles: 0,
            recovery_seed_path: recovery_seed::default_seed_path(),
            recovery_status: None,
            recovery_task: None,
        };
        app.boot_path(path);
        app
    }

    fn boot_path(&mut self, path: &Path) {
        self.boot_path_with_entry_chord(path, false);
    }

    fn boot_path_with_entry_chord(&mut self, path: &Path, hold_entry_chord: bool) {
        self.firmware_path = path.to_path_buf();
        let loaded = self.load_boot_session(path, |rom| {
            if hold_entry_chord {
                FirmwareSession::boot_small_rom_with_entry_chord(rom)
            } else {
                FirmwareSession::boot_small_rom(rom)
            }
        });
        match loaded {
            Ok(session) => {
                self.session = Some(session);
                self.load_error = None;
                self.modifier_state = ModifierMatrixState::default();
                self.reset_realtime_metrics();
                self.realtime_cycle_remainder = 0.0;
            }
            Err(error) => {
                self.session = None;
                self.load_error = Some(format!("{}: {error}", path.display()));
            }
        }
    }

    fn boot_path_with_left_shift_tab(&mut self, path: &Path) {
        self.firmware_path = path.to_path_buf();
        let loaded = self.load_boot_session(path, |rom| {
            FirmwareSession::boot_with_keys(rom, &[0x0e, 0x0c], 512)
        });
        match loaded {
            Ok(session) => {
                self.session = Some(session);
                self.load_error = None;
                self.modifier_state = ModifierMatrixState::default();
                self.reset_realtime_metrics();
            }
            Err(error) => {
                self.session = None;
                self.load_error = Some(format!("{}: {error}", path.display()));
            }
        }
    }

    fn load_boot_session(
        &self,
        path: &Path,
        boot: impl FnOnce(
            FirmwareRuntime,
        )
            -> Result<FirmwareSession, crate::firmware_session::FirmwareSessionError>,
    ) -> Result<FirmwareSession, String> {
        let firmware = FirmwareRuntime::load_small_rom(path).map_err(|error| error.to_string())?;
        let is_full_system = firmware.is_neo_system_image();
        let mut session = boot(firmware).map_err(|error| error.to_string())?;
        if is_full_system {
            recovery_seed::apply_seed_file_if_present(&mut session, &self.recovery_seed_path)
                .map_err(|error| error.to_string())?;
        }
        Ok(session)
    }

    fn start_recovery_reinit(&mut self) {
        if self.recovery_task.is_some() {
            return;
        }
        let firmware_path = self.firmware_path.clone();
        let seed_path = self.recovery_seed_path.clone();
        let (sender, receiver) = mpsc::channel();
        self.recovery_status = Some("Reinitializing memory with firmware recovery...".to_string());
        self.recovery_task = Some(receiver);
        thread::spawn(move || {
            let result = recovery_seed::generate_and_save_seed_with_progress(
                &firmware_path,
                &seed_path,
                |status| {
                    let _ = sender.send(RecoveryTaskMessage::Progress(status.to_string()));
                },
            )
            .map_err(|error| error.to_string());
            let _ = sender.send(RecoveryTaskMessage::Done(result));
        });
    }

    fn poll_recovery_task(&mut self) {
        let Some(receiver) = self.recovery_task.as_ref() else {
            return;
        };
        loop {
            match receiver.try_recv() {
                Ok(RecoveryTaskMessage::Progress(status)) => {
                    self.recovery_status = Some(status);
                }
                Ok(RecoveryTaskMessage::Done(Ok(path))) => {
                    self.recovery_task = None;
                    self.recovery_status = Some(format!("Memory seed saved: {}", path.display()));
                    let firmware_path = self.firmware_path.clone();
                    self.boot_path(&firmware_path);
                    break;
                }
                Ok(RecoveryTaskMessage::Done(Err(error))) => {
                    self.recovery_task = None;
                    self.recovery_status = Some(format!("Memory reinit failed: {error}"));
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.recovery_task = None;
                    self.recovery_status = Some("Memory reinit worker disconnected".to_string());
                    break;
                }
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
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(880.0);
            render_header(ui, self);
            ui.add_space(10.0);
            match self.session.as_mut() {
                Some(session) => {
                    let lcd = session.lcd_snapshot();
                    let status = session.status_text().to_string();
                    let applet_memory_status = session.applet_memory_status();
                    render_lcd(ui, &lcd);
                    ui.add_space(8.0);
                    render_session_controls(ui, self, &status, &applet_memory_status);
                    ui.add_space(8.0);
                    let Some(session) = self.session.as_mut() else {
                        return;
                    };
                    render_controls(ui, session);
                }
                None => render_empty_state(ui, self.load_error.as_deref()),
            }
        });
    }
}

#[derive(Debug)]
enum RecoveryTaskMessage {
    Progress(String),
    Done(Result<PathBuf, String>),
}

impl eframe::App for AlphaEmuApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_recovery_task();
        if self.recovery_task.is_some() {
            ctx.request_repaint_after(REALTIME_FRAME_INTERVAL);
        }
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_realtime_tick);
        self.last_realtime_tick = now;
        if self
            .session
            .as_ref()
            .is_some_and(FirmwareSession::is_running)
        {
            let elapsed = elapsed.min(MAX_REALTIME_CATCHUP);
            let target_cycles =
                elapsed.as_secs_f64() * NEO_CPU_HZ as f64 + self.realtime_cycle_remainder;
            let cycle_budget = target_cycles.floor() as u64;
            self.realtime_cycle_remainder = target_cycles - cycle_budget as f64;
            if cycle_budget > 0
                && let Some(session) = self.session.as_mut()
            {
                let actual_cycles = session.run_realtime_cycles_for(
                    cycle_budget,
                    MAX_REALTIME_STEPS_PER_FRAME,
                    MAX_REALTIME_WORK_PER_FRAME,
                );
                self.last_target_cycles = cycle_budget;
                self.last_actual_cycles = actual_cycles;
                self.sample_realtime_speed(now);
            }
            ctx.request_repaint_after(REALTIME_FRAME_INTERVAL);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ui.ctx());
        let available = ui.available_size();
        egui::Frame::new().fill(app_bg()).show(ui, |ui| {
            ui.set_min_size(available);
            self.render(ui);
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        app_bg().to_normalized_gamma_f32()
    }
}

impl AlphaEmuApp {
    fn reset_realtime_metrics(&mut self) {
        self.last_realtime_tick = Instant::now();
        self.last_speed_sample = self.last_realtime_tick;
        self.last_speed_cycles = self.session.as_ref().map_or(0, FirmwareSession::cycles);
        self.realtime_cycle_remainder = 0.0;
        self.measured_hz = 0.0;
        self.last_target_cycles = 0;
        self.last_actual_cycles = 0;
    }

    fn sample_realtime_speed(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last_speed_sample);
        if elapsed < SPEED_SAMPLE_INTERVAL {
            return;
        }
        let Some(session) = self.session.as_ref() else {
            return;
        };
        let cycles = session.cycles();
        self.measured_hz =
            cycles.saturating_sub(self.last_speed_cycles) as f64 / elapsed.as_secs_f64();
        self.last_speed_cycles = cycles;
        self.last_speed_sample = now;
        tracing::info!(
            target_hz = NEO_CPU_HZ,
            achieved_hz = self.measured_hz,
            "alpha-emu realtime speed sample"
        );
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let mut handled = false;
        ctx.input(|input| {
            handled |= self.modifier_state.sync(session, input.modifiers);
            for event in &input.events {
                match event {
                    egui::Event::Key {
                        key,
                        pressed,
                        repeat: false,
                        modifiers,
                        ..
                    } => {
                        handled |= self.modifier_state.sync(session, *modifiers);
                        if let Some(tap) = matrix_tap_for_key(*key) {
                            if *pressed {
                                tap.press(session);
                            } else {
                                tap.release(session);
                            }
                            session.run_realtime_steps(2_000);
                            handled = true;
                        }
                    }
                    _ => {}
                }
            }
        });
        if handled {
            session.run_realtime_steps(2_000);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ModifierMatrixState {
    shift: bool,
    ctrl: bool,
    command: bool,
    option: bool,
}

impl ModifierMatrixState {
    fn sync(&mut self, session: &mut FirmwareSession, modifiers: egui::Modifiers) -> bool {
        let next = Self {
            shift: modifiers.shift,
            ctrl: modifiers.ctrl,
            command: modifiers.mac_cmd,
            option: modifiers.alt,
        };
        let mut changed = false;
        changed |= sync_matrix_key(session, self.shift, next.shift, SHIFT_CODE);
        changed |= sync_matrix_key(session, self.ctrl, next.ctrl, CTRL_CODE);
        changed |= sync_matrix_key(session, self.command, next.command, COMMAND_CODE);
        changed |= sync_matrix_key(session, self.option, next.option, OPTION_CODE);
        *self = next;
        changed
    }
}

fn sync_matrix_key(session: &mut FirmwareSession, old: bool, new: bool, code: u8) -> bool {
    match (old, new) {
        (false, true) => {
            session.press_matrix_code(code);
            true
        }
        (true, false) => {
            session.release_matrix_code(code);
            true
        }
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MatrixTap {
    Key(u8),
    Chord(&'static [u8]),
}

impl MatrixTap {
    fn press(self, session: &mut FirmwareSession) {
        match self {
            Self::Key(code) => session.press_matrix_code(code),
            Self::Chord(codes) => {
                for code in codes {
                    session.press_matrix_code(*code);
                }
            }
        }
    }

    fn release(self, session: &mut FirmwareSession) {
        match self {
            Self::Key(code) => session.release_matrix_code(code),
            Self::Chord(codes) => {
                for code in codes {
                    session.release_matrix_code(*code);
                }
            }
        }
    }
}

const PLUS_CHORD: &[u8] = &[SHIFT_CODE, 0x40];

fn matrix_tap_for_key(key: egui::Key) -> Option<MatrixTap> {
    matrix_code_for_key(key)
        .map(MatrixTap::Key)
        .or_else(|| match key {
            egui::Key::Plus => Some(MatrixTap::Chord(PLUS_CHORD)),
            _ => None,
        })
}

fn matrix_code_for_key(key: egui::Key) -> Option<u8> {
    match key {
        egui::Key::A => matrix_key_for_char('a').map(|key| key.code()),
        egui::Key::B => matrix_key_for_char('b').map(|key| key.code()),
        egui::Key::C => matrix_key_for_char('c').map(|key| key.code()),
        egui::Key::D => matrix_key_for_char('d').map(|key| key.code()),
        egui::Key::E => matrix_key_for_char('e').map(|key| key.code()),
        egui::Key::F => matrix_key_for_char('f').map(|key| key.code()),
        egui::Key::G => matrix_key_for_char('g').map(|key| key.code()),
        egui::Key::H => matrix_key_for_char('h').map(|key| key.code()),
        egui::Key::I => matrix_key_for_char('i').map(|key| key.code()),
        egui::Key::J => matrix_key_for_char('j').map(|key| key.code()),
        egui::Key::K => matrix_key_for_char('k').map(|key| key.code()),
        egui::Key::L => matrix_key_for_char('l').map(|key| key.code()),
        egui::Key::M => matrix_key_for_char('m').map(|key| key.code()),
        egui::Key::N => matrix_key_for_char('n').map(|key| key.code()),
        egui::Key::O => matrix_key_for_char('o').map(|key| key.code()),
        egui::Key::P => matrix_key_for_char('p').map(|key| key.code()),
        egui::Key::Q => matrix_key_for_char('q').map(|key| key.code()),
        egui::Key::R => matrix_key_for_char('r').map(|key| key.code()),
        egui::Key::S => matrix_key_for_char('s').map(|key| key.code()),
        egui::Key::T => matrix_key_for_char('t').map(|key| key.code()),
        egui::Key::U => matrix_key_for_char('u').map(|key| key.code()),
        egui::Key::V => matrix_key_for_char('v').map(|key| key.code()),
        egui::Key::W => matrix_key_for_char('w').map(|key| key.code()),
        egui::Key::X => matrix_key_for_char('x').map(|key| key.code()),
        egui::Key::Y => matrix_key_for_char('y').map(|key| key.code()),
        egui::Key::Z => matrix_key_for_char('z').map(|key| key.code()),
        egui::Key::Num0 => matrix_key_for_char('0').map(|key| key.code()),
        egui::Key::Num1 => matrix_key_for_char('1').map(|key| key.code()),
        egui::Key::Num2 => matrix_key_for_char('2').map(|key| key.code()),
        egui::Key::Num3 => matrix_key_for_char('3').map(|key| key.code()),
        egui::Key::Num4 => matrix_key_for_char('4').map(|key| key.code()),
        egui::Key::Num5 => matrix_key_for_char('5').map(|key| key.code()),
        egui::Key::Num6 => matrix_key_for_char('6').map(|key| key.code()),
        egui::Key::Num7 => matrix_key_for_char('7').map(|key| key.code()),
        egui::Key::Num8 => matrix_key_for_char('8').map(|key| key.code()),
        egui::Key::Num9 => matrix_key_for_char('9').map(|key| key.code()),
        egui::Key::Exclamationmark => matrix_key_for_char('1').map(|key| key.code()),
        egui::Key::OpenBracket | egui::Key::OpenCurlyBracket => {
            matrix_key_for_char('[').map(|key| key.code())
        }
        egui::Key::CloseBracket | egui::Key::CloseCurlyBracket => {
            matrix_key_for_char(']').map(|key| key.code())
        }
        egui::Key::Backslash | egui::Key::Pipe => matrix_key_for_char('\\').map(|key| key.code()),
        egui::Key::Semicolon | egui::Key::Colon => matrix_key_for_char(';').map(|key| key.code()),
        egui::Key::Quote => matrix_key_for_char('\'').map(|key| key.code()),
        egui::Key::Backtick => matrix_key_for_char('`').map(|key| key.code()),
        egui::Key::Comma => matrix_key_for_char(',').map(|key| key.code()),
        egui::Key::Period => matrix_key_for_char('.').map(|key| key.code()),
        egui::Key::Slash | egui::Key::Questionmark => {
            matrix_key_for_char('/').map(|key| key.code())
        }
        egui::Key::Minus => matrix_key_for_char('-').map(|key| key.code()),
        egui::Key::Equals => matrix_key_for_char('=').map(|key| key.code()),
        egui::Key::Plus => None,
        egui::Key::Backspace => Some(0x09),
        egui::Key::Delete => Some(0x61),
        egui::Key::Enter => Some(0x69),
        egui::Key::Escape => Some(0x74),
        egui::Key::Space => Some(0x79),
        egui::Key::Tab => Some(0x0c),
        egui::Key::ArrowUp => Some(0x77),
        egui::Key::ArrowDown => Some(0x15),
        egui::Key::ArrowLeft => Some(0x75),
        egui::Key::ArrowRight => Some(0x76),
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
                egui::RichText::new("AlphaSmart NEO Emulator")
                    .size(20.0)
                    .strong()
                    .color(text_primary()),
            );
            ui.label(
                egui::RichText::new("Firmware, LCD, and keyboard matrix")
                    .size(12.0)
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

fn render_session_controls(
    ui: &mut egui::Ui,
    app: &mut AlphaEmuApp,
    status: &str,
    applet_memory_status: &str,
) {
    compact_panel(ui, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 5.0);
        ui.horizontal(|ui| {
            metadata_pill(ui, "File", compact_path(&app.firmware_path));
            metadata_pill(ui, "", status);
            metadata_pill(ui, "CPU", format_speed(app.measured_hz));
            metadata_pill(ui, "Applets", applet_memory_status);
            ui.add_space(6.0);
            if secondary_button(ui, "Reboot normally").clicked() {
                let path = app.firmware_path.clone();
                app.boot_path(&path);
            }
            if secondary_button(ui, "Small ROM").clicked() {
                let path = app.firmware_path.clone();
                app.boot_path_with_entry_chord(&path, true);
            }
            if primary_button(ui, "SmartApplets").clicked() {
                let path = app.firmware_path.clone();
                app.boot_path_with_left_shift_tab(&path);
            }
            if secondary_button(ui, "Reinit").clicked() {
                app.start_recovery_reinit();
            }
        });
        if let Some(status) = &app.recovery_status {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(status)
                    .size(11.0)
                    .color(text_secondary()),
            );
        }
    });
}

fn render_controls(ui: &mut egui::Ui, session: &mut FirmwareSession) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new("Functional keys")
                .size(14.0)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(8.0);
        render_functional_keys(ui, session);
    });
}

fn format_speed(hz: f64) -> String {
    if hz <= 0.0 {
        "measuring".to_string()
    } else {
        format!("{:.1} MHz", hz / 1_000_000.0)
    }
}

fn render_functional_keys(ui: &mut egui::Ui, session: &mut FirmwareSession) {
    let mut any_pressed = false;
    render_key_group(
        ui,
        "Files",
        &[
            ("F1", 0x4b),
            ("F2", 0x4a),
            ("F3", 0x0a),
            ("F4", 0x1a),
            ("F5", 0x19),
            ("F6", 0x10),
            ("F7", 0x02),
            ("F8", 0x42),
        ],
        session,
        &mut any_pressed,
    );
    render_key_group(
        ui,
        "Actions",
        &[
            ("Applets", 0x46),
            ("Send", 0x47),
            ("Print", 0x49),
            ("Spell", 0x59),
            ("Find", 0x67),
            ("Clear", 0x54),
        ],
        session,
        &mut any_pressed,
    );
    render_key_group(
        ui,
        "Navigation",
        &[
            ("Home", 0x34),
            ("End", 0x65),
            ("Up", 0x77),
            ("Down", 0x15),
            ("Left", 0x75),
            ("Right", 0x76),
        ],
        session,
        &mut any_pressed,
    );
    render_key_group(
        ui,
        "Edit / modifiers",
        &[
            ("Esc", 0x74),
            ("Tab", 0x0c),
            ("Backspace", 0x09),
            ("Delete", 0x61),
            ("Enter", 0x69),
            ("Shift", SHIFT_CODE),
            ("Ctrl", CTRL_CODE),
            ("Option", OPTION_CODE),
            ("Cmd", COMMAND_CODE),
        ],
        session,
        &mut any_pressed,
    );
    if any_pressed {
        session.run_realtime_steps(2_000);
    }
}

fn render_key_group(
    ui: &mut egui::Ui,
    title: &str,
    keys: &[(&str, u8)],
    session: &mut FirmwareSession,
    any_pressed: &mut bool,
) {
    ui.horizontal_wrapped(|ui| {
        ui.add_sized(
            egui::vec2(86.0, 26.0),
            egui::Label::new(
                egui::RichText::new(title)
                    .size(12.0)
                    .strong()
                    .color(text_secondary()),
            ),
        );
        for (label, raw) in keys {
            if button_for_matrix_key(ui, label, *raw).clicked() {
                session.tap_matrix_code(*raw);
                *any_pressed = true;
            }
        }
    });
    ui.add_space(4.0);
}

fn button_for_matrix_key(ui: &mut egui::Ui, label: &str, raw: u8) -> egui::Response {
    let response = ui.add(
        egui::Button::new(egui::RichText::new(label).size(12.0).color(text_primary()))
            .fill(egui::Color32::from_rgb(238, 241, 245))
            .stroke(border())
            .corner_radius(7.0)
            .min_size(egui::vec2(54.0, 26.0)),
    );
    response.on_hover_text(format!("{} / raw 0x{raw:02x}", matrix_key_label(raw)))
}

fn render_lcd(ui: &mut egui::Ui, lcd: &LcdSnapshot) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new("Emulated NEO LCD")
                .size(15.0)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(10.0);
        let visible_height = NEO_VISIBLE_LCD_HEIGHT.min(lcd.height);
        let visible_width = NEO_VISIBLE_LCD_WIDTH.min(lcd.width);
        let width = ui.available_width().min(visible_width as f32 * 2.5);
        let height = width * visible_height as f32 / visible_width as f32;
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
        let pixel_rect = rect.shrink(LCD_INNER_PADDING);
        let scale_x = pixel_rect.width() / visible_width as f32;
        let scale_y = pixel_rect.height() / visible_height as f32;
        let cursor_visible = ((ui.input(|input| input.time) * 2.0) as u64).is_multiple_of(2);
        let lcd = cursor_blink_snapshot(lcd, cursor_visible);
        for y in 0..visible_height {
            for x in 0..visible_width {
                if lcd.pixels[y * lcd.width + x] {
                    let min = egui::pos2(
                        pixel_rect.left() + x as f32 * scale_x,
                        pixel_rect.top() + y as f32 * scale_y,
                    );
                    let max = egui::pos2(min.x + scale_x.max(1.0), min.y + scale_y.max(1.0));
                    painter.rect_filled(egui::Rect::from_min_max(min, max), 0.0, pixel);
                }
            }
        }
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

fn compact_panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(card_bg())
        .stroke(border())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, add_contents);
}

fn compact_path(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("firmware");
    let parent = path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str());
    if let Some(parent) = parent {
        format!("{parent}/{file_name}")
    } else {
        file_name.to_string()
    }
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
            .corner_radius(8.0)
            .min_size(egui::vec2(0.0, 26.0)),
    )
}

fn metadata_pill(ui: &mut egui::Ui, label: &str, value: impl ToString) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(239, 243, 248))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if !label.is_empty() {
                    ui.label(
                        egui::RichText::new(label)
                            .size(11.0)
                            .color(text_secondary()),
                    );
                }
                ui.label(
                    egui::RichText::new(value.to_string())
                        .size(11.0)
                        .strong()
                        .color(text_primary()),
                );
            });
        });
}

#[cfg(test)]
mod tests {
    use super::{
        MatrixTap, NEO_VISIBLE_LCD_HEIGHT, NEO_VISIBLE_LCD_WIDTH, PLUS_CHORD, matrix_code_for_key,
        matrix_tap_for_key,
    };
    use eframe::egui;

    use crate::keyboard::{matrix_cells, matrix_key_is_character, matrix_key_label};
    use crate::lcd::{
        LcdSnapshot, cursor_blink_snapshot, probable_cursor_columns, probable_cursor_pixels,
    };

    #[test]
    fn whitespace_and_editing_keys_are_covered_as_physical_keys() {
        let expected = [
            (egui::Key::Space, 0x79),
            (egui::Key::Tab, 0x0c),
            (egui::Key::Enter, 0x69),
            (egui::Key::Backspace, 0x09),
            (egui::Key::Delete, 0x61),
            (egui::Key::Escape, 0x74),
        ];
        for (key, code) in expected {
            assert_eq!(matrix_code_for_key(key), Some(code));
        }
    }

    #[test]
    fn arrows_are_covered_as_physical_keys() {
        assert_eq!(matrix_code_for_key(egui::Key::ArrowUp), Some(0x77));
        assert_eq!(matrix_code_for_key(egui::Key::ArrowDown), Some(0x15));
        assert_eq!(matrix_code_for_key(egui::Key::ArrowLeft), Some(0x75));
        assert_eq!(matrix_code_for_key(egui::Key::ArrowRight), Some(0x76));
    }

    #[test]
    fn alphabet_keys_are_available_for_keyboard_shortcuts() {
        let expected = [
            (egui::Key::A, 0x2c),
            (egui::Key::E, 0x3a),
            (egui::Key::I, 0x30),
            (egui::Key::K, 0x20),
            (egui::Key::Q, 0x3c),
            (egui::Key::R, 0x3d),
            (egui::Key::T, 0x0d),
            (egui::Key::W, 0x3b),
            (egui::Key::Z, 0x6c),
        ];
        for (key, code) in expected {
            assert_eq!(matrix_code_for_key(key), Some(code));
        }
    }

    #[test]
    fn number_keys_are_available_for_keyboard_shortcuts() {
        let expected = [
            (egui::Key::Num0, 0x53),
            (egui::Key::Num1, 0x5c),
            (egui::Key::Num2, 0x5b),
            (egui::Key::Num3, 0x5a),
            (egui::Key::Num4, 0x5d),
            (egui::Key::Num5, 0x4d),
            (egui::Key::Num6, 0x4f),
            (egui::Key::Num7, 0x5f),
            (egui::Key::Num8, 0x50),
            (egui::Key::Num9, 0x52),
        ];
        for (key, code) in expected {
            assert_eq!(matrix_code_for_key(key), Some(code));
        }
    }

    #[test]
    fn punctuation_and_calculator_operator_keys_are_physical_keys() {
        let expected = [
            (egui::Key::Minus, 0x43),
            (egui::Key::Equals, 0x40),
            (egui::Key::Slash, 0x73),
            (egui::Key::Questionmark, 0x73),
            (egui::Key::Period, 0x62),
            (egui::Key::Comma, 0x60),
            (egui::Key::Semicolon, 0x23),
            (egui::Key::Colon, 0x23),
            (egui::Key::Backslash, 0x29),
            (egui::Key::Pipe, 0x29),
        ];
        for (key, code) in expected {
            assert_eq!(matrix_code_for_key(key), Some(code));
        }
        assert_eq!(
            matrix_tap_for_key(egui::Key::Plus),
            Some(MatrixTap::Chord(PLUS_CHORD))
        );
    }

    #[test]
    fn non_character_matrix_keys_are_gui_button_candidates() {
        let labels = matrix_cells()
            .into_iter()
            .filter(|cell| !matrix_key_is_character(cell.raw.code()))
            .map(|cell| matrix_key_label(cell.raw.code()))
            .collect::<Vec<_>>();

        for expected in [
            "File 1",
            "Print",
            "Spell Check",
            "Find",
            "Clear File",
            "Applets",
            "Send",
            "Up",
            "Down",
            "Left",
            "Right",
        ] {
            assert!(
                labels.iter().any(|label| label == expected),
                "missing {expected}"
            );
        }
    }

    #[test]
    fn gui_uses_neo_visible_lcd_viewport_height() {
        assert_eq!(NEO_VISIBLE_LCD_HEIGHT, 64);
    }

    #[test]
    fn gui_uses_square_pixel_viewport_matching_neo_screen_ratio() {
        assert_eq!(NEO_VISIBLE_LCD_WIDTH, 264);
        assert_eq!(NEO_VISIBLE_LCD_HEIGHT, 64);
    }

    #[test]
    fn cursor_detector_masks_two_column_alphaword_cursor_dump_shape() {
        let mut lcd = LcdSnapshot {
            width: 320,
            height: 128,
            pixels: vec![false; 320 * 128],
        };
        for y in 0..16 {
            lcd.pixels[y * lcd.width] = true;
            lcd.pixels[y * lcd.width + 1] = true;
        }

        let mask = probable_cursor_columns(&lcd, NEO_VISIBLE_LCD_WIDTH, NEO_VISIBLE_LCD_HEIGHT);

        assert!(mask[0]);
        assert!(mask[1]);
        assert!(!mask[2]);

        let off = cursor_blink_snapshot(&lcd, false);
        assert!(!off.pixels[0]);
        assert!(!off.pixels[1]);
    }

    #[test]
    fn cursor_detector_masks_only_the_cursor_run_not_the_whole_column() {
        let mut lcd = LcdSnapshot {
            width: 320,
            height: 128,
            pixels: vec![false; 320 * 128],
        };
        for y in 0..16 {
            lcd.pixels[y * lcd.width] = true;
            lcd.pixels[y * lcd.width + 1] = true;
        }
        for y in 32..40 {
            lcd.pixels[y * lcd.width] = true;
        }

        let mask = probable_cursor_pixels(&lcd, NEO_VISIBLE_LCD_WIDTH, NEO_VISIBLE_LCD_HEIGHT);
        let off = cursor_blink_snapshot(&lcd, false);

        assert!(mask[15 * NEO_VISIBLE_LCD_WIDTH]);
        assert!(!mask[32 * NEO_VISIBLE_LCD_WIDTH]);
        assert!(!off.pixels[15 * lcd.width]);
        assert!(off.pixels[32 * lcd.width]);
    }

    #[test]
    fn cursor_detector_does_not_mask_wide_text_stems() {
        let mut lcd = LcdSnapshot {
            width: 320,
            height: 128,
            pixels: vec![false; 320 * 128],
        };
        for y in 0..16 {
            for x in 10..18 {
                lcd.pixels[y * lcd.width + x] = true;
            }
        }

        let mask = probable_cursor_columns(&lcd, NEO_VISIBLE_LCD_WIDTH, NEO_VISIBLE_LCD_HEIGHT);

        assert!(!mask.iter().any(|masked| *masked));
    }
}
