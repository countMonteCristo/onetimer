use std::io;
use std::sync::Arc;

use tiny_http::{Request, Response, StatusCode};

use crate::db::DB;
use crate::config::Settings;
use crate::utils::generate_hex_id;
use crate::context::Context;


const URL_ID_LENGTH: u32 = 64;

pub const HTTP_200: u16 = 200;
pub const HTTP_400: u16 = 400;
pub const HTTP_404: u16 = 404;
pub const HTTP_500: u16 = 500;
pub const HTTP_501: u16 = 501;

fn get_err_msg(code: u16) -> &'static str {
    match code {
        HTTP_400 => "Bad Request",
        HTTP_404 => "Not Found",
        HTTP_500 => "Server Error",
        HTTP_501 => "Not Implemented",
        _ => "UNKNOWN",
    }
}

fn respond_error(r: Request, status_code: u16, msg: &'static str) -> io::Result<()> {
    let resp_msg = format!("{} {}", status_code, msg);
    r.respond(Response::from_string(resp_msg).with_status_code(StatusCode(status_code)))
}

pub fn respond(r: Request, ctx: &mut Context, status_code: Option<u16>, msg: Option<&str>) -> io::Result<()> {
    let code = status_code.unwrap_or(HTTP_200);
    let message = msg.unwrap_or("");

    let result = match code {
        HTTP_200 => r.respond(Response::from_string(message)),
        _ => respond_error(r, code, get_err_msg(code),
        ),
    };
    ctx.fix();
    info!("Request qid={} time={} ms", ctx.qid, ctx.time_ms());

    result
}

pub fn handle_method_add(mut r: Request, db: Arc<dyn DB>, cfg: Arc<Settings>, ctx: &mut Context) -> io::Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    if let Err(e) = r.as_reader().read_to_end(&mut buf) {
        eprintln!("[ERROR] [HANDLERS] Failed to read request data: {}", e);
        return respond(r, ctx, Some(HTTP_400), None);
    }
    match create_url_for_msg(buf, db, cfg) {
        Ok(url) => {
            respond(r, ctx, None, Some(&url))
        }
        Err(e) => {
            eprintln!("[ERROR] [HANDLERS] Failed to create url for user request: {}", e);
            respond(r, ctx, Some(HTTP_500), None)
        }
    }

}

fn create_url_for_msg(msg: Vec<u8>, db: Arc<dyn DB>, cfg: Arc<Settings>) -> Result<String, &'static str> {
    let id = generate_hex_id(URL_ID_LENGTH);

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

pub fn handle_method_get(r: Request, db: Arc<dyn DB>, ctx: &mut Context) -> io::Result<()>  {
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
            respond(r, ctx, Some(HTTP_404), None)
        }
    }
}
