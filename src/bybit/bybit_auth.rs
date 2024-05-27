use hex;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

pub fn _generate_post_signature(
    timestamp: &str,
    api_key: &str,
    recv_window: &str,
    params: &serde_json::Map<String, Value>,
    api_secret: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(timestamp.as_bytes());
    mac.update(api_key.as_bytes());
    mac.update(recv_window.as_bytes());
    mac.update(serde_json::to_string(&params)?.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    Ok(hex::encode(code_bytes))
}

pub fn _generate_get_signature(
    timestamp: &str,
    api_key: &str,
    recv_window: &str,
    params: &HashMap<&str, &str>,
    api_secret: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(timestamp.as_bytes());
    mac.update(api_key.as_bytes());
    mac.update(recv_window.as_bytes());
    mac.update(_generate_query_str(params).as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    Ok(hex::encode(code_bytes))
}

fn _generate_query_str(params: &HashMap<&str, &str>) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<String>>()
        .join("&")
}
