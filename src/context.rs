use crate::utils::{generate_hex_id, time_us};
use crate::config::Config;
use crate::db::DB;

use std::sync::{Arc, Mutex};

use crate::api::{new_response, ApiResponse};


pub struct Context {
    pub qid: String,
    pub start_time_us: u128,
    pub finish_time_us: u128,
    pub cfg: Arc<Config>,
    pub db: Arc<Mutex<Box<dyn  DB>>>,
    pub resp: ApiResponse
}

impl Context {
    pub fn new(db: Arc<Mutex<Box<dyn DB>>>, cfg: Arc::<Config>) -> Context {
        Context {
            qid: generate_hex_id(8),
            start_time_us: time_us(),
            finish_time_us: 0,
            cfg,
            db,
            resp: new_response(),
        }
    }

    pub fn fix(&mut self) {
        self.finish_time_us = time_us();
    }

    pub fn time_ms(&self) -> f32 {
        ((self.finish_time_us - self.start_time_us) as f32)/1000.0
    }
}
