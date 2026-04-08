use crate::raw::{align_len_big, u64_to_usize};
use crate::read::{UPBFDataForRead, UPBFDataFormatForRead, UPBFReadResult, UPBFReader, UPBFReaderDataReadError, UPBFReaderError, UPBFReaderFormatReadError, UPBFReaderNameReadError, UPBFReaderVersionReadError};
use crate::{UPBFType, UPBFVersion};

pub struct RawReaderBigAlignedBigEndian;

impl RawReaderBigAlignedBigEndian {
    pub fn read<'a>(reader: &'a UPBFReader, file_type: UPBFType, file_version: UPBFVersion) -> Result<UPBFReadResult<'a>, UPBFReaderError> {
        let source = reader.source;
        // Name
        let build_name_len = (&source[0x18..0x1C]).try_into();
        let build_name_len = if let Ok(ok) = build_name_len { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
        let build_name_len = u32::from_be_bytes(build_name_len) as usize;
        let build_name_len_aligned = align_len_big(build_name_len);
        if source.len() < 0x20 + build_name_len_aligned { return Err(UPBFReaderNameReadError::InvalidLength.into()) }
        let build_name = &source[0x20..0x20 + build_name_len];
        let build_name = Vec::from(build_name);
        let build_name = if let Ok(ok) = String::from_utf8(build_name) { ok } else { return Err(UPBFReaderNameReadError::InvalidString.into()) };
        // Version
        let offset = build_name_len_aligned;
        let build_version_len = (&source[0x1C..0x20]).try_into();
        let build_version_len = if let Ok(ok) = build_version_len { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
        let build_version_len = u32::from_be_bytes(build_version_len) as usize;
        if source.len() < offset + 0x20 + align_len_big(build_version_len) { return Err(UPBFReaderVersionReadError::InvalidLength.into()) }
        let build_version = &source[offset + 0x20..offset + 0x20 + build_version_len];
        let build_version = Vec::from(build_version);
        let build_version = if let Ok(ok) = String::from_utf8(build_version) { ok } else { return Err(UPBFReaderVersionReadError::InvalidString.into()) };
        // Data Formats
        let offset = (&source[0x8..0x10]).try_into();
        let offset = if let Ok(ok) = offset { ok } else { return Err(UPBFReaderFormatReadError::InvalidOffset.into()) };
        let mut offset = u64_to_usize(u64::from_be_bytes(offset), UPBFReaderFormatReadError::InvalidOffset.into())?;
        let mut data_format_list = Vec::<UPBFDataFormatForRead>::new();
        while offset != 0 {
            if source.len() < offset { return Err(UPBFReaderFormatReadError::InvalidOffset.into()) }
            let next_offset = (&source[offset..offset + 0x8]).try_into();
            let next_offset = if let Ok(ok) = next_offset { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
            let next_offset = u64_to_usize(u64::from_be_bytes(next_offset), UPBFReaderFormatReadError::InvalidOffset.into())?;
            let name_len = (&source[offset + 0x8..offset + 0xC]).try_into();
            let name_len = if let Ok(ok) = name_len { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
            let name_len = u32::from_be_bytes(name_len) as usize;
            let name_len_unaligned = name_len;
            let name_len = align_len_big(name_len);
            if source.len() < offset + 0x18 + name_len { return Err(UPBFReaderFormatReadError::InvalidNameLength.into()) }
            let data_id = (&source[offset + 0xC..offset + 0x10]).try_into();
            let data_id = if let Ok(ok) = data_id { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
            let data_id = u32::from_be_bytes(data_id);
            let name = &source[offset + 0x10..offset + 0x10 + name_len_unaligned];
            let name = Vec::from(name);
            let name = if let Ok(ok) = String::from_utf8(name) { ok } else { return Err(UPBFReaderFormatReadError::InvalidNameString.into()) };
            data_format_list.push(UPBFDataFormatForRead::new(data_id, name));
            offset = next_offset;
        }
        // Data
        let offset = (&source[0x10..0x18]).try_into();
        let offset = if let Ok(ok) = offset { ok } else { return Err(UPBFReaderDataReadError::InvalidOffset.into()) };
        let mut offset = u64_to_usize(u64::from_be_bytes(offset), UPBFReaderDataReadError::InvalidOffset.into())?;
        let mut data_list = Vec::<UPBFDataForRead>::new();
        while offset != 0 {
            if source.len() < offset { return Err(UPBFReaderDataReadError::InvalidOffset.into()) }
            let next_offset = (&source[offset..offset + 0x8]).try_into();
            let next_offset = if let Ok(ok) = next_offset { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
            let next_offset = u64_to_usize(u64::from_be_bytes(next_offset), UPBFReaderDataReadError::InvalidOffset.into())?;
            let data_len = (&source[offset + 0x8..offset + 0x10]).try_into();
            let data_len = if let Ok(ok) = data_len { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
            let data_len = u64_to_usize(u64::from_be_bytes(data_len), UPBFReaderDataReadError::InvalidDataLength.into())?;
            let data_len_unaligned = data_len;
            let data_len = align_len_big(data_len);
            let data_id = (&source[offset + 0x10..offset + 0x14]).try_into();
            let data_id = if let Ok(ok) = data_id { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
            let data_id = u32::from_be_bytes(data_id);
            let format_position = data_format_list.iter().position(|it| it.data_id == data_id);
            let format_position = if let Some(some) = format_position { some } else { return Err(UPBFReaderDataReadError::InvalidDataId.into()) };
            let name_len = (&source[offset + 0x14..offset + 0x18]).try_into();
            let name_len = if let Ok(ok) = name_len { ok } else { return Err(UPBFReaderError::InvalidFileLength) };
            let name_len = u32::from_be_bytes(name_len) as usize;
            let name_len_unaligned = name_len;
            let name_len = align_len_big(name_len);
            if source.len() < offset + 0x18 + name_len { return Err(UPBFReaderDataReadError::InvalidNameLength.into()) }
            if source.len() < offset + 0x18 + name_len + data_len { return Err(UPBFReaderDataReadError::InvalidDataLength.into()) }
            let name = &source[offset + 0x18..offset + 0x18 + name_len_unaligned];
            let name = Vec::from(name);
            let name = if let Ok(ok) = String::from_utf8(name) { ok } else { return Err(UPBFReaderDataReadError::InvalidNameString.into()) };
            let data = &source[offset + 0x18 + name_len..offset + 0x18 + name_len + data_len_unaligned];
            data_list.push(UPBFDataForRead::new(name, data, data_id, format_position));
            offset = next_offset;
        }
        // Return
        Ok(
            UPBFReadResult {
                file_type,
                file_version,
                build_name,
                build_version,
                data_formats: data_format_list,
                data: data_list
            }
        )
    }
}