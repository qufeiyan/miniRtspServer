use std::io::{Result, Write};
use linked_hash_map::LinkedHashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct RtspResponse<'a> {
    version: &'a str,
    status_code: &'a str,
    status_text: &'a str,
    headers: Option<LinkedHashMap<&'a str, String>>,
    body: Option<String>,   
}

impl<'a> Default for RtspResponse<'a> {
    fn default() -> Self{
        Self {
            version: "RTSP/1.0".into(),
            status_code: "200".into(),
            status_text: "OK".into(),
            headers: None,
            body: None,
        }
    }
}

impl<'a> From<RtspResponse<'a>> for String {
    fn from(res: RtspResponse) -> String {
        let res1 = res.clone();
        match res.body {
            Some(s) => {
                return format!(
                    "{} {} {}\r\n{}Content-Length: {}\r\n\r\n{}",
                    &res1.version(),
                    &res1.status_code(),
                    &res1.status_text(),
                    &res1.headers(),
                    &s.len(),
                    &res1.body()
                );
            }
            None => {
                format!(
                    "{} {} {}\r\n{}\r\n",
                    &res1.version(),
                    &res1.status_code(),
                    &res1.status_text(),
                    &res1.headers(),
                )
            },
        }
    }
}

impl<'a> RtspResponse<'a> {
    pub fn new(
        status_code: &'a str, 
        headers: Option<LinkedHashMap<&'a str, String>>, 
        body: Option<String>
    ) -> RtspResponse<'a> {
        let mut response: RtspResponse<'a> = RtspResponse::default();
        if status_code != "200" {
            response.status_code = status_code.into();
        }
        response.headers = match &headers {
            Some(_h) => headers,
            None => {
                None
            }
        };

        response.status_text = match response.status_code {
            "200" => "OK".into(),
            "400" => "Bad Request".into(),
            "404" => "Not Found".into(),
            "500" => "Internal Server Error".into(),
            _ => "Not Found".into(),
        };
        response.body = body;
        response
    }

    pub fn send_response(&self, write_stream: &mut impl Write) -> Result<()> {
        let res = self.clone();
        let response_string: String = res.into();
        // log::warn!("res: {:#?}", response_string);

        let _ = write!(write_stream, "{}", response_string);
        Ok(())
    }

    fn version(&self) -> &str {
        self.version
    }

    fn status_code(&self) -> &str {
        self.status_code
    }
    fn status_text(&self) -> &str {
        self.status_text
    }

    fn headers(&self) -> String {
        let map: LinkedHashMap<&str, String> = self.headers.clone().unwrap();
        let mut header_string: String = "".into();
        for (k, v) in map.iter() {
            header_string = format!("{}{}: {}\r\n", header_string, k, v);
        }
        header_string
    }
    pub fn body(&self) -> &str {
        match &self.body {
            Some(b) => b.as_str(),
            None => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn test_response_struct_creation_200() {
        let response_actual = RtspResponse::new("200", None, Some("xxxx".into()));
        let response_expected = RtspResponse {
            version: "RTSP/1.0",
            status_code: "200",
            status_text: "OK",
            headers: {
                None
            },
            body: Some("xxxx".into()),
        };
        assert_eq!(response_actual, response_expected);
    }

    #[test]
    fn test_response_struct_creation_404() {
    let mut headers = LinkedHashMap::new();
    headers.insert("Content-Type", "application/sdp".to_owned());
    let response_actual = RtspResponse::new("404", Some(headers), Some("xxxx".into()));
        let response_expected = RtspResponse {
            version: "RTSP/1.0",
            status_code: "404",
            status_text: "Not Found",
            headers: {
                let mut h = LinkedHashMap::new();
                h.insert("Content-Type", "application/sdp".to_owned());
                Some(h)
            },
            body: Some("xxxx".into()),
        };
        assert_eq!(response_actual, response_expected);
    }

    #[test]
    fn test_rtsp_response_creation() {
        let response_expected = RtspResponse {
            version: "RTSP/1.0",
            status_code: "200",
            status_text: "OK",
            headers: {
                let mut h = LinkedHashMap::new();
                h.insert("Content-Type", "application/sdp".to_owned());
                Some(h)
            },
            body: Some("xxxx".into()),
        };
        let rtsp_string: String = response_expected.into();
        let actual_string =
            "RTSP/1.0 200 OK\r\nContent-Type:application/sdp\r\nContent-Length: 4\r\n\r\nxxxx"
                .to_string();
        assert_eq!(rtsp_string, actual_string);
    }
}
