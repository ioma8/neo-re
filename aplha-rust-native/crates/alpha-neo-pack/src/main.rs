mod elf;
mod os3kapp;

use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use os3kapp::{AppletManifest, Version};

fn main() -> Result<(), Box<dyn Error>> {
    let command = parse_args(env::args().skip(1))?;
    let bytes = fs::read(&command.input)?;
    let code = elf::extract_load_image(&bytes)?;
    let image = os3kapp::build_image(&alpha_usb_manifest(), &code)?;
    os3kapp::validate_image(&image)?;

    if let Some(parent) = command.output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&command.output, image)?;
    println!("wrote {}", command.output.display());
    Ok(())
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Command, CliError> {
    let applet = args.next().ok_or(CliError::Usage)?;
    if applet != "alpha-usb" {
        return Err(CliError::UnknownApplet(applet));
    }

    let input = args.next().ok_or(CliError::Usage)?;
    let output = args.next().ok_or(CliError::Usage)?;
    if args.next().is_some() {
        return Err(CliError::Usage);
    }

    Ok(Command {
        input: PathBuf::from(input),
        output: PathBuf::from(output),
    })
}

fn alpha_usb_manifest() -> AppletManifest {
    AppletManifest {
        id: 0xA130,
        name: "Alpha USB",
        version: Version::decimal(1, 20),
        flags: 0xFF00_00CE,
        base_memory_size: 0x100,
        extra_memory_size: 0x2000,
        copyright: "neo-re benign SmartApplet probe",
        alphaword_write_metadata: true,
    }
}

struct Command {
    input: PathBuf,
    output: PathBuf,
}

#[derive(Debug)]
enum CliError {
    Usage,
    UnknownApplet(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => {
                f.write_str("usage: alpha-neo-pack alpha-usb <input-elf-or-a> <output.os3kapp>")
            }
            Self::UnknownApplet(name) => write!(f, "unknown applet: {name}"),
        }
    }
}

impl Error for CliError {}
