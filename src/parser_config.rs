use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ParserConfig {
    interval: String,
    sender_email: Option<Email>,
}

#[derive(Debug, Deserialize)]
struct Email {
    email: String,
    token: String,
}

pub fn get_parser_config() -> Result<ParserConfig, ConfigError> {
    Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()
        .unwrap()
        .try_deserialize::<ParserConfig>()
}
