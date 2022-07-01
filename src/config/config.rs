use serde::Deserialize;
use tracing::{event, instrument};

#[derive(Debug, Deserialize)]
pub struct Config {
    host: String,
    port: u16,
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
        }
    }
}

impl Config {
    #[instrument]
    pub fn from_env() -> Result<Config, config::ConfigError> {
        event!(tracing::Level::INFO, "Building configuration");

        let env = config::File::new("server_config", config::FileFormat::Toml);
        let builder = config::Config::builder().add_source(env);
        Ok(builder.build().expect("Couldn't build config").into())
    }

    pub fn get_address(&self) -> (&str, u16) {
        (self.host.as_ref(), self.port)
    }
}
