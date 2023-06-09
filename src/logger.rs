use std::fs::OpenOptions;

use simplelog::{ColorChoice, LevelFilter, TerminalMode, TermLogger, WriteLogger};

use crate::config::Config;
use crate::utils::{ErrorStr, ResultV};


pub fn init_logger(cfg: &Config) -> ResultV {
    match cfg.log.kind.as_str() {
        "console" => init_term_logger(cfg.log.level),
        "file"    => init_file_logger(cfg.log.level, &cfg.log.file),
        _         => {
            eprintln!(
                "Unsupported log kind: {}, only `file` and `console` are supported. Use `console` by default",
                cfg.log.kind
            );
            init_term_logger(cfg.log.level)
        }
    }
}

pub fn get_reporter<E: std::fmt::Display>(module: &'static str, item: &'static str, msg: ErrorStr) -> impl Fn(E) -> ErrorStr {
    move |e: E| {
        error!("[{}] {} error: {}", module, item, e);
        msg
    }
}


fn prepare_logger_config() -> simplelog::Config {
    simplelog::ConfigBuilder::new().set_time_format_custom(
        simplelog::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
        )
    ).set_time_offset_to_local().unwrap().build()
}

fn init_term_logger(level: LevelFilter) -> ResultV {
    TermLogger::init(
        level,
        prepare_logger_config(),
        TerminalMode::Stderr, ColorChoice::Auto
    ).map_err(|_| "init logger error")
}

fn init_file_logger(level: LevelFilter, filename: &String) -> ResultV {
    WriteLogger::init(
        level,
        prepare_logger_config(),
        OpenOptions::new().write(true).create(true).append(true).open(filename).map_err(|_|"io error")?
    ).map_err(|_| "init logger error")
}
