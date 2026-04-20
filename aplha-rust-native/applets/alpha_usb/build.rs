use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/entry.s");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let object = out_dir.join("alpha_usb_entry.o");
    let status = Command::new("m68k-elf-as")
        .arg("src/entry.s")
        .arg("-o")
        .arg(&object)
        .status()?;
    if !status.success() {
        return Err("m68k-elf-as failed while assembling Alpha USB entry".into());
    }

    println!("cargo:rustc-link-arg={}", object.display());
    Ok(())
}
