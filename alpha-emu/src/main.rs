use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use alpha_emu::firmware::FirmwareRuntime;
use alpha_emu::firmware_session::FirmwareSession;
use alpha_emu::keyboard::{logical_key_for_matrix_code, matrix_cells, matrix_key_label, matrix_text_key};
use alpha_emu::lcd::{
    LcdSnapshot, cursor_blink_snapshot, render_snapshot_bits, scale_snapshot, visible_snapshot,
};
use alpha_emu::recovery_seed;
use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut headless = false;
    let mut lcd_ascii = false;
    let mut lcd_visible_ascii = false;
    let mut lcd_bits = false;
    let mut lcd_bits_path = None;
    let mut lcd_ranges = false;
    let mut lcd_pbm = None;
    let mut lcd_visible_pbm = None;
    let mut lcd_blink_pbm_prefix = None;
    let mut lcd_dump_dir = None;
    let mut lcd_ocr = false;
    let mut lcd_ocr_scale = 4usize;
    let mut key_events = Vec::new();
    let mut hold_events = Vec::new();
    let mut all_row_key_events = Vec::new();
    let mut type_events = Vec::new();
    let mut key_now = Vec::new();
    let mut type_now = Vec::new();
    let mut stop_at_pc = None;
    let mut stop_at_pc_hit = 1usize;
    let mut stop_at_resource = None;
    let mut trace_stack_at_pc = None;
    let mut trace_stack_at_pc_hit = 1usize;
    let mut scan_special_keys_at = None;
    let mut scan_matrix_visibility_at = None;
    let mut validate_key_map_at = None;
    let mut validate_alpha_usb_native = false;
    let mut validate_forth_mini = false;
    let mut validate_basic_writer = false;
    let mut validate_write_or_die = false;
    let mut validate_floppy_bird = false;
    let mut validate_snake = false;
    let mut probe_basic_writer_key = None;
    let mut load_memory = None;
    let mut dump_memory_start = None;
    let mut dump_memory = None;
    let mut reinit_memory = false;
    let mut recovery_seed_path = None;
    let mut sample_lcd_after = None;
    let mut launch_forth_mini = false;
    let mut launch_calculator = false;
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
        } else if arg == "--lcd-visible-ascii" {
            lcd_visible_ascii = true;
        } else if arg == "--lcd-bits" {
            lcd_bits = true;
        } else if arg == "--lcd-ranges" {
            lcd_ranges = true;
        } else if arg == "--lcd-ocr" {
            lcd_ocr = true;
        } else if arg == "--launch-forth-mini" {
            launch_forth_mini = true;
        } else if arg == "--launch-calculator" {
            launch_calculator = true;
        } else if arg == "--verbose" {
            verbose = true;
        } else if arg == "--validate-alpha-usb-native" {
            validate_alpha_usb_native = true;
        } else if arg == "--validate-forth-mini" {
            validate_forth_mini = true;
        } else if arg == "--validate-basic-writer" {
            validate_basic_writer = true;
        } else if arg == "--validate-write-or-die" {
            validate_write_or_die = true;
        } else if arg == "--validate-floppy-bird" {
            validate_floppy_bird = true;
        } else if arg == "--validate-snake" {
            validate_snake = true;
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--probe-basic-writer-key="))
        {
            probe_basic_writer_key = Some(matrix_code_for_key_name(value)?);
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
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--lcd-visible-pbm="))
        {
            lcd_visible_pbm = Some(PathBuf::from(value));
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--lcd-bits-path="))
        {
            lcd_bits_path = Some(PathBuf::from(value));
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--lcd-blink-pbm-prefix="))
        {
            lcd_blink_pbm_prefix = Some(PathBuf::from(value));
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--lcd-dump-dir="))
        {
            lcd_dump_dir = Some(PathBuf::from(value));
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--lcd-ocr-scale="))
        {
            lcd_ocr_scale = value.parse()?;
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--sample-lcd-after="))
        {
            sample_lcd_after = Some(parse_sample_lcd_after(value)?);
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
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--type-now=")) {
            type_now.push(value.to_string());
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--key-now=")) {
            key_now.extend(parse_key_name_list(value)?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--stop-at-pc="))
        {
            stop_at_pc = Some(parse_u32_arg(value)?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--trace-stack-at-pc="))
        {
            trace_stack_at_pc = Some(parse_u32_arg(value)?);
        } else if let Some(value) = arg
            .to_str()
            .and_then(|arg| arg.strip_prefix("--trace-stack-at-pc-hit="))
        {
            trace_stack_at_pc_hit = value.parse()?;
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
    headless |= validate_alpha_usb_native
        || validate_forth_mini
        || validate_basic_writer
        || validate_write_or_die
        || validate_floppy_bird
        || validate_snake
        || probe_basic_writer_key.is_some()
        || lcd_ascii
        || lcd_visible_ascii
        || lcd_bits
        || lcd_bits_path.is_some()
        || lcd_ranges
        || lcd_pbm.is_some()
        || lcd_visible_pbm.is_some()
        || lcd_blink_pbm_prefix.is_some()
        || lcd_dump_dir.is_some()
        || lcd_ocr
        || !key_events.is_empty()
        || !hold_events.is_empty()
        || !all_row_key_events.is_empty()
        || !type_events.is_empty()
        || !key_now.is_empty()
        || !type_now.is_empty()
        || stop_at_pc.is_some()
        || stop_at_resource.is_some()
        || scan_special_keys_at.is_some()
        || scan_matrix_visibility_at.is_some()
        || validate_key_map_at.is_some()
        || load_memory.is_some()
        || dump_memory_start.is_some()
        || dump_memory.is_some()
        || reinit_memory
        || sample_lcd_after.is_some();

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
        } else if (launch_forth_mini || launch_calculator) && is_full_system {
            FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?
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
        if launch_forth_mini {
            if !is_full_system {
                anyhow::bail!("--launch-forth-mini requires the full NEO system firmware image");
            }
            session.run_realtime_cycles(220_000_000, 25_000_000);
            launch_forth_mini_for_debugging(&mut session)?;
        } else if launch_calculator {
            if !is_full_system {
                anyhow::bail!("--launch-calculator requires the full NEO system firmware image");
            }
            session.run_realtime_cycles(220_000_000, 25_000_000);
            launch_calculator_for_debugging(&mut session)?;
        }
        if let Some(path) = load_memory {
            let overlay = std::fs::read(&path)?;
            session.overlay_memory_bytes(&overlay);
            println!("memory_loaded={}", path.display());
        }
        session.set_trace_stack_at_pc(trace_stack_at_pc, trace_stack_at_pc_hit);
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
            if !is_full_system {
                anyhow::bail!("Forth Mini validation requires the full NEO system firmware image");
            }
            let firmware = FirmwareRuntime::load_small_rom(&path)?;
            let mut session = FirmwareSession::boot_small_rom(firmware)?;
            session.set_trace_stack_at_pc(trace_stack_at_pc, trace_stack_at_pc_hit);
            recovery_seed::apply_seed_file_if_present(&mut session, &recovery_seed_path)?;
            session.run_realtime_cycles(220_000_000, 25_000_000);
            launch_forth_mini_through_menu(&mut session);
            bail_if_exception(&session, "Forth Mini focus")?;
            print_ocr_checkpoint("forth_mini_focus", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            enter_forth_line_and_assert(
                &mut session,
                ": sq dup * ;",
                "ok",
                "Forth Mini define sq",
            )?;
            enter_forth_line_and_assert(&mut session, "7 sq .", "49", "Forth Mini sq eval 7")?;
            enter_forth_line_and_assert(&mut session, "8 sq .", "64", "Forth Mini sq eval 8")?;
            enter_forth_line_and_assert(&mut session, "9 sq .", "81", "Forth Mini sq eval 9")?;
            print_ocr_checkpoint("forth_mini_after_sq", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;

            enter_forth_line_and_assert(
                &mut session,
                ": ch if 11 else 22 then ;",
                "ok",
                "Forth Mini define ch",
            )?;
            enter_forth_line_and_assert(
                &mut session,
                "0 ch .",
                "22",
                "Forth Mini if/else false 1",
            )?;
            enter_forth_line_and_assert(
                &mut session,
                "1 ch .",
                "11",
                "Forth Mini if/else true 1",
            )?;
            enter_forth_line_and_assert(
                &mut session,
                "0 ch .",
                "22",
                "Forth Mini if/else false 2",
            )?;
            enter_forth_line_and_assert(
                &mut session,
                "1 ch .",
                "11",
                "Forth Mini if/else true 2",
            )?;
            print_ocr_checkpoint("forth_mini_if_else", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;

            enter_forth_line(&mut session, ": d begin dup while dup . 1 - repeat drop ;");
            bail_if_exception(&session, "Forth Mini define d")?;
            enter_forth_line(&mut session, "3 d");
            bail_if_exception(&session, "Forth Mini while/repeat")?;
            assert_forth_screen_contains(&session, "3 2 1", "Forth Mini while/repeat")?;
            print_ocr_checkpoint("forth_mini_while_repeat", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;

            exit_forth_to_menu(&mut session);
            bail_if_exception(&session, "Forth Mini exit to menu")?;
            relaunch_current_menu_item(&mut session);
            bail_if_exception(&session, "Forth Mini relaunch")?;
            enter_forth_line(&mut session, "8 sq .");
            bail_if_exception(&session, "Forth Mini persistence reload first")?;
            if !session.snapshot().text_screen.unwrap_or_default().contains("64") {
                print_forth_state_ascii_runs(&session, "forth_mini_relaunch_state_after_first");
            }
            assert_forth_screen_contains(&session, "64", "Forth Mini persistence reload")?;
            print_ocr_checkpoint("forth_mini_relaunch", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            let snapshot = session.snapshot();
            println!(
                "forth_mini_validation=ok pc=0x{:08x} steps={} exception={}",
                snapshot.pc,
                snapshot.steps,
                snapshot.last_exception.as_deref().unwrap_or("none")
            );
            return Ok(());
        }
        if validate_basic_writer {
            if !is_full_system {
                anyhow::bail!("Basic Writer validation requires the full NEO system firmware image");
            }
            let firmware = FirmwareRuntime::load_small_rom(&path)?;
            let mut session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
            recovery_seed::apply_seed_file_if_present(&mut session, &recovery_seed_path)?;
            session.run_realtime_cycles(220_000_000, 25_000_000);
            launch_basic_writer_through_menu(&mut session, lcd_ocr_scale)?;
            session.run_steps(1_500_000);
            bail_if_exception(&session, "Basic Writer focus")?;
            print_ocr_checkpoint("basic_writer_focus", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            assert_basic_writer_state(&session, 1, "", 0, 0)?;
            type_text_via_matrix(&mut session, "abc")?;
            wait_for_basic_writer_state(&mut session, |state| state.len == 3 && state.cursor == 3)?;
            assert_basic_writer_state(&session, 1, "abc", 3, 3)?;
            tap_key_now(&mut session, 0x75);
            wait_for_basic_writer_state(&mut session, |state| state.cursor == 2)?;
            assert_basic_writer_state(&session, 1, "abc", 3, 2)?;
            switch_basic_writer_file(&mut session, 2)?;
            assert_basic_writer_state(&session, 2, "", 0, 0)?;
            assert_basic_writer_slot_state(&session, 1, "abc", 3, 2)?;
            assert_basic_writer_banner(&session, 2, "Basic Writer file2 banner")?;
            type_text_via_matrix(&mut session, "xy")?;
            wait_for_basic_writer_state(&mut session, |state| state.len == 2 && state.cursor == 2)?;
            assert_basic_writer_state(&session, 2, "xy", 2, 2)?;
            wait_for_basic_writer_banner_to_clear(&mut session)?;
            print_ocr_checkpoint("basic_writer_file2", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            switch_basic_writer_file(&mut session, 1)?;
            assert_basic_writer_state(&session, 1, "abc", 3, 2)?;
            assert_basic_writer_slot_state(&session, 2, "xy", 2, 2)?;
            assert_basic_writer_banner(&session, 1, "Basic Writer file1 banner")?;
            exit_basic_writer_to_menu(&mut session);
            bail_if_exception(&session, "Basic Writer exit to menu")?;
            relaunch_current_menu_item(&mut session);
            session.run_steps(1_500_000);
            bail_if_exception(&session, "Basic Writer relaunch")?;
            switch_basic_writer_file(&mut session, 1)?;
            assert_basic_writer_state(&session, 1, "abc", 3, 2)?;
            switch_basic_writer_file(&mut session, 2)?;
            assert_basic_writer_state(&session, 2, "xy", 2, 2)?;
            print_ocr_checkpoint("basic_writer_relaunch", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            let snapshot = session.snapshot();
            println!(
                "basic_writer_validation=ok pc=0x{:08x} steps={} exception={}",
                snapshot.pc,
                snapshot.steps,
                snapshot.last_exception.as_deref().unwrap_or("none")
            );
            return Ok(());
        }
        if validate_write_or_die {
            if !is_full_system {
                anyhow::bail!("WriteOrDie validation requires the full NEO system firmware image");
            }
            let firmware = FirmwareRuntime::load_small_rom(&path)?;
            validate_write_or_die_file_keys_through_menu(firmware.clone(), &recovery_seed_path)?;
            let mut session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
            recovery_seed::apply_seed_file_if_present(&mut session, &recovery_seed_path)?;
            session.run_realtime_cycles(220_000_000, 25_000_000);
            focus_write_or_die_direct(&mut session)?;
            bail_if_exception(&session, "WriteOrDie focus")?;
            print_ocr_checkpoint("write_or_die_setup", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            assert_write_or_die_state(&session, 0, 0, 500, 600, 10, "", "WriteOrDie defaults")?;
            seed_write_or_die_completed_state(&mut session, "one two three four five");
            print_ocr_checkpoint("write_or_die_completed", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            assert_write_or_die_text_prefix(&session, "one two", "WriteOrDie completed text")?;
            assert_write_or_die_penalty_feedback(&mut session)?;
            seed_write_or_die_completed_state(&mut session, "one two three four five");
            if std::env::var_os("WOD_SKIP_EXPORT").is_none() {
                let write_or_die_base = write_or_die_state_base(&session).unwrap_or_default() as usize;
                let alphaword_before = alphaword_file_records(session.memory_bytes());
                session.clear_trace();
                send_write_or_die_key_direct(&mut session, 0x2c)?;
                let state = write_or_die_state_at(&session, write_or_die_base);
                if state.phase != 3 || state.export_slot != 2 || state.export_status != 1 {
                    let trace = session
                        .snapshot()
                        .trace
                        .into_iter()
                        .rev()
                        .take(16)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join("\n  ");
                    let base = write_or_die_base;
                    let snapshot = session.snapshot();
                    let a5_base = snapshot.a[5].saturating_add(0x300) as usize;
                    let bytes = session.memory_bytes();
                    let state_hex = bytes
                        .get(base..base + 16)
                        .unwrap_or(&[])
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let wod_protocol_hex = bytes
                        .get(base + 0x130..base + 0x210)
                        .unwrap_or(&[])
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let a5_state_hex = bytes
                        .get(a5_base..a5_base + 16)
                        .unwrap_or(&[])
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let validation_status = read_be_u32(bytes, 0x1200).unwrap_or_default();
                    let alphaword_base = session
                        .applet_state_base_for_validation("AlphaWord Plus")
                        .unwrap_or_default() as usize;
                    let alphaword_hex = bytes
                        .get(alphaword_base + 0x130..alphaword_base + 0x210)
                        .unwrap_or(&[])
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let alphaword_transfer_hex = bytes
                        .get(alphaword_base + 0x1d0..alphaword_base + 0x220)
                        .unwrap_or(&[])
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    anyhow::bail!("WriteOrDie AlphaWord export failed: {:?} validation_status=0x{validation_status:08x} base=0x{base:08x} state={state_hex} wod_0130_020f={wod_protocol_hex} a5_base=0x{a5_base:08x} a5_state={a5_state_hex} alphaword_base=0x{alphaword_base:08x} aw_0130_020f={alphaword_hex} aw_01d0_021f={alphaword_transfer_hex}\n  {}", state, trace);
                }
                let marker_offsets = validate_alphaword_export_records(
                    &alphaword_before,
                    session.memory_bytes(),
                    backing_alphaword_slot(2),
                    b"WriteOrDie session",
                )?;
                if marker_offsets.len() != 1 {
                    anyhow::bail!(
                        "WriteOrDie AlphaWord export should touch exactly one backing File 1 buffer for visible File 2; got {}: {}",
                        marker_offsets.len(),
                        marker_offsets.join(",")
                    );
                }
                println!("write_or_die_export_visible_file2_backing_slot{}_buffers={}", backing_alphaword_slot(2), marker_offsets.join(","));
                seed_write_or_die_completed_state(&mut session, "six seven");
                let before_second = alphaword_file_records(session.memory_bytes());
                let used_before_second = max_alphaword_slot_used(&before_second, backing_alphaword_slot(2));
                send_write_or_die_key_direct(&mut session, 0x2c)?;
                let second_state = write_or_die_state_at(&session, write_or_die_base);
                if second_state.phase != 3 || second_state.export_slot != 2 || second_state.export_status != 1 {
                    anyhow::bail!("WriteOrDie second AlphaWord export failed: {:?}", second_state);
                }
                let second_offsets = validate_alphaword_export_records(
                    &before_second,
                    session.memory_bytes(),
                    backing_alphaword_slot(2),
                    b"six seven",
                )?;
                let used_after_second = max_alphaword_slot_used(&alphaword_file_records(session.memory_bytes()), backing_alphaword_slot(2));
                if second_offsets.is_empty() || used_after_second <= used_before_second {
                    anyhow::bail!(
                        "WriteOrDie second AlphaWord export did not append: offsets={:?} before_used={} after_used={}",
                        second_offsets,
                        used_before_second,
                        used_after_second
                    );
                }
                println!("write_or_die_second_export_file2_buffers={}", second_offsets.join(","));
            }

            let snapshot = session.snapshot();
            println!(
                "write_or_die_validation=ok pc=0x{:08x} steps={} exception={}",
                snapshot.pc,
                snapshot.steps,
                snapshot.last_exception.as_deref().unwrap_or("none")
            );
            return Ok(());
        }
        if validate_floppy_bird {
            if !is_full_system {
                anyhow::bail!("Floppy Bird validation requires the full NEO system firmware image");
            }
            let firmware = FirmwareRuntime::load_small_rom(&path)?;
            let mut session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
            recovery_seed::apply_seed_file_if_present(&mut session, &recovery_seed_path)?;
            session.run_realtime_cycles(220_000_000, 25_000_000);
            focus_floppy_bird_direct(&mut session)?;
            bail_if_exception(&session, "Floppy Bird focus")?;
            print_ocr_checkpoint("floppy_bird_focus", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            assert_floppy_bird_state(&session, 0, false, "Floppy Bird initial state")?;
            let before_flap = floppy_bird_state(&session);
            dispatch_floppy_bird_message(&mut session, 0x21, 0x4c, "Floppy Bird space flap")?;
            force_floppy_bird_tick(&mut session, "Floppy Bird post-flap tick")?;
            let after_flap = floppy_bird_state(&session);
            if after_flap.bird_y_q8 >= before_flap.bird_y_q8 {
                anyhow::bail!("Floppy Bird flap did not move upward: before={before_flap:?} after={after_flap:?}");
            }
            seed_floppy_bird_near_score(&mut session);
            force_floppy_bird_tick(&mut session, "Floppy Bird score tick")?;
            assert_floppy_bird_state(&session, 1, false, "Floppy Bird scoring")?;
            seed_floppy_bird_crash(&mut session);
            force_floppy_bird_tick(&mut session, "Floppy Bird crash tick")?;
            assert_floppy_bird_state(&session, 1, true, "Floppy Bird game over")?;
            dispatch_floppy_bird_message(&mut session, 0x21, 0x48, "Floppy Bird escape exit")?;
            let status = read_be_u32(session.memory_bytes(), 0x1200).unwrap_or_default();
            if status != 7 {
                anyhow::bail!("Floppy Bird Escape should return applet exit status 7, got {status}");
            }
            let snapshot = session.snapshot();
            println!(
                "floppy_bird_validation=ok pc=0x{:08x} steps={} exception={}",
                snapshot.pc,
                snapshot.steps,
                snapshot.last_exception.as_deref().unwrap_or("none")
            );
            return Ok(());
        }
        if validate_snake {
            if !is_full_system {
                anyhow::bail!("Snake validation requires the full NEO system firmware image");
            }
            let firmware = FirmwareRuntime::load_small_rom(&path)?;
            let mut session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
            recovery_seed::apply_seed_file_if_present(&mut session, &recovery_seed_path)?;
            session.run_realtime_cycles(220_000_000, 25_000_000);
            focus_snake_direct(&mut session)?;
            bail_if_exception(&session, "Snake focus")?;
            assert_snake_state(&session, 0, 3, false, false, "Snake initial")?;
            let initial_pixels = lcd_visible_lit_pixels(&session.snapshot().lcd);
            if initial_pixels < 400 {
                anyhow::bail!("Snake should render pixel graphics over the LCD, got only {initial_pixels} lit pixels");
            }
            let sidebar_pixels = lcd_lit_pixels_in_rect(&session.snapshot().lcd, 200, 0, 64, 64);
            if sidebar_pixels < 300 {
                anyhow::bail!("Snake should render title/help/length in the 64px sidebar, got {sidebar_pixels} lit sidebar pixels");
            }
            let before_turn = snake_state(&session);
            dispatch_snake_message(&mut session, 0x21, 0x0d, "Snake down arrow")?;
            force_snake_tick(&mut session, "Snake down movement")?;
            let after_turn = snake_state(&session);
            if after_turn.head_y <= before_turn.head_y {
                anyhow::bail!("Snake down arrow did not move the head downward: before={before_turn:?} after={after_turn:?}");
            }
            seed_snake_food_ahead(&mut session);
            force_snake_tick(&mut session, "Snake food tick")?;
            assert_snake_state(&session, 1, 4, false, false, "Snake scoring/growth")?;
            dispatch_snake_message(&mut session, 0x21, 0x1e, "Snake pause key")?;
            let paused = snake_state(&session);
            force_snake_tick(&mut session, "Snake paused tick")?;
            let still_paused = snake_state(&session);
            if !still_paused.paused || still_paused.head_x != paused.head_x || still_paused.head_y != paused.head_y {
                anyhow::bail!("Snake pause should stop movement: before={paused:?} after={still_paused:?}");
            }
            dispatch_snake_message(&mut session, 0x21, 0x23, "Snake restart key")?;
            assert_snake_state(&session, 0, 3, false, false, "Snake restart")?;
            seed_snake_edge_wrap(&mut session);
            force_snake_tick(&mut session, "Snake edge wrap")?;
            let wrapped = snake_state(&session);
            if wrapped.game_over || wrapped.head_x != 0 {
                anyhow::bail!("Snake should wrap from right edge to left edge, got {wrapped:?}");
            }
            dispatch_snake_message(&mut session, 0x21, 0x48, "Snake escape exit")?;
            let status = read_be_u32(session.memory_bytes(), 0x1200).unwrap_or_default();
            if status != 7 {
                anyhow::bail!("Snake Escape should return applet exit status 7, got {status}");
            }
            focus_snake_direct(&mut session)?;
            dispatch_snake_message(&mut session, 0x21, 0x29, "Snake applets exit")?;
            let status = read_be_u32(session.memory_bytes(), 0x1200).unwrap_or_default();
            if status != 7 {
                anyhow::bail!("Snake Applets should return applet exit status 7, got {status}");
            }
            let snapshot = session.snapshot();
            println!(
                "snake_validation=ok pc=0x{:08x} steps={} exception={} pixels={}",
                snapshot.pc,
                snapshot.steps,
                snapshot.last_exception.as_deref().unwrap_or("none"),
                initial_pixels
            );
            return Ok(());
        }
        if let Some(key) = probe_basic_writer_key {
            if !is_full_system {
                anyhow::bail!("Basic Writer probing requires the full NEO system firmware image");
            }
            let firmware = FirmwareRuntime::load_small_rom(&path)?;
            let mut session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
            recovery_seed::apply_seed_file_if_present(&mut session, &recovery_seed_path)?;
            session.run_realtime_cycles(220_000_000, 25_000_000);
            launch_basic_writer_through_menu(&mut session, lcd_ocr_scale)?;
            bail_if_exception(&session, "Basic Writer focus")?;
            print_ocr_checkpoint("basic_writer_probe_focus", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            print_basic_writer_state("basic_writer_probe_focus_state", &session);
            session.press_matrix_code(key);
            let before = basic_writer_state(&session);
            wait_for_basic_writer_state(&mut session, |state| {
                state.preview != before.preview
                    || state.len != before.len
                    || state.active_slot != before.active_slot
            })?;
            session.release_matrix_code(key);
            session.run_steps(3_000_000);
            bail_if_exception(&session, "Basic Writer probe key")?;
            print_ocr_checkpoint("basic_writer_probe_after_key", &session.snapshot(), lcd_ocr, lcd_ocr_scale)?;
            print_basic_writer_state("basic_writer_probe_after_key_state", &session);
            let snapshot = session.snapshot();
            println!(
                "basic_writer_probe=ok pc=0x{:08x} steps={} exception={}",
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
        if let Some((interval_steps, count)) = sample_lcd_after {
            print_lcd_samples(&mut session, interval_steps, count);
        }
        for text in &type_now {
            if launch_forth_mini {
                type_text_direct_to_forth(&mut session, text)?;
            } else {
                type_text_now(&mut session, text);
            }
        }
        for code in &key_now {
            if launch_forth_mini {
                send_key_direct_to_forth(&mut session, *code)?;
            } else {
                tap_key_now(&mut session, *code);
            }
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
        let visible_lcd = visible_snapshot(&snapshot.lcd);
        if lcd_visible_ascii {
            println!("lcd visible ascii:");
            print!("{}", render_lcd_ascii(&visible_lcd, 2, 2));
        }
        if lcd_bits {
            println!("lcd bits:");
            print!("{}", render_snapshot_bits(&snapshot.lcd));
        }
        if let Some(path) = lcd_bits_path {
            std::fs::write(&path, render_snapshot_bits(&snapshot.lcd))?;
            println!("lcd_bits={}", path.display());
        }
        if let Some(path) = lcd_pbm {
            write_lcd_pbm(&snapshot.lcd, &path)?;
            println!("lcd_pbm={}", path.display());
        }
        if let Some(path) = lcd_visible_pbm {
            write_lcd_pbm(&visible_lcd, &path)?;
            println!("lcd_visible_pbm={}", path.display());
        }
        if let Some(prefix) = lcd_blink_pbm_prefix {
            let on_path = prefixed_path(&prefix, "on.pbm");
            let off_path = prefixed_path(&prefix, "off.pbm");
            let on = cursor_blink_snapshot(&snapshot.lcd, true);
            let off = cursor_blink_snapshot(&snapshot.lcd, false);
            write_lcd_pbm(&on, &on_path)?;
            write_lcd_pbm(&off, &off_path)?;
            let diff = on
                .pixels
                .iter()
                .zip(&off.pixels)
                .filter(|(left, right)| left != right)
                .count();
            println!(
                "lcd_blink_pbm_on={} off={} diff_pixels={diff}",
                on_path.display(),
                off_path.display()
            );
        }
        if lcd_ocr {
            let ocr_text = ocr_visible_lcd(snapshot.text_screen.as_deref(), &snapshot.lcd, lcd_ocr_scale)?;
            println!("lcd_ocr:\n{}", ocr_text.trim_end());
        }
        if let Some(dir) = lcd_dump_dir {
            write_lcd_debug_dump(snapshot.text_screen.as_deref(), &snapshot.lcd, &dir, lcd_ocr_scale)?;
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

fn bail_if_exception(session: &FirmwareSession, label: &str) -> Result<()> {
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
        anyhow::bail!("{label} failed: {exception}\n  {trace}");
    }
    Ok(())
}

fn print_ocr_checkpoint(label: &str, snapshot: &alpha_emu::firmware_session::FirmwareSnapshot, enabled: bool, scale: usize) -> Result<()> {
    if !enabled {
        return Ok(());
    }
    let text = ocr_visible_lcd(snapshot.text_screen.as_deref(), &snapshot.lcd, scale)?;
    println!("{label}:\n{}", text.trim_end());
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct BasicWriterState {
    active_slot: u32,
    banner_slot: u32,
    preview: String,
    len: u32,
    cursor: u32,
    viewport: u32,
    banner_until_ms: u32,
}

fn basic_writer_state(session: &FirmwareSession) -> BasicWriterState {
    let active_slot = basic_writer_active_slot(session);
    let mut state = basic_writer_slot_state(session, active_slot);
    state.active_slot = active_slot;
    state.banner_slot = basic_writer_banner_slot(session);
    state.banner_until_ms = basic_writer_banner_until_ms(session);
    state
}

fn basic_writer_active_slot(session: &FirmwareSession) -> u32 {
    let snapshot = session.snapshot();
    let a5 = snapshot.a[5];
    let state_base = a5.saturating_add(0x300) as usize;
    let bytes = session.memory_bytes();
    let read_u32 = |offset: usize| -> Option<u32> {
        let start = state_base.checked_add(offset)?;
        let chunk = bytes.get(start..start + 4)?;
        Some(u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
    };
    read_u32(0).unwrap_or_default()
}

fn basic_writer_banner_slot(session: &FirmwareSession) -> u32 {
    let snapshot = session.snapshot();
    let a5 = snapshot.a[5];
    let state_base = a5.saturating_add(0x300) as usize;
    let bytes = session.memory_bytes();
    let start = state_base + 4;
    bytes
        .get(start..start + 4)
        .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .unwrap_or_default()
}

fn basic_writer_banner_until_ms(session: &FirmwareSession) -> u32 {
    let snapshot = session.snapshot();
    let a5 = snapshot.a[5];
    let state_base = a5.saturating_add(0x300) as usize;
    let bytes = session.memory_bytes();
    let start = state_base + 8;
    bytes
        .get(start..start + 4)
        .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .unwrap_or_default()
}

fn basic_writer_slot_state(session: &FirmwareSession, slot: u32) -> BasicWriterState {
    let snapshot = session.snapshot();
    let a5 = snapshot.a[5];
    let state_base = a5.saturating_add(0x300) as usize;
    let bytes = session.memory_bytes();
    const SLOT_OFFSET: usize = 20;
    const SLOT_BYTES: usize = 268;
    let slot_index = slot.saturating_sub(1).min(7) as usize;
    let slot_base = state_base + SLOT_OFFSET + slot_index * SLOT_BYTES;
    let len = bytes
        .get(slot_base..slot_base + 4)
        .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .unwrap_or_default();
    let cursor = bytes
        .get(slot_base + 4..slot_base + 8)
        .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .unwrap_or_default();
    let viewport = bytes
        .get(slot_base + 8..slot_base + 12)
        .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .unwrap_or_default();
    let preview = bytes
        .get(slot_base + 12..slot_base + 20)
        .unwrap_or(&[])
        .iter()
        .map(|byte| match *byte {
            b' '..=b'~' => *byte as char,
            b'\n' => 'n',
            _ => '.',
        })
        .collect::<String>();
    BasicWriterState {
        active_slot: slot,
        banner_slot: 0,
        preview,
        len,
        cursor,
        viewport,
        banner_until_ms: 0,
    }
}

fn print_basic_writer_state(label: &str, session: &FirmwareSession) {
    let state = basic_writer_state(session);
    let snapshot = session.snapshot();
    println!(
        "{label}: slot={} preview={:?} len={} cursor={} viewport={} banner_slot={} banner_until={} pc=0x{:08x} stopped={}",
        state.active_slot,
        state.preview,
        state.len,
        state.cursor,
        state.viewport,
        state.banner_slot,
        state.banner_until_ms,
        snapshot.pc,
        snapshot.stopped
    );
}

fn assert_basic_writer_state(
    session: &FirmwareSession,
    expected_slot: u32,
    expected_preview: &str,
    expected_len: u32,
    expected_cursor: u32,
) -> Result<()> {
    let state = basic_writer_state(session);
    if state.active_slot != expected_slot
        || !state.preview.starts_with(expected_preview)
        || state.len != expected_len
        || state.cursor != expected_cursor
    {
        anyhow::bail!(
            "Basic Writer state mismatch: expected slot {} preview prefix {:?} len {} cursor {}, got slot {} preview {:?} len {} cursor {}",
            expected_slot,
            expected_preview,
            expected_len,
            expected_cursor,
            state.active_slot,
            state.preview,
            state.len,
            state.cursor,
        );
    }
    Ok(())
}

fn assert_basic_writer_slot_state(
    session: &FirmwareSession,
    slot: u32,
    expected_preview: &str,
    expected_len: u32,
    expected_cursor: u32,
) -> Result<()> {
    let state = basic_writer_slot_state(session, slot);
    if !state.preview.starts_with(expected_preview) || state.len != expected_len || state.cursor != expected_cursor {
        anyhow::bail!(
            "Basic Writer slot {} mismatch: expected preview prefix {:?} len {} cursor {}, got preview {:?} len {} cursor {}",
            slot,
            expected_preview,
            expected_len,
            expected_cursor,
            state.preview,
            state.len,
            state.cursor,
        );
    }
    Ok(())
}

fn assert_basic_writer_banner(session: &FirmwareSession, slot: u32, label: &str) -> Result<()> {
    let state = basic_writer_state(session);
    let text = session.snapshot().text_screen.unwrap_or_default();
    let expected = format!("File {slot}");
    if state.banner_slot != slot || !text.contains(&expected) {
        anyhow::bail!(
            "{label}: expected banner {:?}, got banner_slot={} screen {:?}",
            expected,
            state.banner_slot,
            text
        );
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct WriteOrDieState {
    phase: u32,
    selected_setup_row: u32,
    goal_mode: u32,
    word_goal: u32,
    time_goal_seconds: u32,
    grace_seconds: u32,
    len: u32,
    cursor: u32,
    viewport: u32,
    preview: String,
    start_ms: u32,
    last_activity_ms: u32,
    final_word_count: u32,
    remaining_seconds_estimate: u32,
    export_slot: u32,
    export_status: u32,
    flash_on: u32,
}

fn write_or_die_state(session: &FirmwareSession) -> WriteOrDieState {
    let base = write_or_die_state_base(session).unwrap_or_default() as usize;
    write_or_die_state_at(session, base)
}

fn write_or_die_state_at(session: &FirmwareSession, base: usize) -> WriteOrDieState {
    let bytes = session.memory_bytes();
    let read_u32 = |offset: usize| -> u32 {
        bytes
            .get(base + offset..base + offset + 4)
            .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .unwrap_or_default()
    };
    let text_base = base + 36;
    let preview = bytes
        .get(text_base..text_base + 24)
        .unwrap_or(&[])
        .iter()
        .map(|byte| match *byte {
            b' '..=b'~' => *byte as char,
            b'\n' => 'n',
            _ => '.',
        })
        .collect::<String>();
    WriteOrDieState {
        phase: read_u32(0),
        selected_setup_row: read_u32(4),
        goal_mode: read_u32(8),
        word_goal: read_u32(12),
        time_goal_seconds: read_u32(16),
        grace_seconds: read_u32(20),
        len: read_u32(24),
        cursor: read_u32(28),
        viewport: read_u32(32),
        preview,
        start_ms: read_u32(804),
        last_activity_ms: read_u32(808),
        final_word_count: read_u32(816),
        remaining_seconds_estimate: read_u32(824),
        export_slot: read_u32(832),
        export_status: read_u32(836),
        flash_on: read_u32(840),
    }
}

fn assert_write_or_die_state(
    session: &FirmwareSession,
    phase: u32,
    goal_mode: u32,
    word_goal: u32,
    time_goal_seconds: u32,
    grace_seconds: u32,
    preview: &str,
    label: &str,
) -> Result<()> {
    let state = write_or_die_state(session);
    if state.phase != phase
        || state.goal_mode != goal_mode
        || state.word_goal != word_goal
        || state.time_goal_seconds != time_goal_seconds
        || state.grace_seconds != grace_seconds
        || !state.preview.starts_with(preview)
    {
        anyhow::bail!(
            "{label}: got phase={} mode={} word_goal={} time_goal={} grace={} preview={:?}",
            state.phase,
            state.goal_mode,
            state.word_goal,
            state.time_goal_seconds,
            state.grace_seconds,
            state.preview
        );
    }
    Ok(())
}

fn assert_write_or_die_phase(session: &FirmwareSession, phase: u32, label: &str) -> Result<()> {
    let state = write_or_die_state(session);
    if state.phase != phase {
        anyhow::bail!("{label}: expected phase {phase}, got {}", state.phase);
    }
    Ok(())
}

fn assert_write_or_die_text_prefix(session: &FirmwareSession, preview: &str, label: &str) -> Result<()> {
    let state = write_or_die_state(session);
    if !state.preview.starts_with(preview) {
        anyhow::bail!("{label}: expected preview prefix {preview:?}, got {:?}", state.preview);
    }
    Ok(())
}

fn assert_write_or_die_screen_contains(session: &FirmwareSession, needle: &str, label: &str) -> Result<()> {
    let text = session.snapshot().text_screen.unwrap_or_default();
    if !text.contains(needle) {
        anyhow::bail!("{label}: expected screen to contain {needle:?}, got {text:?}");
    }
    Ok(())
}

#[derive(Debug)]
struct FloppyBirdState {
    bird_y_q8: i16,
    barrier_x: i16,
    gap_row: u8,
    score: u8,
    game_over: bool,
}

fn floppy_bird_state_base(session: &FirmwareSession) -> Option<u32> {
    session.applet_state_base_for_validation("Floppy Bird")
}

fn read_be_i16(bytes: &[u8], addr: usize) -> Option<i16> {
    let slice = bytes.get(addr..addr + 2)?;
    Some(i16::from_be_bytes([slice[0], slice[1]]))
}

fn floppy_bird_state(session: &FirmwareSession) -> FloppyBirdState {
    let base = floppy_bird_state_base(session).unwrap_or_default() as usize;
    let bytes = session.memory_bytes();
    FloppyBirdState {
        bird_y_q8: read_be_i16(bytes, base).unwrap_or_default(),
        barrier_x: read_be_i16(bytes, base + 4).unwrap_or_default(),
        gap_row: bytes.get(base + 6).copied().unwrap_or_default(),
        score: bytes.get(base + 7).copied().unwrap_or_default(),
        game_over: bytes.get(base + 10).copied().unwrap_or_default() != 0,
    }
}

fn assert_floppy_bird_state(session: &FirmwareSession, score: u8, game_over: bool, label: &str) -> Result<()> {
    let state = floppy_bird_state(session);
    if state.score != score || state.game_over != game_over || state.gap_row > 2 || state.barrier_x > 27 {
        anyhow::bail!("{label}: expected score={score} game_over={game_over}, got {state:?}");
    }
    Ok(())
}

fn write_floppy_i16(session: &mut FirmwareSession, offset: u32, value: i16) {
    let base = floppy_bird_state_base(session).unwrap_or_default();
    session.overlay_memory_range(base + offset, &value.to_be_bytes());
}

fn write_floppy_u8(session: &mut FirmwareSession, offset: u32, value: u8) {
    let base = floppy_bird_state_base(session).unwrap_or_default();
    session.overlay_memory_range(base + offset, &[value]);
}

fn write_floppy_u32(session: &mut FirmwareSession, offset: u32, value: u32) {
    let base = floppy_bird_state_base(session).unwrap_or_default();
    session.overlay_memory_range(base + offset, &value.to_be_bytes());
}

fn seed_floppy_bird_near_score(session: &mut FirmwareSession) {
    write_floppy_i16(session, 0, 384);
    write_floppy_i16(session, 2, 0);
    write_floppy_i16(session, 4, 4);
    write_floppy_u8(session, 6, 1);
    write_floppy_u8(session, 7, 0);
    write_floppy_u8(session, 9, 0);
    write_floppy_u8(session, 10, 0);
    write_floppy_u32(session, 12, 0);
    write_floppy_u8(session, 16, 1);
}

fn seed_floppy_bird_crash(session: &mut FirmwareSession) {
    write_floppy_i16(session, 0, 900);
    write_floppy_i16(session, 2, 120);
    write_floppy_i16(session, 4, 10);
    write_floppy_u8(session, 6, 1);
    write_floppy_u8(session, 10, 0);
    write_floppy_u32(session, 12, 0);
    write_floppy_u8(session, 16, 1);
}

#[derive(Debug)]
struct SnakeState {
    length: u8,
    head_x: u8,
    head_y: u8,
    score: u8,
    paused: bool,
    game_over: bool,
}

fn snake_state_base(session: &FirmwareSession) -> Option<u32> {
    session.applet_state_base_for_validation("Snake")
}

fn snake_state(session: &FirmwareSession) -> SnakeState {
    let base = snake_state_base(session).unwrap_or_default() as usize;
    let bytes = session.memory_bytes();
    SnakeState {
        length: bytes.get(base + 384).copied().unwrap_or_default(),
        head_x: bytes.get(base + 385).copied().unwrap_or_default(),
        head_y: bytes.get(base + 386).copied().unwrap_or_default(),
        score: bytes.get(base + 389).copied().unwrap_or_default(),
        paused: bytes.get(base + 393).copied().unwrap_or_default() != 0,
        game_over: bytes.get(base + 394).copied().unwrap_or_default() != 0,
    }
}

fn assert_snake_state(
    session: &FirmwareSession,
    score: u8,
    length: u8,
    paused: bool,
    game_over: bool,
    label: &str,
) -> Result<()> {
    let state = snake_state(session);
    if state.score != score || state.length != length || state.paused != paused || state.game_over != game_over {
        anyhow::bail!("{label}: expected score={score} length={length} paused={paused} game_over={game_over}, got {state:?}");
    }
    Ok(())
}

fn write_snake_u8(session: &mut FirmwareSession, offset: u32, value: u8) {
    let base = snake_state_base(session).unwrap_or_default();
    session.overlay_memory_range(base + offset, &[value]);
}

fn write_snake_u32(session: &mut FirmwareSession, offset: u32, value: u32) {
    let base = snake_state_base(session).unwrap_or_default();
    session.overlay_memory_range(base + offset, &value.to_be_bytes());
}

fn seed_snake_food_ahead(session: &mut FirmwareSession) {
    let state = snake_state(session);
    write_snake_u8(session, 387, state.head_x);
    write_snake_u8(session, 388, state.head_y.saturating_add(1));
    write_snake_u32(session, 396, 0);
}

fn seed_snake_edge_wrap(session: &mut FirmwareSession) {
    write_snake_u8(session, 0, 49);
    write_snake_u8(session, 192, 8);
    write_snake_u8(session, 384, 3);
    write_snake_u8(session, 385, 49);
    write_snake_u8(session, 386, 8);
    write_snake_u8(session, 389, 0);
    write_snake_u8(session, 391, 1);
    write_snake_u8(session, 392, 1);
    write_snake_u8(session, 393, 0);
    write_snake_u8(session, 394, 0);
    write_snake_u32(session, 396, 0);
}

fn lcd_visible_lit_pixels(snapshot: &LcdSnapshot) -> usize {
    let width = snapshot.width.min(264);
    let height = snapshot.height.min(64);
    (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .filter(|(x, y)| snapshot.pixels[*y * snapshot.width + *x])
        .count()
}

fn lcd_lit_pixels_in_rect(
    snapshot: &LcdSnapshot,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> usize {
    let max_y = (y + height).min(snapshot.height);
    let max_x = (x + width).min(snapshot.width);
    (y..max_y)
        .flat_map(|pixel_y| (x..max_x).map(move |pixel_x| (pixel_x, pixel_y)))
        .filter(|(pixel_x, pixel_y)| snapshot.pixels[*pixel_y * snapshot.width + *pixel_x])
        .count()
}

fn seed_write_or_die_completed_state(session: &mut FirmwareSession, text: &str) {
    let base = write_or_die_state_base(session).unwrap_or_default();
    let write_u32 = |session: &mut FirmwareSession, offset: u32, value: u32| {
        session.overlay_memory_range(base + offset, &value.to_be_bytes());
    };
    write_u32(session, 0, 2);
    write_u32(session, 24, text.len() as u32);
    write_u32(session, 28, text.len() as u32);
    write_u32(session, 32, 0);
    let mut text_bytes = [0u8; 768];
    let source = text.as_bytes();
    let len = source.len().min(text_bytes.len());
    text_bytes[..len].copy_from_slice(&source[..len]);
    session.overlay_memory_range(base + 36, &text_bytes);
    write_u32(session, 816, 5);
    write_u32(session, 832, 0);
    write_u32(session, 836, 0);
    write_u32(session, 840, 0);
}

fn seed_write_or_die_running_state(session: &mut FirmwareSession, text: &str) {
    seed_write_or_die_running_state_with_idle(session, text, 8_500);
}

fn seed_write_or_die_running_state_with_idle(
    session: &mut FirmwareSession,
    text: &str,
    idle_ms: u32,
) {
    let target_now = idle_ms.saturating_add(1_000);
    let now = session.snapshot().cycles.saturating_div(33_000) as u32;
    if now < target_now {
        let missing_ms = target_now - now;
        session.run_realtime_cycles(u64::from(missing_ms) * 33_000, 5_000_000);
    }
    let base = write_or_die_state_base(session).unwrap_or_default();
    let write_u32 = |session: &mut FirmwareSession, offset: u32, value: u32| {
        session.overlay_memory_range(base + offset, &value.to_be_bytes());
    };
    let now = session.snapshot().cycles.saturating_div(33_000) as u32;
    write_u32(session, 0, 1);
    write_u32(session, 8, 0);
    write_u32(session, 12, 500);
    write_u32(session, 16, 600);
    write_u32(session, 20, 6);
    write_u32(session, 24, text.len() as u32);
    write_u32(session, 28, text.len() as u32);
    write_u32(session, 32, 0);
    let mut text_bytes = [0u8; 768];
    let source = text.as_bytes();
    let len = source.len().min(text_bytes.len());
    text_bytes[..len].copy_from_slice(&source[..len]);
    session.overlay_memory_range(base + 36, &text_bytes);
    write_u32(session, 804, now.wrapping_sub(30_000));
    write_u32(session, 808, now.wrapping_sub(idle_ms));
    write_u32(session, 812, now.wrapping_sub(idle_ms));
    write_u32(session, 824, 600);
    write_u32(session, 828, 0);
    write_u32(session, 840, 0);
}

fn assert_write_or_die_penalty_feedback(session: &mut FirmwareSession) -> Result<()> {
    focus_write_or_die_direct(session)?;
    seed_write_or_die_running_state_with_idle(session, "alpha beta gamm", 0);
    dispatch_write_or_die_message(session, 0x20, b'a'.into(), "WriteOrDie penalty setup draw")?;
    seed_write_or_die_running_state_with_idle(session, "alpha beta gamma", 5_999);
    dispatch_write_or_die_message(session, 0x00, 0, "WriteOrDie safe idle")?;
    let state = write_or_die_state(session);
    if state.preview.contains("gamm.") || state.flash_on != 0 {
        anyhow::bail!("WriteOrDie damaged or flashed text during safe grace: {state:?}");
    }
    seed_write_or_die_running_state_with_idle(session, "alpha beta gamma", 6_750);
    dispatch_write_or_die_message(session, 0x00, 0, "WriteOrDie warning idle")?;
    let state = write_or_die_state(session);
    if state.preview.contains("gamm.") || state.flash_on == 0 {
        anyhow::bail!("WriteOrDie warning phase should flash without deleting: {state:?}");
    }
    dispatch_write_or_die_message(session, 0x20, b'!'.into(), "WriteOrDie warning recovery")?;
    seed_write_or_die_running_state_with_idle(session, "alpha beta gamma", 10_750);
    let before = session.snapshot();
    dispatch_write_or_die_message(session, 0x00, 0, "WriteOrDie penalty idle")?;
    let after = session.snapshot();
    let state = write_or_die_state(session);
    if !state.preview.starts_with("alpha beta gamm") || state.preview.contains("gamma") {
        anyhow::bail!("WriteOrDie penalty did not remove one trailing character: {state:?}");
    }
    let text_area_diff = lcd_diff_pixels_in_rect(&before.lcd, &after.lcd, 0, 16, 264, 48);
    let full_lcd_diff = lcd_diff_pixels_in_rect(&before.lcd, &after.lcd, 0, 0, 320, 128);
    if text_area_diff < 500 {
        let lcd_mmio = after
            .mmio_accesses
            .iter()
            .filter(|line| line.contains("0100") || line.contains("8000"))
            .rev()
            .take(8)
            .cloned()
            .collect::<Vec<_>>();
        anyhow::bail!("WriteOrDie penalty did not visibly highlight the written text area; diff_pixels={text_area_diff} full_lcd_diff={full_lcd_diff} state={state:?} lcd_mmio={lcd_mmio:?}");
    }
    let base = write_or_die_state_base(session).unwrap_or_default();
    let stale_penalty_ms = session
        .memory_bytes()
        .get((base + 808) as usize..(base + 812) as usize)
        .map(|bytes| [bytes[0], bytes[1], bytes[2], bytes[3]])
        .unwrap_or([0, 0, 0, 0]);
    session.overlay_memory_range(base + 812, &stale_penalty_ms);
    dispatch_write_or_die_message(session, 0x00, 0, "WriteOrDie second penalty idle")?;
    let state = write_or_die_state(session);
    if !state.preview.starts_with("alpha beta gam") || state.preview.contains("gamm") {
        anyhow::bail!("WriteOrDie repeated penalty did not remove another trailing character: {state:?}");
    }
    let now = session.snapshot().cycles.saturating_div(33_000) as u32;
    session.overlay_memory_range(base + 808, &now.to_be_bytes());
    dispatch_write_or_die_message(session, 0x20, b'!'.into(), "WriteOrDie recovery char")?;
    let state = write_or_die_state(session);
    if !state.preview.starts_with("alpha beta gam!") || state.flash_on != 0 {
        anyhow::bail!("WriteOrDie typing did not stop punishment immediately: {state:?}");
    }
    Ok(())
}

fn lcd_diff_pixels_in_rect(
    before: &LcdSnapshot,
    after: &LcdSnapshot,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> usize {
    let max_y = (y + height).min(before.height).min(after.height);
    let max_x = (x + width).min(before.width).min(after.width);
    let mut count = 0;
    for pixel_y in y..max_y {
        for pixel_x in x..max_x {
            if before.pixels[pixel_y * before.width + pixel_x]
                != after.pixels[pixel_y * after.width + pixel_x]
            {
                count += 1;
            }
        }
    }
    count
}

fn write_or_die_state_base(session: &FirmwareSession) -> Option<u32> {
    session.applet_state_base_for_validation("WriteOrDie")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AlphaWordFileRecord {
    slot: u8,
    record: usize,
    pointer: usize,
    len_a: u32,
    len_b: u32,
    cap_a: u32,
    cap_b: u32,
    record_bytes: Vec<u8>,
    buffer_sample: Vec<u8>,
}

fn alphaword_file_records(bytes: &[u8]) -> Vec<AlphaWordFileRecord> {
    const FILE_KEYS: [u8; 8] = [0x2d, 0x2c, 0x04, 0x0f, 0x0e, 0x0a, 0x01, 0x27];
    let mut records = Vec::new();
    for name in 0x1000usize..0x2400 {
        if bytes.get(name..name + 5) != Some(b"File ".as_slice()) {
            continue;
        }
        let Some(slot) = bytes.get(name + 5).and_then(|digit| digit.checked_sub(b'0')) else {
            continue;
        };
        if !(1..=8).contains(&slot) {
            continue;
        }
        let record = name.saturating_sub(0x16);
        if bytes.get(record + 0x27).copied() != Some(slot)
            || bytes.get(record + 0x29).copied() != Some(FILE_KEYS[(slot - 1) as usize])
        {
            continue;
        }
        let Some(pointer) = read_be_u32(bytes, record + 0x2a).map(|value| value as usize) else {
            continue;
        };
        let len_a = read_be_u32(bytes, record + 0x2e).unwrap_or_default();
        let len_b = read_be_u32(bytes, record + 0x32).unwrap_or_default();
        let cap_a = read_be_u32(bytes, record + 0x36).unwrap_or_default();
        let cap_b = read_be_u32(bytes, record + 0x3a).unwrap_or_default();
        let record_bytes = bytes.get(record..record + 0x40).unwrap_or(&[]).to_vec();
        let sample_len = effective_alphaword_capacity(len_a, len_b, cap_a, cap_b)
            .unwrap_or(64)
            .min(64);
        let buffer_sample = if pointer < bytes.len() && pointer + sample_len <= bytes.len() {
            bytes[pointer..pointer + sample_len].to_vec()
        } else {
            Vec::new()
        };
        records.push(AlphaWordFileRecord {
            slot,
            record,
            pointer,
            len_a,
            len_b,
            cap_a,
            cap_b,
            record_bytes,
            buffer_sample,
        });
    }
    records
}

fn effective_alphaword_capacity(len_a: u32, len_b: u32, cap_a: u32, cap_b: u32) -> Option<usize> {
    let values = [cap_a, cap_b, len_a, len_b];
    values
        .into_iter()
        .filter(|value| (64..=4096).contains(value))
        .max()
        .map(|value| value as usize)
}

fn validate_alphaword_export_records(
    before: &[AlphaWordFileRecord],
    after_bytes: &[u8],
    target_slot: u8,
    marker: &[u8],
) -> Result<Vec<String>> {
    let after = alphaword_file_records(after_bytes);
    let mut marker_offsets = Vec::new();
    for original in before {
        let Some(current) = after.iter().find(|candidate| candidate.record == original.record) else {
            anyhow::bail!("AlphaWord File {} descriptor at 0x{:04x} disappeared", original.slot, original.record);
        };
        if original.slot != target_slot {
            continue;
        }
        if current.pointer != original.pointer || current.cap_a != original.cap_a || current.cap_b != original.cap_b {
            anyhow::bail!(
                "AlphaWord File {target_slot} descriptor at 0x{:04x} changed pointer/capacity: before ptr=0x{:08x} caps={}/{} after ptr=0x{:08x} caps={}/{}",
                original.record,
                original.pointer,
                original.cap_a,
                original.cap_b,
                current.pointer,
                current.cap_a,
                current.cap_b
            );
        }
        if current.len_a != current.len_b {
            anyhow::bail!(
                "AlphaWord File {target_slot} descriptor at 0x{:04x} has mismatched used lengths {}/{}",
                original.record,
                current.len_a,
                current.len_b
            );
        }
        let Some(capacity) = effective_alphaword_capacity(current.len_a, current.len_b, current.cap_a, current.cap_b) else {
            continue;
        };
        if current.pointer >= after_bytes.len() || current.pointer + capacity > after_bytes.len() {
            continue;
        }
        let body = &after_bytes[current.pointer..current.pointer + capacity];
        if body.windows(marker.len()).any(|window| window == marker) {
            if current.len_a > current.cap_a && current.cap_a >= 64 {
                anyhow::bail!(
                    "AlphaWord File {target_slot} descriptor at 0x{:04x} used length {} exceeds capacity {}",
                    original.record,
                    current.len_a,
                    current.cap_a
                );
            }
            marker_offsets.push(format!("0x{:08x}", current.pointer));
        }
    }
    Ok(marker_offsets)
}

fn max_alphaword_slot_used(records: &[AlphaWordFileRecord], slot: u8) -> u32 {
    records
        .iter()
        .filter(|record| record.slot == slot && record.len_a == record.len_b)
        .map(|record| record.len_a)
        .max()
        .unwrap_or_default()
}

fn alphaword_marker_offsets(bytes: &[u8], slot: u8, marker: &[u8]) -> Vec<String> {
    alphaword_file_records(bytes)
        .into_iter()
        .filter(|record| record.slot == slot)
        .filter_map(|record| {
            let capacity = effective_alphaword_capacity(record.len_a, record.len_b, record.cap_a, record.cap_b)?;
            if record.pointer >= bytes.len() || record.pointer + capacity > bytes.len() {
                return None;
            }
            bytes[record.pointer..record.pointer + capacity]
                .windows(marker.len())
                .any(|window| window == marker)
                .then(|| format!("0x{:08x}", record.pointer))
        })
        .collect()
}

fn backing_alphaword_slot(visible_slot: u8) -> u8 {
    if visible_slot == 1 { 8 } else { visible_slot - 1 }
}

fn validate_write_or_die_file_keys_through_menu(
    firmware: FirmwareRuntime,
    recovery_seed_path: &Path,
) -> Result<()> {
    let mut base_session = FirmwareSession::boot_with_keys(firmware, &[0x0e, 0x0c], 512)?;
    recovery_seed::apply_seed_file_if_present(&mut base_session, recovery_seed_path)?;
    base_session.run_realtime_cycles(220_000_000, 25_000_000);
    launch_write_or_die_through_menu(&mut base_session)?;
    base_session.run_steps(1_500_000);
    bail_if_exception(&base_session, "WriteOrDie menu focus")?;

    const FILE_KEYS: &[(u8, u8)] = &[
        (1, 0x4b),
        (2, 0x4a),
        (3, 0x0a),
        (4, 0x1a),
        (5, 0x19),
        (6, 0x10),
        (7, 0x02),
        (8, 0x42),
    ];
    const NEO_CPU_HZ: u64 = 33_000_000;
    for (slot, raw) in FILE_KEYS {
        let mut session = base_session.clone();
        seed_write_or_die_completed_state(&mut session, "matrix file export");
        let before = alphaword_file_records(session.memory_bytes());
        session.tap_matrix_code_for_cycles(*raw, NEO_CPU_HZ / 25, NEO_CPU_HZ / 50);
        session.run_realtime_cycles(80_000_000, 10_000_000);
        bail_if_exception(&session, "WriteOrDie matrix File-key export")?;
        let snapshot = session.snapshot();
        if snapshot.stopped {
            anyhow::bail!(
                "WriteOrDie matrix File {slot} left firmware stopped at pc=0x{:08x}",
                snapshot.pc
            );
        }
        let state = write_or_die_state(&session);
        if state.phase != 3 || state.export_slot != *slot as u32 || state.export_status != 1 {
            anyhow::bail!(
                "WriteOrDie matrix File {slot} export did not complete: raw=0x{raw:02x} state={state:?}"
            );
        }
        let backing_slot = backing_alphaword_slot(*slot);
        let offsets = if *slot == 2 {
            validate_alphaword_export_records(
                &before,
                session.memory_bytes(),
                backing_slot,
                b"matrix file export",
            )?
        } else {
            alphaword_marker_offsets(session.memory_bytes(), backing_slot, b"matrix file export")
        };
        if offsets.is_empty() {
            anyhow::bail!("WriteOrDie matrix File {slot} export wrote no marker in backing slot {backing_slot}");
        }
        if *slot == 2 {
            assert_write_or_die_exported_screen_accepts_enter(&mut session)?;
        }
    }
    Ok(())
}

fn assert_write_or_die_exported_screen_accepts_enter(session: &mut FirmwareSession) -> Result<()> {
    session.tap_matrix_code_for_cycles(0x69, 33_000_000 / 25, 33_000_000 / 50);
    session.run_realtime_cycles(80_000_000, 10_000_000);
    bail_if_exception(session, "WriteOrDie exported screen Enter")?;
    let state = write_or_die_state(session);
    if state.phase != 0 {
        anyhow::bail!("WriteOrDie exported screen did not accept Enter: {state:?}");
    }
    Ok(())
}

fn assert_write_or_die_screen_not_contains(session: &FirmwareSession, needle: &str, label: &str) -> Result<()> {
    let text = session.snapshot().text_screen.unwrap_or_default();
    if text.contains(needle) {
        anyhow::bail!("{label}: expected screen not to contain {needle:?}, got {text:?}");
    }
    Ok(())
}

fn assert_lcd_ocr_contains(session: &FirmwareSession, needle: &str, scale: usize, label: &str) -> Result<()> {
    let snapshot = session.snapshot();
    let text = ocr_visible_lcd(None, &snapshot.lcd, scale)?;
    if !text.contains(needle) {
        anyhow::bail!("{label}: expected OCR to contain {needle:?}, got {text:?}");
    }
    Ok(())
}

fn wait_for_lcd_ocr_not_contains(
    session: &mut FirmwareSession,
    needle: &str,
    scale: usize,
    label: &str,
) -> Result<()> {
    for attempt in 0..20 {
        session.run_realtime_steps(10_000_000);
        bail_if_exception(session, label)?;
        let snapshot = session.snapshot();
        let text = ocr_visible_lcd(None, &snapshot.lcd, scale)?;
        println!("{label} attempt={attempt} ocr={text:?}");
        if !text.contains(needle) {
            return Ok(());
        }
    }
    let snapshot = session.snapshot();
    let text = ocr_visible_lcd(None, &snapshot.lcd, scale)?;
    anyhow::bail!("{label}: OCR still contains {needle:?}: {text:?}");
}

fn wait_for_write_or_die_state(
    session: &mut FirmwareSession,
    predicate: impl Fn(&WriteOrDieState) -> bool,
) -> Result<()> {
    for _ in 0..80 {
        session.run_steps(250_000);
        bail_if_exception(session, "WriteOrDie wait")?;
        let state = write_or_die_state(session);
        if predicate(&state) {
            return Ok(());
        }
    }
    anyhow::bail!("timed out waiting for WriteOrDie state: {:?}", write_or_die_state(session));
}

fn wait_for_basic_writer_banner_to_clear(session: &mut FirmwareSession) -> Result<()> {
    const CHUNK_STEPS: usize = 100_000;
    const MAX_STEPS: usize = 12_000_000;
    let mut elapsed = 0;
    while elapsed < MAX_STEPS {
        session.run_steps(CHUNK_STEPS);
        elapsed += CHUNK_STEPS;
        bail_if_exception(session, "Basic Writer banner wait")?;
        if basic_writer_state(session).banner_slot == 0 {
            return Ok(());
        }
    }
    anyhow::bail!("Basic Writer banner did not clear within {} steps", MAX_STEPS)
}

fn wait_for_basic_writer_state(
    session: &mut FirmwareSession,
    predicate: impl Fn(&BasicWriterState) -> bool,
) -> Result<()> {
    const CHUNK_STEPS: usize = 50_000;
    const MAX_STEPS: usize = 3_000_000;

    let mut elapsed = 0;
    while elapsed < MAX_STEPS {
        session.run_steps(CHUNK_STEPS);
        elapsed += CHUNK_STEPS;
        bail_if_exception(session, "Basic Writer wait")?;
        let state = basic_writer_state(session);
        if predicate(&state) {
            return Ok(());
        }
    }

    let state = basic_writer_state(session);
    anyhow::bail!(
        "Basic Writer state did not reach expected condition within {} steps; slot {} preview {:?} len {} banner_slot={}",
        MAX_STEPS,
        state.active_slot,
        state.preview,
        state.len,
        state.banner_slot,
    );
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

fn read_be_u32(bytes: &[u8], addr: usize) -> Option<u32> {
    let slice = bytes.get(addr..addr + 4)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn print_forth_state_ascii_runs(session: &FirmwareSession, label: &str) {
    const FORTH_SOURCE_OFFSET: usize = 5772;
    let snapshot = session.snapshot();
    let Some(slot) = debug_word(&snapshot, 0x0000_35e2) else {
        println!("{label}: missing_slot");
        return;
    };
    let bytes = session.memory_bytes();
    let table_addr = 0x0000_355eusize + (slot as usize) * 4;
    let Some(a5) = read_be_u32(bytes, table_addr) else {
        println!("{label}: missing_a5");
        return;
    };
    let state_base = a5 as usize + 0x300;
    let source_base = state_base + FORTH_SOURCE_OFFSET;
    let Some(region) = bytes.get(source_base..source_base + 128) else {
        println!("{label}: missing_state_region a5=0x{a5:08x}");
        return;
    };
    let mut preview = String::new();
    for &byte in region {
        if byte == 0 {
            break;
        }
        preview.push(if (0x20..=0x7e).contains(&byte) {
            byte as char
        } else {
            '.'
        });
    }
    println!("{label}: a5=0x{a5:08x} src={preview}");
}

fn parse_sample_lcd_after(value: &str) -> Result<(usize, usize)> {
    let Some((interval, count)) = value.split_once(':') else {
        anyhow::bail!("--sample-lcd-after expects INTERVAL_STEPS:COUNT");
    };
    Ok((interval.parse()?, count.parse()?))
}

fn print_lcd_samples(session: &mut FirmwareSession, interval_steps: usize, count: usize) {
    let mut previous = session.lcd_snapshot();
    println!(
        "lcd_sample index=0 step={} cycles={} hash=0x{:016x} diff=0",
        session.snapshot().steps,
        session.snapshot().cycles,
        lcd_hash(&previous)
    );
    for index in 1..=count {
        session.run_realtime_steps(interval_steps);
        let current = session.lcd_snapshot();
        let diff = previous
            .pixels
            .iter()
            .zip(&current.pixels)
            .filter(|(left, right)| left != right)
            .count();
        println!(
            "lcd_sample index={index} step={} cycles={} hash=0x{:016x} diff={diff}",
            session.snapshot().steps,
            session.snapshot().cycles,
            lcd_hash(&current)
        );
        previous = current;
    }
}

fn launch_forth_mini_through_menu(session: &mut FirmwareSession) {
    for _ in 0..19 {
        session.tap_matrix_code_long(0x15);
        session.run_steps(250_000);
    }
    session.press_matrix_code(0x69);
    session.run_steps(3_000_000);
    session.release_matrix_code(0x69);
    session.run_steps(3_000_000);
    session.clear_keyboard_transients();
}

fn enter_forth_line(session: &mut FirmwareSession, line: &str) {
    let _ = type_text_via_matrix(session, line);
    session.press_matrix_code(0x69);
    session.run_steps(500_000);
    session.release_matrix_code(0x69);
    session.run_steps(500_000);
}

fn exit_forth_to_menu(session: &mut FirmwareSession) {
    tap_key_now(session, 0x46);
    session.run_steps(2_000_000);
}

fn relaunch_current_menu_item(session: &mut FirmwareSession) {
    session.press_matrix_code(0x69);
    session.run_steps(3_000_000);
    session.release_matrix_code(0x69);
    session.run_steps(3_000_000);
    session.clear_keyboard_transients();
}

fn assert_forth_screen_contains(session: &FirmwareSession, needle: &str, label: &str) -> Result<()> {
    let text = session.snapshot().text_screen.unwrap_or_default();
    if text.contains(needle) {
        return Ok(());
    }
    anyhow::bail!("{label} failed: expected screen to contain {needle:?}, got:\n{text}");
}

fn enter_forth_line_and_assert(
    session: &mut FirmwareSession,
    line: &str,
    needle: &str,
    label: &str,
) -> Result<()> {
    enter_forth_line(session, line);
    bail_if_exception(session, label)?;
    assert_forth_screen_contains(session, needle, label)
}

fn launch_basic_writer_through_menu(session: &mut FirmwareSession, _ocr_scale: usize) -> Result<()> {
    for _ in 0..29 {
        session.tap_matrix_code_long(0x15);
        session.run_steps(250_000);
    }
    session.press_matrix_code(0x69);
    session.run_steps(3_000_000);
    session.release_matrix_code(0x69);
    session.run_steps(3_000_000);
    session.clear_keyboard_transients();
    Ok(())
}

fn launch_write_or_die_through_menu(session: &mut FirmwareSession) -> Result<()> {
    for _ in 0..30 {
        session.tap_matrix_code_long(0x15);
        session.run_steps(250_000);
    }
    session.press_matrix_code(0x69);
    session.run_steps(3_000_000);
    session.release_matrix_code(0x69);
    session.run_steps(3_000_000);
    session.clear_keyboard_transients();
    Ok(())
}

fn focus_write_or_die_direct(session: &mut FirmwareSession) -> Result<()> {
    dispatch_write_or_die_message(session, 0x19, 0, "WriteOrDie direct focus")
}

fn focus_floppy_bird_direct(session: &mut FirmwareSession) -> Result<()> {
    dispatch_floppy_bird_message(session, 0x19, 0, "Floppy Bird direct focus")
}

fn force_floppy_bird_tick(session: &mut FirmwareSession, label: &str) -> Result<()> {
    write_floppy_u32(session, 12, 0);
    dispatch_floppy_bird_message(session, 0x00, 0, label)
}

fn focus_snake_direct(session: &mut FirmwareSession) -> Result<()> {
    dispatch_snake_message(session, 0x19, 0, "Snake direct focus")
}

fn force_snake_tick(session: &mut FirmwareSession, label: &str) -> Result<()> {
    write_snake_u32(session, 396, 0);
    dispatch_snake_message(session, 0x00, 0, label)
}

fn launch_alpha_word_from_write_or_die_menu(session: &mut FirmwareSession) {
    for _ in 0..30 {
        session.tap_matrix_code_long(0x77);
        session.run_steps(250_000);
    }
    session.press_matrix_code(0x69);
    session.run_steps(4_000_000);
    session.release_matrix_code(0x69);
    session.run_steps(4_000_000);
    session.clear_keyboard_transients();
}

fn focus_alpha_word_direct(session: &mut FirmwareSession) -> Result<()> {
    session
        .start_stock_applet_message_for_validation("AlphaWord Plus", 0x19)
        .map_err(|error| anyhow::anyhow!("failed to focus AlphaWord Plus: {error}"))?;
    session.run_realtime_steps(20_000_000);
    bail_if_exception(session, "AlphaWord direct focus")
}

fn launch_forth_mini_for_debugging(session: &mut FirmwareSession) -> Result<()> {
    focus_forth_mini_direct(session)?;
    if let Some(exception) = session.snapshot().last_exception.clone() {
        let trace = session
            .snapshot()
            .trace
            .into_iter()
            .rev()
            .take(12)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n  ");
        anyhow::bail!("Forth Mini debug launch failed: {exception}\n  {trace}");
    }
    Ok(())
}

fn focus_forth_mini_direct(session: &mut FirmwareSession) -> Result<()> {
    session
        .start_applet_message_for_validation("Forth Mini", 0x19)
        .map_err(|error| anyhow::anyhow!("failed to focus Forth Mini: {error}"))?;
    session.run_steps(20_000);
    Ok(())
}

fn launch_calculator_for_debugging(session: &mut FirmwareSession) -> Result<()> {
    session
        .start_stock_applet_message_for_validation("Calculator", 0x19)
        .map_err(|error| anyhow::anyhow!("failed to launch Calculator for debugging: {error}"))?;
    session.run_steps(500_000);
    if let Some(exception) = session.snapshot().last_exception.clone() {
        let trace = session
            .snapshot()
            .trace
            .into_iter()
            .rev()
            .take(12)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n  ");
        anyhow::bail!("Calculator debug launch failed: {exception}\n  {trace}");
    }
    Ok(())
}

fn type_text_now(session: &mut FirmwareSession, text: &str) {
    for value in text.chars() {
        session.tap_char_debug(value);
        session.run_steps(300_000);
    }
}

fn type_text_via_matrix(session: &mut FirmwareSession, text: &str) -> Result<()> {
    const LEFT_SHIFT: u8 = 0x0e;
    for value in text.chars() {
        let key = matrix_text_key(value)
            .ok_or_else(|| anyhow::anyhow!("no matrix key for character {value:?}"))?;
        if key.shift {
            session.press_matrix_code(LEFT_SHIFT);
            session.run_steps(500_000);
        }
        session.press_matrix_code(key.code);
        session.run_steps(300_000);
        session.release_matrix_code(key.code);
        session.run_steps(300_000);
        if key.shift {
            session.release_matrix_code(LEFT_SHIFT);
            session.run_steps(300_000);
        }
    }
    Ok(())
}

fn type_text_like_gui(session: &mut FirmwareSession, text: &str) {
    const NEO_CPU_HZ: u64 = 33_000_000;
    const GUI_KEY_PRESS_CYCLES: u64 = NEO_CPU_HZ / 25;
    const GUI_KEY_RELEASE_CYCLES: u64 = NEO_CPU_HZ / 50;
    for value in text.chars() {
        session.tap_char_for_cycles(value, GUI_KEY_PRESS_CYCLES, GUI_KEY_RELEASE_CYCLES);
    }
    session.run_steps(60_000 * text.chars().count());
}

fn type_write_or_die_direct(session: &mut FirmwareSession, text: &str) -> Result<()> {
    for value in text.chars() {
        send_write_or_die_char_direct(session, value)?;
    }
    Ok(())
}

fn send_write_or_die_char_direct(session: &mut FirmwareSession, value: char) -> Result<()> {
    dispatch_write_or_die_message(session, 0x20, value as u32, "WriteOrDie char dispatch")
}

fn send_write_or_die_key_direct(session: &mut FirmwareSession, key: u32) -> Result<()> {
    dispatch_write_or_die_message(session, 0x21, key, "WriteOrDie key dispatch")
}

fn dispatch_write_or_die_message(
    session: &mut FirmwareSession,
    message: u32,
    param: u32,
    label: &str,
) -> Result<()> {
    const VALIDATION_RETURN_PC: u32 = 0x0042_6752;
    session
        .start_applet_message_with_param_for_validation("WriteOrDie", message, param)
        .map_err(|error| anyhow::anyhow!("failed to dispatch WriteOrDie message: {error}"))?;
    if !session.run_until_pc_or_steps(VALIDATION_RETURN_PC, 5_000_000) {
        bail_if_exception(session, label)?;
        anyhow::bail!("{label}: applet did not return to validation trampoline");
    }
    Ok(())
}

fn dispatch_floppy_bird_message(
    session: &mut FirmwareSession,
    message: u32,
    param: u32,
    label: &str,
) -> Result<()> {
    const VALIDATION_RETURN_PC: u32 = 0x0042_6752;
    session
        .start_applet_message_with_param_for_validation("Floppy Bird", message, param)
        .map_err(|error| anyhow::anyhow!("failed to dispatch Floppy Bird message: {error}"))?;
    if !session.run_until_pc_or_steps(VALIDATION_RETURN_PC, 5_000_000) {
        bail_if_exception(session, label)?;
        anyhow::bail!("{label}: applet did not return to validation trampoline");
    }
    Ok(())
}

fn dispatch_snake_message(
    session: &mut FirmwareSession,
    message: u32,
    param: u32,
    label: &str,
) -> Result<()> {
    const VALIDATION_RETURN_PC: u32 = 0x0042_6752;
    session
        .start_applet_message_with_param_for_validation("Snake", message, param)
        .map_err(|error| anyhow::anyhow!("failed to dispatch Snake message: {error}"))?;
    if !session.run_until_pc_or_steps(VALIDATION_RETURN_PC, 5_000_000) {
        bail_if_exception(session, label)?;
        anyhow::bail!("{label}: applet did not return to validation trampoline");
    }
    Ok(())
}

fn tap_key_now(session: &mut FirmwareSession, code: u8) {
    session.tap_matrix_code_debug(code);
    session.run_steps(300_000);
}

fn press_key_now(session: &mut FirmwareSession, code: u8) {
    session.press_matrix_code(code);
    session.run_steps(700_000);
    session.release_matrix_code(code);
    session.run_steps(700_000);
}

fn switch_basic_writer_file(session: &mut FirmwareSession, slot: u32) -> Result<()> {
    let key = match slot {
        1 => 0x4b,
        2 => 0x4a,
        3 => 0x0a,
        4 => 0x1a,
        5 => 0x19,
        6 => 0x10,
        7 => 0x02,
        8 => 0x42,
        _ => anyhow::bail!("invalid Basic Writer slot {slot}"),
    };
    session.press_matrix_code(key);
    wait_for_basic_writer_state(session, |state| state.active_slot == slot && state.banner_slot == slot)?;
    session.release_matrix_code(key);
    session.run_steps(300_000);
    bail_if_exception(session, "Basic Writer file switch")
}

fn exit_basic_writer_to_menu(session: &mut FirmwareSession) {
    session.press_matrix_code(0x46);
    session.run_steps(3_000_000);
    session.release_matrix_code(0x46);
    session.run_steps(3_000_000);
}

fn exit_write_or_die_to_menu(session: &mut FirmwareSession) {
    tap_key_now(session, 0x46);
    session.run_steps(2_000_000);
}

fn type_text_direct_to_forth(session: &mut FirmwareSession, text: &str) -> Result<()> {
    for value in text.bytes() {
        session
            .start_applet_message_with_param_for_validation("Forth Mini", 0x20, u32::from(value))
            .map_err(|error| anyhow::anyhow!("failed to send Forth Mini char: {error}"))?;
        session.run_steps(200_000);
        if let Some(exception) = session.snapshot().last_exception {
            anyhow::bail!("Forth Mini char dispatch failed: {exception}");
        }
    }
    Ok(())
}

fn submit_forth_line_direct(session: &mut FirmwareSession, text: &str) -> Result<()> {
    type_text_direct_to_forth(session, text)?;
    session
        .start_applet_message_with_param_for_validation("Forth Mini", 0x20, u32::from(b'\r'))
        .map_err(|error| anyhow::anyhow!("failed to submit Forth Mini line: {error}"))?;
    session.run_steps(20_000);
    if let Some(exception) = session.snapshot().last_exception {
        anyhow::bail!("Forth Mini submit failed: {exception}");
    }
    Ok(())
}

fn send_key_direct_to_forth(session: &mut FirmwareSession, matrix_code: u8) -> Result<()> {
    let logical = logical_key_for_matrix_code(matrix_code)
        .ok_or_else(|| anyhow::anyhow!("no logical key for matrix code 0x{matrix_code:02x}"))?;
    session
        .start_applet_message_with_param_for_validation("Forth Mini", 0x21, u32::from(logical))
        .map_err(|error| anyhow::anyhow!("failed to send Forth Mini key: {error}"))?;
    session.run_steps(200_000);
    if let Some(exception) = session.snapshot().last_exception {
        anyhow::bail!("Forth Mini key dispatch failed: {exception}");
    }
    Ok(())
}

fn lcd_hash(snapshot: &LcdSnapshot) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for pixel in &snapshot.pixels {
        hash ^= u64::from(*pixel);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
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
                session.tap_char_debug(value);
                if run_short_settle(session, keep_trace, stop, 300_000) {
                    return Some(true);
                }
            }
            text_index += 1;
        } else if next_key_step <= next_all_row_key_step && next_key_step <= next_hold_step {
            let (step, code) = key_events[key_index];
            run_until_step(session, step, keep_trace);
            session.tap_matrix_code_debug(code);
            if run_short_settle(session, keep_trace, stop, 300_000) {
                return Some(true);
            }
            key_index += 1;
        } else if next_all_row_key_step <= next_hold_step {
            let (step, code) = all_row_key_events[all_row_key_index];
            run_until_step(session, step, keep_trace);
            session.tap_matrix_code_all_rows_debug(code);
            if run_short_settle(session, keep_trace, stop, 300_000) {
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
            if run_short_settle(session, keep_trace, stop, 2_000) {
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

fn run_short_settle(
    session: &mut FirmwareSession,
    keep_trace: bool,
    stop: HeadlessStop,
    settle_steps: usize,
) -> bool {
    if let Some(stop_pc) = stop.pc {
        if stop.pc_hit > 1 {
            return session.run_until_pc_hit_or_steps(stop_pc, stop.pc_hit, settle_steps);
        }
        return session.run_until_pc_or_steps(stop_pc, settle_steps);
    }
    if keep_trace {
        session.run_steps(settle_steps);
    } else {
        session.run_realtime_steps(settle_steps);
    }
    false
}

fn parse_key_event(value: &str) -> Result<(usize, u8)> {
    let Some((step, key)) = value.split_once(':') else {
        anyhow::bail!("--key-at expects STEP:KEY");
    };
    Ok((step.parse()?, matrix_code_for_key_name(key)?))
}

fn parse_key_name_list(value: &str) -> Result<Vec<u8>> {
    value.split(',').map(matrix_code_for_key_name).collect()
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

fn ocr_visible_lcd(text_screen: Option<&str>, snapshot: &LcdSnapshot, scale: usize) -> Result<String> {
    if let Some(text_screen) = text_screen {
        let trimmed = text_screen.trim_end();
        if !trimmed.is_empty() {
            return Ok(format!("{trimmed}\n"));
        }
    }
    let visible = visible_snapshot(snapshot);
    let cursor_off = cursor_blink_snapshot(&visible, false);
    let scaled = scale_snapshot(&cursor_off, scale.max(1));
    let temp_dir = std::env::temp_dir();
    let pbm_path = temp_dir.join(format!("alpha-emu-lcd-ocr-{}.pbm", std::process::id()));
    write_lcd_pbm(&scaled, &pbm_path)?;
    let output = Command::new("tesseract")
        .arg(&pbm_path)
        .arg("stdout")
        .arg("--psm")
        .arg("6")
        .arg("-c")
        .arg("preserve_interword_spaces=1")
        .output()?;
    let _ = std::fs::remove_file(&pbm_path);
    if !output.status.success() {
        anyhow::bail!(
            "tesseract failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn write_lcd_debug_dump(
    text_screen: Option<&str>,
    snapshot: &LcdSnapshot,
    dir: &std::path::Path,
    scale: usize,
) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let full_bits_path = dir.join("lcd-full-bits.txt");
    let full_pbm_path = dir.join("lcd-full.pbm");
    let visible = visible_snapshot(snapshot);
    let visible_pbm_path = dir.join("lcd-visible.pbm");
    let visible_scaled = scale_snapshot(&cursor_blink_snapshot(&visible, false), scale.max(1));
    let visible_scaled_pbm_path = dir.join("lcd-visible-ocr.pbm");
    std::fs::write(&full_bits_path, render_snapshot_bits(snapshot))?;
    write_lcd_pbm(snapshot, &full_pbm_path)?;
    write_lcd_pbm(&visible, &visible_pbm_path)?;
    write_lcd_pbm(&visible_scaled, &visible_scaled_pbm_path)?;
    if let Some(text_screen) = text_screen {
        let text_screen_path = dir.join("text-screen.txt");
        std::fs::write(&text_screen_path, text_screen)?;
        println!("lcd_dump_text_screen={}", text_screen_path.display());
    }
    println!(
        "lcd_dump bits={} full_pbm={} visible_pbm={} visible_ocr_pbm={}",
        full_bits_path.display(),
        full_pbm_path.display(),
        visible_pbm_path.display(),
        visible_scaled_pbm_path.display()
    );
    match ocr_visible_lcd(text_screen, snapshot, scale) {
        Ok(text) => {
            let ocr_path = dir.join("lcd-ocr.txt");
            std::fs::write(&ocr_path, &text)?;
            println!("lcd_dump_ocr={}", ocr_path.display());
        }
        Err(error) => {
            println!("lcd_dump_ocr_error={error}");
        }
    }
    Ok(())
}

fn prefixed_path(prefix: &std::path::Path, suffix: &str) -> PathBuf {
    let mut path = prefix.to_path_buf();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name}-{suffix}"))
        .unwrap_or_else(|| suffix.to_string());
    path.set_file_name(file_name);
    path
}
