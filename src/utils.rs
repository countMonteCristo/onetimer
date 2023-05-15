use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};


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


pub fn time_us() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_micros()
}

pub fn one() -> i64 {1}
