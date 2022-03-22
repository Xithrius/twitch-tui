use std::collections::{hash_map::Entry::Occupied, HashMap};

use anyhow::{bail, Error, Result};
use lazy_static::lazy_static;
use rusqlite::{params, Connection as SqliteConnection};

lazy_static! {
    pub static ref TABLES: Vec<&'static str> = vec!["filters", "channels", "mentions"];
}

#[derive(Debug)]
pub struct Database {
    conn: SqliteConnection,
    tables: HashMap<String, DatabaseTable>,
}

#[derive(Debug, Clone)]
pub struct DatabaseTable {
    content: Vec<String>,
    _enabled: bool,
}

impl Database {
    pub fn new(conn: SqliteConnection) -> Self {
        let mut tables = HashMap::new();

        for table in TABLES.iter() {
            conn.execute(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY,
                    content TEXT NOT NULL,
                )",
                    table
                )
                .as_str(),
                [],
            )
            .unwrap();

            let data = conn
                .prepare(format!("SELECT content FROM {}", table).as_str())
                .unwrap()
                .query_map([], |row| {
                    let item: String = row.get(0).unwrap();

                    Ok(item)
                })
                .unwrap()
                .flatten()
                .collect::<Vec<String>>();

            tables.insert(table.to_string(), DatabaseTable::new(data));
        }

        Self { conn, tables }
    }

    pub fn add(&mut self, table: String, item: String) -> Result<(), Error> {
        if TABLES.contains(&table.as_str()) {
            bail!("Table '{table}' does not exist within static vector tables.");
        }

        if let Occupied(mut m) = self.tables.entry(table.to_string()) {
            let content = &mut m.get_mut().content;

            if content.contains(&item) {
                bail!("Table '{table}' already contains item '{item}'.");
            } else {
                content.push(item.clone());

                self.conn
                    .execute(
                        &format!("INSERT INTO {table} (content) VALUES (?1)"),
                        params![item],
                    )
                    .unwrap();

                Ok(())
            }
        } else {
            bail!("Table '{table}' for some reason doesn't exist within database tables.");
        }
    }
}

impl DatabaseTable {
    pub fn new(data: Vec<String>) -> Self {
        Self {
            content: data,
            _enabled: true,
        }
    }
}
