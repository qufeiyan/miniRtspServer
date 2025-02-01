#[derive(Debug, Clone)]
pub struct RtpPacket {
    version: u8,
    padding: bool,
    extension: bool,
    csrc_count: u8,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub payload: Vec<u8>,
}

impl RtpPacket {
    pub fn new(payload_type: u8, sequence_number: u16, timestamp: u32, ssrc: u32, marker: bool) -> Self {
        let version = 2;
        let padding = false;
        let extension = false;
        let csrc_count = 0;
        RtpPacket {
            version,
            padding,
            extension,
            csrc_count,
            marker,
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            payload: Vec::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let first_byte = (self.version << 6) | ((self.padding as u8) << 5) | ((self.extension as u8) << 4) | self.csrc_count;
        let second_byte = ((self.marker as u8) << 7) | self.payload_type;
        bytes.push(first_byte);

        bytes.push(second_byte);
        bytes.extend(&self.sequence_number.to_be_bytes());
        bytes.extend(&self.timestamp.to_be_bytes());
        bytes.extend(&self.ssrc.to_be_bytes());
        bytes.extend(&*self.payload);
        bytes
    }
}
