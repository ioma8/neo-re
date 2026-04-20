use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let applet_dir = manifest_dir.join("src").join("applets");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    println!("cargo:rerun-if-changed={}", applet_dir.display());
    let applets = discover_applets(&applet_dir)?;
    let generated = generate_registry(&applets);
    fs::write(out_dir.join("applets_generated.rs"), generated)?;
    Ok(())
}

fn discover_applets(applet_dir: &Path) -> Result<Vec<DiscoveredApplet>, Box<dyn Error>> {
    let mut applets = Vec::new();
    let mut module_names = BTreeSet::new();
    for entry in fs::read_dir(applet_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !is_rust_source(&path) {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or("applet filename must be valid UTF-8")?;
        let module_name = module_name_from_stem(stem);
        if !module_names.insert(module_name.clone()) {
            return Err(format!("duplicate generated applet module name: {module_name}").into());
        }
        applets.push(DiscoveredApplet { path, module_name });
    }
    applets.sort_by(|left, right| left.module_name.cmp(&right.module_name));
    Ok(applets)
}

fn is_rust_source(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("rs")
}

fn module_name_from_stem(stem: &str) -> String {
    let mut name = String::new();
    for character in stem.chars() {
        if character.is_ascii_alphanumeric() {
            name.push(character.to_ascii_lowercase());
        } else {
            name.push('_');
        }
    }
    if name
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        name.insert(0, '_');
    }
    name
}

fn generate_registry(applets: &[DiscoveredApplet]) -> String {
    let mut generated = String::new();
    for applet in applets {
        generated.push_str(&format!(
            "#[path = {:?}]\npub mod {};\n",
            applet.path, applet.module_name
        ));
    }
    generated.push_str("\npub fn all() -> &'static [AppletPackage] {\n    &[\n");
    for applet in applets {
        generated.push_str(&format!("        {}::PACKAGE,\n", applet.module_name));
    }
    generated.push_str("    ]\n}\n");
    generated
}

struct DiscoveredApplet {
    path: PathBuf,
    module_name: String,
}
