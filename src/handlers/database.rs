use regex::Regex;
use rusqlite::Connection as SqliteConnection;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Database {
    filters: DatabaseTable<Regex>,
    channels: DatabaseTable<String>,
    mentions: DatabaseTable<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DatabaseTable<T> {
    raw_content: Vec<String>,
    converted_content: Vec<T>,
    max_track: Option<u32>,
    enabled: bool,
}

#[allow(dead_code)]
impl Database {
    pub fn new(conn: SqliteConnection) -> Self {
        let get_table_data = |conn: &SqliteConnection, table: &str| -> Vec<String> {
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

            conn.prepare(format!("SELECT content FROM {}", table).as_str())
                .unwrap()
                .query_map([], |row| {
                    let item: String = row.get(0).unwrap();

                    Ok(item)
                })
                .unwrap()
                .flatten()
                .collect::<Vec<String>>()
        };

        let default_converter = |s: String| -> String { s };

        Self {
            filters: DatabaseTable::new(get_table_data(&conn, "filters"), |s: String| -> Regex {
                Regex::new(&s).unwrap()
            }),
            channels: DatabaseTable::new(get_table_data(&conn, "channels"), default_converter),
            mentions: DatabaseTable::new(get_table_data(&conn, "mentions"), default_converter),
        }
    }

    pub fn dump(_conn: SqliteConnection) {
        todo!()
    }
}

#[allow(dead_code)]
impl<T> DatabaseTable<T> {
    pub fn new<F>(raw_content: Vec<String>, converter: F) -> Self
    where
        F: Fn(String) -> T,
    {
        let converted_content = raw_content
            .iter()
            .map(|f| converter(f.to_string()))
            .collect::<Vec<T>>();

        Self {
            raw_content,
            converted_content,
            max_track: None,
            enabled: true,
        }
    }

    fn add(&mut self, data: &str) {
        self.raw_content.push(data.to_string());
    }

    fn remove(&mut self, data: &str) {
        if let Some(index) = self.raw_content.iter().position(|x| x.as_str() == data) {
            self.raw_content.remove(index);
        }
    }

    fn edit(&mut self, data_old: &str, data_new: &str) {
        self.remove(data_old);
        self.add(data_new);
    }

    fn list(self) -> Vec<String> {
        self.raw_content
    }

    fn contains(&self, data: String) -> bool {
        if self.enabled {
            // for re in &self.captures {
            //     if re.is_match(&data) {
            //         return true;
            //     }
            // }

            for item in &self.raw_content {
                if &data == item {
                    return true;
                }
            }
        }

        false
    }
}
