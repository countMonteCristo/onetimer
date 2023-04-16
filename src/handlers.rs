use std::{io, sync::Arc};

use rand::Rng;
use tiny_http::{Request, Response, StatusCode};

use crate::db::DB;
use crate::config::Settings;


const URL_ID_CHARSET: &[u8] = b"0123456789abcdef";
const URL_ID_LENGTH: usize = 64;


fn respond_error(r: Request, status_code: u16, msg: &'static str) -> io::Result<()> {
    let resp_msg = format!("{} {}", status_code, msg);
    r.respond(Response::from_string(resp_msg).with_status_code(StatusCode(status_code)))
}

pub fn respond_server_error(r: Request) -> io::Result<()> {
    respond_error(r, 500, "Server Error")
}

pub fn respond_not_implemented(r: Request) -> io::Result<()> {
    respond_error(r, 501, "Not Implemented")
}

pub fn respond_bad_request(r: Request) -> io::Result<()> {
    respond_error(r, 400, "Bad Request")
}

pub fn respond_not_found(r: Request) -> io::Result<()> {
    respond_error(r, 404, "Not Found")
}


pub fn handle_method_add(mut r: Request, db: Arc<dyn DB>, cfg: Arc<Settings>) -> io::Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    if let Err(e) = r.as_reader().read_to_end(&mut buf) {
        eprintln!("[ERROR] [HANDLERS] Failed to read request data: {}", e);
        return respond_bad_request(r);
    }
    match create_url_for_msg(buf, db, cfg) {
        Ok(url) => {
            r.respond(Response::from_string(url))
        }
        Err(e) => {
            eprintln!("[ERROR] [HANDLERS] Failed to create url for user request: {}", e);
            respond_server_error(r)
        }
    }

}

fn create_url_for_msg(msg: Vec<u8>, db: Arc<dyn DB>, cfg: Arc<Settings>) -> Result<String, &'static str> {
    let id = generate_msg_id();

    match db.insert(id.clone(), msg) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("[ERROR] [HANDLERS] Server error: {}", e);
            return Err("server error");
        }
    }

    let url = format!("{}/get/{}", cfg.server_address, id);
    return Ok(url);
}

fn generate_msg_id() -> String {
    let mut rng = rand::thread_rng();

    (0..URL_ID_LENGTH).map(
        |_| {
            let idx = rng.gen_range(0..URL_ID_CHARSET.len());
            URL_ID_CHARSET[idx] as char
        }
    ).collect()
}


pub fn handle_method_get(r: Request, db: Arc<dyn DB>) -> io::Result<()>  {
    let parts: Vec<&str> = r.url().split("/").collect();
    let id = parts[2];
    match db.select(id.to_string()) {
        Ok(msg) => {
            r.respond(Response::from_string(msg))
        }
        Err(e) => {
            if e != "not_found" {
                eprintln!("[ERROR] [MAIN] Error while doing select: {}", e);
            }
            respond_not_found(r)
        }
    }
}
