use crate::read::UPBFReaderError;
use crate::write::UPBFWriterError;

pub fn align_len_medium(len: usize) -> usize {
    (len + 3) & !3
}

pub fn align_len_big(len: usize) -> usize {
    (len + 7) & !7
}

pub fn u64_to_usize(value: u64, err: UPBFReaderError) -> Result<usize, UPBFReaderError> {
    if value >= usize::MAX as u64 {
        Err(err)
    } else {
        Ok(value as usize)
    }
}

pub fn usize_to_u32(value: usize, err: UPBFWriterError) -> Result<u32, UPBFWriterError> {
    if value >= u32::MAX as usize {
        Err(err)
    } else {
        Ok(value as u32)
    }
}

pub fn str_to_bytes_align_medium(str: &String) -> (&[u8], usize) {
    let len = str.len();
    let align = align_len_medium(len);
    (str.as_bytes(), align - len)
}

pub fn bytes_align_medium(bytes: &[u8]) -> usize {
    let len = bytes.len();
    align_len_medium(len) - len
}

pub fn str_to_bytes_align_big(str: &String) -> (&[u8], usize) {
    let len = str.len();
    let align = align_len_big(len);
    (str.as_bytes(), align - len)
}

pub fn bytes_align_big(bytes: &[u8]) -> usize {
    let len = bytes.len();
    align_len_big(len) - len
}