use std::sync::Arc;
use std::collections::HashMap;

use rand::Rng;

use crate::request::{Request, Response};
use crate::db::{self, DB};

pub fn handle_method_add(r: &Request, db: Arc<db::SqliteDB>) -> Result<Response, &'static str> {
    match create_url_for_msg(r, r.body.clone(), db) {
        Ok(url) => {
            Ok(Response {
                content_length: url.len(),
                http_version: r.http_version.clone(),
                headers: HashMap::from([
                    (String::from("Content-Type"), String::from("text/plain")),
                    (String::from("Content-Length"), url.len().to_string()),
                ]),
                body: url,
                status_code: 201,
                status_msg: "Created",
            })
        }
        Err(e) => {
            Err(e)
        }
    }
}

fn create_url_for_msg(r: &Request, msg: Vec<u8>, db: Arc<db::SqliteDB>) -> Result<String, &'static str> {
    let id = generate_msg_id();

    match db.insert(id.clone(), msg) {
        Ok(_) => {},
        Err(e) => {
            println!("[ERROR] [MAIN] Server error: {}", e);
            return Err("server error");
        }
    }

    let host_str = match r.headers.get(&String::from("Host")) {
        Some(host) => {
            host.to_string()
        },
        None => {
            String::from("127.0.0.1:8080")
        }
    };

    let url = format!("http://{}/get/{}", host_str, id);
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


pub fn handle_method_get(r: &Request, id: &str, db: Arc<db::SqliteDB>) -> Result<Response, &'static str> {
    let mut resp = Response {
        content_length: 0,
        http_version: r.http_version.clone(),
        headers: HashMap::new(),
        body: String::new(),
        status_code: 200,
        status_msg: "OK",
    };

    resp.headers.insert(String::from("Content-Type"), String::from("text/plain"));
    resp.headers.insert(String::from("Content-Length"), String::from("0"));

    match db.select(id.to_string()) {
        Ok(msg) => {
            resp.content_length = msg.len();
            resp.headers.insert(String::from("Content-Length"), resp.content_length.to_string());
            resp.body.push_str(msg.as_str());
        }
        Err(e) => {
            if e != "not_found" {
                println!("[ERROR] [MAIN] Error while doing select: {}", e);
            }
            resp.status_code = 404;
            resp.status_msg = "Not Found";
        }
    };
    Ok(resp)
}
