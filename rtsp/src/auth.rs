use base64::{engine::general_purpose, Engine};
use md5;
use std::collections::HashMap;

pub enum AuthType {
    Basic(Option<(String, String)>),
    Digest(Option<HashMap<String, String>>), // Digest 认证参数
}

fn parse_basic_auth(auth_header: &str) -> Option<(String, String)> {
    let encoded = &auth_header[6..];
    let decoded = general_purpose::STANDARD.decode(encoded).ok()?;
    let decoded_str = String::from_utf8(decoded).ok()?;
    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    Some((parts[0].to_string(), parts[1].to_string()))
}

/// 使用示例
/// let auth_header = "Digest username=\"admin\", nonce=\"abc123\", uri=\"rtsp://example.com/stream\", response=\"d41d8cd98f00b204e9800998ecf8427e\"";
/// let auth_params = parse_digest_header(auth_header);
fn parse_digest_header(auth_header: &str) -> Option<HashMap<String, String> >{
    let mut params = HashMap::new();
    let stripped = auth_header.trim_start_matches("Digest ");
    for pair in stripped.split(',') {
        let mut kv = pair.trim().splitn(2, '=');
        if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
            let value = value.trim_matches('\"').to_string();
            params.insert(key.trim().to_string(), value);
        }
    }
    Some(params)
}

/// 使用示例
/// let method = "DESCRIBE";
/// let password = "secret"; // 服务器存储的密码
/// let is_valid = validate_digest_response(&auth_params, method, realm, password);
pub fn validate_digest_response(
    auth_params: &HashMap<String, String>,
    method: &str,
    realm: &str,
    password: &str,
) -> bool {
    // 提取必要参数
    let username = auth_params.get("username").unwrap();
    let nonce = auth_params.get("nonce").unwrap();
    let uri = auth_params.get("uri").unwrap();
    let client_response = auth_params.get("response").unwrap();

    // 计算 HA1
    let ha1_input = format!("{}:{}:{}", username, realm, password);
    let ha1 = format!("{:x}", md5::compute(ha1_input.as_bytes()));

    // 计算 HA2
    let ha2_input = format!("{}:{}", method, uri);
    let ha2 = format!("{:x}", md5::compute(ha2_input.as_bytes()));

    // 计算服务器端 response
    let server_response_input = format!("{}:{}:{}", ha1, nonce, ha2);
    let server_response = &format!("{:x}", md5::compute(server_response_input.as_bytes()));

    // 比对客户端和服务器端的 response
    server_response == client_response
}


pub fn validate_digest_auth(
    username: &str,
    password: &str,
    nonce: &str,
    method: &str,
    uri: &str,
    response: &str,
) -> bool {
    let ha1 = format!("{:x}", md5::compute(format!("{}:{}:{}", username, "realm", password)));
    let ha2 = format!("{:x}", md5::compute(format!("{}:{}", method, uri)));
    let expected_response = format!("{:x}", md5::compute(format!("{}:{}:{}", ha1, nonce, ha2)));
    response == expected_response
}

pub fn get_auth_type(auth_header: &str) -> Option<AuthType> {
    if auth_header.starts_with("Basic ") {
        Some(AuthType::Basic(parse_basic_auth(auth_header)))
    } else if auth_header.starts_with("Digest ") {
        Some(AuthType::Digest(parse_digest_header(auth_header)))
    } else {
        None
    }
}

fn generate_nonce() -> String {
    // 生成随机 nonce（实际应用中应更安全）
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:x}", rng.gen::<u64>())
}

pub trait AuthHeaderGenerator {
    fn generate(&mut self) -> String;
}

pub struct BasicAuthenticator {
    realm: String,
}

impl BasicAuthenticator {
    pub fn new(realm: &str) -> Self {
        Self {
            realm: realm.to_string(),
        }
    }
}

impl AuthHeaderGenerator for BasicAuthenticator {
    fn generate(&mut self) -> String {
        format!("Basic realm=\"{}\"", self.realm)
    }
}

pub struct DigestAuthenticator {
    realm: String,
    nonce: String,
    opaque: String,
    pub stale: bool,   // nonce 是否过期
    algorithm: String, // 目前只支持 MD5
}

impl DigestAuthenticator {
    pub fn new(realm: &str, nonce: Option<String>) -> Self {
        Self {
            realm: realm.to_string(),
            nonce: nonce.unwrap_or(generate_nonce()), // 生成随机 nonce, 也可以从配置中读取
            opaque: "".to_string(),
            stale: false, // 默认不过期, 修改 nonce 时设置为 true.
            algorithm: "MD5".to_string(),
        }
    }
}

impl AuthHeaderGenerator for DigestAuthenticator {
    fn generate(&mut self) -> String {
        if self.stale {
            self.nonce = generate_nonce();
        }
        
        let mut header = format!(
            r#"Digest realm="{}", nonce="{}", algorithm="{}"#,
            self.realm, self.nonce, self.algorithm
        );

        if self.stale {
            header.push_str(", stale=true");
        }

        header
    }
}

pub fn generate<T: AuthHeaderGenerator>(auth: &mut T) -> String {
    auth.generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_digest_header() {
        // 使用示例
        let auth_header = "Digest username=\"admin\", realm=\"rust rtsp-server\", nonce=\"c4119e8b076a09c3\", uri=\"rtsp://192.168.3.27:5544/track1\", response=\"73dbd1c2c166a45b27f04345770cc3f7\", algorithm=\"MD5\"\r\n";
        // let auth_header = "Digest username=\"admin\", nonce=\"abc123\", uri=\"rtsp://example.com/stream\", response=\"d41d8cd98f00b204e9800998ecf8427e\"";
        let auth_params = parse_digest_header(auth_header).unwrap();
        println!("auth_params: {:#?}", auth_params);
        assert_eq!(auth_params.get("username").unwrap(), "admin");
        assert_eq!(auth_params.get("nonce").unwrap(), "abc123");
        assert_eq!(auth_params.get("uri").unwrap(), "rtsp://example.com/stream");
        assert_eq!(auth_params.get("response").unwrap(), "d41d8cd98f00b204e9800998ecf8427e");
    }
}