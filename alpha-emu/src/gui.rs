use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use eframe::egui;

use crate::firmware::FirmwareRuntime;
use crate::firmware_session::{FirmwareSession, FirmwareSnapshot};
use crate::keyboard::{matrix_cells, matrix_key_is_character, matrix_key_label};
use crate::lcd::LcdSnapshot;

const SHIFT_CODE: u8 = 0x0e;
const COMMAND_CODE: u8 = 0x14;
const OPTION_CODE: u8 = 0x41;
const CTRL_CODE: u8 = 0x7c;
const NEO_CPU_HZ: u64 = 33_000_000;
const REALTIME_FRAME_INTERVAL: Duration = Duration::from_millis(16);
const MAX_REALTIME_STEPS_PER_FRAME: usize = 250_000;
const MAX_REALTIME_CATCHUP: Duration = Duration::from_millis(50);

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
    realtime_cycle_remainder: f64,
}

impl AlphaEmuApp {
    fn load(path: &Path) -> Self {
        let mut app = Self {
            firmware_path: path.to_path_buf(),
            session: None,
            load_error: None,
            modifier_state: ModifierMatrixState::default(),
            last_realtime_tick: Instant::now(),
            realtime_cycle_remainder: 0.0,
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
                self.last_realtime_tick = Instant::now();
                self.realtime_cycle_remainder = 0.0;
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
                            render_boot_controls(ui, self);
                            ui.add_space(14.0);
                            let Some(session) = self.session.as_mut() else {
                                return;
                            };
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
                session.run_realtime_cycles(cycle_budget, MAX_REALTIME_STEPS_PER_FRAME);
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
                        ..
                    } => {
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
        session.tap_char_all_rows(self.character);
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
            metadata_pill(ui, "Cycles", snapshot.cycles);
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
            ui.label(
                egui::RichText::new("Normal open/reboot does not hold the Small ROM key chord.")
                    .size(12.0)
                    .color(text_secondary()),
            );
        });
    });
}

fn render_controls(ui: &mut egui::Ui, session: &mut FirmwareSession) {
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
            ui.label(
                egui::RichText::new("The firmware advances automatically while running.")
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
                    session.tap_matrix_code(raw);
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

#[cfg(test)]
mod tests {
    use super::{matrix_code_for_key, tap_for_text_char};
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
    fn alphabet_keys_are_not_handled_as_physical_control_keys() {
        for key in [
            egui::Key::A,
            egui::Key::E,
            egui::Key::I,
            egui::Key::K,
            egui::Key::Q,
            egui::Key::R,
            egui::Key::T,
            egui::Key::W,
            egui::Key::Z,
        ] {
            assert_eq!(matrix_code_for_key(key), None);
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
