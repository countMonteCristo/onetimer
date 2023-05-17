use std::io;

use tiny_http::{Request, Response, StatusCode};

use crate::api::{ApiAddRequest, parse_request};
use crate::context::Context;
use crate::utils::generate_hex_id;
use crate::db::NOT_FOUND_ERROR;


const URL_ID_LENGTH: u32 = 64;

pub const HTTP_200: u16 = 200;
pub const HTTP_400: u16 = 400;
pub const HTTP_404: u16 = 404;
pub const HTTP_500: u16 = 500;
pub const HTTP_501: u16 = 501;


pub fn respond(r: Request, ctx: &mut Context, code: u16) -> io::Result<()> {
    let data = serde_json::to_string(&ctx.resp).unwrap();
    let response = Response::from_string(&data).with_status_code(StatusCode(code));
    let result = r.respond(response);

    ctx.fix();
    info!("Respond to [qid={}]: time: {}ms; status: {}; sent: {} bytes", ctx.qid, ctx.time_ms(), code, data.as_bytes().len());

    result
}

pub fn handle_method_add(mut r: Request, ctx: &mut Context) -> io::Result<()> {
    let parsed = parse_request(&mut r);
    if parsed.is_err() {
        ctx.resp.status = "Failed to parse input request".to_string();
        return respond(r, ctx, HTTP_400);
    }
    let json = parsed.unwrap();

    let code = match create_url_for_msg(&json, ctx) {
        Ok(url) => {
            ctx.resp.msg = url;
            HTTP_200
        },
        Err(e) => {
            error!("[HANDLERS] Failed to create url for user request: {}", e);
            ctx.resp.status = e.to_string();
            HTTP_500
        }
    };

    respond(r, ctx, code)
}

fn create_url_for_msg(msg: &ApiAddRequest, ctx: &mut Context) -> Result<String, &'static str> {
    let id = generate_hex_id(URL_ID_LENGTH);

    match ctx.db.lock().unwrap().insert(&id, msg) {
        Ok(_) => {},
        Err(e) => {
            error!("[HANDLERS] Server error: {}", e);
            return Err("Server error");
        }
    }
    let url = format!("{}/get/{}", ctx.cfg.server_address, id);
    Ok(url)
}

pub fn handle_method_get(r: Request, ctx: &mut Context) -> io::Result<()>  {
    let parts: Vec<&str> = r.url().split("/").collect();
    let id = parts[2];

    let code = match ctx.db.lock().unwrap().select(&id.to_string()) {
        Ok(message) => {
            ctx.resp.msg = message;
            HTTP_200
        },
        Err(e) => {
            if e != NOT_FOUND_ERROR {
                error!("[HANDLERS] Error while doing select: {}", e);
            }
            ctx.resp.status = "Link was not found or has been deleted".to_string();
            HTTP_404
        }
    };

    respond(r, ctx, code)
}
