use std::error::Error;
use std::fmt::{Display, Formatter};

use goblin::{Object, archive::Archive, elf::Elf};

#[derive(Debug)]
pub enum ExtractError {
    ArchiveMemberMissing,
    ForbiddenSection(String),
    LowAbsoluteJump(u32),
    MissingLoadSection,
    RelocationsPresent(String),
    UnsupportedObject,
    Goblin(goblin::error::Error),
}

impl Display for ExtractError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchiveMemberMissing => {
                f.write_str("archive has no member defining alpha_usb_entry")
            }
            Self::ForbiddenSection(name) => write!(f, "ELF contains forbidden section {name}"),
            Self::LowAbsoluteJump(address) => {
                write!(f, "ELF contains low absolute jump to 0x{address:08x}")
            }
            Self::MissingLoadSection => f.write_str("ELF has no non-empty loadable image"),
            Self::RelocationsPresent(name) => write!(f, "ELF contains relocations in {name}"),
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
    const SHT_REL: u32 = 9;
    const SHT_RELA: u32 = 4;

    let mut image = Vec::new();
    for section in &elf.section_headers {
        let name = elf.shdr_strtab.get_at(section.sh_name).unwrap_or("");
        if section.sh_size > 0 && matches!(name, ".got" | ".got.plt") {
            return Err(ExtractError::ForbiddenSection(name.to_owned()));
        }
        if section.sh_size > 0 && matches!(section.sh_type, SHT_REL | SHT_RELA) {
            return Err(ExtractError::RelocationsPresent(name.to_owned()));
        }
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
    image = strip_embedded_os3k_wrapper(image);
    validate_load_image_safety(&image)?;
    Ok(image)
}

fn strip_embedded_os3k_wrapper(mut image: Vec<u8>) -> Vec<u8> {
    const EMBEDDED_HEADER_MAGIC: [u8; 4] = [0xC0, 0xFF, 0xEE, 0xAD];
    const EMBEDDED_ENTRY_OFFSET: usize = 0x94;
    const EMBEDDED_META_OFFSET: usize = 0x84;
    const EMBEDDED_FOOTER_MAGIC: [u8; 4] = [0xCA, 0xFE, 0xFE, 0xED];

    let has_embedded_header = image.starts_with(&EMBEDDED_HEADER_MAGIC)
        && image.len() >= EMBEDDED_ENTRY_OFFSET
        && image.get(EMBEDDED_META_OFFSET..EMBEDDED_ENTRY_OFFSET)
            == Some(&[
                0x00, 0x00, 0x00, 0x94, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x02,
            ]);
    if !has_embedded_header {
        return image;
    }

    image.drain(..EMBEDDED_ENTRY_OFFSET);
    if image.ends_with(&EMBEDDED_FOOTER_MAGIC) {
        let new_len = image.len() - EMBEDDED_FOOTER_MAGIC.len();
        image.truncate(new_len);
    }
    image
}

fn validate_load_image_safety(image: &[u8]) -> Result<(), ExtractError> {
    for window in image.windows(6) {
        if window[0] == 0x4E && window[1] == 0xB9 {
            let address = u32::from_be_bytes([window[2], window[3], window[4], window[5]]);
            if address < 0x0010_0000 {
                return Err(ExtractError::LowAbsoluteJump(address));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_low_absolute_jsr() {
        let image = [0x4E, 0xB9, 0x00, 0x00, 0x01, 0x34];
        assert!(matches!(
            validate_load_image_safety(&image),
            Err(ExtractError::LowAbsoluteJump(0x134))
        ));
    }

    #[test]
    fn allows_known_os_absolute_jsr() -> Result<(), ExtractError> {
        let image = [0x4E, 0xB9, 0x00, 0x41, 0x0B, 0x26];
        validate_load_image_safety(&image)
    }
}
