use crate::{UPBFType, UPBFVersion};

pub fn check_header_len(bytes: &[u8]) -> bool {
    bytes.len() >= 8
}

pub fn check_magic(bytes: &[u8]) -> bool {
    &bytes[0x0..0x6] == b".UPBF\0"
}

pub fn read_version(bytes: &[u8]) -> UPBFVersion {
    UPBFVersion::new(bytes[0x7])
}

pub fn read_type(bytes: &[u8]) -> Result<UPBFType, ()> {
    UPBFType::try_from(bytes[0x6])
}