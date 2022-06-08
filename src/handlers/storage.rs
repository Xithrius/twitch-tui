use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::handlers::config::StorageConfig;

lazy_static! {
    pub static ref ITEM_KEYS: Vec<&'static str> = vec!["channels", "mentions"];
}

type StorageMap = HashMap<String, StorageItem>;

#[derive(Debug)]
pub struct Storage {
    items: StorageMap,
    file_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageItem {
    content: Vec<String>,
    enabled: bool,
}

impl Storage {
    pub fn new(file_path: String, config: StorageConfig) -> Self {
        if !Path::new(&file_path).exists() {
            let mut items = StorageMap::new();

            for item_key in ITEM_KEYS.iter() {
                let enabled = match *item_key {
                    "channels" => config.channels,
                    "mentions" => config.mentions,
                    _ => panic!("Invalid storage key {}.", item_key),
                };

                items.insert(
                    item_key.to_string(),
                    StorageItem {
                        content: vec![],
                        enabled,
                    },
                );
            }

            let storage_str = serde_json::to_string(&items).unwrap();

            let mut file = File::create(&file_path).unwrap();

            file.write_all(storage_str.as_bytes()).unwrap();

            return Storage { items, file_path };
        }

        let file_content = read_to_string(&file_path).unwrap();

        let items: StorageMap = serde_json::from_str(&file_content).unwrap();

        Storage { items, file_path }
    }

    pub fn dump_data(&self) {
        let storage_str = serde_json::to_string(&self.items).unwrap();

        let mut file = File::create(&self.file_path).unwrap();

        file.write_all(storage_str.as_bytes()).unwrap();
    }

    pub fn add(&mut self, key: String, value: String) {
        if ITEM_KEYS.contains(&key.as_str()) {
            if let Some(item) = self.items.get_mut(&key) {
                if !item.content.contains(&value) && item.enabled {
                    item.content.push(value);
                }
            }
        } else {
            panic!("Attempted to add value with key {} to JSON storage.", key);
        }
    }

    pub fn get(&self, key: String) -> Vec<String> {
        if ITEM_KEYS.contains(&key.as_str()) {
            if let Some(item) = self.items.get(&key) {
                item.content.clone()
            } else {
                vec![]
            }
        } else {
            panic!("Attempted to get key {} from JSON storage.", key);
        }
    }
}
