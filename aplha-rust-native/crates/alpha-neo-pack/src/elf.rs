use std::error::Error;
use std::fmt::{Display, Formatter};

use goblin::{Object, archive::Archive, elf::Elf};

#[derive(Debug)]
pub enum ExtractError {
    ArchiveMemberMissing,
    MissingLoadSection,
    UnsupportedObject,
    Goblin(goblin::error::Error),
}

impl Display for ExtractError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchiveMemberMissing => {
                f.write_str("archive has no member defining alpha_usb_entry")
            }
            Self::MissingLoadSection => f.write_str("ELF has no non-empty loadable image"),
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

pub fn extract_load_image(bytes: &[u8]) -> Result<Vec<u8>, ExtractError> {
    match Object::parse(bytes)? {
        Object::Elf(elf) => load_image_from_elf(bytes, &elf),
        Object::Archive(archive) => load_image_from_archive(bytes, &archive),
        _ => Err(ExtractError::UnsupportedObject),
    }
}

fn load_image_from_archive(bytes: &[u8], archive: &Archive<'_>) -> Result<Vec<u8>, ExtractError> {
    let member = archive
        .member_of_symbol("alpha_usb_entry")
        .ok_or(ExtractError::ArchiveMemberMissing)?;
    let member_bytes = archive.extract(member, bytes)?;
    match Object::parse(member_bytes)? {
        Object::Elf(elf) => load_image_from_elf(member_bytes, &elf),
        _ => Err(ExtractError::UnsupportedObject),
    }
}

fn load_image_from_elf(bytes: &[u8], elf: &Elf<'_>) -> Result<Vec<u8>, ExtractError> {
    const SHF_ALLOC: u64 = 0x2;
    const SHT_NOBITS: u32 = 8;

    let mut image = Vec::new();
    for section in &elf.section_headers {
        if section.sh_flags & SHF_ALLOC == 0 || section.sh_type == SHT_NOBITS {
            continue;
        }
        let address =
            usize::try_from(section.sh_addr).map_err(|_| ExtractError::MissingLoadSection)?;
        let size =
            usize::try_from(section.sh_size).map_err(|_| ExtractError::MissingLoadSection)?;
        if size == 0 {
            continue;
        }
        let file_start =
            usize::try_from(section.sh_offset).map_err(|_| ExtractError::MissingLoadSection)?;
        let file_end = file_start
            .checked_add(size)
            .ok_or(ExtractError::MissingLoadSection)?;
        let data = bytes
            .get(file_start..file_end)
            .ok_or(ExtractError::MissingLoadSection)?;
        let image_end = address
            .checked_add(size)
            .ok_or(ExtractError::MissingLoadSection)?;
        if image.len() < image_end {
            image.resize(image_end, 0);
        }
        image[address..image_end].copy_from_slice(data);
    }
    if image.is_empty() {
        return Err(ExtractError::MissingLoadSection);
    }
    Ok(image)
}
