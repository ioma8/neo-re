use std::path::PathBuf;

use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let path = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from("../analysis/cab/smallos3kneorom.os3kos"),
        PathBuf::from,
    );

    alpha_emu::gui::run(path)
}
