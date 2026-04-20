use crate::sdk::{
    AppletDefinition, AppletId, AppletManifest, NeoApplet, Status, UiContext, UsbContext, Version,
    define,
};

pub struct AlphaUsb;

impl NeoApplet for AlphaUsb {
    const MANIFEST: AppletManifest = AppletManifest {
        id: AppletId(0xA130),
        name: "Alpha USB",
        version: Version {
            major_bcd: 0x01,
            minor_bcd: 0x20,
        },
        flags: 0xFF00_00CE,
        base_memory_size: 0x100,
        extra_memory_size: 0x2000,
        copyright: "neo-re benign SmartApplet probe",
        alphaword_write_metadata: true,
    };

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
        ctx.status(Status::raw(0x11));
    }
}

pub fn define_alpha_usb() -> AppletDefinition {
    define(AlphaUsb)
}
