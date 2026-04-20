use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::applets::{self, AppletPackage};

const DEFAULT_OUTPUT_DIR: &str = "../exports/applets";

pub fn run() -> Result<(), Box<dyn Error>> {
    match parse_command(env::args_os())? {
        Command::List => {
            print_registered_applets();
            Ok(())
        }
        Command::Build { target, output_dir } => build_target(target, &output_dir),
    }
}

fn print_registered_applets() {
    for package in applets::all() {
        println!("{}\t{}", package.name, package.output_filename);
    }
}

fn build_target(target: BuildTarget, output_dir: &Path) -> Result<(), Box<dyn Error>> {
    match target {
        BuildTarget::All => {
            for package in applets::all() {
                let path = build_package(package, output_dir)?;
                println!("wrote={}", path.display());
            }
            Ok(())
        }
        BuildTarget::One(name) => {
            let package = applets::find(&name).ok_or_else(|| unknown_applet(&name))?;
            let path = build_package(package, output_dir)?;
            println!("wrote={}", path.display());
            Ok(())
        }
    }
}

fn build_package(package: &AppletPackage, output_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let definition = (package.build)();
    let entry_code = crate::compiler::compile_applet(&definition)
        .map_err(|error| CliError(format!("compile failed for {}: {error}", package.name)))?;
    let image = crate::os3kapp::build_image(&definition.manifest, &entry_code)
        .map_err(|error| CliError(format!("package failed for {}: {error}", package.name)))?;
    (package.validate)(&image)
        .map_err(|error| CliError(format!("validation failed for {}: {error}", package.name)))?;

    fs::create_dir_all(output_dir).map_err(|error| {
        CliError(format!(
            "could not create output directory {}: {error}",
            output_dir.display()
        ))
    })?;
    let output_path = output_dir.join(package.output_filename);
    fs::write(&output_path, image).map_err(|error| {
        CliError(format!(
            "could not write output file {}: {error}",
            output_path.display()
        ))
    })?;
    Ok(output_path)
}

fn parse_command(mut args: impl Iterator<Item = OsString>) -> Result<Command, CliError> {
    let _program = args.next();
    let Some(command) = args.next() else {
        return Err(usage_error());
    };

    if command == "list" {
        no_more_args(args)?;
        return Ok(Command::List);
    }

    if command != "build" {
        return Err(usage_error());
    }
    parse_build(args)
}

fn parse_build(mut args: impl Iterator<Item = OsString>) -> Result<Command, CliError> {
    let Some(target) = args.next() else {
        return Err(usage_error());
    };
    let target = target
        .into_string()
        .map_err(|_| CliError("build target must be valid UTF-8".to_string()))?;
    let mut output_dir = PathBuf::from(DEFAULT_OUTPUT_DIR);
    while let Some(flag) = args.next() {
        if flag != "--output-dir" {
            return Err(usage_error());
        }
        let Some(path) = args.next() else {
            return Err(usage_error());
        };
        output_dir = PathBuf::from(path);
    }

    let target = if target == "all" {
        BuildTarget::All
    } else {
        BuildTarget::One(target)
    };
    Ok(Command::Build { target, output_dir })
}

fn no_more_args(mut args: impl Iterator<Item = OsString>) -> Result<(), CliError> {
    if args.next().is_some() {
        return Err(usage_error());
    }
    Ok(())
}

fn unknown_applet(name: &str) -> CliError {
    let available = applets::all()
        .iter()
        .map(|package| package.name)
        .collect::<Vec<_>>()
        .join(", ");
    CliError(format!("unknown applet '{name}'; available: {available}"))
}

fn usage_error() -> CliError {
    let usage = "usage: cargo run -- list | cargo run -- build <applet|all> [--output-dir <path>]";
    CliError(usage.to_string())
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Build {
        target: BuildTarget,
        output_dir: PathBuf,
    },
    List,
}

#[derive(Debug, Eq, PartialEq)]
enum BuildTarget {
    All,
    One(String),
}

#[derive(Debug)]
struct CliError(String);

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(parts: &[&str]) -> Result<Command, CliError> {
        parse_command(parts.iter().map(OsString::from))
    }

    #[test]
    fn parses_list_command() -> Result<(), CliError> {
        assert_eq!(parse(&["tool", "list"])?, Command::List);
        Ok(())
    }

    #[test]
    fn parses_single_build_command() -> Result<(), CliError> {
        assert_eq!(
            parse(&["tool", "build", "alpha_usb"])?,
            Command::Build {
                target: BuildTarget::One("alpha_usb".to_string()),
                output_dir: PathBuf::from(DEFAULT_OUTPUT_DIR),
            }
        );
        Ok(())
    }

    #[test]
    fn parses_all_build_with_output_dir() -> Result<(), CliError> {
        assert_eq!(
            parse(&["tool", "build", "all", "--output-dir", "../exports"])?,
            Command::Build {
                target: BuildTarget::All,
                output_dir: PathBuf::from("../exports"),
            }
        );
        Ok(())
    }
}
