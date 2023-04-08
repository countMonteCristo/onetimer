pub mod db;
pub mod config;
pub mod handlers;
pub mod request;

use std::collections::HashMap;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::io::{Write, Read, BufReader, BufRead};
use std::str;
use std::env;

use handlers::handle_method_get;

use crate::db::DB;
use crate::request::{Request, Response};
use crate::handlers::handle_method_add;


fn read_request(stream: &TcpStream) -> Result<Request, &'static str> {
    let mut r: Request = Request{
        method: String::from("GET"),
        url: String::from("/"),
        content_length: 0,
        http_version: String::from("HTTP/1.1"),
        headers: HashMap::new(),
        body: Vec::new(),
    };

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut name = String::new();
    loop {
        let count = reader.read_line(&mut name).unwrap();
        if count < 3 { //detect empty line
            break;
        }
    }
    // println!("--------------------------------------");
    // println!("{}", name);
    // println!("--------------------------------------");

    let linesplit = name.split("\n");

    let mut first = true;
    for l in linesplit {
        let trimmed = l.trim();
        if first {
            first = false;
            let parts: Vec<&str> = trimmed.split(" ").collect();
            if parts.len() < 3 {
                println!("[ERROR] [MAIN] Unknown request: {}", l);
                return Err("Unknown request type");
            }
            r.method = String::from(parts[0]);
            r.url = String::from(parts[1]);
            r.http_version = String::from(parts[2]);
            continue;
        }
        if trimmed.len() > 0 {
            let kv: Vec<&str> = trimmed.splitn(2, ":").collect();
            if kv.len() < 2 {
                println!("[WARN] [MAIN] Bad header: {}", l);
            } else {
                r.headers.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }
    }
    if r.headers.contains_key("Content-Length") {
        r.content_length = r.headers["Content-Length"].parse::<usize>().unwrap();
    }
    r.body.resize(r.content_length, 0);
    reader.read_exact(&mut r.body).unwrap();             //Get the Body Content.

    // println!("--------------------------------------");
    // println!("{:?}", r);
    // println!("--------------------------------------");

    Ok(r)
}

fn handle_request(r: &Request, db: Arc<db::SqliteDB>) -> Result<Response, &'static str> {
    match r.method.as_str() {
        "GET" => {
            handle_get(r, db)
        }
        "POST" => {
            handle_post(r, db)
        }
        _ => {
            Err("Unhandled request")
        }
    }
}

fn handle_get(r: &Request, db: Arc<db::SqliteDB>) -> Result<Response, &'static str> {
    let parts: Vec<&str> = r.url.split("/").collect();
    match parts[..] {
        [.., "get", id] => {
            handle_method_get(r, id, db)
        }
        _ => {
            Err("Unknown GET method")
        }
    }
}

fn handle_post(r: &Request, db: Arc<db::SqliteDB>) -> Result<Response, &'static str> {
    match r.url.as_str() {
        "/add" => {
            handle_method_add(r, db)
        }
        _ => {
            Err("Unknown POST method")
        }
    }
}


fn send_response(mut stream: &TcpStream, resp: &Response) {
    let mut resp_lines:Vec<String> = Vec::new();

    let status = format!("{} {} {}", resp.http_version, resp.status_code.to_string(), resp.status_msg);
    resp_lines.push(status);

    for (header_name, header_value) in &resp.headers {
        resp_lines.push(format!("{}: {}", header_name, header_value));
    }
    resp_lines.push(String::new());

    resp_lines.push(resp.body.clone());

    let data = resp_lines.join("\r\n");

    match stream.write(data.as_bytes()) {
        Ok(_) => {},
        Err(e) => println!("[ERROR] [MAIN] Failed sending response: {}", e),
    };
}

fn send_server_error(mut stream: &TcpStream, r: &Request) {
    let status = format!("{} 500 Internal Server Error\r\n", r.http_version);

    match stream.write(status.as_bytes()) {
        Ok(_) => {},
        Err(e) => println!("Failed sending response: {}!", e),
    };
}


fn handle_connection(stream: &TcpStream, db: Arc<db::SqliteDB>) {
    let r = read_request(stream).unwrap();
    match handle_request(&r, db) {
        Ok(resp) => send_response(stream, &resp),
        Err(e) => {
            println!("[ERROR] [MAIN] Failed to handle request: {}!", e);
            send_server_error(stream, &r);
        }
    };
    stream.shutdown(Shutdown::Write).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {args[1].as_str()} else {"conf/config.toml"};

    let s = config::load(config_path);
    let addr = format!("{}:{}", s.server_host, s.server_port);

    let listener = TcpListener::bind(addr.as_str()).unwrap();
    let database = db::SqliteDB::create(s.db_url.as_str());
    database.prepare();

    let database = Arc::new(database);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let db_ = database.clone();
                thread::spawn(move || {
                    handle_connection(&stream, db_);
                });
            }
            Err(e) => {
                println!("[ERROR] [MAIN] Connection failed: {}!", e);
            }
        }
    }
}
