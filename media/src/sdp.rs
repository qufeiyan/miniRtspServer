use std::collections::HashMap;
use base64::{engine::general_purpose, Engine};

#[derive(Debug, Clone, Default)]
pub struct Bandwidth{
    modifier: String, // 'CT' or 'AS'
    value: u16, 
}

impl<'a> From<Bandwidth> for String {
    fn from(bandwidth: Bandwidth) -> String {
        format!("{}:{}\r\n", bandwidth.modifier, bandwidth.value)
    }
}

/*
     The general form of an rtpmap attribute is:

     a=rtpmap:<payload type> <encoding name>/<clock rate>[/<encoding
     parameters>]

     For audio streams, <encoding parameters> may specify the number of
     audio channels.  This parameter may be omitted if the number of
     channels is one provided no additional parameters are needed.  For
     video streams, no encoding parameters are currently specified.
    
    example: 
    a=rtpmap:96 H264/90000\r\n\
    a=rtpmap:97 MPEG4-GENERIC/48000/2\r\n\
*/
#[derive(Debug, Clone, Default)]
pub struct RtpMap {
    pub payload_type: u16,
    pub encoding_name: String,
    pub clock_rate: u32,
    pub encoding_param: String,
}

impl<'a> From<RtpMap> for String {
    fn from(rtp_map: RtpMap) -> Self {
        let mut res = format!(
            "{} {}/{}",
            rtp_map.payload_type,
            rtp_map.encoding_name,
            rtp_map.clock_rate
        );
        
        if rtp_map.encoding_param != "" {
            res = format!("{}/{}", res, rtp_map.encoding_param);
        }
        
        format!("{res}\r\n")
    }
}

#[derive(Debug, Clone, Default)]
pub struct H264Fmtp {
    pub payload_type: u16,
    pub packetization_mode: u8,
    pub profile_level_id: Vec<u8>,
    pub sps: Vec<u8>,
    pub pps: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct H265Fmtp {
    pub payload_type: u16,
    pub vps: Vec<u8>,
    pub sps: Vec<u8>,
    pub pps: Vec<u8>,
}

impl<'a> From<H264Fmtp> for String {
    fn from(fmtp: H264Fmtp) -> Self {
        let profile_level_id_str = fmtp.profile_level_id
            .iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join("");
        println!("{}", profile_level_id_str);

        let sps_str = general_purpose::STANDARD.encode(&fmtp.sps);
        let pps_str = general_purpose::STANDARD.encode(&fmtp.pps);

        let h264_fmtp = format!(
            "{} packetization-mode={}; profile-level-id={}; sprop-parameter-sets={},{}",
            fmtp.payload_type, fmtp.packetization_mode, profile_level_id_str, sps_str, pps_str
        );
        
        format!("{}\r\n", h264_fmtp)
    }
}

impl<'a> From<H265Fmtp> for String {
    fn from(fmtp: H265Fmtp) -> Self {
        let sps_str = general_purpose::STANDARD.encode(&fmtp.sps);
        let pps_str = general_purpose::STANDARD.encode(&fmtp.pps);
        let vps_str = general_purpose::STANDARD.encode(&fmtp.vps);

        let h265_fmtp = format!(
            "{} sprop-vps={}; sprop-sps={}; sprop-pps={}",
            fmtp.payload_type, vps_str, sps_str, pps_str
        );

        format!("{}\r\n", h265_fmtp)
    }
}

#[derive(Debug, Clone)]
pub enum Fmtp {
    H264(H264Fmtp),
    H265(H265Fmtp),
    // Mpeg4(Mpeg4Fmtp),
}

#[derive(Debug, Clone, Default)]
pub struct MediaInfo {
    pub media_type: String,
    pub port: usize,
    pub protocol: String,
    pub fmts: Vec<u8>,
    pub bandwidth: Option<Bandwidth>,
    pub rtpmap: RtpMap,
    pub fmtp: Option<Fmtp>,
    pub attribute: HashMap<String, String>,  // other attribute.
}

impl<'a> From<MediaInfo> for String {
    fn from(media_info: MediaInfo) -> Self {
        let fmts: String = media_info
            .fmts
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        let bandwidth: String = if let Some(bandwidth) = media_info.bandwidth {
            format!("b={}", String::from(bandwidth))
        }else {
            String::from("")
        }; 

        let mut res = format!(
            "m={} {} {} {}\r\n{}a=rtpmap:{}",
            media_info.media_type,
            media_info.port,
            media_info.protocol,
            fmts,
            bandwidth,
            String::from(media_info.rtpmap)
        );

        if let Some(fmtp) = media_info.fmtp {
            let fmtp_string: String = match fmtp {
                Fmtp::H264(h264_fmtp) => h264_fmtp.into(),
                Fmtp::H265(h265_fmtp) => h265_fmtp.into(),
            };
            res = format!("{}a=fmtp:{}", res, fmtp_string);
        }

        for (k, v) in media_info.attribute {
            res = format!("{res}a={k}:{v}\r\n");
        }
        
        res
    }
}

#[derive(Debug, Clone, Default)]
pub struct SDP {
    version: u16,
    
    // o=<username> <session id> <version> <network type>
    // <address type> <address>*/
    origin: String,
    session_name: String,
    connection: String,
    time: String,
    medias: Vec<MediaInfo>,
    attribute: HashMap<String, String>,  
}

impl<'a> From<SDP> for String {
    fn from(sdp: SDP) -> String {
        let mut res: String = format!(
            "v={}\r\no={}\r\ns={}\r\nc={}\r\nt={}\r\n",
            sdp.version, sdp.origin, sdp.session_name, sdp.connection, sdp.time
        );

        for(k, v) in &sdp.attribute {
            res = format!("{res}a={k}:{v}\r\n");
        }    

        for media_info in sdp.medias {
            res = format!("{}{}", res, String::from(media_info));
        }

        res
    }
}

impl SDP {
    pub fn new(session_id: String, ip: String, media_info: Vec<MediaInfo>, attribute: Option<HashMap<String, String>>) -> Self {
        let mut sdp = SDP::default();
        sdp.version = 0;
        let owner = "-";
        sdp.origin = format!("{owner} {} {} IN IP4 {}", session_id, 0, ip);
        sdp.session_name = "seminar".to_string();
        sdp.connection = format!("IN IP4 {}", ip);
        
        // unlimited time.
        sdp.time = format!("{} {}", 0, 0);

        // media_info
        sdp.medias = media_info;
        sdp.attribute = attribute.unwrap_or_default();         

        sdp
    }
}

#[cfg(test)]
mod tests{
    use std::collections::HashMap;
    use crate::session::H264Fmtp;
    use crate::session::MediaInfo;
    use super::SDP;
    use super::RtpMap;
    use super::Fmtp;

    #[test]
    fn test_generate_sdp() {
        let mut rtpmap = RtpMap::default();
        rtpmap.payload_type = 96;
        rtpmap.encoding_name = "H264".to_string(); 
        rtpmap.clock_rate = 90000;
        rtpmap.encoding_param = String::from("");

        let mut fmtp: H264Fmtp = H264Fmtp::default();
        fmtp.payload_type = 96;
        fmtp.packetization_mode = 1;
        fmtp.profile_level_id = vec![0x10, 0x42, 0x30];

        let mut media_info = MediaInfo::default();
        media_info.media_type = "video".to_string();
        media_info.port = 2007;
        media_info.protocol = "RTP/AVP".to_string(); 
        media_info.fmts.insert(0, 96);
        media_info.bandwidth = None;
        media_info.rtpmap = rtpmap;
        media_info.fmtp = Some(Fmtp::H264(fmtp));
        media_info.attribute = HashMap::new();
        media_info.attribute.insert("control".to_string(), "track0".to_string());

        let sdp = SDP::new("123455556".to_string(), "10.15.112.58".to_string(), vec![media_info], None);
        // println!("sdp: {}", String::from(sdp));

        let target = "v=0\r\no=rust-rtsp-server 123455556 0 IN IP4 10.15.112.58\r\ns=seminar\r\nc=IN IP4 10.15.112.58\r\nt=0 0\r\nm=video 2007 RTP/AVP 96\r\na=rtpmap:96 H264/90000\r\na=fmtp:96 packetization-mode=1;profile-level-id=104230\r\n";
        assert_eq!(String::from(sdp), target);
    }
}

