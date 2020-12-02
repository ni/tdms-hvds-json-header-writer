#![allow(dead_code)]
#![allow(unused_assignments)]

use std::convert::Into;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::tdms_parse_error::TdmsParseError;
use std::io::prelude::*;

use crate::util::{read_string, read_u32, read_u64};

use crate::tdms_datatype::TdmsDataType;
use crate::tdms_object::{
    DaqFormatChangingScaler, DaqmxScaler, NewRawDataIndex, RawDataIndex, TdmsObject, TdmsProperty,
};

#[derive(Debug, Clone)]
pub struct Metadata {
    pub object_number: u32,
    pub objects: std::vec::Vec<TdmsObject>,
}

impl Metadata {
    pub fn read<T: Read>(
        mut reader: &mut T,
        is_toc_big_endian: bool,
    ) -> Result<Metadata, TdmsParseError> {
        let object_number = read_u32(&mut reader, is_toc_big_endian);

        let mut objects = vec![];
        for _ in 0..object_number {
            debug!("*********** OBJECT START");

            /* object path */
            let object_path_length = read_u32(&mut reader, is_toc_big_endian);
            debug!("object_path_length = {}", object_path_length);

            let object_path = read_string(&mut reader, object_path_length as usize)?;
            debug!("object_path = {}", object_path);

            /* raw data index */
            let raw_data_index: RawDataIndex;
            let mut raw_data_index_vec = vec![0; 4];
            reader.read(&mut raw_data_index_vec)?;
            match raw_data_index_vec.as_slice() {
                [0xff, 0xff, 0xff, 0xff] => {
                    // No raw data assigned in this segment
                    raw_data_index = RawDataIndex::NoRawData;
                }
                [0x69, 0x12, 0x00, 0x00] => {
                    let mut daqmx_data = DaqmxScaler::new();
                    // DAQmx raw data Format Changing scaler in this segment
                    let datatype_u32 = read_u32(&mut reader, is_toc_big_endian);
                    if datatype_u32 != 0xffffffff {
                        return Err(TdmsParseError::IncorrectDataTypeInDaqmxRawDataIndex);
                    }
                    let datatype: TdmsDataType = datatype_u32.into();
                    daqmx_data.datatype = datatype;

                    let array_dimension = read_u32(&mut reader, is_toc_big_endian); // TODO: check file version
                    daqmx_data.array_dimension = array_dimension;

                    let number_of_values = read_u64(&mut reader, is_toc_big_endian);
                    daqmx_data.chunk_size = number_of_values;

                    /* vector of Format Changing scalers*/
                    let scalers_vector_size = read_u32(&mut reader, is_toc_big_endian);
                    daqmx_data.scalers_size = scalers_vector_size;

                    let mut scalers: Vec<DaqFormatChangingScaler> = vec![];
                    for iter_number in 0..scalers_vector_size {
                        if iter_number == 0 {
                            let datatype_u32 = read_u32(&mut reader, is_toc_big_endian);
                            let datatype: TdmsDataType = datatype_u32.into();

                            let raw_buffer_index = read_u32(&mut reader, is_toc_big_endian);
                            let raw_byte_offset_within_stride =
                                read_u32(&mut reader, is_toc_big_endian);
                            let sample_format_bitmap = read_u32(&mut reader, is_toc_big_endian);
                            let scale_id = read_u32(&mut reader, is_toc_big_endian);

                            let scaler = DaqFormatChangingScaler {
                                size: scalers_vector_size,
                                datatype,
                                raw_buffer_index,
                                raw_byte_offset_within_stride,
                                sample_format_bitmap,
                                scale_id,
                            };
                            scalers.push(scaler);
                            // TODO: (If the vector size is larger than 1, the object contains multiple Format Changing scalers and the information in the previous bullet items can be repeated.)
                        }
                    }
                    daqmx_data.scalers = scalers;

                    /* vector of raw data width*/
                    let raw_data_width_vector_size = read_u32(&mut reader, is_toc_big_endian);
                    daqmx_data.raw_data_size = raw_data_width_vector_size;

                    let mut elements = vec![];
                    for _ in 0..raw_data_width_vector_size {
                        let raw_data_element = read_u32(&mut reader, is_toc_big_endian);
                        elements.push(raw_data_element);
                    }
                    daqmx_data.raw_data_vector = elements;
                    raw_data_index = RawDataIndex::Daqmx(daqmx_data);
                }
                [0x69, 0x13, 0x00, 0x00] => {
                    // DAQmx raw data Digital Line scaler in this segment
                    unimplemented!()
                }
                [0x00, 0x00, 0x00, 0x00] => {
                    // exactly matches the index the same object had in the previous segment
                    raw_data_index = RawDataIndex::SameAsPrevious;
                }
                _ => {
                    let mut new_raw_data_index_length = 0;
                    if is_toc_big_endian {
                        new_raw_data_index_length = BigEndian::read_u32(&raw_data_index_vec);
                    } else {
                        new_raw_data_index_length = LittleEndian::read_u32(&raw_data_index_vec);
                    }
                    let datatype_u32 = read_u32(&mut reader, is_toc_big_endian);
                    debug!("datatype = {:?}", datatype_u32);
                    let datatype: TdmsDataType = datatype_u32.into();
                    let array_dimension = read_u32(&mut reader, is_toc_big_endian); // TODO: check file version
                    let number_of_values = read_u64(&mut reader, is_toc_big_endian);
                    debug!("number of values: {}", number_of_values);

                    let mut total_size_bytes = 0;
                    if datatype == TdmsDataType::String {
                        total_size_bytes = read_u64(&mut reader, is_toc_big_endian);
                    }

                    let new_raw_data_index = NewRawDataIndex {
                        raw_data_index_length: new_raw_data_index_length,
                        datatype,
                        array_dimension,
                        chunk_size: number_of_values,
                        total_size_bytes,
                    };
                    raw_data_index = RawDataIndex::NewDataIndex(new_raw_data_index);
                }
            }

            /* properties */
            let number_of_properties = read_u32(&mut reader, is_toc_big_endian);

            let mut properties = vec![];
            for _ in 0..number_of_properties {
                let mut property = TdmsProperty::new();

                let property_name_length = read_u32(&mut reader, is_toc_big_endian);
                let property_name = read_string(&mut reader, property_name_length as usize)?;
                property.name = property_name;

                let property_datatype_u32 = read_u32(&mut reader, is_toc_big_endian);
                let property_datatype: TdmsDataType = property_datatype_u32.into();
                property.datatype = property_datatype;
               
                if property.datatype == TdmsDataType::String {
                    let property_value_length = read_u32(&mut reader, is_toc_big_endian);
                    debug!("property_value_length = {:?}", property_value_length);
                    let property_str_value =
                        read_string(&mut reader, property_value_length as usize)?;
                    property.string_value = property_str_value;
                } else if property.datatype == TdmsDataType::TimeStamp {
                    let seconds = read_u64(&mut reader, is_toc_big_endian);
                    debug!("seconds = {:?}", seconds);
                    let fractions = read_u64(&mut reader, is_toc_big_endian);
                    debug!("fractions = {:?}", fractions);
                } else if property.datatype == TdmsDataType::U64 {
                    let property_value = read_u64(&mut reader, is_toc_big_endian);
                    property.string_value = property_value.to_string();
                } else if property.datatype == TdmsDataType::U32 {
                    let property_value = read_u32(&mut reader, is_toc_big_endian);
                    property.string_value = property_value.to_string();
                }
                else {
                    let property_value = read_u32(&mut reader, is_toc_big_endian);
                    debug!("property_value = {:?}", property_value);
                }
                properties.push(property);
            }

            let tdms_object = TdmsObject {
                path: object_path,
                raw_data_index,
                property_size: number_of_properties,
                properties,
            };

            debug!("*********** OBJECT END\n");
            objects.push(tdms_object);
        }

        Ok(Metadata {
            object_number,
            objects,
        })
    }
}
