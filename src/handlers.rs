use std::io;

use tiny_http::{Request, Response, StatusCode};

use crate::context::Context;
use crate::utils::generate_hex_id;


const URL_ID_LENGTH: u32 = 64;

pub const HTTP_200: u16 = 200;
pub const HTTP_400: u16 = 400;
pub const HTTP_404: u16 = 404;
pub const HTTP_500: u16 = 500;
pub const HTTP_501: u16 = 501;


pub fn respond(r: Request, ctx: &mut Context, code: u16, msg: Option<String>) -> io::Result<()> {
    let status_code = StatusCode(code);
    let msg = msg.unwrap_or("".to_string());

    let data = if code == HTTP_200 {
        msg
    } else {
        let mut x = format!("{}: {}", code, status_code.default_reason_phrase().to_string());
        if msg.len() > 0 {
            x = format!("{}\n\n{}", x, msg);
        }
        x
    };
    let response = Response::from_string(&data).with_status_code(status_code);

    let result = r.respond(response);

    ctx.fix();
    info!("Respond to [{}]: time: {}ms; status: {}; sent: {} bytes", ctx.qid, ctx.time_ms(), code, data.as_bytes().len());

    result
}

pub fn handle_method_add(mut r: Request, ctx: &mut Context) -> io::Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    if let Err(e) = r.as_reader().read_to_end(&mut buf) {
        error!("[HANDLERS] Failed to read request data: {}", e);
        return respond(r, ctx, HTTP_400, None);
    }
    let (code, msg) = match create_url_for_msg(buf, ctx) {
        Ok(url) => (HTTP_200, Some(url)),
        Err(e) => {
            error!("[HANDLERS] Failed to create url for user request: {}", e);
            (HTTP_500, None)
        }
    };

    respond(r, ctx, code, msg)
}

fn create_url_for_msg(msg: Vec<u8>, ctx: &mut Context) -> Result<String, &'static str> {
    let id = generate_hex_id(URL_ID_LENGTH);

    match ctx.db.insert(&id, msg) {
        Ok(_) => {},
        Err(e) => {
            error!("[HANDLERS] Server error: {}", e);
            return Err("server error");
        }
    }
    let url = format!("{}/get/{}", ctx.cfg.server_address, id);

    Ok(url)
}

pub fn handle_method_get(r: Request, ctx: &mut Context) -> io::Result<()>  {
    let parts: Vec<&str> = r.url().split("/").collect();
    let id = parts[2];
    let (status_code, msg) = match ctx.db.select(&id.to_string()) {
        Ok(message) => (HTTP_200, Some(message)),
        Err(e) => {
            if e != "not_found" {
                error!("[HANDLERS] Error while doing select: {}", e);
            }
            (HTTP_404, None)
        }
    };

    respond(r, ctx, status_code, msg)
}
