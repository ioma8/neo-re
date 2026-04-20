use std::error::Error;
use std::fmt::{Display, Formatter};

use goblin::{Object, archive::Archive, elf::Elf};

#[derive(Debug)]
pub enum ExtractError {
    ArchiveMemberMissing,
    MissingTextSection,
    UnsupportedObject,
    Goblin(goblin::error::Error),
}

impl Display for ExtractError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchiveMemberMissing => {
                f.write_str("archive has no member defining alpha_usb_entry")
            }
            Self::MissingTextSection => f.write_str("ELF has no non-empty .text section"),
            Self::UnsupportedObject => f.write_str("input is not an ELF object or archive"),
            Self::Goblin(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ExtractError {}

impl From<goblin::error::Error> for ExtractError {
    fn from(value: goblin::error::Error) -> Self {
        Self::Goblin(value)
    }
}

pub fn extract_text(bytes: &[u8]) -> Result<Vec<u8>, ExtractError> {
    match Object::parse(bytes)? {
        Object::Elf(elf) => text_from_elf(bytes, &elf),
        Object::Archive(archive) => text_from_archive(bytes, &archive),
        _ => Err(ExtractError::UnsupportedObject),
    }
}

fn text_from_archive(bytes: &[u8], archive: &Archive<'_>) -> Result<Vec<u8>, ExtractError> {
    let member = archive
        .member_of_symbol("alpha_usb_entry")
        .ok_or(ExtractError::ArchiveMemberMissing)?;
    let member_bytes = archive.extract(member, bytes)?;
    match Object::parse(member_bytes)? {
        Object::Elf(elf) => text_from_elf(member_bytes, &elf),
        _ => Err(ExtractError::UnsupportedObject),
    }
}

fn text_from_elf(bytes: &[u8], elf: &Elf<'_>) -> Result<Vec<u8>, ExtractError> {
    for section in &elf.section_headers {
        let Some(name) = elf.shdr_strtab.get_at(section.sh_name) else {
            continue;
        };
        if name == ".text" && section.sh_size > 0 {
            let start =
                usize::try_from(section.sh_offset).map_err(|_| ExtractError::MissingTextSection)?;
            let size =
                usize::try_from(section.sh_size).map_err(|_| ExtractError::MissingTextSection)?;
            let end = start
                .checked_add(size)
                .ok_or(ExtractError::MissingTextSection)?;
            let data = bytes
                .get(start..end)
                .ok_or(ExtractError::MissingTextSection)?;
            return Ok(data.to_vec());
        }
    }
    Err(ExtractError::MissingTextSection)
}
