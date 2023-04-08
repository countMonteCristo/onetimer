use std::collections::HashMap;


#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub content_length: usize,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct Response {
    pub content_length: usize,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub status_code: usize,
    pub status_msg: &'static str,
}
