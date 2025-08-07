use crate::parser_config::ParserConfig;
use log::{debug, info};
use reqwest;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde::{Deserialize, Serialize};

const WILLHABEN_WOHNUNG_URL: &str = "https://www.willhaben.at/iad/immobilien/mietwohnungen/wien";

const DEFAULT_PARAMS: [(&str, &str); 5] = [
    // Max value
    ("rows", "200"),
    // AreaID for Wien
    ("areaId", "900"),
    // Property types
    ("PROPERTY_TYPE", "110"),
    ("PROPERTY_TYPE", "105"),
    ("PROPERTY_TYPE", "3"),
];

#[derive(Debug, Deserialize)]
pub struct Search {
    props: Props,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
    page_props: PageProps,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProps {
    search_result: SearchResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    id: u32,
    pub rows_found: u32,
    rows_returned: u32,
    pub advert_summary_list: AdvertSummaryList,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertSummaryList {
    pub advert_summary: Vec<Advert>,
}

#[derive(Debug, Deserialize)]
pub struct Advert {
    pub id: String,
    pub description: String,
    pub attributes: Attribute,
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    pub attribute: Vec<Attr>,
}

#[derive(Debug, Deserialize)]
pub struct Attr {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Apartment {
    id: u64,
    description: String,
    coordinates: Option<String>,
    published: Option<u64>,
    floor: Option<String>,
    rooms: Option<u32>,
    size: Option<u32>,
    postcode: Option<u32>,
    price: Option<f32>,
    url: Option<String>,
}

impl Default for Apartment {
    fn default() -> Apartment {
        Apartment {
            id: 0,
            description: String::new(),
            coordinates: (None),
            published: (None),
            floor: (None),
            rooms: (None),
            size: (None),
            postcode: (None),
            price: (None),
            url: (None),
        }
    }
}

pub fn get_adverts(config: &ParserConfig) -> Vec<Advert> {
    let client = reqwest::blocking::Client::new();
    let mut params_hashmap = config.search_criteria.to_hashmap();
    let mut search_result: SearchResult = SearchResult {
        id: 0,
        rows_found: 0,
        rows_returned: 0,
        advert_summary_list: AdvertSummaryList {
            advert_summary: Vec::<Advert>::new(),
        },
    };
    let mut page_id: u32 = 1;

    info!("Retrieving adverts");

    debug!("Adding default parameters: {:?}", DEFAULT_PARAMS);

    for param in DEFAULT_PARAMS {
        params_hashmap.insert(param.0.to_string(), param.1.to_string());
    }

    debug!("Parameters for request: {:?}", params_hashmap);

    loop {
        info!("Requesting page {}", page_id);

        params_hashmap.insert("page".to_string(), page_id.to_string());

        let body_html = client
            .get(WILLHABEN_WOHNUNG_URL)
            .query(&params_hashmap)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let data: String = Html::parse_document(body_html.as_str())
            .select(&Selector::parse("#__NEXT_DATA__").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let search_return: Search = serde_json::from_str(data.as_str()).unwrap();

        debug!(
            "Got page {} with {} rows",
            page_id, search_return.props.page_props.search_result.rows_returned
        );

        search_result.rows_returned += search_return.props.page_props.search_result.rows_returned;

        for advert in search_return
            .props
            .page_props
            .search_result
            .advert_summary_list
            .advert_summary
        {
            search_result
                .advert_summary_list
                .advert_summary
                .push(advert);
        }

        if search_return.props.page_props.search_result.rows_returned
            < params_hashmap.get("rows").unwrap().parse::<u32>().unwrap()
        {
            break;
        }

        page_id += 1;
    }

    info!("Finished retrieving adverts");

    search_result.advert_summary_list.advert_summary
}
