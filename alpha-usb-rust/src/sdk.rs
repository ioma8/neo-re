#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppletId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version {
    pub major_bcd: u8,
    pub minor_bcd: u8,
}

impl Version {
    pub const fn new(major: u8, minor: u8) -> Self {
        Self {
            major_bcd: major,
            minor_bcd: minor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppletManifest {
    pub id: AppletId,
    pub name: &'static str,
    pub version: Version,
    pub flags: u32,
    pub base_memory_size: u32,
    pub extra_memory_size: u32,
    pub copyright: &'static str,
    pub alphaword_write_metadata: bool,
}

impl AppletManifest {
    pub const fn basic(id: AppletId, name: &'static str, version: Version) -> Self {
        Self {
            id,
            name,
            version,
            flags: 0xFF00_0000,
            base_memory_size: 0x100,
            extra_memory_size: 0,
            copyright: "neo-re SmartApplet",
            alphaword_write_metadata: false,
        }
    }

    pub const fn alpha_usb_bridge(id: AppletId, name: &'static str, version: Version) -> Self {
        Self {
            id,
            name,
            version,
            flags: 0xFF00_00CE,
            base_memory_size: 0x100,
            extra_memory_size: 0x2000,
            copyright: "neo-re benign SmartApplet probe",
            alphaword_write_metadata: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppletDefinition {
    pub manifest: AppletManifest,
    pub handlers: Vec<MessageHandler>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageHandler {
    pub message: Message,
    pub actions: Vec<Action>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Message {
    Init,
    SetFocus,
    Char,
    Key,
    Identity,
    UsbMacInit,
    UsbPlug,
    UsbPcInit,
    OtherUsb(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    ClearScreen,
    WriteLines {
        start_row: u8,
        lines: Vec<&'static str>,
    },
    IdleForever,
    ReturnStatus(u8),
    CompleteHidToDirect,
    MarkDirectConnected,
    ReturnAppletId,
    IfKey {
        key: Key,
        actions: Vec<Action>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status(u8);

impl Status {
    pub const OK: Self = Self(0);

    pub const fn raw(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

pub trait NeoApplet {
    const MANIFEST: AppletManifest;

    fn on_init(&self, ctx: &mut SystemContext) {
        ctx.status(Status::OK);
    }

    fn on_focus(&self, ctx: &mut UiContext) {
        ctx.status(Status::OK);
    }

    fn on_char(&self, ctx: &mut SystemContext) {
        ctx.status(Status::raw(0x04));
    }

    fn on_key(&self, ctx: &mut KeyContext) {
        ctx.status(Status::raw(0x04));
    }

    fn on_identity(&self, ctx: &mut IdentityContext) {
        ctx.return_applet_id();
    }

    fn on_usb_mac_init(&self, ctx: &mut UsbContext) {
        ctx.status(Status::raw(0x11));
    }

    fn on_usb_plug(&self, ctx: &mut UsbContext) {
        ctx.status(Status::raw(0x11));
    }

    fn on_usb_pc_init(&self, ctx: &mut UsbContext) {
        ctx.status(Status::raw(0x11));
    }

    fn on_other_usb(&self, ctx: &mut SystemContext, _message: u32) {
        ctx.status(Status::raw(0x04));
    }
}

pub fn define<A: NeoApplet>(applet: A) -> AppletDefinition {
    let mut handlers = Vec::new();

    let mut init = SystemContext::default();
    applet.on_init(&mut init);
    handlers.push(init.into_handler(Message::Init));

    let mut focus = UiContext::default();
    applet.on_focus(&mut focus);
    handlers.push(focus.into_handler(Message::SetFocus));

    let mut char_ctx = SystemContext::default();
    applet.on_char(&mut char_ctx);
    handlers.push(char_ctx.into_handler(Message::Char));

    let mut key_ctx = KeyContext::default();
    applet.on_key(&mut key_ctx);
    handlers.push(key_ctx.into_handler(Message::Key));

    let mut identity = IdentityContext::default();
    applet.on_identity(&mut identity);
    handlers.push(identity.into_handler(Message::Identity));

    let mut mac = UsbContext::default();
    applet.on_usb_mac_init(&mut mac);
    handlers.push(mac.into_handler(Message::UsbMacInit));

    let mut plug = UsbContext::default();
    applet.on_usb_plug(&mut plug);
    handlers.push(plug.into_handler(Message::UsbPlug));

    let mut pc = UsbContext::default();
    applet.on_usb_pc_init(&mut pc);
    handlers.push(pc.into_handler(Message::UsbPcInit));

    for message in [0x10003, 0x10006, 0x20002, 0x20006, 0x2011F] {
        let mut other = SystemContext::default();
        applet.on_other_usb(&mut other, message);
        handlers.push(other.into_handler(Message::OtherUsb(message)));
    }

    AppletDefinition {
        manifest: A::MANIFEST,
        handlers,
    }
}

#[derive(Default)]
pub struct SystemContext {
    actions: Vec<Action>,
}

impl SystemContext {
    pub fn status(&mut self, status: Status) {
        self.actions.push(Action::ReturnStatus(status.value()));
    }

    fn into_handler(self, message: Message) -> MessageHandler {
        MessageHandler {
            message,
            actions: self.actions,
        }
    }
}

#[derive(Default)]
pub struct UiContext {
    actions: Vec<Action>,
}

impl UiContext {
    pub fn screen(&mut self) -> Screen<'_> {
        Screen {
            actions: &mut self.actions,
        }
    }

    pub fn events(&mut self) -> Events<'_> {
        Events {
            actions: &mut self.actions,
        }
    }

    pub fn status(&mut self, status: Status) {
        self.actions.push(Action::ReturnStatus(status.value()));
    }

    fn into_handler(self, message: Message) -> MessageHandler {
        MessageHandler {
            message,
            actions: self.actions,
        }
    }
}

pub struct Screen<'a> {
    actions: &'a mut Vec<Action>,
}

impl Screen<'_> {
    pub fn clear(&mut self) {
        self.actions.push(Action::ClearScreen);
    }

    pub fn write_lines<const N: usize>(&mut self, start_row: u8, lines: [&'static str; N]) {
        self.actions.push(Action::WriteLines {
            start_row,
            lines: Vec::from(lines),
        });
    }
}

pub struct Events<'a> {
    actions: &'a mut Vec<Action>,
}

impl Events<'_> {
    pub fn idle_forever(&mut self) {
        self.actions.push(Action::IdleForever);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Esc,
    Raw(u32),
}

impl Key {
    pub const fn raw_value(self) -> u32 {
        match self {
            Self::Esc => 0x48,
            Self::Raw(value) => value,
        }
    }
}

#[derive(Default)]
pub struct KeyContext {
    actions: Vec<Action>,
}

impl KeyContext {
    pub fn when_key(&mut self, key: Key, build: impl FnOnce(&mut SystemContext)) {
        let mut nested = SystemContext::default();
        build(&mut nested);
        self.actions.push(Action::IfKey {
            key,
            actions: nested.actions,
        });
    }

    pub fn status(&mut self, status: Status) {
        self.actions.push(Action::ReturnStatus(status.value()));
    }

    fn into_handler(self, message: Message) -> MessageHandler {
        MessageHandler {
            message,
            actions: self.actions,
        }
    }
}

#[derive(Default)]
pub struct UsbContext {
    actions: Vec<Action>,
}

impl UsbContext {
    pub fn usb(&mut self) -> Usb<'_> {
        Usb {
            actions: &mut self.actions,
        }
    }

    pub fn status(&mut self, status: Status) {
        self.actions.push(Action::ReturnStatus(status.value()));
    }

    fn into_handler(self, message: Message) -> MessageHandler {
        MessageHandler {
            message,
            actions: self.actions,
        }
    }
}

pub struct Usb<'a> {
    actions: &'a mut Vec<Action>,
}

impl Usb<'_> {
    pub fn complete_hid_to_direct(&mut self) {
        self.actions.push(Action::CompleteHidToDirect);
    }

    pub fn mark_direct_connected(&mut self) {
        self.actions.push(Action::MarkDirectConnected);
    }
}

#[derive(Default)]
pub struct IdentityContext {
    actions: Vec<Action>,
}

impl IdentityContext {
    pub fn return_applet_id(&mut self) {
        self.actions.push(Action::ReturnAppletId);
    }

    fn into_handler(self, message: Message) -> MessageHandler {
        MessageHandler {
            message,
            actions: self.actions,
        }
    }
}
