pub mod raw;
pub mod read;
pub mod write;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UPBFType {
    MediumAlignedLittleEndian   = 0x0,
    MediumAlignedBigEndian      = 0x1,
    BigAlignedLittleEndian      = 0x2,
    BigAlignedBigEndian         = 0x3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UPBFVersion(u8);

impl TryFrom<u8> for UPBFType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 0x3 { return Err(()); }
        unsafe { std::mem::transmute(value) }
    }
}

impl Into<u8> for UPBFType {
    fn into(self) -> u8 {
        unsafe {  std::mem::transmute(self) }
    }
}

impl UPBFVersion {
    pub const V0: Self = Self(0x0);
    pub const V1: Self = Self(0x1);
    pub const LAST_SUPPORTED: Self = Self::V1;

    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn is_supported(self) -> bool {
        self.0 == Self::LAST_SUPPORTED.0
    }

    pub const fn as_raw(self) -> u8 {
        self.0
    }
}

impl Into<u8> for UPBFVersion {
    fn into(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use read::{UPBFReader, UPBFReaderError, UPBFReaderFormatReadError, UPBFReaderHeaderReadError };
    use write::{UPBFWriter, UPBFWriterDataAddError, UPBFWriterError, UPBFWriterWriteError };

    // Helper: create a writer with some data formats and data blocks
    fn create_test_writer(
        build_name: &str,
        build_version: &str,
        data_specs: &[(&str, &str, &[u8])], // (data_name, format_name, data_bytes)
    ) -> UPBFWriter {
        let mut writer = UPBFWriter::new(build_name.to_string(), build_version.to_string());
        for (data_name, format_name, data_bytes) in data_specs {
            writer
                .add_data(
                    data_name.to_string(),
                    &format_name.to_string(),
                    data_bytes.to_vec().into_boxed_slice(),
                )
                .unwrap();
        }
        writer
    }

    #[test]
    fn roundtrip_all_types() {
        let specs = &[
            ("data1", "formatA", &b"hello"[..]),
            ("data2", "formatB", &b"world"[..]),
            ("data3", "formatA", &b"again"[..]),
        ];
        let build_name = "@test_build";
        let build_version = "1.2.3";

        for ty in [
            UPBFType::MediumAlignedLittleEndian,
            UPBFType::MediumAlignedBigEndian,
            UPBFType::BigAlignedLittleEndian,
            UPBFType::BigAlignedBigEndian,
        ] {
            let mut writer = create_test_writer(build_name, build_version, specs);
            let bytes = writer.write(ty, UPBFVersion::LAST_SUPPORTED).unwrap();
            let reader = UPBFReader::new(&bytes).unwrap();
            let read_result = reader.read().unwrap();

            assert_eq!(read_result.file_type(), ty);
            assert_eq!(read_result.build_name(), build_name);
            assert_eq!(read_result.build_version(), build_version);

            let formats = read_result.data_formats();
            assert_eq!(formats.len(), 2);
            let format_a = formats.iter().find(|f| f.name() == "formatA").unwrap();
            let format_b = formats.iter().find(|f| f.name() == "formatB").unwrap();
            let data = read_result.data();
            assert_eq!(data.len(), 3);
            let data1 = data.iter().find(|d| d.name() == "data1").unwrap();
            let data2 = data.iter().find(|d| d.name() == "data2").unwrap();
            let data3 = data.iter().find(|d| d.name() == "data3").unwrap();
            assert_eq!(data1.data_id(), format_a.data_id());
            assert_eq!(data3.data_id(), format_a.data_id());
            assert_eq!(data2.data_id(), format_b.data_id());
            assert_eq!(data1.data(), b"hello");
            assert_eq!(data2.data(), b"world");
            assert_eq!(data3.data(), b"again");
        }
    }

    #[test]
    fn data_format_reuse() {
        let mut writer = UPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_data("d1".to_string(), &"fmt".to_string(), b"data1".to_vec().into_boxed_slice())
            .unwrap();
        writer
            .add_data("d2".to_string(), &"fmt".to_string(), b"data2".to_vec().into_boxed_slice())
            .unwrap();

        let formats = writer.data_formats();
        assert_eq!(formats.len(), 1);
        assert_eq!(formats[0].name(), "fmt");
        assert_eq!(formats[0].refs(), 2);
    }

    #[test]
    fn remove_data_decrements_refs() {
        let mut writer = UPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_data("d1".to_string(), &"fmt".to_string(), b"d1".to_vec().into_boxed_slice())
            .unwrap();
        writer
            .add_data("d2".to_string(), &"fmt".to_string(), b"d2".to_vec().into_boxed_slice())
            .unwrap();

        assert_eq!(writer.data_formats().len(), 1);
        assert_eq!(writer.data_formats()[0].refs(), 2);

        assert!(writer.remove_data(&"d1".to_string()));
        assert_eq!(writer.data().len(), 1);
        assert_eq!(writer.data_formats().len(), 1);
        assert_eq!(writer.data_formats()[0].refs(), 1);

        assert!(writer.remove_data(&"d2".to_string()));
        assert_eq!(writer.data().len(), 0);
        assert_eq!(writer.data_formats().len(), 0);

        writer
            .add_data("d3".to_string(), &"fmt".to_string(), b"d3".to_vec().into_boxed_slice())
            .unwrap();
        assert_eq!(writer.data_formats().len(), 1);
    }

    #[test]
    fn add_data_duplicate_name_error() {
        let mut writer = UPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_data("dup".to_string(), &"fmt".to_string(), b"first".to_vec().into_boxed_slice())
            .unwrap();
        let err = writer
            .add_data("dup".to_string(), &"fmt".to_string(), b"second".to_vec().into_boxed_slice())
            .unwrap_err();
        match err {
            UPBFWriterError::DataAdd(UPBFWriterDataAddError::DataAlreadyDefined) => (),
            _ => panic!("expected DataAlreadyDefined"),
        }
    }

    #[test]
    fn add_or_overwrite_data() {
        let mut writer = UPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_or_overwrite_data(&"d1".to_string(), &"fmt".to_string(), b"first".to_vec().into_boxed_slice())
            .unwrap();
        assert_eq!(writer.data().len(), 1);
        assert_eq!(writer.data()[0].data(), b"first");

        writer
            .add_or_overwrite_data(&"d1".to_string(), &"fmt2".to_string(), b"second".to_vec().into_boxed_slice())
            .unwrap();
        assert_eq!(writer.data().len(), 1);
        assert_eq!(writer.data()[0].data(), b"second");
        let fmt = writer.data_formats().iter().find(|f| f.data_id() == writer.data()[0].data_id()).unwrap();
        assert_eq!(fmt.name(), "fmt2");
    }

    #[test]
    fn remove_nonexistent_data_returns_false() {
        let mut writer = UPBFWriter::new("test".to_string(), "1.0".to_string());
        assert!(!writer.remove_data(&"nosuch".to_string()));
    }

    #[test]
    fn read_invalid_magic() {
        let bytes = b"BADMAGIC";
        let err = UPBFReader::new(bytes).unwrap_err();
        match err {
            UPBFReaderError::Header(UPBFReaderHeaderReadError::InvalidMagic) => (),
            _ => panic!("expected InvalidMagic"),
        }
    }

    #[test]
    fn read_unsupported_version() {
        let mut bytes = b".UPBF\0".to_vec();
        bytes.push(0x00); // type
        bytes.push(0xFF); // unsupported version
        bytes.resize(8, 0);
        let reader = UPBFReader::new(&bytes).unwrap();
        assert_eq!(reader.is_read_supported(), false);
        let err = reader.read().unwrap_err();
        match err {
            UPBFReaderError::Header(UPBFReaderHeaderReadError::UnsupportedVersion) => (),
            _ => panic!("expected UnsupportedVersion"),
        }
    }

    #[test]
    fn read_truncated_file() {
        let bytes = b".UPBF\0";
        let err = UPBFReader::new(bytes).unwrap_err();
        match err {
            UPBFReaderError::InvalidFileLength => (),
            _ => panic!("expected InvalidFileLength"),
        }
    }

    #[test]
    fn read_truncated_name() {
        let mut writer = UPBFWriter::new("very_long_build_name_that_exceeds_buffer".to_string(), "1.0".to_string());
        let mut bytes = writer
            .write(UPBFType::MediumAlignedLittleEndian, UPBFVersion::LAST_SUPPORTED)
            .unwrap();
        bytes.truncate(0x20);
        let reader = UPBFReader::new(&bytes).unwrap();
        let err = reader.read().unwrap_err();
        match err {
            UPBFReaderError::Header(UPBFReaderHeaderReadError::InvalidBuildNameLength) => (),
            _ => panic!("expected InvalidLength"),
        }
    }

    #[test]
    fn data_format_lookup() {
        let mut writer = create_test_writer(
            "test",
            "1.0",
            &[("data1", "formatX", &b"abc"[..]), ("data2", "formatY", &b"def"[..])],
        );
        let bytes = writer.write(UPBFType::MediumAlignedLittleEndian, UPBFVersion::LAST_SUPPORTED).unwrap();
        let reader = UPBFReader::new(&bytes).unwrap();
        let read_result = reader.read().unwrap();

        let data = read_result.data();
        let data1 = data.iter().find(|d| d.name() == "data1").unwrap();
        let fmt1 = data1.format(&read_result);
        assert_eq!(fmt1.name(), "formatX");
        let data2 = data.iter().find(|d| d.name() == "data2").unwrap();
        let fmt2 = data2.format(&read_result);
        assert_eq!(fmt2.name(), "formatY");
    }

    #[test]
    fn empty_writer() {
        let mut writer = UPBFWriter::new("empty".to_string(), "0.0".to_string());
        let bytes = writer.write(UPBFType::MediumAlignedLittleEndian, UPBFVersion::LAST_SUPPORTED).unwrap();
        let reader = UPBFReader::new(&bytes).unwrap();
        let read_result = reader.read().unwrap();

        assert_eq!(read_result.data_formats().len(), 0);
        assert_eq!(read_result.data().len(), 0);
        assert_eq!(read_result.build_name(), "empty");
        assert_eq!(read_result.build_version(), "0.0");
    }

    #[test]
    fn convert_from_read_result() {
        let mut writer_orig = create_test_writer(
            "convert",
            "2.0",
            &[("c1", "fmtA", &b"one"[..]), ("c2", "fmtB", &b"two"[..]), ("c3", "fmtA", &b"three"[..])],
        );
        let bytes = writer_orig.write(UPBFType::BigAlignedBigEndian, UPBFVersion::LAST_SUPPORTED).unwrap();
        let reader = UPBFReader::new(&bytes).unwrap();
        let read_result = reader.read().unwrap();

        let writer_converted: UPBFWriter = UPBFWriter::try_from(&read_result).unwrap();
        let mut writer_converted_mut = writer_converted;
        let bytes2 = writer_converted_mut.write(UPBFType::BigAlignedBigEndian, UPBFVersion::LAST_SUPPORTED).unwrap();
        // The two byte sequences should be identical (deterministic writer)
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn data_id_reuse_after_remove() {
        let mut writer = UPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_data("d1".to_string(), &"fmt".to_string(), b"".to_vec().into_boxed_slice())
            .unwrap();
        let first_format_id = writer.data_formats()[0].data_id();
        writer.remove_data(&"d1".to_string());
        writer
            .add_data("d2".to_string(), &"fmt".to_string(), b"".to_vec().into_boxed_slice())
            .unwrap();
        assert_eq!(writer.data_formats()[0].data_id(), first_format_id);
    }

    #[test]
    fn writer_errors_on_too_long_strings() {
        let mut writer = UPBFWriter::new("a".repeat(u32::MAX as usize).to_string(), "1.0".to_string());
        let err = writer
            .write(UPBFType::MediumAlignedLittleEndian, UPBFVersion::LAST_SUPPORTED)
            .unwrap_err();
        match err {
            UPBFWriterError::Write(UPBFWriterWriteError::InvalidBuildNameLength) => (),
            _ => panic!("expected InvalidNameLength"),
        }
    }

    #[test]
    fn read_invalid_type_byte() {
        let mut bytes = b".UPBF\0".to_vec();
        bytes.push(0xFF); // invalid type
        bytes.push(UPBFVersion::LAST_SUPPORTED.into());
        bytes.resize(8, 0);
        let err = UPBFReader::new(&bytes).unwrap_err();
        match err {
            UPBFReaderError::Header(UPBFReaderHeaderReadError::InvalidType) => (),
            _ => panic!("expected InvalidType"),
        }
    }

    #[test]
    fn read_corrupted_format_next_offset() {
        let mut writer = create_test_writer("corrupt", "1.0", &[("d1", "fmt", &b"x"[..])]);
        let mut bytes = writer
            .write(UPBFType::MediumAlignedLittleEndian, UPBFVersion::LAST_SUPPORTED)
            .unwrap();
        let format_offset = u32::from_le_bytes(bytes[0x8..0xC].try_into().unwrap()) as usize;
        let next_offset = format_offset + 0x10000;
        bytes[format_offset..format_offset + 4].copy_from_slice(&(next_offset as u32).to_le_bytes());
        let reader = UPBFReader::new(&bytes).unwrap();
        let err = reader.read().unwrap_err();
        match err {
            UPBFReaderError::DataFormat(UPBFReaderFormatReadError::InvalidOffset) => (),
            _ => panic!("expected InvalidOffset"),
        }
    }
}