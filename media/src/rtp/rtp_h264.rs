use std::fs::File;
use std::io::{IoSlice, Seek, SeekFrom, Write};
use std::sync::Arc;
use crate::codec::parse::NaluIterator;
use crate::rtp::rtp_packet::RtpPacket;

use super::rtp_packet::{RtpSink, RTP_MAX_PACKET_SIZE};

const NALU_HEADER_SIZE: usize = 1;

pub struct RtpSinkH264 {
    payload_type: u8,
    clock_rate: u32,
    fps: u32,
    packet: RtpPacket,
    ssrc: u32,
    filename: Arc<String>,
    infinite: bool,
}

impl RtpSinkH264 {
    pub fn new(filename: Arc<String>, payload_type: u8, clock_rate: u32, fps: u32, ssrc: u32, infinite: bool) -> Self {
        let sequence_number = 0;
        let timestamp: u32 = 0;
        let packet = RtpPacket::new(payload_type, sequence_number, timestamp, ssrc, false);
        RtpSinkH264 {
            payload_type,
            clock_rate,
            fps,
            packet,
            ssrc,
            filename,
            infinite,
        }
    }

    pub fn ssrc(&self) -> u32 {
        self.ssrc
    }

}
//！ FU-A  header for H264
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//| FU indicator  |   FU header   |                               |
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 0                   1                   2                   3
// 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//|F|NRI|  Type   |S|E|R|  Type   |                               |
impl RtpSink for RtpSinkH264 {
    fn handle(&mut self, nalu: &[u8], mut write_stream: Box<dyn Write>) {
        let rtp_packet: &mut RtpPacket = &mut self.packet;
        let nalu_type = nalu[0];

        let mut send_packet = |rtp_packet: &mut RtpPacket| {
            let len = rtp_packet.payload.len() + 12;
            let interleaved: &[u8] = &[0x24, 0, ((len >> 8) & 0xFF) as u8, (len & 0xFF) as u8];
            write_stream
                .write_all_vectored(&mut [
                    IoSlice::new(interleaved),
                    IoSlice::new(rtp_packet.to_bytes().as_slice()),
                ])
                .unwrap();
        };

        if nalu.len() <= RTP_MAX_PACKET_SIZE {
            rtp_packet.payload.extend_from_slice(&nalu[..nalu.len()]);
            rtp_packet.marker = true;

            send_packet(rtp_packet);

            rtp_packet.payload.clear();
            rtp_packet.sequence_number += 1;
            rtp_packet.marker = false;

            if (nalu_type & 0x1F) == 7 || (nalu_type & 0x1F) == 8 {
                return;
            }
        } else {
            let pkt_num = nalu.len() / RTP_MAX_PACKET_SIZE;
            let remain_pkt_size = nalu.len() % RTP_MAX_PACKET_SIZE;
            let mut pos = NALU_HEADER_SIZE; // skip nalu header

            for i in 0..pkt_num {
                // F/NRI bit 保持不变, Type 为 28
                rtp_packet.payload.insert(0, (nalu_type & 0xE0) | 28);
                rtp_packet.payload.insert(1, nalu_type & 0x1F);

                if i == 0 {
                    rtp_packet.payload[1] |= 0x80; // start
                } else if remain_pkt_size <= NALU_HEADER_SIZE && i == pkt_num - 1 {
                    rtp_packet.payload[1] |= 0x40; // end
                    rtp_packet.marker = true;
                }

                rtp_packet
                    .payload
                    .extend_from_slice(&nalu[pos..pos + RTP_MAX_PACKET_SIZE]);

                send_packet(rtp_packet);

                rtp_packet.payload.clear();
                rtp_packet.sequence_number += 1;
                pos += RTP_MAX_PACKET_SIZE;
            }

            if remain_pkt_size > NALU_HEADER_SIZE {
                rtp_packet.payload.insert(0, (nalu_type & 0xE0) | 28);
                rtp_packet.payload.insert(1, nalu_type & 0x1F);
                rtp_packet.payload[1] |= 0x40; // end
                rtp_packet.marker = true;

                log::info!(
                    "pos: {} remain: {} nalu.len: {}",
                    pos,
                    remain_pkt_size,
                    nalu.len()
                );
                rtp_packet
                    .payload
                    .extend_from_slice(&nalu[pos..pos + remain_pkt_size - NALU_HEADER_SIZE]);

                send_packet(rtp_packet);

                rtp_packet.payload.clear();
                rtp_packet.sequence_number += 1;
            }
            rtp_packet.marker = false;
        }
        rtp_packet.timestamp += self.clock_rate / self.fps;
    }

    fn get_nalu_iter(&self) -> NaluIterator {
        let mut file = File::open(self.filename.as_ref()).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        NaluIterator::new(file, self.infinite)
    }
}

impl IntoIterator for RtpSinkH264 {
    type Item = Vec<u8>;
    type IntoIter = NaluIterator;

    fn into_iter(self) -> Self::IntoIter {
        let file = File::open(self.filename.as_ref()).unwrap();
        NaluIterator::new(file, self.infinite)
    }
}

