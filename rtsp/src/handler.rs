use std::sync::{Arc, Mutex};

use crate::{request::RtspRequest, response::RtspResponse};
use linked_hash_map::LinkedHashMap;
use media::{sdp::SDP, session::Session};

const USER_AGENT: &str = "rust rtsp-server";

pub trait Handler {
    fn handle(&mut self, req: &RtspRequest) -> RtspResponse;
}

pub struct OptionsHandler;
pub struct DescribeHandler {
    sdp: SDP,
}

impl DescribeHandler {
    pub fn new(sdp: SDP) -> Self {
        Self { sdp }
    }
}

pub struct SetupHandler<'a> {
    session: Arc<Mutex<Session<'a>>>,
}

impl<'a> SetupHandler<'a> {
    pub fn new(session: Arc<Mutex<Session<'a>>>) -> Self {
        Self { session }
    }
}
pub struct PlayHandler<'a> {
    session: Arc<Mutex<Session<'a>>>,
}

impl<'a> PlayHandler<'a> {
    pub fn new(session: Arc<Mutex<Session<'a>>>) -> Self {
        Self { session }
    }
}

pub struct TeardownHandler<'a> {
    session: Arc<Mutex<Session<'a>>>,
}

impl<'a> TeardownHandler<'a> {
    pub fn new(session: Arc<Mutex<Session<'a>>>) -> Self {
        Self { session }
    }
}

impl Handler for OptionsHandler {
    fn handle(&mut self, req: &RtspRequest) -> RtspResponse {
        let mut headers: LinkedHashMap<&str, String> = LinkedHashMap::new();

        // log::info!("{:#?}", req.headers);

        let seq = req.headers.get("CSeq").unwrap();
        headers.insert("CSeq", seq.to_string());
        headers.insert("User-Agent", "rtsp-server".to_owned());
        headers.insert("Public", "OPTIONS, DESCRIBE, SETUP, TEARDOWN, PLAY".to_string());
        RtspResponse::new("200", Some(headers), None)
    }
}

impl Handler for DescribeHandler {
    fn handle(&mut self, req: &RtspRequest) -> RtspResponse {
        let mut headers: LinkedHashMap<&str, String> = LinkedHashMap::new();
        let seq = req.headers.get("CSeq").unwrap();
        headers.insert("CSeq", seq.to_string());
        headers.insert("User-Agent", "rust rtsp-server".to_string());
        headers.insert("Content-Type", "application/sdp".to_owned());
        
        let sdp_str: String = self.sdp.clone().into();
        // headers.insert("Content-Length", sdp.len().to_string());
        RtspResponse::new("200", Some(headers), Some(sdp_str))
    }
}

impl<'a> Handler for SetupHandler<'a> {
    fn handle(&mut self, req: &RtspRequest) -> RtspResponse {
        let session = self.session.lock().unwrap();
        let mut session_str = session.session_id.as_str().to_string();
        session_str += ";timeout=60";

        let mut headers: LinkedHashMap<&str, String> = LinkedHashMap::new();
        let seq = req.headers.get("CSeq").unwrap();
        headers.insert("CSeq", seq.to_owned());
        headers.insert("User-Agent", USER_AGENT.to_string());
        
        if req.headers.contains_key("session") {
            headers.insert("Session", req.headers.get("session").unwrap().to_owned() + ";timeout=60");
        }else {
            headers.insert("Session", session_str.to_string());
        }

        let mut transport = req.headers.get("Transport").unwrap().to_string();
        transport += ";ssrc=12345678";
        headers.insert("Transport", transport);

        //DATA 
        RtspResponse::new("200", Some(headers), None)
    }
}

impl<'a> Handler for PlayHandler<'a> {
    fn handle(&mut self, req: &RtspRequest) -> RtspResponse {
        let mut headers: LinkedHashMap<&str, String> = LinkedHashMap::new();
        
        let seq = req.headers.get("CSeq").unwrap();
        headers.insert("CSeq", seq.to_string());
        headers.insert("User-Agent", "rtsp-server".to_owned());
        let session = req.headers.get("Session").unwrap();
        // check session.. to do list.
        // 
        headers.insert("Session", session.to_string());
        headers.insert("RTP-Info", format!("url=rtsp://{}/track0;seq={};rtptime={}", "192.168.3.27:5544", 0, 0).to_string());
        headers.insert("Range", "npt=0.000-".to_string());
        headers.insert("Scale", "1.000".to_string());
        headers.insert("Cache-Control", "no-cache".to_string());

        let session = self.session.lock().unwrap();
        session.play(); 
        RtspResponse::new("200", Some(headers), None)
    }
}


impl<'a> Handler for TeardownHandler<'a> {
    fn handle(&mut self, req: &RtspRequest) -> RtspResponse {
        let mut headers: LinkedHashMap<&str, String> = LinkedHashMap::new();

        let seq = req.headers.get("CSeq").unwrap();
        headers.insert("CSeq", seq.to_string());
        headers.insert("User-Agent", "rtsp-server".to_owned());

        // check session.. to do list.
        // 
        let session = self.session.lock().unwrap();
        headers.insert("Session", session.session_id.as_str().to_string());

        session.teardown(); 
        RtspResponse::new("200", Some(headers), None)
    }
}