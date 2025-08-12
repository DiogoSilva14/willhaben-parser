mod db_consumer;
mod parser_config;
mod scraper_parser;

use crate::db_consumer::DBParser;
use crate::parser_config::get_parser_config;
use crate::scraper_parser::{adverts_to_aparts, get_adverts, update_database};
use clap::Parser;
use env_logger::Builder;
use log::{LevelFilter, debug, error, info};
use std::process::ExitCode;
use std::{thread, time};

#[derive(Parser, Debug)]
#[command(
    name = "willhaben-parser",
    version = "0.1",
    author = "Diogo Silva <diogo.silva.pt14@gmail.com>",
    about = "A CLI utility to scrape willhaben"
)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.verbose {
        Builder::new().filter_level(LevelFilter::Debug).init();
    } else {
        Builder::new().filter_level(LevelFilter::Info).init();
    }

    debug!("Starting...");

    let config = match get_parser_config() {
        Ok(settings) => settings,
        Err(error) => {
            error!(
                "Failed to parse the configuration file with error: {}",
                error
            );
            return ExitCode::from(ExitCode::FAILURE);
        }
    };

    info!("Sucessfully loaded config file!");
    debug!("{:?}", config);

    let data_processer = thread::spawn(move || {
        loop {
            let db = DBParser::new();
            let adverts = get_adverts(&config);
            let apartments = adverts_to_aparts(adverts);
            update_database(apartments);

            //            db.consume_adverts(&adverts);
            //            db.process();

            thread::sleep(time::Duration::from_secs(20));
        }
    });

    data_processer.join().unwrap();

    ExitCode::SUCCESS
}
