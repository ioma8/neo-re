use std::path::PathBuf;

use alpha_emu::firmware::FirmwareRuntime;
use alpha_emu::firmware_session::FirmwareSession;
use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut headless = false;
    let mut path = None;
    let mut steps = 10_000;
    for arg in std::env::args_os().skip(1) {
        if arg == "--headless" {
            headless = true;
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--steps=")) {
            steps = value.parse()?;
        } else {
            path = Some(PathBuf::from(arg));
        }
    }
    let path = path.unwrap_or_else(|| PathBuf::from("../analysis/cab/smallos3kneorom.os3kos"));

    if headless {
        let firmware = FirmwareRuntime::load_small_rom(path)?;
        let mut session = FirmwareSession::boot_small_rom(firmware)?;
        session.run_steps(steps);
        let snapshot = session.snapshot();
        println!(
            "pc=0x{:08x} ssp=0x{:08x} steps={} stopped={} exception={}",
            snapshot.pc,
            snapshot.ssp,
            snapshot.steps,
            snapshot.stopped,
            snapshot.last_exception.as_deref().unwrap_or("none")
        );
        println!("mmio:");
        for access in &snapshot.mmio_accesses {
            println!("  {access}");
        }
        println!("trace:");
        for line in &snapshot.trace {
            println!("  {line}");
        }
        Ok(())
    } else {
        alpha_emu::gui::run(path)
    }
}
