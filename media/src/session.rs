use std::{collections::HashMap, fs::File, sync::{mpsc::Sender, Arc, Mutex}};
use crate::{codec::parse::ParameterSet, rtp::{rtp_h264::RtpSinkH264, rtp_h265::RtpSinkH265, rtp_packet::RtpSink}, sdp::{Fmtp, H264Fmtp, H265Fmtp, MediaInfo, RtpMap, SDP}};
use crate::codec::parse;

const RTP_PAYLOAD_TYPE_H26X: u8 = 96; // 媒体类型-视频
const RTP_PAYLOAD_TYPE_AAC: u8 = 97;  // 媒体类型-音频
const RTP_PAYLOAD_TYPE_PCMA: u8 = 8;  // 媒体类型-音频

pub enum Track{
    Audio,
    Video,    
    Uninitialized,
}

impl From<&str> for Track {
    fn from(s: &str) -> Self {
        match s {
            "audio" => Track::Audio,
            "video" => Track::Video,
            _ => Track::Uninitialized,
        }
    }
}

impl From<Track> for String {
    fn from(track: Track) -> Self {
        match track {
            Track::Audio => "audio".to_string(),
            Track::Video => "video".to_string(),
            Track::Uninitialized => "uninitialized".to_string(),
        }
    }
}

pub struct Session<'a> {
    session_name: &'a str,  // used for identifying the session.
    pub session_id: String,     // used for setupting the session, not session id in sdp.
    pub rtp_sinks: HashMap<String, Arc<Mutex<Box<dyn RtpSink>>>>, // used for sending the rtp packets. key is the track id.
    fmtps: HashMap<String, Fmtp>, // used for setting the fmtp.
    rtpmaps: HashMap<String, RtpMap>, // used for setting the rtpmap.
    video_file: Option<Arc<String>>, // used for storing the video file.
    audio_file: Option<Arc<String>>, // used for storing the audio file.
    tx_play: Sender<bool>, // TODO: use async channel
}

impl<'a> Session<'a> {
    pub fn new(session_name: &'a str, video_file: Option<Arc<String>>, audio_file: Option<Arc<String>>, tx_play: Sender<bool>) -> Self {
        const CHARSET: &[u8] = b"0123456789";
        const SESSION_LEN: usize = 10;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let session_id: String = (0..SESSION_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        let mut session = Session {
            session_name,
            session_id,
            rtp_sinks: HashMap::new(),
            fmtps: HashMap::new(),
            rtpmaps: HashMap::new(),
            video_file,
            audio_file,
            tx_play,
        };

        session.add_fmtp(Track::Video);
        
        // let file = "./test.h264";
        let file = session.video_file.as_ref().unwrap().clone();
        let clock_rate = 90000;
        let fps = 25;
        let ssrc = 12345678;
        match session.fmtps.get(&String::from(Track::Video)).unwrap() {
            Fmtp::H264(fmtp) => {
                let rtp_sink: Box<dyn RtpSink> = Box::new(RtpSinkH264::new(file, fmtp.payload_type as u8, clock_rate, fps, ssrc, true));
                let rtp_sink_arc = Arc::new(Mutex::new(rtp_sink));
                session.add_rtp_sink(Track::Video, Arc::clone(&rtp_sink_arc));
            }
            Fmtp::H265(fmtp) => {
                let rtp_sink: Box<dyn RtpSink> = Box::new(RtpSinkH265::new(file, fmtp.payload_type as u8, clock_rate, fps, ssrc, true));
                let rtp_sink_arc = Arc::new(Mutex::new(rtp_sink));
                session.add_rtp_sink(Track::Video, Arc::clone(&rtp_sink_arc));
            }
        };

        session
    }

    fn add_rtp_sink(&mut self, track: Track, rtp_sink: Arc<Mutex<Box<dyn RtpSink>>>) {
        self.rtp_sinks.insert(String::from(track), rtp_sink);
    }

    pub fn get_rtp_sink(&mut self, track: Track) -> &mut Arc<Mutex<Box<dyn RtpSink>>> {
        let rtpsink = self.rtp_sinks.get_mut(&String::from(track)).unwrap();
        rtpsink
    }
    
    fn add_rtpmap(&mut self, track: Track) {
        todo!()
    }

    fn add_fmtp(&mut self, track: Track) {
        match track {
            Track::Video => {
                let video_file = self.video_file.clone().unwrap();
                let mut file = File::open(video_file.as_ref()).unwrap();
                if let ParameterSet::H264 { sps, pps } = parse::parse_h264(&mut file) {
                    let mut h264_fmtp = H264Fmtp::default();
                    h264_fmtp.payload_type = 96;
                    h264_fmtp.packetization_mode = 1;
                    h264_fmtp.profile_level_id = sps[1..4].to_vec();
                    h264_fmtp.sps = sps;
                    h264_fmtp.pps = pps;
                    self.fmtps.insert(String::from(track), Fmtp::H264(h264_fmtp));
                }else if let ParameterSet::H265 { vps, sps, pps } = parse::parse_h265(&mut file) {
                    let mut h265_fmtp = H265Fmtp::default();
                    h265_fmtp.payload_type = 96;
                    h265_fmtp.vps = vps;
                    h265_fmtp.sps = sps;
                    h265_fmtp.pps = pps;
                    self.fmtps.insert(String::from(track), Fmtp::H265(h265_fmtp));
                    
                }
            },
            Track::Audio => {
                todo!()
            },
            Track::Uninitialized => todo!(),
        }
    }

    pub fn generare_sdp(&self) -> SDP {
        let encoding_name = match self.fmtps.get("video").unwrap() {
            Fmtp::H264(_) => "H264",
            Fmtp::H265(_) => "H265",
        };
        let mut rtpmap = RtpMap::default();
        rtpmap.payload_type = RTP_PAYLOAD_TYPE_H26X as u16;
        rtpmap.encoding_name = encoding_name.to_string(); 
        rtpmap.clock_rate = 90000;
        rtpmap.encoding_param = String::from("");

        let mut video_media_info = MediaInfo::default();
        if self.rtp_sinks.contains_key("video") {
            video_media_info.media_type = "video".to_string();
            video_media_info.port = 4;
            video_media_info.protocol = "RTP/AVP/TCP".to_string(); 
            video_media_info.fmts.insert(0, 96);
            video_media_info.bandwidth = None;
            video_media_info.rtpmap = rtpmap;
            video_media_info.fmtp = Some(self.fmtps.get("video").unwrap().clone());   
            video_media_info.attribute = HashMap::new();
            video_media_info.attribute.insert("control".to_string(), "track1".to_string());
        }
        
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let session_id = since_the_epoch.as_secs();
        let session_id_str = session_id.to_string();
        let mut attribute = HashMap::new();
        attribute.insert("control".to_string(), "*".to_string());

        let address = "0.0.0.0";
        SDP::new(session_id_str, address.to_string(), vec![video_media_info], Some(attribute))
    }

    pub fn play(&self) {
        self.tx_play.send(true).unwrap();
    }

    pub fn teardown(&self) {
        self.tx_play.send(false).unwrap();
    }

}