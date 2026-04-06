use crate::raw::{bytes_align_big, str_to_bytes_align_big, usize_to_u32};
use crate::write::{UPBFWriter, UPBFWriterError, UPBFWriterWriteError};
use crate::{UPBFType, UPBFVersion};
use std::io::Write;

pub struct RawWriterBigAlignedLittleEndian;

impl RawWriterBigAlignedLittleEndian {
    pub fn write(writer: &mut UPBFWriter) -> Result<Vec<u8>, UPBFWriterError> {
        let mut out = Vec::new();

        out.write(b".UPBF\0")?;
        out.write(&u8::to_le_bytes(UPBFType::BigAlignedLittleEndian.into()))?;
        out.write(&u8::to_le_bytes(UPBFVersion::LAST_SUPPORTED.into()))?;
        out.write(&[0; 8])?; // FORMAT_ENTRY
        out.write(&[0; 8])?; // DATA_ENTRY
        out.write(&u32::to_le_bytes(usize_to_u32(writer.build_name.len(), UPBFWriterWriteError::InvalidNameLength.into())?))?;
        out.write(&u32::to_le_bytes(usize_to_u32(writer.build_version.len(), UPBFWriterWriteError::InvalidVersionLength.into())?))?;
        let (name, name_align) = str_to_bytes_align_big(&writer.build_name);
        out.write(name)?;
        out.write(&vec![0; name_align])?;
        let (version, version_align) = str_to_bytes_align_big(&writer.build_version);
        out.write(version)?;
        out.write(&vec![0; version_align])?;

        let mut last_offset = 0x20 + name.len() + name_align + version.len() + version_align;

        if !writer.data_formats.is_empty() {
            let mut next_write_addr = 0x8..0x10; // FORMAT_ENTRY

            for format in &writer.data_formats {
                let block_start = u64::to_le_bytes(last_offset as u64);
                out[next_write_addr].copy_from_slice(&block_start);
                next_write_addr = last_offset..last_offset + 0x8; // NEXT

                out.write(&[0; 8])?; // NEXT
                out.write(&u32::to_le_bytes(usize_to_u32(format.name.len(), UPBFWriterWriteError::InvalidFormatNameLength.into())?))?;
                out.write(&u32::to_le_bytes(format.data_id))?;
                let (name, name_align) = str_to_bytes_align_big(&format.name);
                out.write(name)?;
                out.write(&vec![0; name_align])?;

                last_offset += 0x10 + name.len() + name_align;
            }
        }

        if !writer.data.is_empty() {
            let mut next_write_addr = 0x10..0x18; // DATA_ENTRY

            for data in &writer.data {
                let block_start = u64::to_le_bytes(last_offset as u64);
                out[next_write_addr].copy_from_slice(&block_start);
                next_write_addr = last_offset..last_offset + 0x8; // NEXT

                out.write(&[0; 8])?; // NEXT
                out.write(&u64::to_le_bytes(data.data.len() as u64))?;
                out.write(&u32::to_le_bytes(data.data_id))?;
                out.write(&u32::to_le_bytes(usize_to_u32(data.name.len(), UPBFWriterWriteError::InvalidDataNameLength.into())?))?;
                let (name, name_align) = str_to_bytes_align_big(&data.name);
                out.write(name)?;
                out.write(&vec![0; name_align])?;
                let data_align = bytes_align_big(&data.data);
                out.write(&data.data)?;
                out.write(&vec![0; data_align])?;

                last_offset += 0x18 + name.len() + name_align + data.data.len() + data_align;
            }
        }

        Ok(out)
    }
}