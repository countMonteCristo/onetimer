use config::Config;


pub struct Settings {
    pub db_url: String,
    pub server_host: String,
    pub server_port: u32,
    pub server_address: String,
}


pub fn load(path: &str) -> Settings {
    let config = Config::builder()
        .add_source(config::File::with_name(path))
        .build()
        .unwrap();

    Settings {
        db_url: config.get::<String>("database.path").unwrap(),
        server_host: config.get::<String>("server.host").unwrap(),
        server_port: config.get::<u32>("server.port").unwrap(),
        server_address: config.get::<String>("server.address").unwrap(),
    }
}
