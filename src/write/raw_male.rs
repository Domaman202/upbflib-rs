use crate::raw::{bytes_align_medium, str_to_bytes_align_medium, usize_to_u32};
use crate::write::{UPBFWriter, UPBFWriterError, UPBFWriterWriteError};
use crate::{UPBFType, UPBFVersion};
use std::io::Write;

pub struct RawWriterMediumAlignedLittleEndian;

impl RawWriterMediumAlignedLittleEndian {
    pub fn write(writer: &mut UPBFWriter) -> Result<Vec<u8>, UPBFWriterError> {
        let mut out = Vec::new();

        out.write(b".UPBF\0")?;
        out.write(&u8::to_le_bytes(UPBFType::MediumAlignedLittleEndian.into()))?;
        out.write(&u8::to_le_bytes(UPBFVersion::LAST_SUPPORTED.into()))?;
        out.write(&[0; 4])?; // FORMAT_ENTRY
        out.write(&[0; 4])?; // DATA_ENTRY
        out.write(&u32::to_le_bytes(usize_to_u32(writer.build_name.len(), UPBFWriterWriteError::InvalidBuildNameLength.into())?))?;
        out.write(&u32::to_le_bytes(usize_to_u32(writer.build_version.len(), UPBFWriterWriteError::InvalidBuildVersionLength.into())?))?;
        let (name, name_align) = str_to_bytes_align_medium(&writer.build_name);
        out.write(name)?;
        out.write(&vec![0; name_align])?;
        let (version, version_align) = str_to_bytes_align_medium(&writer.build_version);
        out.write(version)?;
        out.write(&vec![0; version_align])?;

        let mut last_offset = 0x18 + name.len() + name_align + version.len() + version_align;

        if !writer.data_formats.is_empty() {
            let mut next_write_addr = 0x8..0xC; // FORMAT_ENTRY

            for format in &writer.data_formats {
                let block_start = usize_to_u32(last_offset, UPBFWriterWriteError::InvalidOffset.into())?;
                let block_start = u32::to_le_bytes(block_start);
                out[next_write_addr].copy_from_slice(&block_start);
                next_write_addr = last_offset..last_offset + 0x4; // NEXT

                out.write(&[0; 4])?; // NEXT
                out.write(&u32::to_le_bytes(usize_to_u32(format.name.len(), UPBFWriterWriteError::InvalidDataFormatNameLength.into())?))?;
                out.write(&u32::to_le_bytes(format.data_id))?;
                let (name, name_align) = str_to_bytes_align_medium(&format.name);
                out.write(name)?;
                out.write(&vec![0; name_align])?;

                last_offset += 0xC + name.len() + name_align;
            }
        }

        if !writer.data.is_empty() {
            let mut next_write_addr = 0xC..0x10; // DATA_ENTRY

            for data in &writer.data {
                let block_start = usize_to_u32(last_offset, UPBFWriterWriteError::InvalidOffset.into())?;
                let block_start = u32::to_le_bytes(block_start);
                out[next_write_addr].copy_from_slice(&block_start);
                next_write_addr = last_offset..last_offset + 0x4; // NEXT

                out.write(&[0; 4])?; // NEXT
                out.write(&u32::to_le_bytes(usize_to_u32(data.data.len(), UPBFWriterWriteError::InvalidDataLength.into())?))?;
                out.write(&u32::to_le_bytes(data.data_id))?;
                out.write(&u32::to_le_bytes(usize_to_u32(data.name.len(), UPBFWriterWriteError::InvalidDataNameLength.into())?))?;
                let (name, name_align) = str_to_bytes_align_medium(&data.name);
                out.write(name)?;
                out.write(&vec![0; name_align])?;
                let data_align = bytes_align_medium(&data.data);
                out.write(&data.data)?;
                out.write(&vec![0; data_align])?;

                last_offset += 0x10 + name.len() + name_align + data.data.len() + data_align;
            }
        }

        Ok(out)
    }
}