mod db_consumer;
mod parser_config;
mod scraper_parser;

use crate::db_consumer::DBParser;
use crate::parser_config::get_parser_config;
use crate::scraper_parser::get_adverts;
use log::{debug, error, info};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    debug!("Starting...");

    //    let config = match get_parser_config() {
    //        Ok(settings) => settings,
    //        Err(error) => {
    //            error!(
    //                "Failed to parse the configuration file with error: {}",
    //                error
    //            );
    //            return ExitCode::from(ExitCode::FAILURE);
    //        }
    //    };
    //
    //    info!("Sucessfully loaded config file!");
    //    debug!("{:?}", config);

    let db = DBParser::new();

    //    let search_result = get_adverts(&config).await;
    //
    //    info!(
    //        "Got {} results",
    //        search_result.advert_summary_list.advert_summary.len()
    //    );

    ExitCode::SUCCESS
}
