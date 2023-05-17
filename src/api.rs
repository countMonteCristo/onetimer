use tiny_http::Request;

use serde::{Deserialize, Serialize};


use crate::utils::{one, week_seconds};


#[derive(Serialize, Deserialize)]
pub struct ApiAddRequest {
    data: String,

    #[serde(default = "one")]
    max_clicks: i64,

    #[serde[default = "week_seconds"]]
    lifetime: i64,
}

impl ApiAddRequest {
    pub fn get_data(&self) -> &String { &self.data }
    pub fn get_max_clicks(&self) -> i64 { if self.max_clicks <= 0 {one()} else {self.max_clicks} }
    pub fn get_lifetime(&self) -> i64 { if self.lifetime <= 0 {week_seconds()} else {self.lifetime} }
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    pub msg: String,
    pub status: String
}

pub fn new_response() -> ApiResponse {
    ApiResponse { msg: String::new(), status: "OK".to_string() }
}

pub fn parse_request(r: &mut Request) -> Result<ApiAddRequest, serde_json::Error> {
    let mut content: String = String::new();
    r.as_reader().read_to_string(&mut content).unwrap();
    let res = serde_json::from_str(content.as_str());

    if res.is_err() {
        error!("[HANDLERS] Failed to parse request data from `{}`: {}", content, res.as_ref().err().unwrap());
    }
    res
}
