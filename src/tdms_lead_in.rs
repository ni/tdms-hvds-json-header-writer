#![allow(dead_code)]

use byteorder::{ByteOrder, LittleEndian, NativeEndian};
use std::io::prelude::*;

use crate::tdms_parse_error::TdmsParseError;
use crate::util::{load_part, read_u32, read_u64};

#[derive(Debug)]
pub struct LeadIn {
    toc_mask: u32,
    version: u32,
    pub next_segment_offset: u64,
    pub raw_data_offset: u64,
    pub contains_metadata: bool,
    pub contains_raw_data: bool,
    contains_daqmx_raw_data: bool,
    is_raw_data_in_segment_interleaved: bool,
    pub is_toc_big_endian: bool,
    contains_new_object_list: bool,
}

impl LeadIn {
    pub fn read<T: Read>(mut reader: &mut T) -> Result<LeadIn, TdmsParseError> {
        let tdms_tag = NativeEndian::read_u32(&mut load_part(reader, 4));
        let is_little_endian = crate::util::is_little_endian();
        if (!is_little_endian && tdms_tag as u64 != 0x54444536D)
            || (is_little_endian && tdms_tag != 1834173524)
        {
            return Err(TdmsParseError::IncorrectTdmsTag);
        }

        let toc_mask = LittleEndian::read_u32(&mut load_part(reader, 4));
        let is_toc_big_endian = toc_mask & 64 != 0;

        Ok(LeadIn {
            toc_mask: toc_mask,
            version: read_u32(&mut reader, is_toc_big_endian),
            next_segment_offset: read_u64(&mut reader, is_toc_big_endian),
            raw_data_offset: read_u64(&mut reader, is_toc_big_endian),
            contains_metadata: toc_mask & 2 != 0,
            contains_new_object_list: toc_mask & 4 != 0,
            contains_raw_data: toc_mask & 8 != 0,
            is_raw_data_in_segment_interleaved: toc_mask & 32 != 0,
            is_toc_big_endian,
            contains_daqmx_raw_data: toc_mask & 128 != 0,
        })
    }
}
