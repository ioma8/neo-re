use crate::sdk::{Action, AppletDefinition, Message};

const MSG_INIT: u32 = 0x18;
const MSG_SETFOCUS: u32 = 0x19;
const MSG_CHAR: u32 = 0x20;
const MSG_KEY: u32 = 0x21;
const MSG_IDENTITY: u32 = 0x26;
const MSG_USB_MAC_INIT: u32 = 0x10001;
const MSG_USB_PLUG: u32 = 0x30001;
const MSG_USB_PC_INIT: u32 = 0x20001;
const OTHER_USB_EVENTS: &[u32] = &[0x10003, 0x10006, 0x20002, 0x20006, 0x2011F];

#[derive(Debug)]
pub enum CompileError {
    BranchOutOfRange,
    MissingHandler(Message),
    UnsupportedAction,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BranchOutOfRange => f.write_str("generated branch target is out of range"),
            Self::MissingHandler(message) => write!(f, "missing handler for {message:?}"),
            Self::UnsupportedAction => f.write_str("unsupported action sequence"),
        }
    }
}

impl std::error::Error for CompileError {}

pub fn compile_applet(definition: &AppletDefinition) -> Result<Vec<u8>, CompileError> {
    let mut asm = Assembler::default();
    let mut branches = Branches::default();

    asm.bytes(&[0x20, 0x6F, 0x00, 0x0C]); // movea.l 0x0c(a7),a0 ; status_out
    asm.bytes(&[0x42, 0x90]); // clr.l (a0)
    asm.bytes(&[0x20, 0x2F, 0x00, 0x04]); // move.l 0x04(a7),d0 ; command

    branches.init_return = asm.branch_if_equal(MSG_INIT);
    branches.focus = asm.branch_if_equal(MSG_SETFOCUS);
    branches.generic_usb.push(asm.branch_if_equal(MSG_CHAR));
    branches.key = asm.branch_if_equal(MSG_KEY);
    branches.identity = asm.branch_if_equal(MSG_IDENTITY);
    branches.mac_init = asm.branch_if_equal(MSG_USB_MAC_INIT);
    branches.usb_plug = asm.branch_if_equal(MSG_USB_PLUG);
    branches.pc_init = asm.branch_if_equal(MSG_USB_PC_INIT);
    for event in OTHER_USB_EVENTS {
        branches.generic_usb.push(asm.branch_if_equal(*event));
    }
    branches.default_return = asm.branch_always();

    let return_offset = asm.offset();
    asm.rts();

    let focus_offset = asm.offset();
    emit_actions(
        &mut asm,
        actions_for(definition, Message::SetFocus)?,
        &definition.manifest,
    )?;

    let generic_usb_offset = asm.offset();
    emit_actions(
        &mut asm,
        actions_for(definition, Message::Char)?,
        &definition.manifest,
    )?;

    let key_actions = actions_for(definition, Message::Key)?;
    let key_offset = if key_actions == actions_for(definition, Message::Char)? {
        generic_usb_offset
    } else {
        let offset = asm.offset();
        emit_actions(&mut asm, key_actions, &definition.manifest)?;
        offset
    };

    let init_status_offset = asm.offset();
    emit_actions(
        &mut asm,
        actions_for(definition, Message::UsbMacInit)?,
        &definition.manifest,
    )?;

    let usb_plug_offset = asm.offset();
    emit_actions(
        &mut asm,
        actions_for(definition, Message::UsbPlug)?,
        &definition.manifest,
    )?;

    let pc_init_offset = asm.offset();
    emit_actions(
        &mut asm,
        actions_for(definition, Message::UsbPcInit)?,
        &definition.manifest,
    )?;

    let identity_offset = asm.offset();
    emit_actions(
        &mut asm,
        actions_for(definition, Message::Identity)?,
        &definition.manifest,
    )?;

    asm.emit_trap_stubs();
    asm.patch_branch(branches.init_return, return_offset)?;
    asm.patch_branch(branches.focus, focus_offset)?;
    for branch in branches.generic_usb {
        asm.patch_branch(branch, generic_usb_offset)?;
    }
    asm.patch_branch(branches.key, key_offset)?;
    asm.patch_branch(branches.identity, identity_offset)?;
    asm.patch_branch(branches.mac_init, init_status_offset)?;
    asm.patch_branch(branches.usb_plug, usb_plug_offset)?;
    asm.patch_branch(branches.pc_init, pc_init_offset)?;
    asm.patch_branch(branches.default_return, return_offset)?;
    asm.patch_trap_calls()?;

    Ok(asm.finish())
}

fn actions_for(definition: &AppletDefinition, message: Message) -> Result<&[Action], CompileError> {
    definition
        .handlers
        .iter()
        .find(|handler| handler.message == message)
        .map(|handler| handler.actions.as_slice())
        .ok_or(CompileError::MissingHandler(message))
}

fn emit_actions(
    asm: &mut Assembler,
    actions: &[Action],
    manifest: &crate::sdk::AppletManifest,
) -> Result<(), CompileError> {
    for action in actions {
        match action {
            Action::ClearScreen => asm.call_trap(Trap::ClearScreen),
            Action::WriteLines { start_row, lines } => {
                for (index, line) in lines.iter().enumerate() {
                    let row = start_row
                        .checked_add(
                            u8::try_from(index).map_err(|_| CompileError::UnsupportedAction)?,
                        )
                        .ok_or(CompileError::UnsupportedAction)?;
                    asm.set_text_row(row);
                    for byte in line.bytes() {
                        asm.draw_char(byte);
                    }
                }
            }
            Action::IdleForever => {
                asm.call_trap(Trap::FlushText);
                asm.call_trap(Trap::Yield);
                asm.bytes(&[0x60, 0xFA]); // bra.s yield loop
            }
            Action::ReturnStatus(status) => asm.status_return(*status),
            Action::CompleteHidToDirect => {
                asm.bytes(&[0x42, 0xA7]); // clr.l -(a7)
                asm.bytes(&[0x42, 0xA7]); // clr.l -(a7)
                asm.bytes(&[0x48, 0x78, 0x00, 0x01]); // pea.l 1
                asm.jsr_absolute(0x0041_F9A0);
                asm.bytes(&[0x48, 0x78, 0x00, 0x64]); // pea.l 100
                asm.jsr_absolute(0x0042_4780);
                asm.bytes(&[0x13, 0xFC, 0x00, 0x01, 0x00, 0x01, 0x3C, 0xF9]);
                asm.jsr_absolute(0x0044_044E);
                asm.bytes(&[0x48, 0x78, 0x00, 0x64]); // pea.l 100
                asm.jsr_absolute(0x0042_4780);
                asm.bytes(&[0x4F, 0xEF, 0x00, 0x14]); // lea.l 0x14(a7),a7
                asm.jsr_absolute(0x0044_047C);
            }
            Action::MarkDirectConnected => asm.jsr_absolute(0x0041_0B26),
            Action::ReturnAppletId => {
                asm.bytes(&[0x22, 0x3C]); // move.l #applet_id,d1
                asm.u32(u32::from(manifest.id.0));
                asm.bytes(&[0x20, 0x81]); // move.l d1,(a0)
                asm.rts();
            }
            Action::IfKey { key, actions } => {
                asm.bytes(&[0x22, 0x2F, 0x00, 0x08]); // move.l 0x08(a7),d1 ; param
                asm.bytes(&[0x0C, 0x81]); // cmpi.l #key,d1
                asm.u32(key.raw_value());
                let skip_branch = asm.branch(0x66); // bne.w after nested actions
                emit_actions(asm, actions, manifest)?;
                let after_nested = asm.offset();
                asm.patch_branch(skip_branch, after_nested)?;
            }
        };
    }
    Ok(())
}

#[derive(Default)]
struct Branches {
    init_return: usize,
    focus: usize,
    key: usize,
    generic_usb: Vec<usize>,
    identity: usize,
    mac_init: usize,
    usb_plug: usize,
    pc_init: usize,
    default_return: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Trap {
    ClearScreen,
    SetTextRow,
    DrawChar,
    FlushText,
    Yield,
}

impl Trap {
    fn opcode(self) -> u16 {
        match self {
            Self::ClearScreen => 0xA000,
            Self::SetTextRow => 0xA004,
            Self::DrawChar => 0xA010,
            Self::FlushText => 0xA098,
            Self::Yield => 0xA25C,
        }
    }
}

#[derive(Default)]
struct Assembler {
    code: Vec<u8>,
    trap_calls: Vec<(usize, Trap)>,
    trap_stubs: Vec<(Trap, usize)>,
}

impl Assembler {
    fn finish(self) -> Vec<u8> {
        self.code
    }

    fn offset(&self) -> usize {
        self.code.len()
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn u32(&mut self, value: u32) {
        self.code.extend_from_slice(&value.to_be_bytes());
    }

    fn rts(&mut self) {
        self.bytes(&[0x4E, 0x75]);
    }

    fn branch_if_equal(&mut self, value: u32) -> usize {
        self.bytes(&[0x0C, 0x80]);
        self.u32(value);
        self.branch(0x67)
    }

    fn branch_always(&mut self) -> usize {
        self.branch(0x60)
    }

    fn branch(&mut self, opcode: u8) -> usize {
        let offset = self.offset();
        self.bytes(&[opcode, 0x00, 0x00, 0x00]);
        offset
    }

    fn patch_branch(
        &mut self,
        branch_offset: usize,
        target_offset: usize,
    ) -> Result<(), CompileError> {
        let displacement = isize::try_from(target_offset)
            .map_err(|_| CompileError::BranchOutOfRange)?
            - isize::try_from(branch_offset + 2).map_err(|_| CompileError::BranchOutOfRange)?;
        let displacement =
            i16::try_from(displacement).map_err(|_| CompileError::BranchOutOfRange)?;
        self.code[branch_offset + 2..branch_offset + 4]
            .copy_from_slice(&displacement.to_be_bytes());
        Ok(())
    }

    fn call_trap(&mut self, trap: Trap) {
        let offset = self.offset();
        self.bytes(&[0x61, 0x00, 0x00, 0x00]);
        self.trap_calls.push((offset, trap));
    }

    fn emit_trap_stubs(&mut self) {
        for trap in [
            Trap::ClearScreen,
            Trap::SetTextRow,
            Trap::DrawChar,
            Trap::FlushText,
            Trap::Yield,
        ] {
            let offset = self.offset();
            self.trap_stubs.push((trap, offset));
            self.bytes(&trap.opcode().to_be_bytes());
        }
    }

    fn patch_trap_calls(&mut self) -> Result<(), CompileError> {
        for (call_offset, trap) in self.trap_calls.clone() {
            let Some((_, target_offset)) = self
                .trap_stubs
                .iter()
                .find(|(candidate, _)| *candidate == trap)
            else {
                return Err(CompileError::UnsupportedAction);
            };
            self.patch_branch(call_offset, *target_offset)?;
        }
        Ok(())
    }

    fn set_text_row(&mut self, row: u8) {
        self.bytes(&[0x2F, 0x3C, 0x00, 0x00, 0x00, 0x1C]);
        self.bytes(&[0x2F, 0x3C, 0x00, 0x00, 0x00, 0x01]);
        self.bytes(&[0x2F, 0x3C, 0x00, 0x00, 0x00, row]);
        self.call_trap(Trap::SetTextRow);
        self.bytes(&[0x4F, 0xEF, 0x00, 0x0C]);
    }

    fn draw_char(&mut self, byte: u8) {
        self.bytes(&[0x70, byte]);
        self.bytes(&[0x2F, 0x00]);
        self.call_trap(Trap::DrawChar);
        self.bytes(&[0x58, 0x8F]);
    }

    fn status_return(&mut self, status: u8) {
        self.bytes(&[0x72, status, 0x20, 0x81]);
        self.rts();
    }

    fn jsr_absolute(&mut self, address: u32) {
        self.bytes(&[0x4E, 0xB9]);
        self.u32(address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alpha_usb;

    #[test]
    fn compiles_alpha_usb_from_sdk_definition() -> Result<(), Box<dyn std::error::Error>> {
        let code = compile_applet(&alpha_usb::define_alpha_usb())?;

        assert!(code.starts_with(&[0x20, 0x6F, 0x00, 0x0C]));
        assert!(
            code.windows(6)
                .any(|window| window == [0x4E, 0xB9, 0x00, 0x44, 0x04, 0x7C])
        );
        assert!(
            code.windows(6)
                .any(|window| window == [0x4E, 0xB9, 0x00, 0x41, 0x0B, 0x26])
        );
        assert!(
            code.windows(4)
                .any(|window| window == [0x72, 0x11, 0x20, 0x81])
        );
        assert!(code.ends_with(&[0xA0, 0x00, 0xA0, 0x04, 0xA0, 0x10, 0xA0, 0x98, 0xA2, 0x5C]));
        Ok(())
    }
}
