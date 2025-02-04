use linked_hash_map::LinkedHashMap;
use crate::{auth::{self, AuthHeaderGenerator, AuthType}, connection::Connection, handler::TeardownHandler, request::{self, RtspRequest}, response::RtspResponse};
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

    fn unauthorized(req: &RtspRequest, mut auth: impl AuthHeaderGenerator) -> RtspResponse {
        let mut headers: LinkedHashMap<&str, String> = LinkedHashMap::new();
        let seq = req.headers.get("CSeq").unwrap();
        headers.insert("CSeq", seq.to_string());
        headers.insert("WWW-Authenticate", auth.generate());
        RtspResponse::new("401", Some(headers), None)
    }

    pub fn route(req: RtspRequest, connect: &Connection ) -> () {
        let mut handler = Router::get_handler(&req.method, connect);
        let username = "admin";
        let password = "123456";

        if let Some(auth_info) = req.headers.get("Authorization") {
            log::debug!("auth_info: {:#?}", auth_info);
            match auth::get_auth_type(auth_info) {
                Some(auth::AuthType::Basic(client_auth)) => {
                    let auth = &client_auth.unwrap();
                    let res = username == auth.0 && password == auth.1;
                    if !res {
                        log::error!("auth failed");
                        return ;
                    }
                },
                Some(auth::AuthType::Digest(client_auth)) => {
                    let auth = &client_auth.unwrap();
                    log::debug!("auth: {:#?}", auth);
                    let realm = "rust rtsp-server";
                    let method = String::from(&req.method);
                    let res = auth::validate_digest_response(auth, &method, realm, password);
                    if !res {
                        log::error!("auth failed");
                        return ;
                    }
                }
                None => todo!(),
            }
        } else if req.method != request::Method::Options {
            // let auth = auth::BasicAuthenticator::new("rust rtsp-server");
            let auth = auth::DigestAuthenticator::new("rust rtsp-server", None);
            let resp = Router::unauthorized(&req, auth);
            let mut stream = connect.get_stream();
            let _ = resp.send_response(&mut stream);
            return ;
        }

        let mut stream = connect.get_stream(); 
        let resp: RtspResponse = handler.handle(&req);
        log::debug!("resp: {:#?}", resp);
        let _ = resp.send_response(&mut stream);
    }
}