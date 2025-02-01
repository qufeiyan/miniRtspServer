#![feature(write_all_vectored)]
use std::{
    io::{
        BufRead, BufReader, Write
    }, net::{
        IpAddr, TcpListener, TcpStream, UdpSocket
    }, sync::{Arc, Mutex}, thread, time::Duration};
use rtsp::{connection::Connection, request::RtspRequest};
use media::session::{Session, Track};
use rtsp::router::Router;
struct Server<'a> {
    socket_addr: &'a str,
    vedio_file: Arc<String>,
}
impl<'a> Server<'a> {
    fn new(socket_addr: &'a str, stream_file: Arc<String>) -> Self{
        Server{
            socket_addr,
            vedio_file: stream_file,
        }
    }
    fn handle_client(mut connection: Connection) {
        // 读取客户端请求
        loop {
            let buf_reader: BufReader<&mut TcpStream> = BufReader::new(&mut connection.stream);
            let rtsp_request: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();
            if rtsp_request.is_empty() {
                // log::warn!("receive nothing, close the client {} right now.", stream.peer_addr().unwrap());
                // break;
                continue;
            }
            let req: RtspRequest = rtsp_request.join("\r\n").into();
            log::info!("req: {:#?}", rtsp_request);
            Router::route(req, &mut connection);
        }
    }
    
    fn run(&self) {
        let listener = TcpListener::bind(self.socket_addr).unwrap();
        for stream in listener.incoming(){
            match stream {
                Ok(stream) => {
                    // let audio_file = Some("media/audio.aac");
                    let (tx, rx) = std::sync::mpsc::channel();
                    let video_file = Arc::clone(&self.vedio_file);
                    let session = Arc::new(Mutex::new(Session::new("session", Some(video_file), None, tx)));
                    // 为每个连接创建一个新的线程
                    let session_clone = Arc::clone(&session);
                    let mut stream_clone = stream.try_clone().unwrap();
                    let handle_connect = thread::spawn( move || {
                        let connection = Connection::new(stream, session_clone);
                        Server::handle_client(connection);
                    });
                    
                    let session_clone = Arc::clone(&session);
                    let handle_rtp = thread::spawn( move || {
                        let is_play: Option<bool> = rx.recv().ok();
                        if let Some(is_play) = is_play {
                            if !is_play {
                                log::info!("stop play");
                                return;
                            }
                            log::info!("start play");
                        }
                        
                        let mut session = session_clone.lock().unwrap();
                        let rtp_sink = Arc::clone(session.get_rtp_sink(Track::Video));
                        
                        let nalu_iter = rtp_sink.lock().unwrap().get_nalu_iter();
                        for nalu in nalu_iter {
                            if let Some(play) = rx.recv_timeout(Duration::from_millis(40)).ok() {
                                if !play {
                                    break;
                                }
                            }
                            log::debug!("send nalu: {:?}", nalu); 
                            rtp_sink.lock().unwrap().handle(&nalu, &mut stream_clone);
                        }
                    });

                    handle_connect.join().unwrap();
                    handle_rtp.join().unwrap();
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    }
}
fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    let mut builder = env_logger::Builder::from_env(env);
    builder.format(|buf, record| {
        // use std::io::Write;
        use chrono::Local;
        use env_logger::fmt::style::{Style, AnsiColor};
        let subtle = Style::new()
            .fg_color(Some(AnsiColor::BrightBlack.into()));
        let level_style = buf.default_level_style(record.level());
        writeln!(
            buf,
            "{subtle}[{subtle:#}{} {level_style}{:<5}{level_style:#} {}:{}:{}{subtle}]{subtle:#} {}",
            Local::now().format("%Y-%m-%d %H:%M:%S%Z"),
            record.level(),
            record.module_path().unwrap_or("<unnamed>"),
            record.file().unwrap_or("<unnamed>"),
            record.line().unwrap_or(0),
            record.args()
        )
    })
    .init();
    let get_local_ip = || -> Option<IpAddr> {
        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        let local_addr = socket.local_addr().ok()?;
        Some(local_addr.ip())
    };
    match get_local_ip() {
        Some(ip) => {
            let ip_with_port = format!("{}:5544", ip);
            let home_dir = std::env::var("HOME").expect("HOME not set");
            let relative_path = "coder/rust/miniRtspServer/test.h264";
            let stream_file = format!("{}/{}", home_dir, relative_path);
            let video_file = Arc::new(stream_file);
            let server = Server::new(&ip_with_port, video_file);
            log::info!("Listening on {}", ip_with_port);
            server.run();
        }
        None => {
            log::error!("get local ip failed");
        }
    }
    log::info!("hi, server shutdown.");
}