pub mod alpha_usb;

use std::error::Error;

use crate::sdk::{AppletDefinition, define};

pub type AppletValidator = fn(&[u8]) -> Result<(), Box<dyn Error>>;

pub struct AppletPackage {
    pub name: &'static str,
    pub output_filename: &'static str,
    pub build: fn() -> AppletDefinition,
    pub validate: AppletValidator,
}

pub fn all() -> &'static [AppletPackage] {
    &[ALPHA_USB]
}

pub fn find(name: &str) -> Option<&'static AppletPackage> {
    all().iter().find(|package| package.name == name)
}

fn build_alpha_usb() -> AppletDefinition {
    define(alpha_usb::AlphaUsb)
}

fn validate_alpha_usb(image: &[u8]) -> Result<(), Box<dyn Error>> {
    crate::os3kapp::validate_alpha_usb_image(image)?;
    Ok(())
}

const ALPHA_USB: AppletPackage = AppletPackage {
    name: "alpha_usb",
    output_filename: "alpha-usb.os3kapp",
    build: build_alpha_usb,
    validate: validate_alpha_usb,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_alpha_usb() -> Result<(), &'static str> {
        let Some(package) = find("alpha_usb") else {
            return Err("alpha_usb should be registered");
        };

        assert_eq!(package.output_filename, "alpha-usb.os3kapp");
        assert!(find("missing").is_none());
        Ok(())
    }
}
