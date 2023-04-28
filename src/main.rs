#[macro_use] extern crate log;

pub mod db;
pub mod config;
pub mod context;
pub mod handlers;
pub mod utils;

use std::{io, thread, env, sync::Arc, fs::File};

use tiny_http::{Request, Server, Method};
use simplelog::{self, WriteLogger};

use crate::db::{DB, SqliteDB};
use crate::config::Settings;
use crate::handlers::{handle_method_get, handle_method_add, respond, HTTP_501};
use crate::context::Context;


fn handle_request(request: Request, db: Arc<dyn DB>, cfg: Arc::<Settings>) -> io::Result<()> {
    let mut ctx = Context::new();
    info!("New Request: method={} url={} qid={}", request.method(), request.url(), ctx.qid);
    match (request.method(), request.url()) {
        (Method::Post, "/add") => {
            handle_method_add(request, db, cfg, &mut ctx)
        }
        (Method::Get, url) if url.starts_with("/get/") => {
            handle_method_get(request, db, &mut ctx)
        }
        (_, _) => {
            respond(request, &mut ctx, Some(HTTP_501), None)
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config_path = if args.len() > 1 {args[1].as_str()} else {"conf/config.toml"};
    let s = config::load(config_path);
    let addr = format!("{}:{}", s.server_host, s.server_port);

    let _ = WriteLogger::init(
        s.log_level,
        simplelog::Config::default(),
        File::create(s.log_file.clone()
    ).unwrap());

    let database = SqliteDB::create(s.db_url.as_str());
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
