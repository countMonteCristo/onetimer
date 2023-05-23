use std::io;

use tiny_http::{Request, Response, StatusCode};

use crate::api::ApiAddRequest;
use crate::context::Context;
use crate::utils::generate_hex_id;
use crate::db::NOT_FOUND_ERROR;
use crate::logger::get_reporter;


const MODULE: &str = "HANDLERS";

const URL_ID_LENGTH: u32 = 64;

pub const HTTP_200: u16 = 200;
pub const HTTP_400: u16 = 400;
pub const HTTP_404: u16 = 404;
pub const HTTP_500: u16 = 500;
pub const HTTP_501: u16 = 501;


pub fn respond(r: Request, ctx: &mut Context, code: u16) -> io::Result<()> {
    let data = serde_json::to_string(&ctx.resp)?;
    let response = Response::from_string(&data).with_status_code(StatusCode(code));
    let result = r.respond(response);

    ctx.fix();
    info!("Respond to [qid={}]: time: {}ms; status: {}; sent: {} bytes", ctx.qid, ctx.time_ms(), code, data.as_bytes().len());

    result
}

pub fn handle_method_add(mut r: Request, ctx: &mut Context) -> io::Result<()> {
    let parsed = ApiAddRequest::parse_from(&mut r);
    let mut code = HTTP_400;

    match parsed {
        Ok(json) => {
            ctx.resp.set_expired(ctx.resp.created() + (json.get_lifetime() as i64));

            code = match create_url_for_msg(&json, ctx) {
                Ok(url) => {
                    ctx.resp.set_message(url);
                    HTTP_200
                },
                Err(e) => {
                    error!("[{}] Failed to create url for user request: {}", MODULE, e);
                    ctx.resp.set_status(e.to_string());
                    HTTP_500
                }
            };
        }
        Err(_) => ctx.resp.set_status("Failed to parse input request".to_string()),
    }
    respond(r, ctx, code)
}

fn create_url_for_msg(msg: &ApiAddRequest, ctx: &mut Context) -> Result<String, &'static str> {
    let id = generate_hex_id(URL_ID_LENGTH);

    ctx.db().insert(&id, msg).map_err(
        get_reporter(MODULE, "Server", "erver error")
    )?;

    let url = format!("{}/get/{}", ctx.cfg.server.address, id);
    Ok(url)
}

pub fn handle_method_get(r: Request, ctx: &mut Context) -> io::Result<()>  {
    let parts: Vec<&str> = r.url().split("/").collect();
    let id = parts[2];

    let res = ctx.db().select(&id.to_string());
    let code =  match res {
        Ok(message) => {
            ctx.resp.set_message(message);
            HTTP_200
        },
        Err(e) => {
            if e != NOT_FOUND_ERROR {
                error!("[{}] Error while doing select: {}", MODULE, e);
            }
            ctx.resp.set_status("Link was not found or has been deleted".to_string());
            HTTP_404
        }
    };

    // Do not want to show sensitive fields in response
    ctx.resp.hide_sensitive();

    respond(r, ctx, code)
}
