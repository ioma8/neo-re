mod app;
mod backup;
mod protocol;
mod ui;
mod ui_render;
mod usb;
mod usb_support;

use anyhow::Context;
use tracing_subscriber::{fmt, prelude::*};

fn main() -> anyhow::Result<()> {
    let log_dir = backup::app_dir()?.join("logs");
    std::fs::create_dir_all(&log_dir).context("create log directory")?;
    let log_file =
        std::fs::File::create(log_dir.join("alpha-cli.log")).context("create log file")?;
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(log_file).with_ansi(false))
        .init();

    if let Err(error) = ui::run() {
        tracing::error!(error = ?error, "alpha-cli failed");
        return Err(error);
    }
    Ok(())
}
