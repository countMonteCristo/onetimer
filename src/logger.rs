use crate::config::Config;

use std::fs::OpenOptions;

use simplelog::{ColorChoice, LevelFilter, TerminalMode, TermLogger, WriteLogger};


pub fn init_logger(cfg: &Config) {
    match cfg.log.kind.as_str() {
        "console" => init_term_logger(cfg.log.level),
        "file"    => init_file_logger(cfg.log.level, &cfg.log.file),
        _         => {
            eprintln!(
                "Unsupported log type: {}, only `file` and `console` are supported. Use `console` by default",
                cfg.log.kind
            );
            init_term_logger(cfg.log.level);
        }
    };
}


fn prepare_logger_config() -> simplelog::Config {
    simplelog::ConfigBuilder::new().set_time_format_custom(
        simplelog::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
        )
    ).set_time_offset_to_local().unwrap().build()
}

fn init_term_logger(level: LevelFilter) {
    TermLogger::init(
        level,
        prepare_logger_config(),
        TerminalMode::Stderr, ColorChoice::Auto
    ).unwrap();
}

fn init_file_logger(level: LevelFilter, filename: &String) {
    WriteLogger::init(
        level,
        prepare_logger_config(),
        OpenOptions::new().write(true).create(true).append(true).open(filename).unwrap()
    ).unwrap()
}
