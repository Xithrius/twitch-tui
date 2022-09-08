use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{handlers::config::StorageConfig, utils::pathing::config_path};

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
    pub fn new(file: &str, config: &StorageConfig) -> Self {
        let file_path = config_path(file);

        if !Path::new(&file_path).exists() {
            let mut items = StorageMap::new();

            for item_key in ITEM_KEYS.iter() {
                let enabled = match *item_key {
                    "channels" => config.channels,
                    "mentions" => config.mentions,
                    _ => panic!("Invalid storage key {}.", item_key),
                };

                items.insert(
                    (*item_key).to_string(),
                    StorageItem {
                        content: vec![],
                        enabled,
                    },
                );
            }

            let storage_str = serde_json::to_string(&items).unwrap();

            let mut file = File::create(&file_path).unwrap();

            file.write_all(storage_str.as_bytes()).unwrap();

            return Self { items, file_path };
        }

        let file_content = read_to_string(&file_path).unwrap();

        let items: StorageMap = serde_json::from_str(&file_content).unwrap();

        Self { items, file_path }
    }

    pub fn dump_data(&self) {
        let storage_str = serde_json::to_string(&self.items).unwrap();

        let mut file = File::create(&self.file_path).unwrap();

        file.write_all(storage_str.as_bytes()).unwrap();
    }

    pub fn add(&mut self, key: &str, value: String) {
        if ITEM_KEYS.contains(&key) {
            if let Some(item) = self.items.get_mut(&key.to_string()) {
                if !item.content.contains(&value) && item.enabled {
                    item.content.push(value);
                }
            }
        } else {
            panic!("Attempted to add value with key {} to JSON storage.", key);
        }
    }

    pub fn get(&self, key: &str) -> Vec<String> {
        if ITEM_KEYS.contains(&key) {
            self.items
                .get(&key.to_string())
                .map_or_else(Vec::new, |item| item.content.clone())
        } else {
            panic!("Attempted to get key {} from JSON storage.", key);
        }
    }
}
