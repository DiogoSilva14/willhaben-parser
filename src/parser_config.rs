use config::{Config, ConfigError};
use log::{info, warn};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ParserConfig {
    pub interval: String,
    pub sender_email: Option<Email>,
    pub search_criteria: SearchCriteria,
}

#[derive(Debug, Deserialize)]
pub struct Email {
    email: String,
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchCriteria {
    area: Option<RangeCriteria>,
    price: Option<RangeCriteria>,
}

#[derive(Debug, Deserialize)]
pub struct RangeCriteria {
    min: Option<f64>,
    max: Option<f64>,
}

impl SearchCriteria {
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();

        match &self.area {
            Some(criteria) => {
                match &criteria.min {
                    Some(min) => {
                        map.insert("ESTATE_SIZE/LIVING_AREA_FROM".to_string(), min.to_string());
                    }
                    None => {
                        warn!("Minimum area was not specified")
                    }
                }
                match &criteria.max {
                    Some(max) => {
                        map.insert("ESTATE_SIZE/LIVING_AREA_TO".to_string(), max.to_string());
                    }
                    None => {
                        warn!("Maximum area was not specified")
                    }
                }
            }
            None => {
                warn!("Area criteria was not specified")
            }
        }

        match &self.price {
            Some(criteria) => {
                match &criteria.min {
                    Some(min) => {
                        map.insert("PRICE_FROM".to_string(), min.to_string());
                    }
                    None => {
                        warn!("Minimum price was not specified")
                    }
                }
                match &criteria.max {
                    Some(max) => {
                        map.insert("PRICE_TO".to_string(), max.to_string());
                    }
                    None => {
                        warn!("Maximum price was not specified")
                    }
                }
            }
            None => {
                warn!("Price criteria was not specified")
            }
        }

        map
    }
}

pub fn get_parser_config() -> Result<ParserConfig, ConfigError> {
    info!("Retrieving parser configuration");

    Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()
        .unwrap()
        .try_deserialize::<ParserConfig>()
}
