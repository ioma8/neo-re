use crate::applets::{AppletPackage, validate_alpha_usb};
use crate::sdk::{AppletId, AppletManifest, NeoApplet, Status, UiContext, UsbContext, Version};
use crate::sdk::{AppletDefinition, define};

pub const PACKAGE: AppletPackage = AppletPackage {
    name: "alpha_usb",
    output_filename: "alpha-usb.os3kapp",
    build,
    validate: validate_alpha_usb,
};

fn build() -> AppletDefinition {
    define(AlphaUsb)
}

pub struct AlphaUsb;

impl NeoApplet for AlphaUsb {
    const MANIFEST: AppletManifest =
        AppletManifest::alpha_usb_bridge(AppletId(0xA130), "Alpha USB", Version::decimal(1, 20));

    fn on_focus(&self, ctx: &mut UiContext) {
        ctx.screen().clear();
        ctx.screen().write_lines(
            2,
            [
                "Now connect the NEO",
                "to your computer or",
                "smartphone via USB.",
            ],
        );
        ctx.events().idle_forever();
    }

    fn on_usb_plug(&self, ctx: &mut UsbContext) {
        ctx.usb().complete_hid_to_direct();
        ctx.usb().mark_direct_connected();
        ctx.status(Status::USB_HANDLED);
    }
}
