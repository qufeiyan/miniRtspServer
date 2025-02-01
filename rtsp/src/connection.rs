use std::{net::TcpStream, sync::{Arc, Mutex}};

use media::session::Session;

pub struct Connection<'a> {
    pub stream: TcpStream, // TODO: use async tcp stream
    pub session: Arc<Mutex<Session<'a>>>,
}

impl<'a> Connection<'a> {
    pub fn new(stream: TcpStream, session: Arc<Mutex<Session<'a>>>) -> Self {
        Connection {
            stream,
            session,
        }
    }

    pub fn get_stream(&self) -> &TcpStream {
        &self.stream
    }
    
    pub fn get_local_ip(&self) -> String {
        self.stream.local_addr().unwrap().ip().to_string()
    }

    pub fn get_remote_ip(&self) -> String {
        self.stream.peer_addr().unwrap().ip().to_string()
    }

    pub fn get_local_port(&self) -> u16 {
        self.stream.local_addr().unwrap().port()
    }

    pub fn get_remote_port(&self) -> u16 {
        self.stream.peer_addr().unwrap().port()
    }

    pub fn get_session(&self) -> Arc<Mutex<Session<'a>>> {
        Arc::clone(&self.session)
    }
}