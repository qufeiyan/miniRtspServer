use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Method {
    Options,
    Describe,
    Announce,
    Setup,
    Play,
    Pause,
    Teardown,
    Uninitialized,
}

impl From<&Method> for String {
    fn from(method: &Method) -> Self {
        match method {
            Method::Options => "OPTIONS".to_string(),
            Method::Describe => "DESCRIBE".to_string(),
            Method::Announce => "ANNOUNCE".to_string(),
            Method::Setup => "SETUP".to_string(),
            Method::Play => "PLAY".to_string(),
            Method::Pause => "PAUSE".to_string(),
            Method::Teardown => "TEARDOWN".to_string(),
            Method::Uninitialized => "Uninitialized".to_string(),
        }
    }   
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "OPTIONS" => Method::Options,
            "DESCRIBE" => Method::Describe,
            "ANNOUNCE" => Method::Announce,
            "SETUP" => Method::Setup,
            "PLAY" => Method::Play,
            "PAUSE" => Method::Pause,
            "TEARDOWN" => Method::Teardown,
            _ => Method::Uninitialized,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Version {
    V1_0,
    V2_0,
    Uninitialized,
} 

impl From<&str> for Version {
    fn from(s: &str) -> Version {
        match s {
            "RTSP/1.0" => Version::V1_0,
            _ => Version::Uninitialized,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Url {
    Path(String),
}

#[derive(Debug)]
pub struct RtspRequest{
    pub method: Method,
    pub version: Version,
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub msg_body: String, 
}

impl From<String> for RtspRequest {
    fn from(req: String) -> Self {
        let mut parsed_method = Method::Uninitialized;
        let mut parsed_version = Version::V1_0;
        let mut parsed_url = Url::Path("".to_string());
        let mut parsed_headers = HashMap::new();
        let mut parsed_msg_body = "";

        for line in req.lines() {
            if line.contains("RTSP") {
                let (method, resource, version) = process_req_line(line);
                parsed_method = method;
                parsed_url = resource;
                parsed_version = version;
            } else if line.contains(":") {
                let (key, value) = process_header_line(line);
                parsed_headers.insert(key, value);
            } else {
                parsed_msg_body = line;
            }
        }

        RtspRequest {
            method: parsed_method,
            version: parsed_version,
            url: parsed_url,
            headers: parsed_headers,
            msg_body: parsed_msg_body.to_string(),
        }
    }
}

fn process_req_line(s: &str) -> (Method, Url, Version) {
    let mut words = s.split_whitespace();
    let methods = words.next().unwrap();
    let resource = words.next().unwrap();
    let version = words.next().unwrap();
    (
        methods.into(),
        Url::Path(resource.to_string()),
        version.into(),
    )
}

fn process_header_line(s: &str) -> (String, String) {
    let mut header_items = s.split(": ");
    let mut key = String::from("");
    let mut value = String::from("");
    if let Some(k) = header_items.next() {
        key = k.trim().to_string();
    }
    if let Some(v) = header_items.next() {
        value = v.trim().to_string();
    }
    (key, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_into() {
        let m: Method = "OPTIONS".into();
        assert_eq!(m, Method::Options);
    }
    #[test]
    fn test_version_into() {
        let v: Version = "RTSP/1.0".into();
        assert_eq!(v, Version::V1_0);
    }

    #[test]
    fn test_read_rtsp() {
        let s =
            String::from("OPTIONS rtsp://10.15.112.58:5544 RTSP/1.0\r\nCSeq: 1\r\nUser-Agent: RealMedia Player Version 6.0.9.1235 (linux-2.0-libc6-i386-gcc2.95)\r\n\r\n");
        let mut expected_header: HashMap<String, String> = HashMap::new();
        expected_header.insert("CSeq".into(), "1".into());
        expected_header.insert("User-Agent".into(), "RealMedia Player Version 6.0.9.1235 (linux-2.0-libc6-i386-gcc2.95)".into());

        println!("{:?}", expected_header);
        let req: RtspRequest = s.into();
        assert_eq!(Method::Options, req.method);
        assert_eq!(Version::V1_0, req.version);
        assert_eq!(Url::Path("rtsp://10.15.112.58:5544".to_string()), req.url);
        assert_eq!(expected_header, req.headers);
    }
}