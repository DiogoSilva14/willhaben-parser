use crate::db_consumer::{TransactionEntry, TransactionType, get_db_hashmap};
use crate::parser_config::ParserConfig;
use log::{debug, info};
use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub id: u64,
    pub description: String,
    pub coordinates: Option<String>,
    pub published: Option<u64>,
    pub floor: Option<String>,
    pub rooms: Option<u32>,
    pub size: Option<u32>,
    pub postcode: Option<u32>,
    pub price: Option<f32>,
    pub url: Option<String>,
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

pub fn adverts_to_aparts(mut adverts: Vec<Advert>) -> Vec<Apartment> {
    info!("Parsing adverts");

    let apartments: Vec<Apartment> = Vec::new();

    for advert in adverts {
        let mut apart = Apartment::default();

        apart.id = advert.id.parse().unwrap();
        apart.description = advert.description;

        for mut attr in advert.attributes.attribute {
            debug!("{}: {:?}", attr.name, attr.values[0]);

            match attr.name.as_str() {
                "COORDINATES" => apart.coordinates = Some(attr.values.remove(0)),
                "PUBLISHED" => apart.published = Some(attr.values[0].parse().unwrap()),
                "FLOOR" => apart.floor = Some(attr.values.remove(0)),
                "NUMBER_OF_ROOMS" => apart.rooms = Some(attr.values[0].parse().unwrap()),
                "ESTATE_SIZE/LIVING_AREA" => apart.size = Some(attr.values[0].parse().unwrap()),
                "POSTCODE" => apart.postcode = Some(attr.values[0].parse().unwrap()),
                "PRICE" => apart.price = Some(attr.values[0].parse().unwrap()),
                "SEO_URL" => {
                    let mut val = attr.values.remove(0);
                    val.insert_str(0, "https://www.willhaben.at/");
                    apart.url = Some(val);
                }
                &_ => debug!("Ignored attr {}", attr.name.as_str()),
            }
        }

        debug!("Parsed apartment: {:?}", apart);
    }

    return apartments;
}

fn generate_transaction_entry<T: ToString>(
    id: u64,
    field: &str,
    old_field: &Option<T>,
    new_field: &Option<T>,
) -> TransactionEntry {
    let mut old_value: String = String::new();
    let mut new_value: String = String::new();

    if old_field.is_some() {
        old_value = old_field.as_ref().unwrap().to_string();
    }

    if new_field.is_some() {
        new_value = new_field.as_ref().unwrap().to_string();
    }

    TransactionEntry {
        id: id,
        r#type: TransactionType::Update,
        field: Some(field.to_string()),
        old_value: Some(old_value),
        new_value: Some(new_value),
    }
}

fn log_update(old_entry: &Apartment, new_entry: &Apartment) -> Vec<TransactionEntry> {
    assert_eq!(
        old_entry.id, new_entry.id,
        "Trying to update apartment entry with a different ID"
    );

    let mut transactions: Vec<TransactionEntry> = Vec::new();

    if old_entry.description != new_entry.description {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "description",
            &Some(old_entry.description.clone()),
            &Some(new_entry.description.clone()),
        ));
    }

    if old_entry.coordinates != new_entry.coordinates {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "coordinates",
            &old_entry.coordinates,
            &new_entry.coordinates,
        ));
    }

    if old_entry.published != new_entry.published {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "published",
            &old_entry.published,
            &new_entry.published,
        ));
    }

    if old_entry.floor != new_entry.floor {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "floor",
            &old_entry.floor,
            &new_entry.floor,
        ));
    }

    if old_entry.rooms != new_entry.rooms {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "rooms",
            &old_entry.rooms,
            &new_entry.rooms,
        ));
    }

    if old_entry.size != new_entry.size {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "size",
            &old_entry.size,
            &new_entry.size,
        ));
    }

    if old_entry.postcode != new_entry.postcode {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "postcode",
            &old_entry.postcode,
            &new_entry.postcode,
        ));
    }

    if old_entry.price != new_entry.price {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "price",
            &old_entry.price,
            &new_entry.price,
        ));
    }

    if old_entry.url != new_entry.url {
        transactions.push(generate_transaction_entry(
            old_entry.id,
            "url",
            &old_entry.url,
            &new_entry.url,
        ));
    }

    transactions
}

fn log_insert(new_entry: &Apartment) -> TransactionEntry {
    TransactionEntry {
        id: new_entry.id,
        r#type: TransactionType::Insert,
        field: None,
        old_value: None,
        new_value: Some(serde_json::to_string(new_entry).unwrap()),
    }
}

fn log_delete(old_entry: &Apartment) -> TransactionEntry {
    TransactionEntry {
        id: old_entry.id,
        r#type: TransactionType::Delete,
        field: None,
        old_value: Some(serde_json::to_string(old_entry).unwrap()),
        new_value: None,
    }
}

pub fn update_database(mut aparts: Vec<Apartment>) {
    let mut stored_entries = get_db_hashmap().unwrap();
    let mut new_stored_entries: HashMap<u64, Apartment> = HashMap::new();
    let mut transactions: Vec<TransactionEntry> = Vec::new();

    for apart in aparts {
        if stored_entries.contains_key(&apart.id) {
            transactions.append(&mut log_update(&stored_entries[&apart.id], &apart));
        } else {
            transactions.push(log_insert(&apart));
        }

        new_stored_entries.insert(apart.id, apart);
    }

    for entry in stored_entries.keys() {
        if !new_stored_entries.contains_key(entry) {
            transactions.push(log_delete(&stored_entries[entry]));
        }
    }
}
