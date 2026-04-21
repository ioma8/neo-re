use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use eframe::egui;

use crate::firmware::FirmwareRuntime;
use crate::firmware_session::FirmwareSession;
use crate::keyboard::{matrix_cells, matrix_key_for_char, matrix_key_is_character, matrix_key_label};
use crate::lcd::LcdSnapshot;

const SHIFT_CODE: u8 = 0x6e;
const COMMAND_CODE: u8 = 0x14;
const OPTION_CODE: u8 = 0x41;
const CTRL_CODE: u8 = 0x7c;
const NEO_CPU_HZ: u64 = 33_000_000;
const REALTIME_FRAME_INTERVAL: Duration = Duration::from_millis(16);
const MAX_REALTIME_STEPS_PER_FRAME: usize = 250_000;
const MAX_REALTIME_CATCHUP: Duration = Duration::from_millis(16);
const MAX_REALTIME_WORK_PER_FRAME: Duration = Duration::from_millis(12);
const SPEED_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const NEO_VISIBLE_LCD_HEIGHT: usize = 64;
const NEO_VISIBLE_LCD_WIDTH: usize = 256;

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
    modifier_state: ModifierMatrixState,
    last_realtime_tick: Instant,
    last_speed_sample: Instant,
    last_speed_cycles: u64,
    realtime_cycle_remainder: f64,
    measured_hz: f64,
    last_target_cycles: u64,
    last_actual_cycles: u64,
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
        };
        app.boot_path(path);
        app
    }

    fn boot_path(&mut self, path: &Path) {
        self.boot_path_with_entry_chord(path, false);
    }

    fn boot_path_with_entry_chord(&mut self, path: &Path, hold_entry_chord: bool) {
        self.firmware_path = path.to_path_buf();
        let loaded = FirmwareRuntime::load_small_rom(path)
            .map_err(|error| error.to_string())
            .and_then(|rom| {
                if hold_entry_chord {
                    FirmwareSession::boot_small_rom_with_entry_chord(rom)
                } else {
                    FirmwareSession::boot_small_rom(rom)
                }
                .map_err(|error| error.to_string())
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
        let loaded = FirmwareRuntime::load_small_rom(path)
            .map_err(|error| error.to_string())
            .and_then(|rom| {
                FirmwareSession::boot_with_keys(rom, &[0x0e, 0x0c], 512)
                    .map_err(|error| error.to_string())
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
                            let lcd = session.lcd_snapshot();
                            let status = session.status_text().to_string();
                            render_lcd(ui, &lcd);
                            ui.add_space(14.0);
                            render_summary(ui, &self.firmware_path, &status, self.measured_hz);
                            ui.add_space(14.0);
                            render_boot_controls(ui, self);
                            ui.add_space(14.0);
                            let Some(session) = self.session.as_mut() else {
                                return;
                            };
                            render_controls(
                                ui,
                                session,
                                self.measured_hz,
                                self.last_target_cycles,
                                self.last_actual_cycles,
                            );
                        }
                        None => render_empty_state(ui, self.load_error.as_deref()),
                    }
                });
            });
    }
}

impl eframe::App for AlphaEmuApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        egui::Frame::new()
            .fill(app_bg())
            .show(ui, |ui| self.render(ui));
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
                    egui::Event::Text(text)
                        if !input.modifiers.ctrl && !input.modifiers.mac_cmd =>
                    {
                        for character in text.chars() {
                            if let Some(tap) = tap_for_text_char(character) {
                                tap.apply_as_text(session, input.modifiers.shift);
                                handled = true;
                            }
                        }
                    }
                    egui::Event::Key {
                        key,
                        pressed,
                        repeat: false,
                        modifiers,
                        ..
                    } => {
                        handled |= self.modifier_state.sync(session, *modifiers);
                        if let Some(code) = matrix_code_for_key(*key) {
                            if *pressed {
                                session.press_matrix_code(code);
                            } else {
                                session.release_matrix_code(code);
                            }
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
struct TextTap {
    character: char,
    needs_shift: bool,
}

impl TextTap {
    fn apply_as_text(self, session: &mut FirmwareSession, shift_already_held: bool) {
        if self.needs_shift && !shift_already_held {
            session.press_matrix_code(SHIFT_CODE);
        }
        session.tap_char(self.character);
        if self.needs_shift && !shift_already_held {
            session.release_matrix_code(SHIFT_CODE);
        }
    }
}

fn tap_for_text_char(character: char) -> Option<TextTap> {
    let (character, needs_shift) = match character {
        'A'..='Z' => (character.to_ascii_lowercase(), true),
        'a'..='z'
        | '0'..='9'
        | '-'
        | '='
        | '['
        | ']'
        | '\\'
        | ';'
        | '\''
        | '`'
        | ','
        | '.'
        | '/' => (character, false),
        '!' => ('1', true),
        '@' => ('2', true),
        '#' => ('3', true),
        '$' => ('4', true),
        '%' => ('5', true),
        '^' => ('6', true),
        '&' => ('7', true),
        '*' => ('8', true),
        '(' => ('9', true),
        ')' => ('0', true),
        '_' => ('-', true),
        '+' => ('=', true),
        '{' => ('[', true),
        '}' => (']', true),
        '|' => ('\\', true),
        ':' => (';', true),
        '"' => ('\'', true),
        '~' => ('`', true),
        '<' => (',', true),
        '>' => ('.', true),
        '?' => ('/', true),
        _ => return None,
    };
    Some(TextTap {
        character,
        needs_shift,
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

fn render_summary(ui: &mut egui::Ui, path: &Path, status: &str, measured_hz: f64) {
    card(ui, |ui| {
        ui.label(
            egui::RichText::new("Firmware session")
                .size(18.0)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            metadata_pill(ui, "File", path.display());
            metadata_pill(ui, "State", status);
            metadata_pill(ui, "CPU", format_speed(measured_hz));
        });
    });
}

fn render_boot_controls(ui: &mut egui::Ui, app: &mut AlphaEmuApp) {
    card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            if secondary_button(ui, "Reboot normally").clicked() {
                let path = app.firmware_path.clone();
                app.boot_path(&path);
            }
            if primary_button(ui, "Reboot Small ROM with activating key chord").clicked() {
                let path = app.firmware_path.clone();
                app.boot_path_with_entry_chord(&path, true);
            }
            if primary_button(ui, "Boot into SmartApplets list").clicked() {
                let path = app.firmware_path.clone();
                app.boot_path_with_left_shift_tab(&path);
            }
            ui.label(
                egui::RichText::new("SmartApplets boot holds left shift + tab at reset.")
                    .size(12.0)
                    .color(text_secondary()),
            );
        });
    });
}

fn render_controls(
    ui: &mut egui::Ui,
    session: &mut FirmwareSession,
    measured_hz: f64,
    last_target_cycles: u64,
    last_actual_cycles: u64,
) {
    card(ui, |ui| {
        let status = if session.is_running() {
            "running"
        } else {
            "stopped"
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("Realtime")
                    .size(14.0)
                    .strong()
                    .color(text_primary()),
            );
            metadata_pill(ui, "CPU", status);
            metadata_pill(ui, "Target", "33.0 MHz");
            metadata_pill(ui, "Actual", format_speed(measured_hz));
            ui.label(
                egui::RichText::new(speed_note(
                    measured_hz,
                    last_target_cycles,
                    last_actual_cycles,
                ))
                .size(12.0)
                .color(text_secondary()),
            );
        });
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if primary_button(ui, "Step 200").clicked() {
                session.run_steps(200);
            }
            if secondary_button(ui, "Step 5,000").clicked() {
                session.run_steps(5_000);
            }
            if secondary_button(ui, "Run 250,000 fast").clicked() {
                session.run_realtime_steps(250_000);
            }
        });
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("NEO keyboard matrix")
                .size(14.0)
                .strong()
                .color(text_primary()),
        );
        ui.label(
            egui::RichText::new(
                "Buttons send firmware matrix codes through the validated HID key bridge.",
            )
            .size(12.0)
            .color(text_secondary()),
        );
        ui.add_space(8.0);
        render_special_matrix_buttons(ui, session);
    });
}

fn format_speed(hz: f64) -> String {
    if hz <= 0.0 {
        "measuring".to_string()
    } else {
        format!("{:.1} MHz", hz / 1_000_000.0)
    }
}

fn speed_note(measured_hz: f64, last_target_cycles: u64, last_actual_cycles: u64) -> &'static str {
    if measured_hz <= 0.0 {
        "The emulator measures actual CPU throughput once per second."
    } else if measured_hz >= NEO_CPU_HZ as f64 * 0.95 {
        "The firmware is running at approximately the real NEO clock."
    } else if last_actual_cycles < last_target_cycles {
        "Host throughput is below the real NEO clock; run a release build for best speed."
    } else {
        "The firmware advances automatically while running."
    }
}

fn render_special_matrix_buttons(ui: &mut egui::Ui, session: &mut FirmwareSession) {
    let cells = matrix_cells();
    let mut any_pressed = false;
    for row in 0..16u8 {
        ui.horizontal_wrapped(|ui| {
            for cell in cells.iter().filter(|cell| cell.row == row) {
                let raw = cell.raw.code();
                if matrix_key_is_character(raw) {
                    continue;
                }
                let label = format!("{} (0x{:02x})", matrix_key_label(raw), cell.logical);
                let response = button_for_matrix_key(
                    ui,
                    &label,
                    format!("row {:02x}, col {:x}", cell.row, cell.col),
                );
                if response.clicked() {
                    session.tap_matrix_code_long(raw);
                    any_pressed = true;
                }
            }
        });
        ui.add_space(6.0);
    }
    if any_pressed {
        session.run_realtime_steps(2_000);
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
            egui::RichText::new("Emulated NEO LCD")
                .size(15.0)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(10.0);
        let visible_height = NEO_VISIBLE_LCD_HEIGHT.min(lcd.height);
        let visible_width = NEO_VISIBLE_LCD_WIDTH.min(lcd.width);
        let width = ui.available_width().min(visible_width as f32 * 3.0);
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
        let scale_x = rect.width() / visible_width as f32;
        let scale_y = rect.height() / visible_height as f32;
        for y in 0..visible_height {
            for x in 0..visible_width {
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

#[cfg(test)]
mod tests {
    use super::{
        NEO_VISIBLE_LCD_HEIGHT, NEO_VISIBLE_LCD_WIDTH, matrix_code_for_key, tap_for_text_char,
    };
    use eframe::egui;

    use crate::keyboard::{matrix_cells, matrix_key_is_character, matrix_key_label};

    #[test]
    fn printable_text_keys_are_covered() {
        for character in "abcdefghijklmnopqrstuvwxyz0123456789-=[]\\;'`,./".chars() {
            let tap = tap_for_text_char(character);
            assert_eq!(
                tap.map(|tap| (tap.character, tap.needs_shift)),
                Some((character, false)),
                "missing unshifted character {character:?}"
            );
        }
    }

    #[test]
    fn shifted_text_keys_are_covered() {
        let expected = [
            ('A', 'a'),
            ('Z', 'z'),
            ('!', '1'),
            ('@', '2'),
            ('#', '3'),
            ('$', '4'),
            ('%', '5'),
            ('^', '6'),
            ('&', '7'),
            ('*', '8'),
            ('(', '9'),
            (')', '0'),
            ('_', '-'),
            ('+', '='),
            ('{', '['),
            ('}', ']'),
            ('|', '\\'),
            (':', ';'),
            ('"', '\''),
            ('~', '`'),
            ('<', ','),
            ('>', '.'),
            ('?', '/'),
        ];
        for (input, base) in expected {
            let tap = tap_for_text_char(input).expect("shifted key must be covered");
            assert_eq!((tap.character, tap.needs_shift), (base, true));
        }
    }

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
        assert_eq!(NEO_VISIBLE_LCD_WIDTH, 256);
        assert_eq!(NEO_VISIBLE_LCD_HEIGHT, 64);
    }
}
