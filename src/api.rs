use tiny_http::Request;

use serde::{Deserialize, Serialize};


use crate::utils::{one, week_seconds};


#[derive(Serialize, Deserialize)]
pub struct ApiAddRequest {
    pub data: String,

    #[serde(default = "one")]
    pub max_clicks: i64,

    #[serde[default = "week_seconds"]]
    pub lifetime: i64,
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
