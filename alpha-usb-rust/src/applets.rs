use std::error::Error;

use crate::sdk::AppletDefinition;

pub type AppletValidator = fn(&[u8]) -> Result<(), Box<dyn Error>>;

#[derive(Clone, Copy)]
pub struct AppletPackage {
    pub name: &'static str,
    pub output_filename: &'static str,
    pub build: fn() -> AppletDefinition,
    pub validate: AppletValidator,
}

pub fn find(name: &str) -> Option<&'static AppletPackage> {
    all().iter().find(|package| package.name == name)
}

pub fn validate_basic(image: &[u8]) -> Result<(), Box<dyn Error>> {
    crate::os3kapp::validate_image(image)?;
    Ok(())
}

pub fn validate_alpha_usb(image: &[u8]) -> Result<(), Box<dyn Error>> {
    crate::os3kapp::validate_alpha_usb_image(image)?;
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/applets_generated.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_registry_contains_alpha_usb() -> Result<(), &'static str> {
        let Some(package) = find("alpha_usb") else {
            return Err("alpha_usb should be discovered");
        };

        assert_eq!(package.output_filename, "alpha-usb.os3kapp");
        assert!(find("missing").is_none());
        Ok(())
    }
}
