use config;
use serde::{Deserializer, Deserialize};
use simplelog::LevelFilter;

#[derive(serde_derive::Deserialize)]
pub struct Database {
    pub kind: String,
    pub url: String,
}

#[derive(serde_derive::Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u32,
    pub workers: usize,
    pub address: String,
}

#[derive(serde_derive::Deserialize)]
pub struct Log {
    pub kind: String,
    #[serde(deserialize_with = "deserialize_log_level")]
    pub level: LevelFilter,
    pub file: String,
}

#[derive(serde_derive::Deserialize)]
pub struct Config {
    pub database: Database,
    pub server: Server,
    pub log: Log,
}

impl Config {
    pub fn load(path: &String) -> Self {
        config::Config::builder()
            .add_source(config::File::with_name(path))
            .set_default("database.kind",   String::from("memory")                  ).unwrap()
            .set_default("database.url",    String::from("db.sqlite")               ).unwrap()
            .set_default("server.host",     String::from("127.0.0.1")               ).unwrap()
            .set_default("server.port",     String::from("8080")                    ).unwrap()
            .set_default("server.workers",  32                                      ).unwrap()
            .set_default("server.address",  String::from("http://127.0.0.1:8080")   ).unwrap()
            .set_default("log.kind",        String::from("console")                 ).unwrap()
            .set_default("log.file",        String::from("onetimer.log")            ).unwrap()
            .set_default("log.level",       String::from("info")                    ).unwrap()
            .build().unwrap()
            .try_deserialize().unwrap()
    }

    fn get_log_level(level_str: String) -> Result<LevelFilter, &'static str> {
        match level_str.to_lowercase().as_str() {
            "debug" => Ok(LevelFilter::Debug),
            "error" => Ok(LevelFilter::Error),
            "info"  => Ok(LevelFilter::Info),
            "off"   => Ok(LevelFilter::Off),
            "trace" => Ok(LevelFilter::Trace),
            "warn"  => Ok(LevelFilter::Warn),
            _       => {
                eprintln!("Unknown log level: {}", level_str);
                Err("unknown log level")
            }
        }
    }
}

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
where D: Deserializer<'de> {
    use serde::de::Error;
    let buf = String::deserialize(deserializer)?;
    Config::get_log_level(buf).map_err(|e| Error::custom(e))
}

