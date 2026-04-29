use std::time::Duration;

use anyhow::bail;

use crate::protocol::{self, FileEntry, SmartAppletRecord};

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(1_000);
const LONG_TIMEOUT: Duration = Duration::from_millis(120_000);

pub trait DirectTransport {
    fn write(&mut self, payload: &[u8]) -> anyhow::Result<()>;
    fn read_exact(&mut self, len: usize, timeout: Duration) -> anyhow::Result<Vec<u8>>;
}

pub struct SharedNeoClient<T: DirectTransport> {
    transport: T,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NeoClientProgress {
    OsSegmentErased {
        completed: usize,
        total: usize,
        address: u32,
    },
    ChunkProgrammed {
        label: &'static str,
        completed: usize,
        total: usize,
    },
}

impl<T: DirectTransport> SharedNeoClient<T> {
    pub fn new(transport: T) -> anyhow::Result<Self> {
        let mut client = Self { transport };
        client.enter_updater_mode()?;
        Ok(client)
    }

    pub fn list_files(&mut self) -> anyhow::Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        for slot in 1..=8 {
            self.write(&protocol::command(
                0x13,
                slot,
                protocol::ALPHAWORD_APPLET_ID,
            ))?;
            let response = self.read_response()?;
            if response.status == 0x90 {
                continue;
            }
            if response.status != 0x5A {
                bail!(
                    "slot {slot} attributes returned status 0x{:02x}",
                    response.status
                );
            }
            let payload = self.read_exact(response.argument as usize)?;
            validate_payload_sum(&payload, response.trailing, "attribute payload")?;
            entries.push(protocol::parse_file_entry(slot as u8, &payload)?);
        }
        Ok(entries)
    }

    pub fn download_file(&mut self, slot: u8) -> anyhow::Result<Vec<u8>> {
        let argument = (0x80000_u32 << 8) | u32::from(slot);
        self.write(&protocol::command(
            0x12,
            argument,
            protocol::ALPHAWORD_APPLET_ID,
        ))?;
        let start = self.read_response()?;
        if start.status != 0x53 {
            bail!(
                "slot {slot} retrieve start returned status 0x{:02x}",
                start.status
            );
        }
        let mut remaining = start.argument as usize;
        let mut payload = Vec::with_capacity(remaining);
        while remaining > 0 {
            self.write(&protocol::retrieve_chunk_command())?;
            let chunk = self.read_response()?;
            if chunk.status != 0x4D {
                bail!("slot {slot} chunk returned status 0x{:02x}", chunk.status);
            }
            let bytes = self.read_exact(chunk.argument as usize)?;
            validate_payload_sum(&bytes, chunk.trailing, "chunk payload")?;
            remaining = remaining.saturating_sub(bytes.len());
            payload.extend(bytes);
        }
        Ok(payload)
    }

    pub fn list_smart_applets(&mut self) -> anyhow::Result<Vec<SmartAppletRecord>> {
        let mut entries = Vec::new();
        let mut page_offset = 0_u32;
        let page_size = 7_u16;
        loop {
            self.write(&protocol::list_applets_command(page_offset, page_size))?;
            let response = self.read_response()?;
            if response.status == 0x90 || response.argument == 0 {
                break;
            }
            if response.status != 0x44 {
                bail!("unexpected applet list status 0x{:02x}", response.status);
            }
            if !(response.argument as usize).is_multiple_of(protocol::SMARTAPPLET_HEADER_SIZE) {
                bail!("applet list payload length is not a multiple of 0x84");
            }
            let payload = self.read_exact(response.argument as usize)?;
            validate_payload_sum(&payload, response.trailing, "applet list payload")?;
            let mut record_count = 0_u32;
            for record in payload.chunks_exact(protocol::SMARTAPPLET_HEADER_SIZE) {
                entries.push(protocol::parse_smartapplet_record(record)?);
                record_count += 1;
            }
            if record_count < u32::from(page_size) {
                break;
            }
            page_offset += record_count;
        }
        Ok(entries)
    }

    pub fn download_smart_applet(&mut self, applet_id: u16) -> anyhow::Result<Vec<u8>> {
        self.write(&protocol::retrieve_applet_command(applet_id))?;
        let start = self.read_response()?;
        if start.status != 0x53 {
            bail!(
                "applet 0x{applet_id:04x} retrieve start returned status 0x{:02x}",
                start.status
            );
        }
        let mut remaining = start.argument as usize;
        let mut payload = Vec::with_capacity(remaining);
        while remaining > 0 {
            self.write(&protocol::retrieve_chunk_command())?;
            let chunk = self.read_response()?;
            if chunk.status != 0x4D {
                bail!(
                    "applet 0x{applet_id:04x} chunk returned status 0x{:02x}",
                    chunk.status
                );
            }
            let bytes = self.read_exact(chunk.argument as usize)?;
            validate_payload_sum(&bytes, chunk.trailing, "applet chunk payload")?;
            remaining = remaining.saturating_sub(bytes.len());
            payload.extend(bytes);
        }
        Ok(payload)
    }

    pub fn install_smart_applet(&mut self, image: &[u8]) -> anyhow::Result<SmartAppletRecord> {
        self.install_smart_applet_with_progress(image, |_| {})
    }

    pub fn install_smart_applet_with_progress(
        &mut self,
        image: &[u8],
        mut on_progress: impl FnMut(NeoClientProgress),
    ) -> anyhow::Result<SmartAppletRecord> {
        if image.len() < protocol::SMARTAPPLET_HEADER_SIZE {
            bail!("SmartApplet image is shorter than its header");
        }
        let header =
            protocol::parse_smartapplet_header(&image[..protocol::SMARTAPPLET_HEADER_SIZE])?;
        if header.file_size as usize != image.len() {
            bail!("SmartApplet image length does not match header file size");
        }
        let (argument, trailing) = protocol::derive_add_applet_start_fields(&header);
        self.write(&protocol::add_applet_begin_command(argument, trailing))?;
        self.require_status(0x46, "add-applet begin")?;
        self.write_chunks_and_program(image, "add-applet", &mut on_progress)?;
        self.write(&protocol::finalize_applet_update_command())?;
        self.require_status(0x48, "finalize applet update")?;
        protocol::parse_smartapplet_record(&image[..protocol::SMARTAPPLET_HEADER_SIZE])
    }

    pub fn clear_smart_applet_area(&mut self) -> anyhow::Result<()> {
        self.write(&protocol::clear_applet_area_command())?;
        self.require_status_timeout(0x4F, "clear SmartApplet area", LONG_TIMEOUT)
    }

    pub fn install_neo_os_image(
        &mut self,
        image: &[u8],
        reformat_rest_of_rom: bool,
    ) -> anyhow::Result<usize> {
        self.install_neo_os_image_with_progress(image, reformat_rest_of_rom, |_| {})
    }

    pub fn install_neo_os_image_with_progress(
        &mut self,
        image: &[u8],
        reformat_rest_of_rom: bool,
        mut on_progress: impl FnMut(NeoClientProgress),
    ) -> anyhow::Result<usize> {
        let segments = protocol::parse_neo_os_segments(image)?;
        self.write(&protocol::enter_small_rom_command())?;
        self.require_status_timeout(0x56, "enter Small ROM", Duration::from_millis(5_000))?;
        self.write(&protocol::clear_os_segment_map_command())?;
        self.require_status_timeout(0x54, "clear OS segment map", Duration::from_millis(5_000))?;
        let segment_total = segments.len();
        for (index, segment) in segments.into_iter().enumerate() {
            let mut erase_kb = ((segment.length + 0x3FF) >> 10) as u16;
            if reformat_rest_of_rom && segment.address == 0x005F_FC00 {
                erase_kb = 0;
            }
            self.write(&protocol::erase_os_segment_command(
                segment.address,
                erase_kb,
            ))?;
            self.require_status_timeout(0x55, "erase OS segment", LONG_TIMEOUT)?;
            on_progress(NeoClientProgress::OsSegmentErased {
                completed: index + 1,
                total: segment_total,
                address: segment.address,
            });
        }
        let chunks = self.write_chunks_and_program(image, "OS image", &mut on_progress)?;
        self.write(&protocol::finalize_applet_update_command())?;
        self.require_status_timeout(0x48, "finalize OS update", LONG_TIMEOUT)?;
        Ok(chunks)
    }

    pub fn restart_device(&mut self) -> anyhow::Result<()> {
        self.write(&protocol::restart_device_command())?;
        self.require_status(0x52, "restart device")
    }

    pub fn read_recovery_diagnostics(&mut self) -> anyhow::Result<String> {
        let mut lines = Vec::new();
        self.append_smartapplet_diagnostics(&mut lines)?;
        self.append_alpha_word_attribute_diagnostics(&mut lines)?;
        Ok(lines.join("\n"))
    }

    fn append_smartapplet_diagnostics(&mut self, lines: &mut Vec<String>) -> anyhow::Result<()> {
        lines.push("SmartApplet records".to_owned());
        let mut page_offset = 0_u32;
        let page_size = 7_u16;
        let mut row = 0_usize;

        loop {
            self.write(&protocol::list_applets_command(page_offset, page_size))?;
            let response = self.read_response()?;
            lines.push(format!(
                "page_offset={page_offset} status=0x{:02x} argument={} trailing=0x{:04x}",
                response.status, response.argument, response.trailing
            ));
            if response.status == 0x90 || response.argument == 0 {
                break;
            }
            if response.status != 0x44 {
                lines.push(format!(
                    "stop: unexpected applet list status 0x{:02x}",
                    response.status
                ));
                break;
            }
            if !(response.argument as usize).is_multiple_of(protocol::SMARTAPPLET_HEADER_SIZE) {
                lines.push(format!(
                    "stop: applet list payload length {} is not a multiple of 0x84",
                    response.argument
                ));
                break;
            }

            let payload = self.read_exact(response.argument as usize)?;
            let actual = checksum16(&payload);
            lines.push(format!(
                "payload_sum16=0x{actual:04x} expected=0x{:04x}",
                response.trailing
            ));
            if actual != response.trailing {
                lines.push("stop: applet list payload checksum mismatch".to_owned());
                break;
            }

            let records = payload
                .chunks_exact(protocol::SMARTAPPLET_HEADER_SIZE)
                .collect::<Vec<_>>();
            for record in &records {
                match protocol::parse_smartapplet_record(record) {
                    Ok(metadata) => lines.push(format!(
                        "row={row} applet_id=0x{:04x} version={} name={} file_size={} applet_class=0x{:02x} record={}",
                        metadata.applet_id,
                        metadata.version,
                        metadata.name,
                        metadata.file_size,
                        metadata.applet_class,
                        hex_bytes(record)
                    )),
                    Err(error) => lines.push(format!(
                        "row={row} parse_error={error} record={}",
                        hex_bytes(record)
                    )),
                }
                row += 1;
            }
            if records.len() < usize::from(page_size) {
                break;
            }
            page_offset += records.len() as u32;
        }
        Ok(())
    }

    fn append_alpha_word_attribute_diagnostics(
        &mut self,
        lines: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        lines.push("AlphaWord file attributes".to_owned());
        for slot in 1..=8_u8 {
            self.write(&protocol::command(
                0x13,
                u32::from(slot),
                protocol::ALPHAWORD_APPLET_ID,
            ))?;
            let response = self.read_response()?;
            lines.push(format!(
                "slot {slot} status=0x{:02x} argument={} trailing=0x{:04x}",
                response.status, response.argument, response.trailing
            ));
            if response.status != 0x5A {
                continue;
            }

            let payload = self.read_exact(response.argument as usize)?;
            let actual = checksum16(&payload);
            lines.push(format!(
                "slot {slot} payload_sum16=0x{actual:04x} expected=0x{:04x}",
                response.trailing
            ));
            if actual != response.trailing {
                lines.push(format!("slot {slot} checksum_mismatch"));
                continue;
            }
            match protocol::parse_file_entry(slot, &payload) {
                Ok(entry) => lines.push(format!(
                    "slot {} name={} attribute_bytes={}",
                    entry.slot, entry.name, entry.attribute_bytes
                )),
                Err(error) => lines.push(format!("slot {slot} parse_error={error}")),
            }
        }
        Ok(())
    }

    fn enter_updater_mode(&mut self) -> anyhow::Result<()> {
        self.write(&protocol::reset_packet())?;
        self.write(&protocol::switch_packet())?;
        let response = self.read_exact(8)?;
        if response.as_slice() != b"Switched" {
            bail!(
                "unexpected switch response: {}",
                String::from_utf8_lossy(&response)
            );
        }
        Ok(())
    }

    fn read_response(&mut self) -> anyhow::Result<protocol::Response> {
        self.read_response_timeout(DEFAULT_TIMEOUT)
    }

    fn read_response_timeout(&mut self, timeout: Duration) -> anyhow::Result<protocol::Response> {
        protocol::parse_response(&self.read_exact_timeout(8, timeout)?)
    }

    fn require_status(&mut self, expected: u8, operation: &str) -> anyhow::Result<()> {
        self.require_status_timeout(expected, operation, DEFAULT_TIMEOUT)
    }

    fn require_status_timeout(
        &mut self,
        expected: u8,
        operation: &str,
        timeout: Duration,
    ) -> anyhow::Result<()> {
        let response = self.read_response_timeout(timeout)?;
        if response.status != expected {
            bail!(
                "unexpected {operation} status 0x{:02x}, expected 0x{expected:02x}",
                response.status
            );
        }
        Ok(())
    }

    fn write_chunks_and_program(
        &mut self,
        image: &[u8],
        label: &'static str,
        on_progress: &mut impl FnMut(NeoClientProgress),
    ) -> anyhow::Result<usize> {
        let mut chunks = 0;
        let total = image.chunks(0x400).len();
        for chunk in image.chunks(0x400) {
            let checksum = chunk
                .iter()
                .fold(0_u16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
            self.write(&protocol::command(0x02, chunk.len() as u32, checksum))?;
            self.require_status_timeout(0x42, label, Duration::from_millis(5_000))?;
            self.write(chunk)?;
            self.require_status_timeout(0x43, label, Duration::from_millis(10_000))?;
            self.write(&protocol::program_applet_command())?;
            self.require_status_timeout(0x47, label, LONG_TIMEOUT)?;
            chunks += 1;
            on_progress(NeoClientProgress::ChunkProgrammed {
                label,
                completed: chunks,
                total,
            });
        }
        Ok(chunks)
    }

    fn write(&mut self, payload: &[u8]) -> anyhow::Result<()> {
        self.transport.write(payload)
    }

    fn read_exact(&mut self, len: usize) -> anyhow::Result<Vec<u8>> {
        self.read_exact_timeout(len, DEFAULT_TIMEOUT)
    }

    fn read_exact_timeout(&mut self, len: usize, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        self.transport.read_exact(len, timeout)
    }
}

fn validate_payload_sum(payload: &[u8], expected: u16, label: &str) -> anyhow::Result<()> {
    let actual = checksum16(payload);
    if actual != expected {
        bail!("{label} checksum mismatch: got 0x{actual:04x}, expected 0x{expected:04x}");
    }
    Ok(())
}

fn checksum16(payload: &[u8]) -> u16 {
    payload
        .iter()
        .fold(0_u16, |sum, byte| sum.wrapping_add(u16::from(*byte)))
}

fn hex_bytes(payload: &[u8]) -> String {
    payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, time::Duration};

    use super::*;

    struct FakeTransport {
        reads: VecDeque<Vec<u8>>,
        writes: Vec<Vec<u8>>,
    }

    impl FakeTransport {
        fn new(reads: impl IntoIterator<Item = Vec<u8>>) -> Self {
            Self {
                reads: reads.into_iter().collect(),
                writes: Vec::new(),
            }
        }
    }

    impl DirectTransport for FakeTransport {
        fn write(&mut self, payload: &[u8]) -> anyhow::Result<()> {
            self.writes.push(payload.to_vec());
            Ok(())
        }

        fn read_exact(&mut self, len: usize, _timeout: Duration) -> anyhow::Result<Vec<u8>> {
            let read = self.reads.pop_front().expect("queued fake read");
            assert_eq!(read.len(), len);
            Ok(read)
        }
    }

    #[test]
    fn downloads_file_through_shared_protocol_flow() {
        let payload = b"abc".to_vec();
        let checksum = payload
            .iter()
            .fold(0_u16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
        let transport = FakeTransport::new([
            b"Switched".to_vec(),
            protocol::command(0x53, payload.len() as u32, 0).to_vec(),
            protocol::command(0x4D, payload.len() as u32, checksum).to_vec(),
            payload.clone(),
        ]);
        let mut client = SharedNeoClient::new(transport).unwrap();

        let downloaded = client.download_file(2).unwrap();

        assert_eq!(downloaded, payload);
        assert_eq!(client.transport.writes[0], protocol::reset_packet());
        assert_eq!(client.transport.writes[1], protocol::switch_packet());
        assert_eq!(
            client.transport.writes[2],
            protocol::command(0x12, (0x80000_u32 << 8) | 2, protocol::ALPHAWORD_APPLET_ID)
        );
        assert_eq!(
            client.transport.writes[3],
            protocol::retrieve_chunk_command()
        );
    }

    #[test]
    fn install_applet_reports_chunk_progress() {
        let image = smartapplet_image(0x500);
        let transport = FakeTransport::new([
            b"Switched".to_vec(),
            protocol::command(0x46, 0, 0).to_vec(),
            protocol::command(0x42, 0, 0).to_vec(),
            protocol::command(0x43, 0, 0).to_vec(),
            protocol::command(0x47, 0, 0).to_vec(),
            protocol::command(0x42, 0, 0).to_vec(),
            protocol::command(0x43, 0, 0).to_vec(),
            protocol::command(0x47, 0, 0).to_vec(),
            protocol::command(0x48, 0, 0).to_vec(),
        ]);
        let mut client = SharedNeoClient::new(transport).unwrap();
        let mut events = Vec::new();

        client
            .install_smart_applet_with_progress(&image, |event| events.push(event))
            .unwrap();

        assert_eq!(
            events,
            vec![
                NeoClientProgress::ChunkProgrammed {
                    label: "add-applet",
                    completed: 1,
                    total: 2,
                },
                NeoClientProgress::ChunkProgrammed {
                    label: "add-applet",
                    completed: 2,
                    total: 2,
                },
            ]
        );
    }

    #[test]
    fn install_os_reports_erase_and_chunk_progress() {
        let image = neo_os_image_with_payload(0x500);
        let transport = FakeTransport::new([
            b"Switched".to_vec(),
            protocol::command(0x56, 0, 0).to_vec(),
            protocol::command(0x54, 0, 0).to_vec(),
            protocol::command(0x55, 0, 0).to_vec(),
            protocol::command(0x42, 0, 0).to_vec(),
            protocol::command(0x43, 0, 0).to_vec(),
            protocol::command(0x47, 0, 0).to_vec(),
            protocol::command(0x42, 0, 0).to_vec(),
            protocol::command(0x43, 0, 0).to_vec(),
            protocol::command(0x47, 0, 0).to_vec(),
            protocol::command(0x48, 0, 0).to_vec(),
        ]);
        let mut client = SharedNeoClient::new(transport).unwrap();
        let mut events = Vec::new();

        client
            .install_neo_os_image_with_progress(&image, false, |event| events.push(event))
            .unwrap();

        assert!(events.contains(&NeoClientProgress::OsSegmentErased {
            completed: 1,
            total: 1,
            address: 0x0041_0000,
        }));
        assert!(events.contains(&NeoClientProgress::ChunkProgrammed {
            label: "OS image",
            completed: 2,
            total: 2,
        }));
    }

    #[test]
    fn recovery_diagnostics_include_raw_applet_records() {
        let record = smartapplet_record(0xa000, "AlphaWord Plus", 0x200);
        let transport = FakeTransport::new(
            [
                vec![b"Switched".to_vec()],
                vec![
                    protocol::command(0x44, record.len() as u32, checksum(&record)).to_vec(),
                    record,
                    protocol::command(0x90, 0, 0).to_vec(),
                ],
                empty_attribute_reads(),
            ]
            .concat(),
        );
        let mut client = SharedNeoClient::new(transport).unwrap();

        let report = client.read_recovery_diagnostics().unwrap();

        assert!(report.contains("SmartApplet records"));
        assert!(report.contains("page_offset=0 status=0x44"));
        assert!(report.contains("applet_id=0xa000"));
        assert!(report.contains("AlphaWord Plus"));
        assert!(report.contains("AlphaWord file attributes"));
    }

    #[test]
    fn recovery_diagnostics_include_file_attribute_statuses() {
        let attributes = alpha_word_attributes_record("File 1", 512);
        let transport = FakeTransport::new(
            [
                vec![b"Switched".to_vec()],
                empty_applet_reads(),
                vec![
                    protocol::command(0x5a, attributes.len() as u32, checksum(&attributes)).to_vec(),
                    attributes,
                ],
                empty_attribute_reads_for_slots(2..=8),
            ]
            .concat(),
        );
        let mut client = SharedNeoClient::new(transport).unwrap();

        let report = client.read_recovery_diagnostics().unwrap();

        assert!(report.contains("slot 1 status=0x5a"));
        assert!(report.contains("name=File 1"));
        assert!(report.contains("attribute_bytes=512"));
    }

    fn smartapplet_image(len: usize) -> Vec<u8> {
        let mut image = vec![0_u8; len];
        image[0x00..0x04].copy_from_slice(&0xC0FF_EEAD_u32.to_be_bytes());
        image[0x04..0x08].copy_from_slice(&(len as u32).to_be_bytes());
        image[0x08..0x0C].copy_from_slice(&0x0100_u32.to_be_bytes());
        image[0x14..0x16].copy_from_slice(&0xA130_u16.to_be_bytes());
        image[0x16] = 1;
        image[0x17] = 1;
        image[0x18..0x21].copy_from_slice(b"Alpha USB");
        image[0x3C] = 0x01;
        image[0x3D] = 0x00;
        image[0x3F] = 1;
        image[0x80..0x84].copy_from_slice(&0x2000_u32.to_be_bytes());
        image
    }

    fn neo_os_image_with_payload(payload_len: usize) -> Vec<u8> {
        let mut image = vec![0_u8; 0x60 + payload_len];
        image[6..24].copy_from_slice(b"System 3 Neo      ");
        image[0x50..0x54].copy_from_slice(&0x0041_0000_u32.to_be_bytes());
        image[0x54..0x58].copy_from_slice(&(payload_len as u32).to_be_bytes());
        image
    }

    fn smartapplet_record(applet_id: u16, name: &str, file_size: u32) -> Vec<u8> {
        let mut record = vec![0_u8; protocol::SMARTAPPLET_HEADER_SIZE];
        record[0x00..0x04].copy_from_slice(&0xC0FF_EEAD_u32.to_be_bytes());
        record[0x04..0x08].copy_from_slice(&file_size.to_be_bytes());
        record[0x14..0x16].copy_from_slice(&applet_id.to_be_bytes());
        record[0x16] = 1;
        record[0x17] = 1;
        record[0x18..0x18 + name.len()].copy_from_slice(name.as_bytes());
        record[0x3C] = 0x03;
        record[0x3D] = 0x04;
        record[0x3F] = 1;
        record
    }

    fn alpha_word_attributes_record(name: &str, attribute_bytes: u32) -> Vec<u8> {
        let mut record = vec![0_u8; 0x28];
        record[0x00..0x00 + name.len()].copy_from_slice(name.as_bytes());
        record[0x1c..0x20].copy_from_slice(&attribute_bytes.to_be_bytes());
        record
    }

    fn empty_applet_reads() -> Vec<Vec<u8>> {
        vec![protocol::command(0x90, 0, 0).to_vec()]
    }

    fn empty_attribute_reads() -> Vec<Vec<u8>> {
        empty_attribute_reads_for_slots(1..=8)
    }

    fn empty_attribute_reads_for_slots(slots: impl IntoIterator<Item = u8>) -> Vec<Vec<u8>> {
        slots
            .into_iter()
            .map(|_| protocol::command(0x90, 0, 0).to_vec())
            .collect()
    }

    fn checksum(payload: &[u8]) -> u16 {
        payload
            .iter()
            .fold(0_u16, |sum, byte| sum.wrapping_add(u16::from(*byte)))
    }
}
