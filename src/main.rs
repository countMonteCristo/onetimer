#[macro_use] extern crate log;

pub mod db;
pub mod config;
pub mod context;
pub mod handlers;
pub mod utils;

use std::{io, thread, env, sync::Arc, fs::OpenOptions};

use log::LevelFilter;
use tiny_http::{Request, Server, Method};
use simplelog::{self, WriteLogger, TermLogger, TerminalMode, ColorChoice};

use crate::db::{DB, SqliteDB};
use crate::config::Config;
use crate::handlers::{handle_method_get, handle_method_add, respond, HTTP_501};
use crate::context::Context;


fn handle_request(request: Request, db: Arc<dyn DB>, cfg: Arc::<Config>) -> io::Result<()> {
    let mut ctx = Context::new();

    let headers: String = request.headers().iter().map(|h| -> String {
        h.to_string()
    }).collect::<Vec<String>>().join("\\r\\n");

    info!("New Request [qid={}]: method: {}; url: {}; headers='{}'", ctx.qid, request.method(), request.url(), headers);
    match (request.method(), request.url()) {
        (Method::Post, "/add") => {
            handle_method_add(request, db, cfg, &mut ctx)
        }
        (Method::Get, url) if url.starts_with("/get/") => {
            handle_method_get(request, db, &mut ctx)
        }
        (_, _) => {
            respond(request, &mut ctx, HTTP_501, None)
        }
    }
}

fn init_term_logger(level: LevelFilter) {
    TermLogger::init(
        level, simplelog::Config::default(), TerminalMode::Stderr, ColorChoice::Auto
    ).unwrap();
}

fn init_file_logger(level: LevelFilter, filename: &String) {
    WriteLogger::init(
        level, simplelog::Config::default(),
        OpenOptions::new().write(true).create(true).append(true).open(filename).unwrap()
    ).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let config_path = if args.len() > 1 {args[1].as_str()} else {"conf/config.toml"};
    let config = config::load(config_path);
    let addr = format!("{}:{}", config.server_host, config.server_port);

    match config.log_type.as_str() {
        "console" => init_term_logger(config.log_level),
        "file"    => init_file_logger(config.log_level, &config.log_file),
        _         => {
            eprintln!(
                "Unsupported log type: {}, only `file` and `console` are supported. Use `console` by default",
                config.log_type
            );
            init_term_logger(config.log_level);
        }
    };

    let database = SqliteDB::create(config.db_url.as_str());
    database.prepare();

    let server = Server::http(&addr).map_err(|err| {
        error!("[MAIN] Could not start server at {}: {}", addr, err);
    }).unwrap();
    info!("[MAIN] Staring onetimer service at {}", addr);
    info!("[MAIN] Config loaded from {}", config_path);

    let db = Arc::new(database);
    let cfg = Arc::new(config);
    for request in server.incoming_requests() {
        let db_ = db.clone();
        let cfg_ = cfg.clone();
        thread::spawn(move || {
            handle_request(request, db_, cfg_).ok();
        });
    };
}
