#![allow(dead_code)]

use log::SetLoggerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TdmsParseError {
    #[error("IO error.")]
    IoError(#[from] std::io::Error),
    #[error("SetLoggerError.")]
    SetLoggerError(#[from] SetLoggerError),
    #[error("tdms tag incorrect")]
    IncorrectTdmsTag,
    #[error("In TDMS file format version 2.0, 1 is the only valid value")]
    IncorrectArrayDimensionInTdmsObject,
    #[error("Incorrect datatype in the DAQmx raw data index.")]
    IncorrectDataTypeInDaqmxRawDataIndex,
    #[error("Cannot read timestamp channel.")]
    CannotReadTimestampChannel,
    #[error("Cannot read index channel.")]
    CannotReadIndexChannel,
    #[error("Cannot read header channel.")]
    CannotReadHeaderChannel,
    #[error("Cannot read metadata channel.")]
    CannotReadMetadataChannel,
    #[error("Unexpected segment")]
    UnexpectedSegment,
    #[error("Unexpected group")]
    UnexpectedGroup,
    #[error("Unexpected channel")]
    UnexpectedChannel,
    #[error("Unexpected metadata property {property}")]
    UnexpectedProperty { property: String },
    #[error("Contain no metadata")]
    NoMetadata,
    #[error("index {index}, timestamp {timestamp} or header {header} channel length mismatch")]
    ChannelLengthMismatch { index: usize, timestamp: usize, header: usize },
    #[error("Error occurred: {message}")]
    GeneralError { message: String },
}
