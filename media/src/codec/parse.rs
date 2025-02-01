use std::{fs::File, io::{Read, Seek, SeekFrom}};
pub fn find_nalu(data: &[u8]) -> Option<&[u8]> {
    let mut start = 0;
    while start < data.len() {
        if start + 3 < data.len()
            && (&data[start..start + 3] == b"\x00\x00\x01"
                || &data[start..start + 4] == b"\x00\x00\x00\x01")
        {
            let nalu_start = if &data[start..start + 3] == b"\x00\x00\x01" {
                start + 3
            } else {
                start + 4
            };
            let mut end = nalu_start;
            while end < data.len() - 3 {
                if &data[end..end + 3] == b"\x00\x00\x01"
                    || &data[end..end + 4] == b"\x00\x00\x00\x01"
                {
                    break;
                }
                end += 1;
            }
            return Some(&data[nalu_start..end]);
        } else {
            start += 1;
        }
    }
    None
}

pub struct NaluIterator {
    file: File,
    buffer: Vec<u8>, 
    infinite: bool, // Add a boolean field to indicate if the iterator should be infinite
}

impl NaluIterator {
    pub fn new(file: File, infinite: bool) -> Self {
        let res = NaluIterator {
            file,
            buffer: vec![0; CHUNK_SIZE],
            infinite,
        };
        log::info!("NaluIterator created");
        res
    }
}

impl Iterator for NaluIterator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let bytes_read = self.file.read(&mut self.buffer).ok()?;
            if bytes_read == 0 {
                // log::info!("End of file");
                if self.infinite {
                    self.file.seek(SeekFrom::Start(0)).ok()?;
                    continue;
                }
                return None;
            }
            
            let data = &self.buffer[..bytes_read];
            if let Some(nalu) = find_nalu(data) { 
                log::info!("Found NALU of length: {}", nalu.len());
                log::debug!("nalu: {:#?}", nalu.iter().map(|x| format!("{:x}", x)).collect::<Vec<String>>());
                self.file.seek(SeekFrom::Current((nalu.len() as i64) - (bytes_read as i64))).ok()?;

                return Some(nalu.to_vec());
            }
            
            log::warn!("No NALU found in this chunk, continue to read next chunk with size {}", CHUNK_SIZE);
    
            if self.infinite {
                self.file.seek(SeekFrom::Start(0)).ok()?;
            }else {
                return None;
            }
        }
    }
}


pub enum ParameterSet{
    H264{
        sps: Vec<u8>,
        pps: Vec<u8>,
    },
    H265{
        vps: Vec<u8>,
        sps: Vec<u8>,
        pps: Vec<u8>,
    },
    Other
}
const CHUNK_SIZE: usize = 500 * 1024; // 500KB
pub fn parse_h264(file: &mut File) -> ParameterSet {
    let mut buffer = vec![0; CHUNK_SIZE];
    let mut sps = vec![];
    let mut pps = vec![];
    let mut is_h264 = false;
    file.seek(SeekFrom::Start(0)).unwrap();
    loop {
        let bytes_read = file.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        let data = &buffer[..bytes_read];
        match find_nalu(data) {
            Some(nalu) => {
                println!("Found NALU of length: {}", nalu.len());
                // println!("nalu: {:#?}", nalu.iter().map(|x| format!("{:x}", x)).collect::<Vec<String>>());
                // Process the NALU data here, exaple: parse SPS/PPS for H264
                let nalu_type = nalu[0] & 0x1f;
                if nalu_type == 7 {
                    sps = nalu.to_vec();
                } else if nalu_type == 8 {
                    pps = nalu.to_vec();
                }
                file.seek(SeekFrom::Current((nalu.len() as i64) - (bytes_read as i64))).unwrap();
            }
            None => {
                println!("No NALU found in this chunk");
            }
        }
        if !sps.is_empty() && !pps.is_empty() {
            is_h264 = true;
            break
        }
    }
    match is_h264 {
        true => ParameterSet::H264{
            sps,
            pps,
        },
        false => ParameterSet::Other,
    }
}
pub fn parse_h265(file: &mut File) -> ParameterSet {
    let mut buffer = vec![0; CHUNK_SIZE];
    let mut vps: Vec<u8> = vec![];
    let mut sps = vec![];
    let mut pps = vec![];
    let mut is_h265 = false;
    file.seek(SeekFrom::Start(0)).unwrap();
    loop {
        let bytes_read = file.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        let data = &buffer[..bytes_read];
        match find_nalu(data) {
            Some(nalu) => {
                println!("Found NALU of length: {}", nalu.len());
                // println!("nalu: {:#?}", nalu.iter().map(|x| format!("{:x}", x)).collect::<Vec<String>>());
                // Process the NALU data here, exaple: parse VPS/SPS/PPS for H265
                let nalu_type = (nalu[0] & 0x7e) >> 1;
                match nalu_type {
                    32 => vps = nalu.to_vec(),
                    33 => sps = nalu.to_vec(),
                    34 => pps = nalu.to_vec(),
                    _ => (),
                };
                file.seek(SeekFrom::Current((nalu.len() as i64) - (bytes_read as i64))).unwrap();
            }
            None => {
                println!("No NALU found in this chunk");
            }
        }
        if !vps.is_empty() && !sps.is_empty() && !pps.is_empty() {
            is_h265 = true;
            break
        }
    }
    match is_h265 {
        true => ParameterSet::H265{
            vps,
            sps,
            pps,
        },
        false => ParameterSet::Other,
    }
}