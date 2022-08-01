use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Config {
    host: String,
    port: u16,
    db_url: String,
}

impl From<config::Config> for Config {
    fn from(config: config::Config) -> Self {
        Self {
            host: config
                .get_string("HOST")
                .expect("Error parsing HOST in settings"),
            port: config
                .get::<u16>("PORT")
                .expect("Error parsing PORT in settings"),
            db_url: config
                .get("DATABASE_URL")
                .expect("Error parsing DB_URL in settings"),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Config, config::ConfigError> {
        let env = config::File::new("server_config", config::FileFormat::Toml);
        let builder = config::Config::builder().add_source(env);
        Ok(builder.build().expect("Couldn't build config").into())
    }

    pub fn get_address(&self) -> (&str, u16) {
        (self.host.as_ref(), self.port)
    }
    pub fn get_db_url(&self) -> &str {
        &self.db_url
    }
}
