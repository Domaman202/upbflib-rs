use crate::read::{UPBFReadResult, UPBFReaderError};
use crate::write::raw_babe::RawWriterBigAlignedBigEndian;
use crate::write::raw_bale::RawWriterBigAlignedLittleEndian;
use crate::write::raw_mabe::RawWriterMediumAlignedBigEndian;
use crate::write::raw_male::RawWriterMediumAlignedLittleEndian;
use crate::{UPBFType, UPBFVersion};

mod raw_babe;
mod raw_bale;
mod raw_mabe;
mod raw_male;

#[derive(Debug)]
pub struct UPBFWriter {
    pub build_name: String,
    pub build_version: String,
    data_format_id_last: u32,
    data_format_id_pool: Vec<u32>,
    data_formats: Vec<UPBFDataFormatForWrite>,
    data: Vec<UPBFDataForWrite>
}

#[derive(Debug, Clone)]
pub struct UPBFDataFormatForWrite {
    data_id: u32,
    name: String,
    refs: u32
}

#[derive(Debug, Clone)]
pub struct UPBFDataForWrite {
    data_id: u32,
    name: String,
    data: Box<[u8]>
}

#[derive(Debug)]
pub enum UPBFWriterError {
    DataAdd(UPBFWriterDataAddError),
    Write(UPBFWriterWriteError)
}

#[derive(Debug)]
pub enum UPBFWriterDataAddError {
    DataAlreadyDefined,
    FormatCounterOverflow
}

#[derive(Debug)]
pub enum UPBFWriterWriteError {
    UnsupportedVersion,
    InvalidBuildNameLength,
    InvalidBuildVersionLength,
    InvalidDataFormatNameLength,
    InvalidDataNameLength,
    InvalidDataLength,
    InvalidOffset,
    IOError(std::io::Error),
}

impl UPBFWriter {
    pub fn new(build_name: String, build_version: String) -> Self {
        Self {
            build_name,
            build_version,
            data_format_id_last: 0xFF, // 0xFF - last reserved
            data_format_id_pool: Vec::new(),
            data_formats: Vec::new(),
            data: Vec::new(),
        }
    }

    fn find_or_add_format(&mut self, name: &String) -> Result<u32, UPBFWriterError> {
        let format = self.data_formats.iter_mut().find(|it| it.name == *name);
        if let Some(format) = format {
            format.refs += 1;
            return Ok(format.data_id);
        }

        if let Some(id) = self.data_format_id_pool.pop() {
            self.data_formats.push(UPBFDataFormatForWrite::new(id, name.clone(), 1));
            return Ok(id);
        }

        if self.data_format_id_last == u32::MIN {
            return Err(UPBFWriterDataAddError::FormatCounterOverflow.into());
        }

        self.data_format_id_last += 1;
        let id = self.data_format_id_last;
        self.data_formats.push(UPBFDataFormatForWrite::new(id, name.clone(), 1));
        Ok(id)
    }

    fn find_or_add_format_unchecked(&mut self, name: &String) -> u32 {
        let format = self.data_formats.iter_mut().find(|it| it.name == *name);
        if let Some(format) = format {
            format.refs += 1;
            return format.data_id;
        }

        if let Some(id) = self.data_format_id_pool.pop() {
            self.data_formats.push(UPBFDataFormatForWrite::new(id, name.clone(), 1));
            return id;
        }

        self.data_format_id_last += 1;
        let id = self.data_format_id_last;
        self.data_formats.push(UPBFDataFormatForWrite::new(id, name.clone(), 1));
        id
    }

    pub fn data_formats(&self) -> &Vec<UPBFDataFormatForWrite> {
        &self.data_formats
    }

    pub fn add_data(&mut self, name: String, format: &String, bytes: Box<[u8]>) -> Result<(), UPBFWriterError> {
        if let Some(_) = self.data.iter().find(|it| it.name == *name) {
            Err(UPBFWriterDataAddError::DataAlreadyDefined.into())
        } else {
            let data_id = self.find_or_add_format(format)?;
            self.data.push(UPBFDataForWrite::new(data_id, name, bytes));
            Ok(())
        }
    }

    pub unsafe fn add_data_unchecked(&mut self, name: String, format: &String, bytes: Box<[u8]>) {
        let data_id = self.find_or_add_format_unchecked(format);
        self.data.push(UPBFDataForWrite::new(data_id, name, bytes));
    }

    pub fn add_or_overwrite_data(&mut self, name: &String, format: &String, bytes: Box<[u8]>) -> Result<(), UPBFWriterError> {
        let data_id = self.find_or_add_format(format)?;
        if let Some(data) = self.data.iter_mut().find(|it| it.name == *name) {
            data.data_id = data_id;
            data.data = bytes;
            Ok(())
        } else {
            self.data.push(UPBFDataForWrite::new(data_id, name.clone(), bytes));
            Ok(())
        }
    }

    pub fn remove_data(&mut self, name: &String) -> bool {
        if let Some(idx) = self.data.iter().position(|it| it.name == *name) {
            let data = self.data.remove(idx);
            let format_idx = self.data_formats.iter().position(|it| it.data_id == data.data_id);
            let format_idx = unsafe { format_idx.unwrap_unchecked() };
            let format = &mut self.data_formats[format_idx];
            format.refs -= 1;
            if format.refs == 0 {
                self.data_format_id_pool.push(format.data_id);
                self.data_formats.remove(format_idx);
            }
            true
        } else {
            false
        }
    }

    pub fn data(&self) -> &Vec<UPBFDataForWrite> {
        &self.data
    }

    pub fn write(&mut self, r#type: UPBFType, version: UPBFVersion) -> Result<Vec<u8>, UPBFWriterError> {
        if !version.is_supported() { return Err(UPBFWriterWriteError::UnsupportedVersion.into()); }
        match r#type {
            UPBFType::MediumAlignedLittleEndian => RawWriterMediumAlignedLittleEndian::write(self),
            UPBFType::MediumAlignedBigEndian    => RawWriterMediumAlignedBigEndian   ::write(self),
            UPBFType::BigAlignedLittleEndian    => RawWriterBigAlignedLittleEndian   ::write(self),
            UPBFType::BigAlignedBigEndian       => RawWriterBigAlignedBigEndian      ::write(self)
        }
    }
}

impl TryFrom<&UPBFReadResult<'_>> for UPBFWriter {
    type Error = UPBFReaderError;

    fn try_from(value: &UPBFReadResult) -> Result<Self, UPBFReaderError> {
        let mut data_format_id_last = 0xFF; // 0xFF - last reserved
        let mut data_formats: Vec<UPBFDataFormatForWrite> =
            value
                .data_formats()
                .iter()
                .map(|it| {
                    let id = it.data_id();
                    if data_format_id_last < id { data_format_id_last = id }
                    UPBFDataFormatForWrite::new(id, it.name().clone(), 0)
                })
                .collect();
        let data: Vec<UPBFDataForWrite> =
            value
                .data()
                .iter()
                .map(|it| {
                    let id = it.data_id();
                    let format = unsafe { data_formats.iter_mut().find(|it| it.data_id == id).unwrap_unchecked() };
                    format.refs += 1;
                    UPBFDataForWrite::new(id, it.name().clone(), it.data().into())
                })
                .collect();
        Ok(
            Self {
                build_name: value.build_name().clone(),
                build_version: value.build_version().clone(),
                data_format_id_last,
                data_format_id_pool: Vec::new(),
                data_formats,
                data
            }
        )
    }
}

impl UPBFDataFormatForWrite {
    pub fn new(data_id: u32, name: String, refs: u32) -> Self {
        Self { data_id, name, refs }
    }

    pub fn data_id(&self) -> u32 {
        self.data_id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn refs(&self) -> u32 {
        self.refs
    }
}

impl UPBFDataForWrite {
    pub fn new(data_id: u32, name: String, data: Box<[u8]>) -> Self {
        Self { data_id, name, data }
    }

    pub fn data_id(&self) -> u32 {
        self.data_id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl From<std::io::Error> for UPBFWriterError {
    fn from(value: std::io::Error) -> Self {
        Self::Write(UPBFWriterWriteError::IOError(value))
    }
}

impl Into<UPBFWriterError> for UPBFWriterDataAddError {
    fn into(self) -> UPBFWriterError {
        UPBFWriterError::DataAdd(self)
    }
}

impl Into<UPBFWriterError> for UPBFWriterWriteError {
    fn into(self) -> UPBFWriterError {
        UPBFWriterError::Write(self)
    }
}

