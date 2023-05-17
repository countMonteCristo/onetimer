#[macro_use] extern crate log;

pub mod api;
pub mod config;
pub mod context;
pub mod db;
pub mod handlers;
pub mod logger;
pub mod utils;

use std::{io, thread};
use std::sync::{Arc, Mutex};

use clap::Parser;
use tiny_http::{Method, Request, Server};

use crate::db::get_db;
use crate::handlers::{handle_method_add, handle_method_get, respond, HTTP_501};
use crate::context::Context;


/// Simple service for generating one-time access link to your secret data
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Path to the configurational file
    config_fn: String,
}

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
            ctx.resp.status = "Method is not implemented".to_string();
            respond(r, &mut ctx, HTTP_501)
        }
    }
}


fn main() {
    let args = Args::parse();
    let cfg = config::load(&args.config_fn);
    logger::init_logger(&cfg);

    let database = get_db(&cfg.db_type, &cfg.db_url);
    if database.is_err() {
        error!(
            "[MAIN] Could not init database of type {} and path {}: {}",
            cfg.db_type, cfg.db_url, database.as_ref().err().unwrap()
        );
        return;
    }
    let db = database.unwrap();
    info!("[MAIN] Use `{}` as database backend", db.get_type());
    db.prepare();

    let addr = format!("{}:{}", cfg.server_host, cfg.server_port);
    let server = Server::http(&addr).map_err(|err| {
        error!("[MAIN] Could not start server at {}: {}", addr, err);
    }).unwrap();
    info!("[MAIN] Staring onetimer service at {}", addr);
    info!("[MAIN] Config loaded from {}", args.config_fn);

    let db_arc = Arc::new(Mutex::new(db));
    let cfg_arc = Arc::new(cfg);
    for r in server.incoming_requests() {
        let db_ = db_arc.clone();
        let cfg_ = cfg_arc.clone();
        thread::spawn(move || {
            handle_request(r, Context::new(db_, cfg_)).ok();
        });
    };
}

