use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Output {
    pub schema: String,
    #[serde(rename(serialize = "File"))]
    pub file: File,
}

#[derive(Debug, Serialize)]
pub struct File {
    #[serde(rename(serialize = "RawFile"))]
    pub raw_file: String,
    #[serde(rename(serialize = "Metadata"))]
    pub metadata: FileMetadata,
    #[serde(rename(serialize = "Stream"))]
    pub stream: Stream,
}

#[derive(Debug, Serialize)]
pub struct FileMetadata {
    #[serde(rename(serialize = "Name"))]
    pub name: String,
    
    #[serde(rename(serialize = "LoggerVersionMajor"))]
    pub logger_version_major: u32,
    
    #[serde(rename(serialize = "LoggerVersionMinor"))]
    pub logger_version_minor: u32,

    #[serde(rename(serialize = "LoggerVersionBuild"))]
    pub logger_version_build: u32,
    
    #[serde(rename(serialize = "LoggerVersionJSON"))]
    pub logger_version_json: String,

    #[serde(rename(serialize = "TDMSVersionMajor"))]
    pub tdms_version_major: u32,

    #[serde(rename(serialize = "TDMSVersionMinor"))]
    pub tdms_version_minor: u32,

    #[serde(rename(serialize = "CarModel"))]
    pub car_model: String,

    #[serde(rename(serialize = "CarLicensePlate"))]
    pub car_license_plate: String,

    #[serde(rename(serialize = "ReferenceFileName"))]
    pub reference_file_name: String,

    #[serde(rename(serialize = "ReferenceSplitBefore"))]
    pub reference_split_before: String,

    #[serde(rename(serialize = "ReferenceSplitAfter"))]
    pub reference_split_after: String,

    #[serde(rename(serialize = "ReferenceSplitStartTime"))]
    pub reference_split_start_time: u64,

    #[serde(rename(serialize = "ReferenceSplitStopTime"))]
    pub reference_split_stop_time: u64,

    #[serde(rename(serialize = "FutureTimeEvent"))]
    pub future_time_event: u64,

    #[serde(rename(serialize = "ZFramePath"))]
    pub zframe_path: String,
}

impl FileMetadata {
    pub fn new() -> Self {
        FileMetadata {
            name: String::new(),
            logger_version_major: 0,
            logger_version_minor: 0,
            logger_version_build: 0,
            logger_version_json: String::new(),
            tdms_version_major: 0,
            tdms_version_minor: 0,
            car_model: String::new(),
            car_license_plate: String::new(),
            reference_file_name: String::new(),
            reference_split_before: String::new(),
            reference_split_after: String::new(),
            reference_split_start_time: 0,
            reference_split_stop_time: 0,
            future_time_event: 0,
            zframe_path: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StreamMetadata {
    #[serde(rename(serialize = "Name"))]
    pub name: String,

    #[serde(rename(serialize = "ChannelType"))]
    pub channel_type: String,
    
    #[serde(rename(serialize = "ChannelSubType"))]
    pub channel_sub_type: String,

    #[serde(rename(serialize = "ChannelInfo"))]
    pub channel_info: String,
    
    #[serde(rename(serialize = "Sensor"))]
    pub sensor: String,
    
    #[serde(rename(serialize = "PropertiesJSON"))]
    pub properties_json: String,
    
    #[serde(rename(serialize = "Codec"))]
    pub codec: String,
    
    #[serde(rename(serialize = "System"))]
    pub system: String,
    
    #[serde(rename(serialize = "PXIeCard"))]
    pub pxie_card: String,
    
    #[serde(rename(serialize = "SampleTimestamp"))]
    pub sample_timestamp: String,
    
    #[serde(rename(serialize = "SampleType"))]
    pub sample_type: String,
    
    #[serde(rename(serialize = "SampleTypeVersion"))]
    pub sample_type_version: String,
}

impl StreamMetadata {
    pub fn new() -> Self {
        StreamMetadata {
            name: String::new(),
            channel_type: String::new(),
            channel_sub_type: String::new(),
            channel_info: String::new(),
            sensor: String::new(),
            properties_json: String::new(),
            codec: String::new(),
            system: String::new(),
            pxie_card: String::new(),
            sample_timestamp: String::new(),
            sample_type: String::new(),
            sample_type_version: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Stream {
    #[serde(rename(serialize = "Metadata"))]
    pub metadata: StreamMetadata,
    #[serde(rename(serialize = "Data"))]
    pub data: Data,
}

#[derive(Debug, Serialize)]
pub struct Data {
    #[serde(rename(serialize = "FrameByteOffset"))]
    pub byte_offset: Vec<u64>,
    #[serde(rename(serialize = "FrameSize"))]
    pub frame_size: Vec<u64>,
    #[serde(rename(serialize = "Timestamp"))]
    pub timestamp: Vec<u64>,
    #[serde(rename(serialize = "State"))]
    pub state: Vec<u32>,
    #[serde(rename(serialize = "FrameNumber"))]
    pub frame_number: Vec<u32>
}
