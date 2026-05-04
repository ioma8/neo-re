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
    let manifest = manifest_for(command.applet);
    let image = os3kapp::build_image(&manifest, &code)?;
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
    let applet = AppletName::parse(&applet).ok_or(CliError::UnknownApplet(applet))?;

    let input = args.next().ok_or(CliError::Usage)?;
    let output = args.next().ok_or(CliError::Usage)?;
    if args.next().is_some() {
        return Err(CliError::Usage);
    }

    Ok(Command {
        applet,
        input: PathBuf::from(input),
        output: PathBuf::from(output),
    })
}

fn manifest_for(applet: AppletName) -> AppletManifest {
    match applet {
        AppletName::AlphaUsb => AppletManifest {
            id: 0xA130,
            name: "Alpha USB",
            version: Version::decimal(1, 20),
            flags: 0xFF00_00CE,
            base_memory_size: 0x100,
            extra_memory_size: 0x2000,
            copyright: "neo-re benign SmartApplet probe",
            file_count: 0,
            alphaword_write_metadata: true,
        },
        AppletName::ForthMini => AppletManifest {
            id: 0xA131,
            name: "Forth Mini",
            version: Version::decimal(0, 2),
            flags: 0xFF00_00CE,
            base_memory_size: 0x4000,
            extra_memory_size: 0x2000,
            copyright: "neo-re Betawise Forth SmartApplet",
            file_count: 1,
            alphaword_write_metadata: true,
        },
        AppletName::BasicWriter => AppletManifest {
            id: 0xA132,
            name: "Basic Writer",
            version: Version::decimal(0, 1),
            flags: 0xFF00_00CE,
            base_memory_size: 0x4000,
            extra_memory_size: 0x2000,
            copyright: "neo-re Betawise Basic Writer SmartApplet",
            file_count: 8,
            alphaword_write_metadata: true,
        },
        AppletName::WriteOrDie => AppletManifest {
            id: 0xA133,
            name: "WriteOrDie",
            version: Version::decimal(0, 1),
            flags: 0xFF00_00CE,
            base_memory_size: 0x4000,
            extra_memory_size: 0x2000,
            copyright: "neo-re Betawise WriteOrDie SmartApplet",
            file_count: 1,
            alphaword_write_metadata: true,
        },
        AppletName::FloppyBird => AppletManifest {
            id: 0xA134,
            name: "Floppy Bird",
            version: Version::decimal(0, 1),
            flags: 0xFF00_00CE,
            base_memory_size: 0x2000,
            extra_memory_size: 0x1000,
            copyright: "neo-re Betawise Floppy Bird SmartApplet",
            file_count: 0,
            alphaword_write_metadata: true,
        },
        AppletName::Snake => AppletManifest {
            id: 0xA135,
            name: "Snake",
            version: Version::decimal(0, 1),
            flags: 0xFF00_00CE,
            base_memory_size: 0x3000,
            extra_memory_size: 0x1000,
            copyright: "neo-re Betawise Snake SmartApplet",
            file_count: 0,
            alphaword_write_metadata: true,
        },
        AppletName::Raycaster => AppletManifest {
            id: 0xA136,
            name: "Raycaster",
            version: Version::decimal(0, 1),
            flags: 0xFF00_00CE,
            base_memory_size: 0x3000,
            extra_memory_size: 0x1000,
            copyright: "neo-re Betawise Raycaster SmartApplet",
            file_count: 0,
            alphaword_write_metadata: true,
        },
    }
}

struct Command {
    applet: AppletName,
    input: PathBuf,
    output: PathBuf,
}

#[derive(Clone, Copy)]
enum AppletName {
    AlphaUsb,
    ForthMini,
    BasicWriter,
    WriteOrDie,
    FloppyBird,
    Snake,
    Raycaster,
}

impl AppletName {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "alpha-usb" => Some(Self::AlphaUsb),
            "forth-mini" => Some(Self::ForthMini),
            "basic-writer" => Some(Self::BasicWriter),
            "write-or-die" => Some(Self::WriteOrDie),
            "floppy-bird" => Some(Self::FloppyBird),
            "snake" => Some(Self::Snake),
            "raycaster" => Some(Self::Raycaster),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum CliError {
    Usage,
    UnknownApplet(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => f.write_str(
                "usage: alpha-neo-pack <alpha-usb|forth-mini|basic-writer|write-or-die|floppy-bird|snake|raycaster> <input-elf-or-a> <output.os3kapp>",
            ),
            Self::UnknownApplet(name) => write!(f, "unknown applet: {name}"),
        }
    }
}

impl Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_floppy_bird_manifest() {
        let applet = AppletName::parse("floppy-bird").expect("floppy-bird applet name");
        let manifest = manifest_for(applet);

        assert_eq!(manifest.id, 0xA134);
        assert_eq!(manifest.name, "Floppy Bird");
        assert_eq!(manifest.file_count, 0);
        assert!(manifest.alphaword_write_metadata);
    }

    #[test]
    fn parses_snake_manifest() {
        let applet = AppletName::parse("snake").expect("snake applet name");
        let manifest = manifest_for(applet);

        assert_eq!(manifest.id, 0xA135);
        assert_eq!(manifest.name, "Snake");
        assert_eq!(manifest.file_count, 0);
        assert!(manifest.alphaword_write_metadata);
    }

    #[test]
    fn parses_raycaster_manifest() {
        let applet = AppletName::parse("raycaster").expect("raycaster applet name");
        let manifest = manifest_for(applet);

        assert_eq!(manifest.id, 0xA136);
        assert_eq!(manifest.name, "Raycaster");
        assert_eq!(manifest.file_count, 0);
        assert!(manifest.alphaword_write_metadata);
    }
}
