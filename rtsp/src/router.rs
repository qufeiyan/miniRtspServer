use crate::{connection::Connection, handler::TeardownHandler, request::{self, RtspRequest}, response::RtspResponse};
use crate::handler::{DescribeHandler, Handler, OptionsHandler, PlayHandler, SetupHandler};
pub struct Router;

impl Router {
    fn get_handler<'a>(method: &request::Method, connect: &'a Connection) -> Box<dyn Handler + 'a> {
        match method {
            request::Method::Options => Box::new(OptionsHandler {}),
            request::Method::Describe => {
                let session = connect.get_session();
                let sdp = session.lock().unwrap().generare_sdp();
                Box::new(DescribeHandler::new(sdp))
            },
            request::Method::Setup => {
                Box::new(SetupHandler::new(connect.get_session()))
            },
            request::Method::Play => {
                let session = connect.get_session(); 
                Box::new(PlayHandler::new(session))
            },
            request::Method::Teardown => { 
                let session = connect.get_session(); 
                Box::new(TeardownHandler::new(session))
            },
            request::Method::Announce => todo!(),
            request::Method::Pause => todo!(),
            request::Method::Uninitialized => todo!(),
        }
    }
    pub fn route(req: RtspRequest, connect: &Connection ) -> () {
        let mut handler = Router::get_handler(&req.method, connect);
        
        let mut stream = connect.get_stream(); 
        let resp: RtspResponse = handler.handle(&req);
        log::warn!("resp: {:#?}", resp);
        let _ = resp.send_response(&mut stream);
    }
}