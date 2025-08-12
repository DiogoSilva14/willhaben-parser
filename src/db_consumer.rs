use crate::scraper_parser::{Advert, Apartment};
use log::{debug, error, info};
use rusqlite::{Connection, Error, Result};
use std::collections::HashMap;
use std::fmt;

#[derive(PartialEq)]
enum TableType {
    NewEntries,
    CurrentEntries,
    TransactionLog,
}

impl fmt::Display for TableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let table_name = match self {
            TableType::NewEntries => "new_apartments",
            TableType::CurrentEntries => "current_apartments",
            TableType::TransactionLog => "transaction_log",
        };
        write!(f, "{}", table_name)
    }
}

#[derive(PartialEq)]
pub enum TransactionType {
    Insert,
    Delete,
    Update,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TransactionType {
    fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Insert => "insert",
            TransactionType::Delete => "delete",
            TransactionType::Update => "update",
        }
    }
}

pub struct TransactionEntry {
    pub id: u64,
    pub r#type: TransactionType,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug)]
pub struct DBParser {
    conn: Connection,
}
pub fn get_db_hashmap() -> rusqlite::Result<HashMap<u64, Apartment>> {
    let conn = Connection::open("./.db")?;
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            description,
            coordinates,
            published,
            floor,
            rooms,
            size,
            postcode,
            price,
            url
        FROM apartments",
    )?;

    let apartments = stmt
        .query_map([], |row| {
            Ok(Apartment {
                id: row.get(0)?,
                description: row.get(1)?,
                coordinates: row.get(2)?,
                published: row.get(3)?,
                floor: row.get(4)?,
                rooms: row.get(5)?,
                size: row.get(6)?,
                postcode: row.get(7)?,
                price: row.get(8)?,
                url: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<Apartment>, rusqlite::Error>>()?;

    Ok(apartments
        .into_iter()
        .map(|apart| (apart.id, apart))
        .collect())
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
                 description TEXT,
                 coordinates TEXT,
                 published INTEGER,
                 floor TEXT,
                 rooms INTEGER,
                 size INTEGER,
                 postcode INTEGER,
                 price REAL,
                 url TEXT)
            ",
                (),
            )
            .unwrap();

        db.conn
            .execute(
                "
                 CREATE TABLE IF NOT EXISTS transaction_log
                 (date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                 id INTEGER,
                 type TEXT,
                 field TEXT,
                 old_value TEXT,
                 new_value TEXT)
            ",
                (),
            )
            .unwrap();

        debug!("Database successfully initialized");

        db
    }

    pub fn add_entry(&self, apart: &Apartment, table: TableType) -> Result<usize, Error> {
        debug!("Inserting entry with ID {} on table {}", apart.id, table);

        self.conn.execute(
            format!(
                "
                INSERT INTO
                    {}(
                        id,
                        description,
                        coordinates,
                        published,
                        floor,
                        rooms,
                        size,
                        postcode,
                        price,
                        url)
                VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                table
            )
            .as_str(),
            (
                &apart.id,
                &apart.description,
                &apart.coordinates,
                &apart.published,
                &apart.floor,
                &apart.rooms,
                &apart.size,
                &apart.postcode,
                &apart.price,
                &apart.url,
            ),
        )?;

        if table != TableType::CurrentEntries {
            return Ok(1);
        };

        let apart_json = match serde_json::to_string(apart) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to serialize apartment: {}", e);
                return Ok(1);
            }
        };

        match self.conn.execute(
            format!(
                "
            INSERT INTO {}(
                id,
                type,
                new_value
            ) VALUES (?1, ?2, ?3)
            ",
                TableType::TransactionLog
            )
            .as_str(),
            (apart.id, TransactionType::Insert.as_str(), apart_json),
        ) {
            Ok(_) => {}
            Err(e) => error!("Failed to log transaction: {}", e),
        }

        Ok(1)
    }

    fn delete_entry_by_id(&self, id: u64, table: TableType) -> Result<usize, Error> {
        debug!("Removing ID {} from table {}", id, table);
        let mut apart = Apartment::default();

        if table == TableType::CurrentEntries {
            let apart = self.get_apart_by_id(id, TableType::CurrentEntries)?;
        }

        self.conn.execute(
            format!(
                "
            DELETE FROM {} WHERE id = ?1
                ",
                table
            )
            .as_str(),
            (id,),
        )?;

        if table != TableType::CurrentEntries {
            return Ok(1);
        };

        match self.conn.execute(
            format!(
                "
            INSERT INTO {}(
                id,
                type,
                old_value
            ) VALUES (?1, ?2, ?3)
            ",
                TableType::TransactionLog
            )
            .as_str(),
            (
                apart.id,
                TransactionType::Delete.as_str(),
                format!("{:?}", apart),
            ),
        ) {
            Ok(_) => {}
            Err(e) => error!("Failed to log transaction: {}", e),
        }

        Ok(1)
    }

    fn update_field<T: std::fmt::Display + rusqlite::ToSql + std::cmp::PartialEq>(
        &self,
        id: u64,
        table: TableType,
        field_name: &str,
        old_value: Option<&T>,
        new_value: Option<&T>,
    ) -> Result<usize, Error> {
        debug!("Checking if values are different");

        if old_value == new_value {
            debug!("Fields have the same value");
            return Ok(0);
        }

        debug!("Updating {} of ID {} on table {}", field_name, id, table);

        self.conn.execute(
            format!(
                "
            UPDATE {}
            SET
                {} = ?1
            WHERE id = ?2
                ",
                table, field_name
            )
            .as_str(),
            (&new_value, id),
        )?;

        if table != TableType::CurrentEntries {
            return Ok(1);
        };

        match self.conn.execute(
            format!(
                "
            INSERT INTO {}(
                id,
                type,
                field,
                old_value,
                new_value
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            ",
                TableType::TransactionLog
            )
            .as_str(),
            (
                id,
                TransactionType::Update.as_str(),
                field_name,
                &old_value,
                &new_value,
            ),
        ) {
            Ok(_) => {}
            Err(e) => error!("Failed to log transaction: {}", e),
        }

        Ok(1)
    }

    fn update_entry(&self, old_entry: &Apartment, new_entry: &Apartment) -> Result<usize, Error> {
        assert_eq!(
            old_entry.id, new_entry.id,
            "Trying to update apartment with different ID's"
        );
        debug!("Updating apartment with ID {}", old_entry.id);

        self.update_field(
            old_entry.id,
            TableType::CurrentEntries,
            "description",
            Some(&old_entry.description).as_ref(),
            Some(&new_entry.description).as_ref(),
        )?;
        if new_entry.coordinates.is_some() {
            self.update_field(
                old_entry.id,
                TableType::CurrentEntries,
                "coordinates",
                old_entry.coordinates.as_ref(),
                new_entry.coordinates.as_ref(),
            )?;
        }
        if new_entry.published.is_some() {
            self.update_field(
                old_entry.id,
                TableType::CurrentEntries,
                "published",
                old_entry.published.as_ref(),
                new_entry.published.as_ref(),
            )?;
        }
        if new_entry.floor.is_some() {
            self.update_field(
                old_entry.id,
                TableType::CurrentEntries,
                "floor",
                old_entry.floor.as_ref(),
                new_entry.floor.as_ref(),
            )?;
        }
        if new_entry.rooms.is_some() {
            self.update_field(
                old_entry.id,
                TableType::CurrentEntries,
                "rooms",
                old_entry.rooms.as_ref(),
                new_entry.rooms.as_ref(),
            )?;
        }
        if new_entry.rooms.is_some() {
            self.update_field(
                old_entry.id,
                TableType::CurrentEntries,
                "size",
                old_entry.size.as_ref(),
                new_entry.size.as_ref(),
            )?;
        }
        if new_entry.rooms.is_some() {
            self.update_field(
                old_entry.id,
                TableType::CurrentEntries,
                "postcode",
                old_entry.postcode.as_ref(),
                new_entry.postcode.as_ref(),
            )?;
        }
        if new_entry.rooms.is_some() {
            self.update_field(
                old_entry.id,
                TableType::CurrentEntries,
                "price",
                old_entry.price.as_ref(),
                new_entry.price.as_ref(),
            )?;
        }
        if new_entry.rooms.is_some() {
            self.update_field(
                old_entry.id,
                TableType::CurrentEntries,
                "url",
                old_entry.url.as_ref(),
                new_entry.url.as_ref(),
            )?;
        }

        Ok(1)
    }

    fn get_apart_by_id(&self, id: u64, table: TableType) -> Result<Apartment> {
        self.conn.query_row(
            format!(
                "SELECT
                id,
                description,
                coordinates,
                published,
                floor,
                rooms,
                size,
                postcode,
                price,
                url
            FROM {} WHERE id = ?1
            ",
                table
            )
            .as_str(),
            (id,),
            |row| {
                Ok(Apartment {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    coordinates: row.get(2)?,
                    published: row.get(3)?,
                    floor: row.get(4)?,
                    rooms: row.get(5)?,
                    size: row.get(6)?,
                    postcode: row.get(7)?,
                    price: row.get(8)?,
                    url: row.get(9)?,
                })
            },
        )
    }

    fn get_table_values(&self, table: TableType) -> Result<Vec<Apartment>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare(
                format!(
                    "
            SELECT
                id,
                description,
                coordinates,
                published,
                floor,
                rooms,
                size,
                postcode,
                price,
                url
            FROM {}",
                    table
                )
                .as_str(),
            )
            .unwrap();

        stmt.query_map([], |row| {
            Ok(Apartment {
                id: row.get(0)?,
                description: row.get(1)?,
                coordinates: row.get(2)?,
                published: row.get(3)?,
                floor: row.get(4)?,
                rooms: row.get(5)?,
                size: row.get(6)?,
                postcode: row.get(7)?,
                price: row.get(8)?,
                url: row.get(9)?,
            })
        })
        .unwrap()
        .collect()
    }

    pub fn consume_adverts(&self, adverts: &Vec<Advert>) {
        info!("Consuming parsed averts into database");

        for advert in adverts {
            let mut apart = Apartment::default();

            apart.id = advert.id.parse().unwrap();
            apart.description = advert.description.clone();

            for attr in &advert.attributes.attribute {
                debug!("{}: {:?}", attr.name, attr.values[0]);

                match attr.name.as_str() {
                    "COORDINATES" => apart.coordinates = Some(attr.values[0].clone()),
                    "PUBLISHED" => apart.published = Some(attr.values[0].parse().unwrap()),
                    "FLOOR" => apart.floor = Some(attr.values[0].clone()),
                    "NUMBER_OF_ROOMS" => apart.rooms = Some(attr.values[0].parse().unwrap()),
                    "ESTATE_SIZE/LIVING_AREA" => apart.size = Some(attr.values[0].parse().unwrap()),
                    "POSTCODE" => apart.postcode = Some(attr.values[0].parse().unwrap()),
                    "PRICE" => apart.price = Some(attr.values[0].parse().unwrap()),
                    "SEO_URL" => {
                        let mut val = attr.values[0].clone();
                        val.insert_str(0, "https://www.willhaben.at/");
                        apart.url = Some(val);
                    }
                    &_ => debug!("Ignored attr {}", attr.name.as_str()),
                }
            }

            debug!("Parsed apartment: {:?}", apart);

            match self.add_entry(&apart, TableType::NewEntries) {
                Ok(_) => debug!("Inserted apartment with ID {}", apart.id),
                Err(e) => error!(
                    "Failed to insert apartment with ID {} due to: {}",
                    apart.id, e
                ),
            }
        }
    }

    pub fn process(&self) {
        info!("Processing new apartments");

        let new_aparts: Vec<Apartment> = match self.get_table_values(TableType::NewEntries) {
            Ok(vec) => vec,
            Err(e) => {
                error!("Failed to fetch new apartment entries: {}", e);
                return;
            }
        };

        for apart in new_aparts {
            debug!("New apartments entry: {}", apart.id);
            debug!("Checking if apartment is new");

            let stored_entry = self.get_apart_by_id(apart.id, TableType::CurrentEntries);

            match stored_entry {
                Ok(stored_apart) => {
                    self.update_entry(&stored_apart, &apart).unwrap();
                    {}
                }
                Err(Error::QueryReturnedNoRows) => {
                    self.add_entry(&apart, TableType::CurrentEntries).unwrap();
                    {}
                }
                Err(e) => {
                    error!("An error ocurred {}", e);
                    continue;
                }
            }

            self.delete_entry_by_id(apart.id, TableType::NewEntries)
                .unwrap();
        }

        info!("Finished processing new apartments");
    }
}
