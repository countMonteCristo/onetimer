pub mod db;
pub mod config;
pub mod handlers;

use std::io;
use std::sync::Arc;
use std::thread;
use std::env;

use tiny_http::{Request, Server, Method};

use crate::db::DB;
use crate::config::Settings;
use crate::handlers::{handle_method_get, respond_not_found, handle_method_add};


fn handle_request(request: Request, db: Arc<db::SqliteDB>, cfg: Arc::<Settings>) -> io::Result<()> {
    match (request.method(), request.url().clone()) {
        (Method::Post, "/add") => {
            handle_method_add(request, db, cfg)
        }
        (Method::Get, url) if url.starts_with("/get/") => {
            handle_method_get(request, db)
        }
        (_, _) => {
            respond_not_found(request)
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config_path = if args.len() > 1 {args[1].as_str()} else {"conf/config.toml"};
    let s = config::load(config_path);
    let addr = format!("{}:{}", s.server_host, s.server_port);

    let database = db::SqliteDB::create(s.db_url.as_str());
    database.prepare();

    let server = Server::http(&addr).map_err(|err| {
        eprintln!("[ERROR] [MAIN] Could not start server at {addr}: {err}");
    }).unwrap();

    let database = Arc::new(database);
    let sets = Arc::new(s);
    for request in server.incoming_requests() {
        let db_ = database.clone();
        let s_ = sets.clone();
        thread::spawn(move || {
            handle_request(request, db_, s_).ok();
        });
    }

}
