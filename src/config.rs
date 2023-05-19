use config;
use simplelog::LevelFilter;


pub struct Config {
    pub db_type: String,
    pub db_url: String,
    pub server_host: String,
    pub server_port: u32,
    pub server_address: String,
    pub log_type: String,
    pub log_file: String,
    pub log_level: LevelFilter,
}

impl Config {
    pub fn load(path: &String) -> Self {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .unwrap();

        Config {
            db_type: cfg.get::<String>("database.type").unwrap(),
            db_url: cfg.get::<String>("database.path").unwrap(),
            server_host: cfg.get::<String>("server.host").unwrap(),
            server_port: cfg.get::<u32>("server.port").unwrap(),
            server_address: cfg.get::<String>("server.address").unwrap(),
            log_type: cfg.get::<String>("log.type").unwrap(),
            log_file: cfg.get::<String>("log.file").unwrap(),
            log_level: Self::get_log_level(cfg.get::<String>("log.level").unwrap()),
        }
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
}
