use std::path::PathBuf;
use std::time::Instant;

use alpha_emu::firmware::FirmwareRuntime;
use alpha_emu::firmware_session::FirmwareSession;
use alpha_emu::keyboard::{matrix_cells, matrix_key_label};
use alpha_emu::lcd::LcdSnapshot;
use alpha_emu::recovery_seed;
use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut headless = false;
    let mut lcd_ascii = false;
    let mut lcd_ranges = false;
    let mut lcd_pbm = None;
    let mut key_events = Vec::new();
    let mut hold_events = Vec::new();
    let mut all_row_key_events = Vec::new();
    let mut type_events = Vec::new();
    let mut stop_at_pc = None;
    let mut stop_at_pc_hit = 1usize;
    let mut stop_at_resource = None;
    let mut scan_special_keys_at = None;
    let mut scan_matrix_visibility_at = None;
    let mut validate_key_map_at = None;
    let mut validate_alpha_usb_native = false;
    let mut validate_forth_mini = false;
    let mut load_memory = None;
    let mut dump_memory_start = None;
    let mut dump_memory = None;
    let mut reinit_memory = false;
    let mut recovery_seed_path = None;
    let mut boot_left_shift_tab = false;
    let mut boot_keys = None;
    let mut boot_keys_exact = None;
    let mut verbose = false;
    let mut path = None;
    let mut steps = 10_000;
    for arg in std::env::args_os().skip(1) {
        if arg == "--headless" {
            headless = true;
        } else if arg == "--boot-left-shift-tab" {
            boot_left_shift_tab = true;
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--boot-keys="))
        {
            boot_keys = Some(parse_key_list(value)?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--boot-keys-exact="))
        {
            boot_keys_exact = Some(parse_key_list(value)?);
        } else if arg == "--lcd-ascii" {
            lcd_ascii = true;
        } else if arg == "--lcd-ranges" {
            lcd_ranges = true;
        } else if arg == "--verbose" {
            verbose = true;
        } else if arg == "--validate-alpha-usb-native" {
            validate_alpha_usb_native = true;
        } else if arg == "--validate-forth-mini" {
            validate_forth_mini = true;
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--load-memory="))
        {
            load_memory = Some(PathBuf::from(value));
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--dump-memory-start="))
        {
            dump_memory_start = Some(PathBuf::from(value));
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--dump-memory="))
        {
            dump_memory = Some(PathBuf::from(value));
        } else if arg == "--reinit-memory" {
            reinit_memory = true;
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--recovery-seed="))
        {
            recovery_seed_path = Some(PathBuf::from(value));
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--lcd-pbm=")) {
            lcd_pbm = Some(PathBuf::from(value));
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--key-at=")) {
            key_events.push(parse_key_event(value)?);
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--hold-key=")) {
            hold_events.push(parse_hold_event(value)?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--key-all-rows-at="))
        {
            all_row_key_events.push(parse_key_event(value)?);
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--type-at=")) {
            type_events.push(parse_type_event(value)?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--stop-at-pc="))
        {
            stop_at_pc = Some(parse_u32_arg(value)?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--stop-at-pc-hit="))
        {
            stop_at_pc_hit = value.parse()?;
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--stop-at-resource="))
        {
            stop_at_resource = Some(parse_u32_arg(value)? as u16);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--scan-special-keys-at="))
        {
            scan_special_keys_at = Some(value.parse()?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--scan-matrix-visibility-at="))
        {
            scan_matrix_visibility_at = Some(value.parse()?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--validate-key-map-at="))
        {
            validate_key_map_at = Some(value.parse()?);
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--steps=")) {
            steps = value.parse()?;
        } else {
            path = Some(PathBuf::from(arg));
        }
    }
    let path = path.unwrap_or_else(|| PathBuf::from("../analysis/cab/smallos3kneorom.os3kos"));
    let recovery_seed_path = recovery_seed_path.unwrap_or_else(recovery_seed::default_seed_path);

    if headless {
        if reinit_memory {
            let saved = recovery_seed::generate_and_save_seed(&path, &recovery_seed_path)?;
            println!("recovery_seed_saved={}", saved.display());
        }
        let firmware = FirmwareRuntime::load_small_rom(&path)?;
        let is_full_system = firmware.is_neo_system_image();
        let mut session = if let Some(keys) = boot_keys_exact {
            FirmwareSession::boot_with_exact_keys(firmware, &keys, 50_000)?
        } else if let Some(keys) = boot_keys {
            FirmwareSession::boot_with_keys(firmware, &keys, 512)?
        } else if boot_left_shift_tab {
            FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?
        } else {
            FirmwareSession::boot_small_rom(firmware)?
        };
        if is_full_system
            && recovery_seed::apply_seed_file_if_present(&mut session, &recovery_seed_path)?
        {
            println!("recovery_seed_loaded={}", recovery_seed_path.display());
        }
        if let Some(path) = load_memory {
            let overlay = std::fs::read(&path)?;
            session.overlay_memory_bytes(&overlay);
            println!("memory_loaded={}", path.display());
        }
        if let Some(path) = dump_memory_start {
            std::fs::write(&path, session.memory_bytes())?;
            println!("memory_start={}", path.display());
        }
        let started_at = Instant::now();
        if validate_alpha_usb_native {
            session.run_realtime_cycles(220_000_000, 25_000_000);
            session
                .run_applet_message_for_validation("Alpha USB", 0x19, 200_000)
                .map_err(|error| anyhow::anyhow!("Alpha USB native validation failed: {error}"))?;
            let snapshot = session.snapshot();
            println!(
                "alpha_usb_native_validation=ok pc=0x{:08x} steps={} exception={}",
                snapshot.pc,
                snapshot.steps,
                snapshot.last_exception.as_deref().unwrap_or("none")
            );
            return Ok(());
        }
        if validate_forth_mini {
            session.run_realtime_cycles(220_000_000, 25_000_000);
            session
                .start_applet_message_for_validation("Forth Mini", 0x19)
                .map_err(|error| anyhow::anyhow!("Forth Mini validation failed: {error}"))?;
            session.run_until_pc_or_steps(0x0042_6752, 500_000);
            let snapshot = session.snapshot();
            if let Some(exception) = &snapshot.last_exception {
                let trace = snapshot
                    .trace
                    .iter()
                    .rev()
                    .take(12)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n  ");
                anyhow::bail!("Forth Mini validation failed: {exception}\n  {trace}");
            }
            for value in [b'1', b'2'] {
                session
                    .start_applet_message_with_param_for_validation(
                        "Forth Mini",
                        0x20,
                        u32::from(value),
                    )
                    .map_err(|error| anyhow::anyhow!("Forth Mini validation failed: {error}"))?;
                session.run_until_pc_or_steps(0x0042_6752, 500_000);
                let snapshot = session.snapshot();
                if let Some(exception) = &snapshot.last_exception {
                    let trace = snapshot
                        .trace
                        .iter()
                        .rev()
                        .take(12)
                        .cloned()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join("\n  ");
                    anyhow::bail!(
                        "Forth Mini validation failed after {:?}: {exception}\n  {trace}",
                        char::from(value)
                    );
                }
            }
            let typed_lcd = session.lcd_snapshot();
            session
                .start_applet_message_with_param_for_validation("Forth Mini", 0x20, u32::from(b'5'))
                .map_err(|error| anyhow::anyhow!("Forth Mini validation failed: {error}"))?;
            session.run_until_pc_or_steps(0x0042_6752, 500_000);
            let continued_lcd = session.lcd_snapshot();
            let input_hex = session.validation_applet_memory_hex(0x300 + 64 + 4, 3);
            if input_hex != "31 32 35" {
                let snapshot = session.snapshot();
                let trace = snapshot
                    .trace
                    .iter()
                    .rev()
                    .take(12)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n  ");
                anyhow::bail!(
                    "Forth Mini validation failed: input bytes did not reach REPL; pc=0x{:08x} exception={} input={}\n  {}",
                    snapshot.pc,
                    snapshot.last_exception.as_deref().unwrap_or("none"),
                    input_hex,
                    trace
                );
            }
            if typed_lcd.pixels == continued_lcd.pixels {
                let snapshot = session.snapshot();
                anyhow::bail!(
                    "Forth Mini validation failed: later 5 input did not change LCD; pc=0x{:08x} exception={} repl={}",
                    snapshot.pc,
                    snapshot.last_exception.as_deref().unwrap_or("none"),
                    session.validation_applet_memory_hex(0x300 + 64, 16)
                );
            }
            session
                .start_applet_message_with_param_for_validation(
                    "Forth Mini",
                    0x20,
                    u32::from(b'\r'),
                )
                .map_err(|error| anyhow::anyhow!("Forth Mini validation failed: {error}"))?;
            session.run_until_pc_or_steps(0x0042_6752, 500_000);
            let snapshot = session.snapshot();
            if let Some(exception) = &snapshot.last_exception {
                let trace = snapshot
                    .trace
                    .iter()
                    .rev()
                    .take(12)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n  ");
                anyhow::bail!("Forth Mini validation failed: {exception}\n  {trace}");
            }
            println!(
                "forth_mini_validation=ok pc=0x{:08x} steps={} exception={}",
                snapshot.pc,
                snapshot.steps,
                snapshot.last_exception.as_deref().unwrap_or("none")
            );
            return Ok(());
        }
        let effective_steps = scan_special_keys_at
            .or(scan_matrix_visibility_at)
            .or(validate_key_map_at)
            .unwrap_or(steps);
        let stopped_at_pc = run_headless_steps(
            &mut session,
            effective_steps,
            HeadlessEvents {
                type_events: &mut type_events,
                key_events: &mut key_events,
                all_row_key_events: &mut all_row_key_events,
                hold_events: &mut hold_events,
            },
            HeadlessStop {
                pc: stop_at_pc,
                pc_hit: stop_at_pc_hit,
                resource: stop_at_resource,
            },
            verbose,
        );
        if scan_special_keys_at.is_some() {
            scan_special_keys(&session);
            return Ok(());
        }
        if scan_matrix_visibility_at.is_some() {
            scan_matrix_visibility(&session);
            return Ok(());
        }
        if validate_key_map_at.is_some() {
            validate_key_map(&session);
            return Ok(());
        }
        let elapsed = started_at.elapsed();
        let snapshot = session.snapshot();
        let achieved_hz = if elapsed.is_zero() {
            0.0
        } else {
            snapshot.cycles as f64 / elapsed.as_secs_f64()
        };
        println!(
            "pc=0x{:08x} ssp=0x{:08x} steps={} cycles={} elapsed_ms={} achieved_hz={:.0} target_hz=33000000 stopped={} stop_at={} exception={}",
            snapshot.pc,
            snapshot.ssp,
            snapshot.steps,
            snapshot.cycles,
            elapsed.as_millis(),
            achieved_hz,
            snapshot.stopped,
            stopped_at_pc
                .map(|value| value.to_string())
                .unwrap_or_else(|| "n/a".to_string()),
            snapshot.last_exception.as_deref().unwrap_or("none")
        );
        if verbose {
            println!(
                "regs: d={:08x?} a={:08x?} usp=0x{:08x}",
                snapshot.d, snapshot.a, snapshot.usp
            );
            println!("debug words:");
            for (addr, value) in &snapshot.debug_words {
                println!("  0x{addr:08x}: 0x{value:08x}");
            }
            println!("mmio:");
            for access in &snapshot.mmio_accesses {
                println!("  {access}");
            }
            println!("trace:");
            for line in &snapshot.trace {
                println!("  {line}");
            }
        }
        if lcd_ranges {
            println!("lcd occupied x ranges:");
            for y in 0..snapshot.lcd.height {
                let mut ranges = Vec::new();
                let mut start = None;
                for x in 0..snapshot.lcd.width {
                    if snapshot.lcd.pixels[y * snapshot.lcd.width + x] {
                        start.get_or_insert(x);
                    } else if let Some(range_start) = start.take() {
                        ranges.push((range_start, x - 1));
                    }
                }
                if let Some(range_start) = start {
                    ranges.push((range_start, snapshot.lcd.width - 1));
                }
                if !ranges.is_empty() {
                    let ranges = ranges
                        .into_iter()
                        .map(|(min, max)| format!("{min:03}..{max:03}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    println!("  y={y:03}: {ranges}");
                }
            }
        }
        if lcd_ascii {
            println!("lcd ascii:");
            print!("{}", render_lcd_ascii(&snapshot.lcd, 4, 4));
        }
        if let Some(path) = lcd_pbm {
            write_lcd_pbm(&snapshot.lcd, &path)?;
            println!("lcd_pbm={}", path.display());
        }
        if let Some(path) = dump_memory {
            std::fs::write(&path, session.memory_bytes())?;
            println!("memory={}", path.display());
        }
        Ok(())
    } else {
        alpha_emu::gui::run(path)
    }
}

fn scan_special_keys(base_session: &FirmwareSession) {
    for raw in 0..=0x7f {
        let mut session = base_session.clone();
        session.tap_matrix_code_long(raw);
        let hit = session.run_until_resource_or_steps(0x006b, 3_000_000);
        if hit {
            println!("hit raw=0x{raw:02x}");
        }
    }
}

fn scan_matrix_visibility(base_session: &FirmwareSession) {
    let mut failures = Vec::new();
    let cells = matrix_cells();
    for cell in &cells {
        let raw = cell.raw.code();
        let mut session = base_session.clone();
        session.press_matrix_code(raw);
        let mut visible = false;
        for _ in 0..100 {
            session.run_steps(10_000);
            let snapshot = session.snapshot();
            visible = snapshot.mmio_accesses.iter().any(|access| {
                access.contains("/0xf419->")
                    && !access.ends_with("->0xff")
                    && !access.ends_with("->0x00")
            });
            if visible {
                break;
            }
        }
        if !visible {
            failures.push(format!(
                "raw=0x{raw:02x} row=0x{:02x} col={} logical=0x{:02x} label={}",
                cell.row,
                cell.col,
                cell.logical,
                matrix_key_label(raw)
            ));
        }
    }
    println!(
        "matrix_visibility checked={} visible={} failed={}",
        cells.len(),
        cells.len().saturating_sub(failures.len()),
        failures.len()
    );
    for failure in failures {
        println!("  {failure}");
    }
}

fn validate_key_map(base_session: &FirmwareSession) {
    const FILE_KEYS: &[(&str, u8)] = &[
        ("File 1", 0x4b),
        ("File 2", 0x4a),
        ("File 3", 0x0a),
        ("File 4", 0x1a),
        ("File 5", 0x19),
        ("File 6", 0x10),
        ("File 7", 0x02),
        ("File 8", 0x42),
    ];
    println!("key_map_validation");
    for (label, raw) in FILE_KEYS {
        let mut session = base_session.clone();
        session.tap_matrix_code_long(*raw);
        session.run_realtime_steps(20_000_000);
        let snapshot = session.snapshot();
        println!(
            "  {label}: raw=0x{raw:02x} current_slot_state=0x{:08x} pc=0x{:08x}",
            debug_word(&snapshot, 0x0000_35ec).unwrap_or(0),
            snapshot.pc
        );
    }

    for (label, raw) in [
        ("Applets", 0x46),
        ("Send", 0x47),
        ("Print", 0x49),
        ("Spell Check", 0x59),
        ("Find", 0x67),
        ("Clear File", 0x54),
    ] {
        let mut session = base_session.clone();
        session.tap_matrix_code_long(raw);
        let menu_hit = session.run_until_resource_or_steps(0x006b, 3_000_000);
        let snapshot = session.snapshot();
        println!(
            "  {label}: raw=0x{raw:02x} menu_hit={} pc=0x{:08x}",
            menu_hit, snapshot.pc
        );
    }
}

fn debug_word(snapshot: &alpha_emu::firmware_session::FirmwareSnapshot, addr: u32) -> Option<u32> {
    snapshot
        .debug_words
        .iter()
        .find_map(|(word_addr, value)| (*word_addr == addr).then_some(*value))
}

fn parse_type_event(value: &str) -> Result<(usize, String)> {
    let Some((step, text)) = value.split_once(':') else {
        anyhow::bail!("--type-at expects STEP:TEXT");
    };
    Ok((step.parse()?, text.to_string()))
}

fn parse_u32_arg(value: &str) -> Result<u32> {
    if let Some(hex) = value.strip_prefix("0x") {
        Ok(u32::from_str_radix(hex, 16)?)
    } else {
        Ok(value.parse()?)
    }
}

fn parse_key_list(value: &str) -> Result<Vec<u8>> {
    value
        .split(',')
        .map(|item| Ok(parse_u32_arg(item)? as u8))
        .collect()
}

struct HeadlessEvents<'a> {
    type_events: &'a mut [(usize, String)],
    key_events: &'a mut [(usize, u8)],
    all_row_key_events: &'a mut [(usize, u8)],
    hold_events: &'a mut [(usize, usize, u8)],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HeadlessStop {
    pc: Option<u32>,
    pc_hit: usize,
    resource: Option<u16>,
}

fn run_headless_steps(
    session: &mut FirmwareSession,
    total_steps: usize,
    events: HeadlessEvents<'_>,
    stop: HeadlessStop,
    keep_trace: bool,
) -> Option<bool> {
    let HeadlessEvents {
        type_events,
        key_events,
        all_row_key_events,
        hold_events,
    } = events;
    type_events.sort_by_key(|event| event.0);
    key_events.sort_by_key(|event| event.0);
    all_row_key_events.sort_by_key(|event| event.0);
    hold_events.sort_by_key(|event| event.0);
    let mut expanded_hold_events = Vec::with_capacity(hold_events.len() * 2);
    for (start, end, code) in hold_events.iter().copied() {
        expanded_hold_events.push((start, true, code));
        expanded_hold_events.push((end, false, code));
    }
    expanded_hold_events.sort_by_key(|event| event.0);
    let mut text_index = 0;
    let mut key_index = 0;
    let mut all_row_key_index = 0;
    let mut hold_index = 0;
    while text_index < type_events.len()
        || key_index < key_events.len()
        || all_row_key_index < all_row_key_events.len()
        || hold_index < expanded_hold_events.len()
    {
        let next_text_step = type_events
            .get(text_index)
            .map(|event| event.0)
            .unwrap_or(usize::MAX);
        let next_key_step = key_events
            .get(key_index)
            .map(|event| event.0)
            .unwrap_or(usize::MAX);
        let next_all_row_key_step = all_row_key_events
            .get(all_row_key_index)
            .map(|event| event.0)
            .unwrap_or(usize::MAX);
        let next_hold_step = expanded_hold_events
            .get(hold_index)
            .map(|event| event.0)
            .unwrap_or(usize::MAX);
        if next_text_step <= next_key_step
            && next_text_step <= next_all_row_key_step
            && next_text_step <= next_hold_step
        {
            let (step, text) = &type_events[text_index];
            run_until_step(session, *step, keep_trace);
            for value in text.chars() {
                session.tap_char(value);
                if run_short_settle(session, keep_trace, stop) {
                    return Some(true);
                }
            }
            text_index += 1;
        } else if next_key_step <= next_all_row_key_step && next_key_step <= next_hold_step {
            let (step, code) = key_events[key_index];
            run_until_step(session, step, keep_trace);
            session.tap_matrix_code_long(code);
            if run_short_settle(session, keep_trace, stop) {
                return Some(true);
            }
            key_index += 1;
        } else if next_all_row_key_step <= next_hold_step {
            let (step, code) = all_row_key_events[all_row_key_index];
            run_until_step(session, step, keep_trace);
            session.tap_matrix_code_all_rows(code);
            if run_short_settle(session, keep_trace, stop) {
                return Some(true);
            }
            all_row_key_index += 1;
        } else {
            let (step, pressed, code) = expanded_hold_events[hold_index];
            run_until_step(session, step, keep_trace);
            if pressed {
                session.press_matrix_code(code);
            } else {
                session.release_matrix_code(code);
            }
            if run_short_settle(session, keep_trace, stop) {
                return Some(true);
            }
            hold_index += 1;
        }
    }
    let current_steps = session.snapshot().steps;
    if current_steps < total_steps {
        if let Some(stop_pc) = stop.pc {
            if stop.pc_hit <= 1 {
                return Some(session.run_until_pc_or_steps(stop_pc, total_steps - current_steps));
            }
            return Some(session.run_until_pc_hit_or_steps(
                stop_pc,
                stop.pc_hit,
                total_steps - current_steps,
            ));
        }
        if let Some(resource_id) = stop.resource {
            return Some(
                session.run_until_resource_or_steps(resource_id, total_steps - current_steps),
            );
        }
        if keep_trace {
            session.run_steps(total_steps - current_steps);
        } else {
            session.run_realtime_steps(total_steps - current_steps);
        }
    }
    (stop.pc.is_some() || stop.resource.is_some()).then_some(false)
}

fn run_until_step(session: &mut FirmwareSession, step: usize, keep_trace: bool) {
    let current_steps = session.snapshot().steps;
    if current_steps < step {
        if keep_trace {
            session.run_steps(step - current_steps);
        } else {
            session.run_realtime_steps(step - current_steps);
        }
    }
}

fn run_short_settle(session: &mut FirmwareSession, keep_trace: bool, stop: HeadlessStop) -> bool {
    if let Some(stop_pc) = stop.pc {
        if stop.pc_hit > 1 {
            return session.run_until_pc_hit_or_steps(stop_pc, stop.pc_hit, 2_000);
        }
        return session.run_until_pc_or_steps(stop_pc, 2_000);
    }
    if keep_trace {
        session.run_steps(2_000);
    } else {
        session.run_realtime_steps(2_000);
    }
    false
}

fn parse_key_event(value: &str) -> Result<(usize, u8)> {
    let Some((step, key)) = value.split_once(':') else {
        anyhow::bail!("--key-at expects STEP:KEY");
    };
    Ok((step.parse()?, matrix_code_for_key_name(key)?))
}

fn parse_hold_event(value: &str) -> Result<(usize, usize, u8)> {
    let Some((range, key)) = value.split_once(':') else {
        anyhow::bail!("--hold-key expects START-END:KEY");
    };
    let Some((start, end)) = range.split_once('-') else {
        anyhow::bail!("--hold-key expects START-END:KEY");
    };
    Ok((start.parse()?, end.parse()?, matrix_code_for_key_name(key)?))
}

fn matrix_code_for_key_name(value: &str) -> Result<u8> {
    match value.to_ascii_lowercase().as_str() {
        "enter" | "return" => Ok(0x69),
        "up" => Ok(0x77),
        "down" => Ok(0x15),
        "left" => Ok(0x75),
        "right" => Ok(0x76),
        "esc" | "escape" => Ok(0x74),
        "tab" => Ok(0x0c),
        "backspace" => Ok(0x09),
        "applets" => Ok(0x46),
        "send" => Ok(0x47),
        "find" => Ok(0x67),
        "print" => Ok(0x49),
        "spell-check" | "spellcheck" => Ok(0x59),
        "clear-file" | "clearfile" => Ok(0x54),
        "file1" | "file-1" => Ok(0x4b),
        "file2" | "file-2" => Ok(0x4a),
        "file3" | "file-3" => Ok(0x0a),
        "file4" | "file-4" => Ok(0x1a),
        "file5" | "file-5" => Ok(0x19),
        "file6" | "file-6" => Ok(0x10),
        "file7" | "file-7" => Ok(0x02),
        "file8" | "file-8" => Ok(0x42),
        other if other.starts_with("0x") => Ok(u8::from_str_radix(&other[2..], 16)?),
        other => anyhow::bail!("unknown key name {other:?}"),
    }
}

fn render_lcd_ascii(snapshot: &LcdSnapshot, x_scale: usize, y_scale: usize) -> String {
    let mut output = String::new();
    for y in (0..snapshot.height).step_by(y_scale) {
        for x in (0..snapshot.width).step_by(x_scale) {
            let lit = (y..(y + y_scale).min(snapshot.height)).any(|pixel_y| {
                (x..(x + x_scale).min(snapshot.width))
                    .any(|pixel_x| snapshot.pixels[pixel_y * snapshot.width + pixel_x])
            });
            output.push(if lit { '#' } else { ' ' });
        }
        output.push('\n');
    }
    output
}

fn write_lcd_pbm(snapshot: &LcdSnapshot, path: &std::path::Path) -> Result<()> {
    let mut output = format!("P1\n{} {}\n", snapshot.width, snapshot.height);
    for y in 0..snapshot.height {
        for x in 0..snapshot.width {
            output.push(if snapshot.pixels[y * snapshot.width + x] {
                '1'
            } else {
                '0'
            });
            output.push(if x + 1 == snapshot.width { '\n' } else { ' ' });
        }
    }
    std::fs::write(path, output)?;
    Ok(())
}
