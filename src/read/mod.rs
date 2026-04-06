use crate::read::raw::{check_header_len, check_magic, read_type, read_version};
use crate::read::raw_babe::RawReaderBigAlignedBigEndian;
use crate::read::raw_bale::RawReaderBigAlignedLittleEndian;
use crate::read::raw_mabe::RawReaderMediumAlignedBigEndian;
use crate::read::raw_male::RawReaderMediumAlignedLittleEndian;
use crate::{UPBFType, UPBFVersion};

pub mod raw;
mod raw_babe;
mod raw_bale;
mod raw_mabe;
mod raw_male;

#[derive(Debug)]
pub struct UPBFReader<'a> {
    source: &'a [u8],
    file_type: UPBFType,
    file_version: UPBFVersion
}

#[derive(Debug, Clone)]
pub struct UPBFReadResult<'a> {
    file_type: UPBFType,
    file_version: UPBFVersion,
    build_name: String,
    build_version: String,
    data_formats: Vec<UPBFDataFormatForRead>,
    data: Vec<UPBFDataForRead<'a>>
}

#[derive(Debug, Clone)]
pub struct UPBFDataFormatForRead {
    data_id: u32,
    name: String
}

#[derive(Debug, Clone)]
pub struct UPBFDataForRead<'a> {
    name: String,
    data: &'a [u8],
    data_id: u32,
    format_position: usize,
}

#[derive(Debug)]
pub enum UPBFReaderError {
    InvalidFileLength,
    Header(UPBFReaderHeaderReadError),
    Name(UPBFReaderNameReadError),
    Version(UPBFReaderVersionReadError),
    Format(UPBFReaderFormatReadError),
    Data(UPBFReaderDataReadError)
}

#[derive(Debug)]
pub enum UPBFReaderHeaderReadError {
    InvalidMagic,
    InvalidType,
    UnsupportedVersion,
}

#[derive(Debug)]
pub enum UPBFReaderNameReadError {
    InvalidLength,
    InvalidString,
}

#[derive(Debug)]
pub enum UPBFReaderVersionReadError {
    InvalidLength,
    InvalidString,
}

#[derive(Debug)]
pub enum UPBFReaderFormatReadError {
    InvalidOffset,
    InvalidNameLength,
    InvalidNameString,
}

#[derive(Debug)]
pub enum UPBFReaderDataReadError {
    InvalidOffset,
    InvalidNameLength,
    InvalidNameString,
    InvalidDataLength,
    InvalidDataId
}

impl<'a> UPBFReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, UPBFReaderError> {
        if !check_header_len(bytes) { return Err(UPBFReaderError::InvalidFileLength) }
        if !check_magic(bytes) { return Err(UPBFReaderError::Header(UPBFReaderHeaderReadError::InvalidMagic)) }
        let file_type = read_type(bytes).map_err(|()| UPBFReaderHeaderReadError::InvalidType.into())?;
        let file_version = read_version(bytes);
        Ok(
            Self {
                source: bytes,
                file_type,
                file_version
            }
        )
    }

    pub fn file_type(&self) -> UPBFType {
        self.file_type
    }

    pub fn file_version(&self) -> UPBFVersion {
        self.file_version
    }

    pub fn is_read_supported(&self) -> bool {
        self.file_version.is_supported()
    }

    pub fn read(&'_ self) -> Result<UPBFReadResult<'_>, UPBFReaderError> {
        if !self.is_read_supported() { return Err(UPBFReaderHeaderReadError::UnsupportedVersion.into()) };
        match self.file_type {
            UPBFType::MediumAlignedLittleEndian => RawReaderMediumAlignedLittleEndian::read(&self, self.file_type, self.file_version),
            UPBFType::MediumAlignedBigEndian => RawReaderMediumAlignedBigEndian::read(&self, self.file_type, self.file_version),
            UPBFType::BigAlignedLittleEndian => RawReaderBigAlignedLittleEndian::read(&self, self.file_type, self.file_version),
            UPBFType::BigAlignedBigEndian => RawReaderBigAlignedBigEndian::read(&self, self.file_type, self.file_version)
        }
    }
}

impl<'a> UPBFReadResult<'a> {
    pub fn file_type(&self) -> UPBFType {
        self.file_type
    }

    pub fn file_version(&self) -> UPBFVersion {
        self.file_version
    }

    pub fn build_name(&self) -> &String {
        &self.build_name
    }

    pub fn build_version(&self) -> &String {
        &self.build_version
    }

    pub fn data_formats(&self) -> &Vec<UPBFDataFormatForRead> {
        &self.data_formats
    }

    pub fn data(&self) -> &Vec<UPBFDataForRead<'a>> {
        &self.data
    }
}

impl UPBFDataFormatForRead {
    pub fn new(data_id: u32, name: String) -> Self {
        Self { data_id, name }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data_id(&self) -> u32 {
        self.data_id
    }
}

impl<'a> UPBFDataForRead<'a> {
    pub fn new(name: String, data: &'a [u8], data_id: u32, format_position: usize) -> Self {
        Self { name, data, data_id, format_position }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data(&self) -> &[u8] {
        self.data
    }
    
    pub fn data_id(&self) -> u32 {
        self.data_id
    }

    pub fn format<'b>(&self, read: &'b UPBFReadResult) -> &'b UPBFDataFormatForRead {
        unsafe { &read.data_formats.get_unchecked(self.format_position) }
    }
}

impl Into<UPBFReaderError> for UPBFReaderHeaderReadError {
    fn into(self) -> UPBFReaderError {
        UPBFReaderError::Header(self)
    }
}

impl Into<UPBFReaderError> for UPBFReaderNameReadError {
    fn into(self) -> UPBFReaderError {
        UPBFReaderError::Name(self)
    }
}

impl Into<UPBFReaderError> for UPBFReaderVersionReadError {
    fn into(self) -> UPBFReaderError {
        UPBFReaderError::Version(self)
    }
}

impl Into<UPBFReaderError> for UPBFReaderFormatReadError {
    fn into(self) -> UPBFReaderError {
        UPBFReaderError::Format(self)
    }
}

impl Into<UPBFReaderError> for UPBFReaderDataReadError {
    fn into(self) -> UPBFReaderError {
        UPBFReaderError::Data(self)
    }
}