use std::fs::File;
use std::io::IoSlice;
use std::io::Write;
use std::sync::Arc;
use crate::codec::parse::NaluIterator;
use super::rtp_packet::RtpPacket;
use super::rtp_packet::{RtpSink, RTP_MAX_PACKET_SIZE};

const NALU_HEADER_SIZE: usize = 2; // 2 bytes for NALU header
pub struct RtpSinkH265 {
    payload_type: u8,
    clock_rate: u32,
    fps: u32,
    packet: RtpPacket,
    ssrc: u32,
    filename: Arc<String>,
    infinite: bool,
}

impl RtpSinkH265 {
    pub fn new(filename: Arc<String>, payload_type: u8, clock_rate: u32, fps: u32, ssrc: u32, infinite: bool) -> Self {
        let sequence_number = 0;
        let timestamp: u32 = 0;
        let packet = RtpPacket::new(payload_type, sequence_number, timestamp, ssrc, false);
        RtpSinkH265 {
            payload_type,
            clock_rate,
            fps,
            packet,
            ssrc,
            filename,
            infinite,
        }
    }

}

//！ FU-A  header for HEVC NAL units
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//| FU indicator                  |   FU header   |               | 
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 0                   1                   2                   3
// 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//|F|   Type    |  LayerId  | TID |S|E|    Type   |               |
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  F: 1 bit，表示FU-A的起始NAL单元的禁止位。默认为0。
//  Type: 6 bit，表示FU-A的单元的类型。
//  LayerId: 6 bit，表示NAL单元的层ID。
//  TID: 3 bit，表示NAL单元的Temporal ID。
//  S: 1 bit，表示NAL单元的开始位。如果为1，表示这是NAL单元的第一个分包。
//  E: 1 bit，表示NAL单元的结束位。如果为1，表示这是NAL单元的最后一个分包。
//  Type: 6 bit，表示FU-A的NAL单元的类型。
impl RtpSink for RtpSinkH265 {
    fn handle(&mut self, nalu: &[u8], mut write_stream: Box<dyn Write>) {
        //! 实现 H265 的 发送nalu rtp 包 
        let rtp_packet: &mut RtpPacket = &mut self.packet;

        let nalu_type = (nalu[0] & 0x7e) >> 1;
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

            // STAP-A 单一时间的组合包, sps pps vps 一起发送
            if let 32 | 33 | 34 = nalu_type {
                return;
            }
        } else {
            let pkt_num = nalu.len() / RTP_MAX_PACKET_SIZE;
            let remain_pkt_size = nalu.len() % RTP_MAX_PACKET_SIZE;
            let mut pos = NALU_HEADER_SIZE; // 2 bytes for NALU header

            for i in 0..pkt_num {
                rtp_packet.payload.insert(0, (nalu[0] & 0x81) | (49 >> 1));
                rtp_packet.payload.insert(1, nalu[1]);
                rtp_packet.payload.insert(2, nalu_type);

                if i == 0 {
                    rtp_packet.payload[2] |= 0x80; // start
                } else if remain_pkt_size <= NALU_HEADER_SIZE && i == pkt_num - 1 {
                    rtp_packet.payload[2] |= 0x40; // end
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
                rtp_packet.payload.insert(0, (nalu[0] & 0x81) | (49 >> 1));
                rtp_packet.payload.insert(1, nalu[1]);
                rtp_packet.payload.insert(2, nalu_type);
                rtp_packet.payload[2] |= 0x40; // end
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
        let file = File::open(self.filename.as_ref()).unwrap();
        NaluIterator::new(file, self.infinite)
    }
}