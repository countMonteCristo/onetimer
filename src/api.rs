use tiny_http::Request;

use serde::{Deserialize, Serialize};


use crate::utils::{one, week_seconds, now, is_zero};
use crate::logger::get_reporter;


const MODULE: &str = "API";


#[derive(Deserialize)]
pub struct ApiAddRequest {
    data: String,

    #[serde(default = "one")]
    max_clicks: u32,

    #[serde[default = "week_seconds"]]
    lifetime: u64,
}

impl ApiAddRequest {
    pub fn get_data(&self) -> &String { &self.data }
    pub fn get_max_clicks(&self) -> u32 { if self.max_clicks <= 0 {one().into()} else {self.max_clicks} }
    pub fn get_lifetime(&self) -> u64 { if self.lifetime <= 0 {week_seconds()} else {self.lifetime} }

    pub fn parse_from(r: &mut Request) -> Result<ApiAddRequest, &'static str> {
        serde_json::from_reader(r.as_reader()).map_err(
            get_reporter(MODULE, "ApiAddRequest::parse_from", "parse error")
        )
    }
}

#[derive(Serialize)]
pub struct ApiResponse {
    msg: String,
    status: String,

    #[serde(skip_serializing_if = "is_zero")]
    created: i64,

    #[serde(skip_serializing_if = "is_zero")]
    expired: i64,
}

impl ApiResponse {
    pub fn new() -> Self {
        Self { msg: String::new(), status: "OK".to_string(), created: now(), expired: 0 }
    }

    pub fn set_message(&mut self, msg: String) {
        self.msg = msg;
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn set_created(&mut self, created: i64) {
        self.created = created;
    }

    pub fn set_expired(&mut self, expired: i64) {
        self.expired = expired;
    }

    pub fn message(&self) -> &String {
        &self.msg
    }

    pub fn status(&self) -> &String {
        &self.status
    }

    pub fn created(&self) -> i64 {
        self.created
    }

    pub fn expired(&self) -> i64 {
        self.expired
    }

    pub fn hide_sensitive(&mut self) {
        self.set_created(0);
        self.set_expired(0);
    }
}
