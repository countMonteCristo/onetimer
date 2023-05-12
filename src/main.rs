#[macro_use] extern crate log;

pub mod config;
pub mod context;
pub mod db;
pub mod handlers;
pub mod logger;
pub mod utils;

use std::{io, env, thread};
use std::sync::Arc;

use tiny_http::{Method, Request, Server};

use crate::db::{DB, SqliteDB};
use crate::handlers::{handle_method_add, handle_method_get, respond, HTTP_501};
use crate::context::Context;


fn handle_request(r: Request, mut ctx: Context) -> io::Result<()> {
    let headers: String = r.headers().iter().map(|h| -> String {
        h.to_string()
    }).collect::<Vec<String>>().join("\\r\\n");
    info!("New Request [qid={}]: method: {}; url: {}; headers='{}'", ctx.qid, r.method(), r.url(), headers);

    match (r.method(), r.url()) {
        (Method::Post, "/add") => {
            handle_method_add(r, &mut ctx)
        }
        (Method::Get, url) if url.starts_with("/get/") => {
            handle_method_get(r, &mut ctx)
        }
        (_, _) => {
            respond(r, &mut ctx, HTTP_501, None)
        }
    }
}


fn main() {
    let cfg_path = env::args().nth(1).expect("Config file was not provided");
    let cfg = config::load(&cfg_path);
    logger::init_logger(&cfg);

    let database = SqliteDB::create(cfg.db_url.as_str());
    database.prepare();

    let addr = format!("{}:{}", cfg.server_host, cfg.server_port);
    let server = Server::http(&addr).map_err(|err| {
        error!("[MAIN] Could not start server at {}: {}", addr, err);
    }).unwrap();
    info!("[MAIN] Staring onetimer service at {}", addr);
    info!("[MAIN] Config loaded from {}", cfg_path);

    let db_arc = Arc::new(database);
    let cfg_arc = Arc::new(cfg);
    for r in server.incoming_requests() {
        let db_ = db_arc.clone();
        let cfg_ = cfg_arc.clone();
        thread::spawn(move || {
            handle_request(r, Context::new(db_, cfg_)).ok();
        });
    };
}
