use rand::Rng;
use std::time::{Duration, SystemTime, UNIX_EPOCH};


const URL_ID_CHARSET: &[u8] = b"0123456789abcdef";

pub fn generate_hex_id(length: u32) -> String {
    let mut rng = rand::thread_rng();

    (0..length).map(
        |_| {
            let idx = rng.gen_range(0..URL_ID_CHARSET.len());
            URL_ID_CHARSET[idx] as char
        }
    ).collect()
}


fn current_duration() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards")
}

pub fn time_us() -> u128 {
    current_duration().as_micros()
}

pub fn now() -> i64 {
    current_duration().as_secs() as i64
}

pub fn one() -> i64 {1}
pub fn day_seconds() -> i64 {
    24 * 60 * 60
}
pub fn week_seconds() -> i64 {
    7 * day_seconds()
}

pub fn is_zero(x: &i64) -> bool {
    *x == 0
}
