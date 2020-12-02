use crate::tdms_datatype::TdmsDataType;

#[derive(Debug, Clone)]
pub struct TdmsObject {
    pub path: String,
    pub raw_data_index: RawDataIndex,
    pub property_size: u32,
    pub properties: std::vec::Vec<TdmsProperty>,
}

#[derive(Debug, Clone)]
pub enum RawDataIndex {
    NoRawData,
    Daqmx(DaqmxScaler),
    SameAsPrevious,
    NewDataIndex(NewRawDataIndex),
}

#[derive(Debug, Clone)]
pub struct DaqmxScaler {
    pub datatype: TdmsDataType,
    pub array_dimension: u32,
    pub chunk_size: u64,
    pub scalers_size: u32,
    pub scalers: std::vec::Vec<DaqFormatChangingScaler>,
    pub raw_data_size: u32,
    pub raw_data_vector: std::vec::Vec<u32>,
}

impl DaqmxScaler {
    pub fn new() -> Self {
        DaqmxScaler {
            datatype: TdmsDataType::Boolean,
            array_dimension: 0,
            chunk_size: 0,
            scalers_size: 0,
            scalers: std::vec::Vec::new(),
            raw_data_size: 0,
            raw_data_vector: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DaqFormatChangingScaler {
    pub size: u32,
    pub datatype: TdmsDataType,
    pub raw_buffer_index: u32,
    pub raw_byte_offset_within_stride: u32,
    pub sample_format_bitmap: u32,
    pub scale_id: u32,
}

#[derive(Debug, Clone)]
pub struct NewRawDataIndex {
    pub raw_data_index_length: u32,
    pub datatype: TdmsDataType,
    pub array_dimension: u32,
    pub chunk_size: u64,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct TdmsProperty {
    pub name: String,
    pub datatype: TdmsDataType,
    // numeric_value: ?,
    pub string_value: String,
}

impl TdmsProperty {
    pub fn new() -> Self {
        TdmsProperty {
            name: String::from(""),
            datatype: TdmsDataType::Boolean,
            string_value: String::from(""),
        }
    }
}
