use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian};
use std::any::TypeId;
use std::io::prelude::*;

use crate::TdmsParseError;

pub fn is_little_endian() -> bool {
    TypeId::of::<NativeEndian>() == TypeId::of::<LittleEndian>()
}

pub fn load_part<T: Read>(reader: &mut T, size: usize) -> std::vec::Vec<u8> {
    let mut buf = Vec::with_capacity(size);
    let mut part_reader = reader.take(size as u64);
    part_reader.read_to_end(&mut buf).unwrap();
    buf
}

pub fn read_u32<T: Read>(reader: &mut T, is_toc_big_endian: bool) -> u32 {
    match is_toc_big_endian {
        true => BigEndian::read_u32(&mut load_part(reader, 4)),
        false => LittleEndian::read_u32(&mut load_part(reader, 4)),
    }
}

pub fn read_u64<T: Read>(reader: &mut T, is_toc_big_endian: bool) -> u64 {
    match is_toc_big_endian {
        true => BigEndian::read_u64(&mut load_part(reader, 8)),
        false => LittleEndian::read_u64(&mut load_part(reader, 8)),
    }
}

pub fn read_i64<T: Read>(reader: &mut T, is_toc_big_endian: bool) -> i64 {
    match is_toc_big_endian {
        true => BigEndian::read_i64(&mut load_part(reader, 8)),
        false => LittleEndian::read_i64(&mut load_part(reader, 8)),
    }
}

pub fn read_string<T: Read>(reader: &mut T, size: usize) -> Result<String, TdmsParseError> {
    let mut buffer = vec![0; size];
    reader.read(&mut buffer)?;
    let s = buffer.iter().map(|&c| c as char).collect::<String>();
    Ok(s)
}

pub fn read_unified_timestamp<T: Read>(reader: &mut T, is_toc_big_endian: bool) -> u64 {
    let mut value = read_u64(reader, is_toc_big_endian);
    value &= !(1u64 << 63);
    value &= !(1u64 << 62);
    value
}
