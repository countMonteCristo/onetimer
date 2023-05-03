use config;
use simplelog::LevelFilter;


pub struct Config {
    pub db_url: String,
    pub server_host: String,
    pub server_port: u32,
    pub server_address: String,
    pub log_type: String,
    pub log_file: String,
    pub log_level: LevelFilter,
}

fn get_log_level(level_str: String) -> LevelFilter {
    match level_str.to_lowercase().as_str() {
        "debug" => LevelFilter::Debug,
        "error" => LevelFilter::Error,
        "info"  => LevelFilter::Info,
        "off"   => LevelFilter::Off,
        "trace" => LevelFilter::Trace,
        "warn"  => LevelFilter::Warn,
        _       => {
            eprintln!("Unknown log level: {}. Set to INFO", level_str);
            LevelFilter::Info
        }
    }
}


pub fn load(path: &str) -> Config {
    let config = config::Config::builder()
        .add_source(config::File::with_name(path))
        .build()
        .unwrap();

    Config {
        db_url: config.get::<String>("database.path").unwrap(),
        server_host: config.get::<String>("server.host").unwrap(),
        server_port: config.get::<u32>("server.port").unwrap(),
        server_address: config.get::<String>("server.address").unwrap(),
        log_type: config.get::<String>("log.type").unwrap(),
        log_file: config.get::<String>("log.file").unwrap(),
        log_level: get_log_level(config.get::<String>("log.level").unwrap()),
    }
}
