use tiny_http::Request;

use serde::{Deserialize, Serialize};


use crate::utils::{one, week_seconds, now, is_zero};


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

    pub fn parse_from(r: &mut Request) -> Result<ApiAddRequest, serde_json::Error> {
        let res = serde_json::from_reader(r.as_reader());
        if res.is_err() {
            error!("[HANDLERS] Failed to parse request data: {}", res.as_ref().err().unwrap());
        }
        res
    }
}

#[derive(Serialize, Deserialize)]
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
