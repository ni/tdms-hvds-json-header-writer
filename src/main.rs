#[macro_use]
extern crate log;
extern crate log4rs;
extern crate serde;
extern crate serde_json;

use fs::File;
use std::{fs, fs::OpenOptions, io::BufReader, io::Seek, io::SeekFrom, path::PathBuf};

use chrono::{TimeZone, Utc};
use clap::{App, Arg};
use log::{LevelFilter, SetLoggerError};
use log4rs::{
    append::console::ConsoleAppender, append::console::Target, append::file::FileAppender,
    config::Appender, config::Config, config::Root, Handle,
};
use serde_json::to_writer_pretty;

use output::{Data, FileMetadata, Output, Stream, StreamMetadata};
use tdms_lead_in::LeadIn;
use tdms_metadata::Metadata;
use tdms_object::RawDataIndex;
use tdms_parse_error::TdmsParseError;
use util::{read_i64, read_unified_timestamp};

use crate::tdms_object::TdmsObject;
use crate::util::read_u32;

mod output;
mod tdms_datatype;
mod tdms_lead_in;
mod tdms_metadata;
mod tdms_object;
mod tdms_parse_error;
mod util;

fn main() {
    let matches = App::new("ADAS HVDS Indexer")
        .version("0.1")
        .author("Tian Yu <tian.yu@ni.com>")
        .about("Generate index for HVDS in a TDMS file with JSON format")
        .arg(
            Arg::with_name("INPUT")
                .help("Input file path")
                .required(true)
                .index(1),
        )
        .arg(Arg::with_name("output").short("o").help("output file path"))
        .get_matches();
    debug!("{:?}", matches);

    let path = matches.value_of("INPUT").unwrap();
    let output_path = matches.value_of("output").unwrap_or("");

    let result = parse(path);
    if result.is_err() {
        error!("{:?}", result.err().unwrap());
        return;
    }

    let (file_metadata, stream_metadata, data_start_pos, timestamps, indices,states, frame_numbers) = result.ok().unwrap();

    info!("Preparing to output");
    let result = output(
        path,
        output_path,
        file_metadata,
        stream_metadata,
        data_start_pos,
        timestamps,
        indices,
        states,
        frame_numbers
    );
    if result.is_err() {
        error!("{:?}", result.err().unwrap());
    }
}

fn parse(path: &str) -> Result<(FileMetadata, StreamMetadata, u64, Vec<u64>, Vec<i64>, Vec<u32>, Vec<u32>), TdmsParseError> {
    let _ = init_log(String::from(path))?;
    let mut file_metadata = FileMetadata::new();
    let mut stream_metadata = StreamMetadata::new();

    let file = Some(OpenOptions::new().read(true).open(path)?);
    let mut reader = BufReader::new(file.as_ref().unwrap());

    /*
        read the first segment to obtain the data starting pos
    */
    let (lead_in, metadata) = read_segment_header(&mut reader)?;
    check_extract_metadata(&metadata, &mut file_metadata, &mut stream_metadata)?;
    let data_start_pos = lead_in.raw_data_offset + 28;

    // skip to the next segment starting point
    let _ = reader.seek(SeekFrom::Start(lead_in.next_segment_offset + 28));

    /*
        read the next segment (data segment cont.)
    */
    let mut current_pos: u64 = get_current_read_pos(&mut reader)?;
    debug!("data channel byte offset = {:?}", current_pos);
    let (lead_in, metadata) = read_segment_header(&mut reader)?;
    check_extract_metadata(&metadata, &mut file_metadata, &mut stream_metadata)?;
    // let data_start_pos_2 = get_current_read_pos(&mut reader)?;

    // skip to the next segment starting point
    let _ = reader.seek(SeekFrom::Start(
        current_pos + lead_in.next_segment_offset + 28,
    ));

    /*
        read the next segment to get the information for Timestamps
    */
    current_pos = get_current_read_pos(&mut reader)?;
    debug!("timestamp channel byte offset = {:?}", current_pos);
    let (lead_in, metadata) = read_segment_header(&mut reader)?;
    check_extract_metadata(&metadata, &mut file_metadata, &mut stream_metadata)?;
    let timestamps = read_timestamps(&mut reader, &lead_in, &metadata)?;

    // skip to the next segment starting point
    let _ = reader.seek(SeekFrom::Start(
        current_pos + lead_in.next_segment_offset + 28,
    ));

    /*
       read the next segment to get the information for index
    */
    current_pos = get_current_read_pos(&mut reader)?;
    debug!("index channel byte offset = {:?}", current_pos);
    let (lead_in, metadata) = read_segment_header(&mut reader)?;
    check_extract_metadata(&metadata, &mut file_metadata, &mut stream_metadata)?;
    let indices = read_indices(&mut reader, &lead_in, &metadata)?;

    // skip to the next segment starting point
    let _ = reader.seek(SeekFrom::Start(
        current_pos + lead_in.next_segment_offset + 28,
    ));

    /*
        read the next segment to get the information for header channel
    */
    current_pos = get_current_read_pos(&mut reader)?;
    debug!("header channel byte offset = {:?}", current_pos);
    let (lead_in, metadata) = read_segment_header(&mut reader)?;
    check_extract_metadata(&metadata, &mut file_metadata, &mut stream_metadata)?;
    let (states, frame_numbers) = read_headers(&mut reader, &lead_in, &metadata)?;

    // skip to the next segment starting point
    let _ = reader.seek(SeekFrom::Start(
        current_pos + lead_in.next_segment_offset + 28,
    ));

    // // check whether we have more segments?
    // if lead_in.next_segment_offset != 0xffff_ffff_ffff_ffff {
    //     return Err(TdmsParseError::UnexpectedSegment);
    // }

    Ok((file_metadata, stream_metadata, data_start_pos, timestamps, indices, states, frame_numbers))
}

fn check_extract_metadata<'a>(metadata: &Metadata, file_metadata: &'a mut FileMetadata, stream_metadata: &'a mut StreamMetadata) -> Result<(&'a mut FileMetadata, &'a mut StreamMetadata), TdmsParseError> {
    // check groups
    if metadata.objects.iter().any(|x| is_valid_group(x)) {
        return Err(TdmsParseError::UnexpectedGroup);
    }

    // check channels
    if metadata.objects.iter().any(|x| is_valid_channel(x)) {
        return Err(TdmsParseError::UnexpectedChannel);
    }

    // extract metadata
    if let Some(root_object) = metadata.objects.iter().find(|x| x.path == "/") {
        for property in root_object.properties.iter() {
            let name = property.name.clone();
            let string_value = property.string_value.clone();

            match name.as_str() {
                "name" => file_metadata.name = string_value,
                "LoggerVersionMajor" => {
                    file_metadata.logger_version_major = string_value.parse().unwrap()
                }
                "LoggerVersionMinor" => {
                    file_metadata.logger_version_minor = string_value.parse().unwrap()
                }
                "LoggerVersionBuild" => {
                    file_metadata.logger_version_build = string_value.parse().unwrap()
                }
                "LoggerVersionJSON" => {
                    file_metadata.logger_version_json = string_value.parse().unwrap()
                }
                "TDMSVersionMajor" => {
                    file_metadata.tdms_version_major = string_value.parse().unwrap()
                }
                "TDMSVersionMinor" => {
                    file_metadata.tdms_version_minor = string_value.parse().unwrap()
                }
                "CarModel" => file_metadata.car_model = string_value,
                "CarLicensePlate" => file_metadata.car_license_plate = string_value,
                "ReferenceFileName" => file_metadata.reference_file_name = string_value,
                "SplitBefore" => file_metadata.reference_split_before = string_value,
                "SplitAfter" => file_metadata.reference_split_after = string_value,
                "SplitStartTime" => {
                    file_metadata.reference_split_start_time = string_value.parse().unwrap()
                }
                "SplitStopTime" => {
                    file_metadata.reference_split_stop_time = string_value.parse().unwrap()
                }
                "FutureTimeEvent" => {
                    file_metadata.future_time_event = string_value.parse().unwrap()
                }
                "ZFramePath" => file_metadata.zframe_path = string_value,
                _ => {
                    return Err(TdmsParseError::UnexpectedProperty { property: name });
                }
            }
        }
    }

    if let Some(root_object) = metadata
        .objects
        .iter()
        .find(|x| x.path == "/'Stream'")
    {
        for property in root_object.properties.iter() {
            let name = property.name.clone();
            let string_value = property.string_value.clone();

            match name.as_str() {
                "name" => stream_metadata.name = "Stream".to_string(),
                "ChannelType" => stream_metadata.channel_type = string_value,
                "ChannelSubType" => {
                    stream_metadata.channel_sub_type = string_value.parse().unwrap()
                }
                "ChannelInfo" => stream_metadata.channel_info = string_value,
                "Sensor" => stream_metadata.sensor = string_value,
                "PropertiesJSON" => stream_metadata.properties_json = string_value,
                "Codec" => stream_metadata.codec = string_value,
                "System" => stream_metadata.system = string_value,
                "PXIeCard" => stream_metadata.pxie_card = string_value,
                "SampleTimestamp" => stream_metadata.sample_timestamp = string_value,
                "SampleType" => stream_metadata.sample_type = string_value,
                "SampleTypeVersion" => stream_metadata.sample_type_version = string_value,
                _ => {
                    return Err(TdmsParseError::UnexpectedProperty { property: name });
                }
            }
        }
    }

    Ok((file_metadata, stream_metadata))
}

fn is_root(x: &TdmsObject) -> bool {
    x.path == "/"
}

fn is_valid_group(x: &TdmsObject) -> bool {
    if is_root(x) {
        return false;
    }
    let allowed_groups = ["'stream'"];
    let group_name = get_group_name(x);
    if let Some(name) = group_name {
        !allowed_groups.contains(&name.to_lowercase().as_str())
    } else {
        false
    }
}

fn get_group_name(object: &TdmsObject) -> Option<String> {
    let parts = object.path.split('/').collect::<Vec<&str>>();
    if parts.len() != 2 {
        None
    } else {
        Some(parts[1].to_string())
    }
}

fn is_valid_channel(x: &TdmsObject) -> bool {
    if is_root(x) {
        return false;
    }
    let allowed_channels = ["'data'", "'index'", "'timestamp'", "'header'", "'metadata'"];
    let channel_name = get_channel_name(x);
    if let Some(name) = channel_name {
        !allowed_channels.contains(&name.to_lowercase().as_str())
    } else {
        false
    }
}

fn output(
    path: &str,
    output_path: &str,
    file_metadata: FileMetadata,
    stream_metadata: StreamMetadata,
    data_start_pos: u64,
    timestamps: Vec<u64>,
    indices: Vec<i64>,
    states: Vec<u32>,
    frame_numbers: Vec<u32>
) -> Result<(), TdmsParseError> {
    // output to a file with json Format
    let mut output_json_file = PathBuf::from(path);
    output_json_file.set_extension("hvds.json");

    // process index channel and timestamp channel
    let count = indices.len() / 2;
    if count != timestamps.len() / 4 || count != states.len() {
        return Err(TdmsParseError::ChannelLengthMismatch {
            index: indices.len(),
            timestamp: timestamps.len(),
            header: states.len()
        });
    }
    let mut byte_offset: Vec<u64> = Vec::with_capacity(count);
    let mut frame_size: Vec<u64> = Vec::with_capacity(count);
    let mut timestamp: Vec<u64> = Vec::with_capacity(count);
    let mut state: Vec<u32> = Vec::with_capacity(count);
    let mut frame_number: Vec<u32> = Vec::with_capacity(count);

    for i in 0..count {
        let mut start_offset = indices[2 * i];
        if start_offset < 0 {
            start_offset = start_offset.abs();
        }
        let mut end_offset = indices[2 * i + 1].abs();
        if end_offset < 0 {
            end_offset = end_offset.abs();
        }

        trace!("start_offset = {}, data_start_pos = {}", start_offset, data_start_pos);

        byte_offset.push(start_offset as u64 + data_start_pos);
        frame_size.push((end_offset - start_offset) as u64);
        timestamp.push(timestamps[4 * i]);
        state.push(states[i]);
        frame_number.push(frame_numbers[i]);
    }
    let output_data = Data {
        byte_offset,
        frame_size,
        timestamp,
        state,
        frame_number
    };

    let output_stream = Stream {
        metadata: stream_metadata,
        data: output_data,
    };

    let output_file_path = if output_path == "" {
        PathBuf::from(path)
    } else {
        PathBuf::from(output_path)
    };
    let file_name = output_file_path.file_name().unwrap();
    let output_file = output::File {
        raw_file: String::from(file_name.to_str().unwrap()),
        metadata: file_metadata,
        stream: output_stream,
    };
    let output = Output {
        schema: "http://audi.de/adas/logging/hvds/V0_0_1".to_string(),
        file: output_file,
    };
    to_writer_pretty(&File::create(output_json_file)?, &output).unwrap();
    Ok(())
}

fn read_headers(
    mut reader: &mut BufReader<&File>,
    lead_in: &LeadIn,
    metadata: &Metadata,
) -> Result<(Vec<u32>, Vec<u32>), TdmsParseError> {
    // Frame 1: State
    // Frame 1: FrameNumber,
    // Frame 2: State,
    // Frame 2: FrameNumber
    let mut states: Vec<u32> = vec![];
    let mut frame_numbers: Vec<u32> = vec![];
    if let RawDataIndex::NewDataIndex(new_raw_data_index) =
    metadata.objects[0].raw_data_index.clone()
    {
        let count = new_raw_data_index.chunk_size / 2;
        states.reserve(count as usize);
        frame_numbers.reserve(count as usize);
        for _ in 0..count {
            let state = read_u32(&mut reader, lead_in.is_toc_big_endian);
            let frame_number = read_u32(&mut reader, lead_in.is_toc_big_endian);
            trace!("{:?}", state);
            trace!("{:?}", frame_number);
            states.push(state);
            frame_numbers.push(frame_number);
        }
    } else {
        return Err(TdmsParseError::CannotReadHeaderChannel);
    }

    Ok((states, frame_numbers))
}

fn read_indices(
    mut reader: &mut BufReader<&File>,
    lead_in: &LeadIn,
    metadata: &Metadata,
) -> Result<Vec<i64>, TdmsParseError> {
    let mut indices: Vec<i64> = vec![];
    if let RawDataIndex::NewDataIndex(new_raw_data_index) =
        metadata.objects[0].raw_data_index.clone()
    {
        let index_count = new_raw_data_index.chunk_size / 2;
        indices.reserve((index_count * 2) as usize);
        for _ in 0..index_count {
            let index_start = read_i64(&mut reader, lead_in.is_toc_big_endian);
            let index_end = read_i64(&mut reader, lead_in.is_toc_big_endian);
            trace!("{:?}", index_start);
            trace!("{:?}", index_end);
            indices.push(index_start);
            indices.push(index_end);
        }
    } else {
        return Err(TdmsParseError::CannotReadIndexChannel);
    }

    Ok(indices)
}

fn read_timestamps(
    mut reader: &mut BufReader<&File>,
    lead_in: &LeadIn,
    metadata: &Metadata,
) -> Result<Vec<u64>, TdmsParseError> {
    // In the Timestamp channel, each frame has 4 u64 timestamps.
    // Relative Start Timestamp -> Unified hardware timestamps
    // Relative End Timestamp
    // Absolute Start Timestamp -> Unified hardware timestamps
    // Absolute End Timestamp
    let mut timestamps: Vec<u64> = vec![];
    debug!(
        "reading timestamps {:?}",
        metadata.objects[0].raw_data_index.clone()
    );
    if let Some(object) = get_raw_data_index(&metadata) {
        if let RawDataIndex::NewDataIndex(new_raw_data_index) = object.raw_data_index.clone() {
            let timestamp_count = new_raw_data_index.chunk_size / 4;
            timestamps.reserve((timestamp_count * 4) as usize);
            for _ in 0..timestamp_count {
                let relative_start = read_unified_timestamp(&mut reader, lead_in.is_toc_big_endian);
                let relative_end = read_unified_timestamp(&mut reader, lead_in.is_toc_big_endian);
                let absolute_start = read_unified_timestamp(&mut reader, lead_in.is_toc_big_endian);
                let absolute_end = read_unified_timestamp(&mut reader, lead_in.is_toc_big_endian);
                trace!("{:?}", relative_start);
                trace!("{:?}", Utc.timestamp_nanos(relative_start as i64));
                trace!("{:?}", relative_end);
                trace!("{:?}", Utc.timestamp_nanos(relative_end as i64));
                trace!("{:?}", absolute_start);
                trace!("{:?}", Utc.timestamp_nanos(absolute_start as i64));
                trace!("{:?}", absolute_end);
                trace!("{:?}", Utc.timestamp_nanos(absolute_end as i64));
                timestamps.push(relative_start);
                timestamps.push(relative_end);
                timestamps.push(absolute_start);
                timestamps.push(absolute_end);
            }
        } else {
            return Err(TdmsParseError::CannotReadTimestampChannel);
        }
    } else {
        return Err(TdmsParseError::CannotReadTimestampChannel);
    }
    Ok(timestamps)
}

fn get_raw_data_index(metadata: &Metadata) -> Option<&TdmsObject> {
    metadata.objects.iter().find(|x| is_timestamp_channel(x))
}

fn is_timestamp_channel(tdms_object: &TdmsObject) -> bool {
    if let Some(name) = get_channel_name(tdms_object) {
        name.to_lowercase() == "'timestamp'"
    } else {
        return false;
    }
}

fn get_channel_name(object: &TdmsObject) -> Option<String> {
    let parts = object.path.split('/').collect::<Vec<&str>>();
    if parts.len() != 3 {
        None
    } else {
        Some(parts[2].to_string())
    }
}

fn read_segment_header(
    mut reader: &mut BufReader<&File>,
) -> Result<(LeadIn, Metadata), TdmsParseError> {
    let lead_in = LeadIn::read(&mut reader)?;
    debug!("{:?}", lead_in);

    if lead_in.contains_metadata {
        let metadata = Metadata::read(&mut reader, lead_in.is_toc_big_endian)?;
        debug!("{:?}", metadata);
        Ok((lead_in, metadata))
    } else {
        return Err(TdmsParseError::NoMetadata);
    }
}

fn get_current_read_pos(reader: &mut BufReader<&File>) -> Result<u64, TdmsParseError> {
    let pos = reader.seek(SeekFrom::Current(0))?;
    Ok(pos)
}

fn init_log(path: String) -> Result<Handle, SetLoggerError> {
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    let mut log_file_name = PathBuf::from(path);
    log_file_name.set_extension("log");
    let logfile = FileAppender::builder().build(log_file_name).unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(Appender::builder().build("stderr", Box::new(stderr)))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    Ok(log4rs::init_config(config).unwrap())
}
