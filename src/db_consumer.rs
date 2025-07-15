use crate::scraper_parser::Advert;
use log::{debug, error, info};
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct DBParser {
    conn: Connection,
}

#[derive(Debug)]
pub struct Apartment {
    id: Option<u32>,
    description: Option<String>,
    coordinates: Option<String>,
    published: Option<u64>,
    floor: Option<String>,
    rooms: Option<u32>,
    size: Option<u32>,
    postcode: Option<u32>,
    price: Option<u32>,
    url: Option<String>,
}

impl Default for Apartment {
    fn default() -> Apartment {
        Apartment {
            id: (None),
            description: (None),
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

impl DBParser {
    pub fn new() -> Self {
        debug!("Opening SQLite database");

        let db = Self {
            conn: Connection::open("./.db").unwrap(),
        };

        debug!("Making sure table is created");

        db.conn
            .execute(
                "
                 CREATE TABLE IF NOT EXISTS apartments
                 (id INTEGER PRIMARY KEY,
                 coordinates TEXT,
                 published INTEGER,
                 floor TEXT,
                 rooms INTEGER,
                 size INTEGER,
                 postcode INTEGER,
                 price REAL,
                 url TEXT,
                 json TEXT)
            ",
                (),
            )
            .unwrap();

        debug!("Database successfully initialized");

        db
    }

    pub fn consume(adverts: &Vec<Advert>) {
        for advert in adverts {
            let mut apart = Apartment::default();

            apart.id = Some(advert.id);
            apart.description = Some(advert.description.clone());
        }
    }
}
