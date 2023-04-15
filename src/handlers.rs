use std::io;
use std::sync::Arc;

use rand::Rng;
use tiny_http::{Request, Response};

use crate::db::{DB, SqliteDB};
use crate::config::Settings;


pub fn respond_server_error(r: Request) -> io::Result<()> {
    r.respond(Response::from_string("").with_status_code(500))
}

pub fn respond_not_found(r: Request) -> io::Result<()> {
    r.respond(Response::from_string("").with_status_code(404))
}


pub fn handle_method_add(mut r: Request, db: Arc<SqliteDB>, cfg: Arc<Settings>) -> io::Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    if let Err(e) = r.as_reader().read_to_end(&mut buf) {
        eprintln!("[ERROR] [HANDLERS] Failed to read request data: {}", e);
        return respond_server_error(r);
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

fn create_url_for_msg(msg: Vec<u8>, db: Arc<SqliteDB>, cfg: Arc<Settings>) -> Result<String, &'static str> {
    let id = generate_msg_id();

    match db.insert(id.clone(), msg) {
        Ok(_) => {},
        Err(e) => {
            println!("[ERROR] [MAIN] Server error: {}", e);
            return Err("server error");
        }
    }

    let url = format!("{}/get/{}", cfg.server_address, id);
    return Ok(url);
}

fn generate_msg_id() -> String {
    const CHARSET: &[u8] = b"0123456789abcdef";
    const LENGTH: usize = 64;
    let mut rng = rand::thread_rng();

    let id: String = (0..LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    id
}


pub fn handle_method_get(r: Request, db: Arc<SqliteDB>) -> io::Result<()>  {
    let parts: Vec<&str> = r.url().split("/").collect();
    let id = parts[2];
    match db.select(id.to_string()) {
        Ok(msg) => {
            r.respond(Response::from_string(msg))
        }
        Err(e) => {
            if e != "not_found" {
                println!("[ERROR] [MAIN] Error while doing select: {}", e);
            }
            respond_not_found(r)
        }
    }
}
