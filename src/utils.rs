use rand::Rng;


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
