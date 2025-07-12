mod parser_config;

use crate::parser_config::get_parser_config;
use debug_print::debug_println;
use reqwest;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::process::ExitCode;

const WILLHABEN_WOHNUNG_URL: &str = "https://www.willhaben.at/iad/immobilien/mietwohnungen/wien";

#[derive(Debug, Deserialize)]
struct HtmlBody {
    treeType: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    let config = match get_parser_config() {
        Ok(settings) => settings,
        Err(error) => {
            println!(
                "Failed to parse the configuration file with error: {}",
                error
            );
            return ExitCode::from(ExitCode::FAILURE);
        }
    };

    println!("Sucessfully loaded config file!");

    debug_println!("{:?}", config);

    let client = reqwest::Client::new();

    let params = [("rows", 90)];

    let body_html = client
        .get(WILLHABEN_WOHNUNG_URL)
        .query(&params)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let data: String = Html::parse_document(body_html.as_str())
        .select(&Selector::parse("#__NEXT_DATA__").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>();

    println!("{}", data);

    ExitCode::SUCCESS
}
