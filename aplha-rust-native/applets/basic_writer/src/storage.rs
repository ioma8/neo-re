use crate::editor::{Document, FILE_COUNT, MAX_FILE_BYTES};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageError {
    InvalidSlot,
    FirmwarePathUnproven,
}

#[cfg_attr(not(test), allow(dead_code, reason = "firmware storage wiring is gated by reverse-engineering proof"))]
pub fn document_from_file_bytes(bytes: &[u8]) -> Document {
    Document::from_bytes(bytes)
}

#[cfg_attr(not(test), allow(dead_code, reason = "firmware storage wiring is gated by reverse-engineering proof"))]
pub fn write_document_bytes(document: &Document, output: &mut [u8; MAX_FILE_BYTES]) -> usize {
    let bytes = document.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        output[index] = bytes[index];
        index += 1;
    }
    index
}

#[cfg_attr(not(test), allow(dead_code, reason = "firmware storage wiring is gated by reverse-engineering proof"))]
pub const fn validate_slot(slot: usize) -> Result<(), StorageError> {
    if slot >= 1 && slot <= FILE_COUNT {
        Ok(())
    } else {
        Err(StorageError::InvalidSlot)
    }
}

pub fn load_slot(_slot: usize, _document: &mut Document) -> Result<(), StorageError> {
    Err(StorageError::FirmwarePathUnproven)
}

pub fn save_slot(_slot: usize, _document: &Document) -> Result<(), StorageError> {
    Err(StorageError::FirmwarePathUnproven)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_decoding_drops_unsupported_bytes() {
        let document = document_from_file_bytes(b"a\x00b\tc");

        assert_eq!(document.as_bytes(), b"abc");
    }

    #[test]
    fn save_encoding_writes_only_document_bytes() {
        let document = document_from_file_bytes(b"abc");
        let mut output = [0xff; MAX_FILE_BYTES];

        let len = write_document_bytes(&document, &mut output);

        assert_eq!(len, 3);
        assert_eq!(&output[..3], b"abc");
        assert_eq!(output[3], 0xff);
    }

    #[test]
    fn rejects_invalid_slots() {
        assert_eq!(validate_slot(0), Err(StorageError::InvalidSlot));
        assert_eq!(validate_slot(9), Err(StorageError::InvalidSlot));
        assert_eq!(validate_slot(1), Ok(()));
        assert_eq!(validate_slot(8), Ok(()));
    }

    #[test]
    fn persistence_path_is_not_claimed_until_firmware_calls_are_proven() {
        let mut document = Document::new();

        assert_eq!(
            load_slot(1, &mut document),
            Err(StorageError::FirmwarePathUnproven)
        );
        assert_eq!(
            save_slot(1, &document),
            Err(StorageError::FirmwarePathUnproven)
        );
    }
}
