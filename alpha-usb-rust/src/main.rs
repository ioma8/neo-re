use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(path) => {
            println!("wrote={}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<PathBuf, Box<dyn Error>> {
    let output_path = parse_output_path(env::args_os())?;
    let definition = alpha_usb_rust::alpha_usb::define_alpha_usb();
    let entry_code = alpha_usb_rust::compiler::compile_applet(&definition)?;
    let image = alpha_usb_rust::os3kapp::build_image(&definition.manifest, &entry_code)?;
    alpha_usb_rust::os3kapp::validate_alpha_usb_image(&image)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, image)?;
    Ok(output_path)
}

fn parse_output_path(mut args: impl Iterator<Item = OsString>) -> Result<PathBuf, Box<dyn Error>> {
    let _program = args.next();
    match args.next() {
        None => Ok(PathBuf::from("../exports/alpha-usb-rust.os3kapp")),
        Some(flag) if flag == "--output" => match args.next() {
            Some(path) if args.next().is_none() => Ok(PathBuf::from(path)),
            _ => Err("usage: cargo run -- --output <path>".into()),
        },
        Some(_) => Err("usage: cargo run -- [--output <path>]".into()),
    }
}
