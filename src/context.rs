use std::sync::{Arc, Mutex, MutexGuard};

use crate::api::ApiResponse;
use crate::config::Config;
use crate::db::DB;
use crate::logger::get_reporter;
use crate::utils::{generate_hex_id, time_us, Result};


const MODULE: &str = "CONTEXT";

pub struct Context {
    pub qid: String,
    pub start_time_us: u128,
    pub finish_time_us: u128,
    pub cfg: Arc<Config>,
    pub db: Arc<Mutex<DB>>,
    pub resp: ApiResponse,
}

impl Context {
    pub fn new(db: Arc<Mutex<DB>>, cfg: Arc::<Config>) -> Context {
        Context {
            qid: generate_hex_id(8),
            start_time_us: time_us(),
            finish_time_us: 0,
            cfg,
            db,
            resp: ApiResponse::new(),
        }
    }

    pub fn fix(&mut self) {
        self.finish_time_us = time_us();
    }

    pub fn time_ms(&self) -> f32 {
        ((self.finish_time_us - self.start_time_us) as f32)/1000.0
    }

    pub fn db(&mut self) -> Result<MutexGuard<DB>> {
        self.db.lock().map_err(get_reporter(MODULE, "Context", "context error"))
    }
}
