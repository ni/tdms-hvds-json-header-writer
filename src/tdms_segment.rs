use crate::tdms_lead_in;
use crate::tdms_metadata;

#[derive(Debug)]
pub struct TdmsSegment {
    pub tdms_lead_in: TdmsLeadIn,
    pub tdms_metadata: TdmsMetadata,
    pub data_size: u64,
    pub absolute_pos: u64,
}
