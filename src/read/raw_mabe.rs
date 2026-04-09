use crate::raw::align_len_medium;
use crate::read::{UPBFDataForRead, UPBFDataFormatForRead, UPBFReadResult, UPBFReader, UPBFReaderDataReadError, UPBFReaderError, UPBFReaderDataFormatReadError, UPBFReaderHeaderReadError };
use crate::{UPBFType, UPBFVersion};

pub struct RawReaderMediumAlignedBigEndian;

impl RawReaderMediumAlignedBigEndian {
    pub fn read<'a>(reader: &'a UPBFReader, file_type: UPBFType, file_version: UPBFVersion) -> Result<UPBFReadResult<'a>, UPBFReaderError> {
        let source = reader.source;
        // Name
        let build_name_len = (&source[0x10..0x14]).try_into();
        let build_name_len = build_name_len.map_err(|_| UPBFReaderError::InvalidFileLength)?;
        let build_name_len = u32::from_be_bytes(build_name_len) as usize;
        let build_name_len_aligned = align_len_medium(build_name_len);
        if source.len() < 0x18 + build_name_len_aligned { return Err(UPBFReaderHeaderReadError::InvalidBuildNameLength.into()) }
        let build_name = &source[0x18..0x18 + build_name_len];
        let build_name = Vec::from(build_name);
        let build_name = String::from_utf8(build_name);
        let build_name = build_name.map_err(|_| UPBFReaderHeaderReadError::InvalidBuildNameString.into())?;
        // Version
        let offset = build_name_len_aligned;
        let build_version_len = (&source[0x14..0x18]).try_into();
        let build_version_len = build_version_len.map_err(|_| UPBFReaderError::InvalidFileLength)?;
        let build_version_len = u32::from_be_bytes(build_version_len) as usize;
        if source.len() < offset + 0x18 + align_len_medium(build_version_len) { return Err(UPBFReaderHeaderReadError::InvalidBuildVersionLength.into()) }
        let build_version = &source[offset + 0x18..offset + 0x18 + build_version_len];
        let build_version = Vec::from(build_version);
        let build_version = String::from_utf8(build_version);
        let build_version = build_version.map_err(|_| UPBFReaderHeaderReadError::InvalidBuildVersionString.into())?;
        // Data Formats
        let offset = (&source[0x8..0xC]).try_into();
        let offset = offset.map_err(|_| UPBFReaderDataFormatReadError::InvalidOffset.into())?;
        let mut offset = u32::from_be_bytes(offset) as usize;
        let mut data_format_list = Vec::<UPBFDataFormatForRead>::new();
        while offset != 0 {
            if source.len() < offset { return Err(UPBFReaderDataFormatReadError::InvalidOffset.into()) }
            let next_offset = (&source[offset..offset + 0x4]).try_into();
            let next_offset = next_offset.map_err(|_| UPBFReaderError::InvalidFileLength)?;
            let next_offset = u32::from_be_bytes(next_offset) as usize;
            let name_len = (&source[offset + 0x4..offset + 0x8]).try_into();
            let name_len = name_len.map_err(|_| UPBFReaderError::InvalidFileLength)?;
            let name_len = u32::from_be_bytes(name_len) as usize;
            let name_len_unaligned = name_len;
            let name_len = align_len_medium(name_len);
            if source.len() < offset + 0xC + name_len { return Err(UPBFReaderDataFormatReadError::InvalidNameLength.into()) }
            let data_id = (&source[offset + 0x8..offset + 0xC]).try_into();
            let data_id = data_id.map_err(|_| UPBFReaderError::InvalidFileLength)?;
            let data_id = u32::from_be_bytes(data_id);
            let name = &source[offset + 0xC..offset + 0xC + name_len_unaligned];
            let name = Vec::from(name);
            let name = String::from_utf8(name);
            let name = name.map_err(|_| UPBFReaderDataFormatReadError::InvalidNameString.into())?;
            data_format_list.push(UPBFDataFormatForRead::new(data_id, name));
            offset = next_offset;
        }
        // Data
        let offset = (&source[0xC..0x10]).try_into();
        let offset = offset.map_err(|_| UPBFReaderDataReadError::InvalidOffset.into())?;
        let mut offset = u32::from_be_bytes(offset) as usize;
        let mut data_list = Vec::<UPBFDataForRead>::new();
        while offset != 0 {
            if source.len() < offset { return Err(UPBFReaderDataReadError::InvalidOffset.into()) }
            let next_offset = (&source[offset..offset + 0x4]).try_into();
            let next_offset = next_offset.map_err(|_| UPBFReaderError::InvalidFileLength)?;
            let next_offset = u32::from_be_bytes(next_offset) as usize;
            let data_len = (&source[offset + 0x4..offset + 0x8]).try_into();
            let data_len = data_len.map_err(|_| UPBFReaderError::InvalidFileLength)?;
            let data_len = u32::from_be_bytes(data_len) as usize;
            let data_len_unaligned = data_len;
            let data_len = align_len_medium(data_len);
            let data_id = (&source[offset + 0x8..offset + 0xC]).try_into();
            let data_id = data_id.map_err(|_| UPBFReaderError::InvalidFileLength)?;
            let data_id = u32::from_be_bytes(data_id);
            let format_position = data_format_list.iter().position(|it| it.data_id == data_id);
            let format_position = if let Some(some) = format_position { some } else { return Err(UPBFReaderDataReadError::InvalidDataId.into()) };
            let name_len = (&source[offset + 0xC..offset + 0x10]).try_into();
            let name_len = name_len.map_err(|_| UPBFReaderError::InvalidFileLength)?;
            let name_len = u32::from_be_bytes(name_len) as usize;
            let name_len_unaligned = name_len;
            let name_len = align_len_medium(name_len);
            if source.len() < offset + 0x10 + name_len { return Err(UPBFReaderDataReadError::InvalidNameLength.into()) }
            if source.len() < offset + 0x10 + name_len + data_len { return Err(UPBFReaderDataReadError::InvalidDataLength.into()) }
            let name = &source[offset + 0x10..offset + 0x10 + name_len_unaligned];
            let name = Vec::from(name);
            let name = String::from_utf8(name);
            let name = name.map_err(|_| UPBFReaderDataReadError::InvalidNameString.into())?;
            let data = &source[offset + 0x10 + name_len..offset + 0x10 + name_len + data_len_unaligned];
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