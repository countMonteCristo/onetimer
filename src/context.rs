use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::generate_hex_id;


pub struct Context {
    pub qid: String,
    pub start_time_us: u128,
    pub finish_time_us: u128,
}

impl Context {
    pub fn new() -> Context {
        return Context {
            qid: generate_hex_id(8),
            start_time_us: SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_micros(),
            finish_time_us: 0
        };
    }

    pub fn fix(&mut self) {
        self.finish_time_us = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_micros();
    }

    pub fn time_ms(&self) -> f32 {
        ((self.finish_time_us - self.start_time_us) as f32)/1000.0
    }
}
