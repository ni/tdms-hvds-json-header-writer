#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone)]
pub enum TdmsDataType {
    Void,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    SingleFloat,
    DoubleFloat,
    ExtendedFloat,
    SingleFloatWithUnit = 0x19,
    DoubleFloatWithUnit,
    ExtendedFloatWithUnit,
    String = 0x20,
    Boolean = 0x21,
    TimeStamp = 0x44,
    FixedPoint = 0x4F,
    ComplexSingleFloat = 0x08000c,
    ComplexDoubleFloat = 0x10000d,
    DAQmxRawData = 0xFFFFFFFF,
}

impl From<u32> for TdmsDataType {
    fn from(input: u32) -> Self {
        match input {
            0x0 => TdmsDataType::Void,
            0x1 => TdmsDataType::I8,
            0x2 => TdmsDataType::I16,
            0x3 => TdmsDataType::I32,
            0x4 => TdmsDataType::I64,
            0x5 => TdmsDataType::U8,
            0x6 => TdmsDataType::U16,
            0x7 => TdmsDataType::U32,
            0x8 => TdmsDataType::U64,
            0x9 => TdmsDataType::SingleFloat,
            0xa => TdmsDataType::DoubleFloat,
            0xb => TdmsDataType::ExtendedFloat,
            0x19 => TdmsDataType::SingleFloatWithUnit,
            0x1a => TdmsDataType::DoubleFloatWithUnit,
            0x1b => TdmsDataType::ExtendedFloatWithUnit,
            0x20 => TdmsDataType::String,
            0x21 => TdmsDataType::Boolean,
            0x44 => TdmsDataType::TimeStamp,
            0x4f => TdmsDataType::FixedPoint,
            0x08000c => TdmsDataType::ComplexSingleFloat,
            0x10000d => TdmsDataType::ComplexDoubleFloat,
            0xFFFFFFFF => TdmsDataType::DAQmxRawData,
            _ => {
                error!("Unknown Property data type");
                unreachable!()
            }
        }
    }
}
