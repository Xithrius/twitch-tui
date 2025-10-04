use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{File, read_to_string},
    io::Write,
    path::Path,
    rc::Rc,
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};

use crate::{handlers::config::StorageConfig, utils::pathing::config_path};

static ITEM_KEYS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["channels", "mentions", "chatters"]);

pub type SharedStorage = Rc<RefCell<Storage>>;
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
                    "chatters" => config.chatters,
                    _ => panic!("Invalid storage key {item_key}."),
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
            if let Some(item) = self.items.get_mut(key) {
                if item.enabled {
                    if let Some(position) = item.content.iter().position(|x| x == &value) {
                        item.content.remove(position);
                    }
                    item.content.push(value);
                }
            }
        } else {
            panic!("Attempted to add value with key {key} to JSON storage.");
        }
    }

    pub fn get(&self, key: &str) -> Vec<String> {
        if ITEM_KEYS.contains(&key) {
            self.items
                .get(key)
                .map_or_else(Vec::new, |item| item.content.clone())
        } else {
            panic!("Attempted to get key {key} from JSON storage.");
        }
    }

    pub fn get_last_n(&self, key: &str, n: usize, reverse: bool) -> Vec<String> {
        let items = self.get(key);

        let mut out = if items.len() <= n {
            items
        } else {
            items[items.len() - n..].to_vec()
        };

        if reverse {
            out.reverse();
        }

        out
    }

    pub fn contains(&self, key: &str, value: &str) -> bool {
        self.get(key).contains(&value.to_string())
    }

    pub fn remove_inner_with(&mut self, key: &str, value: &str) -> String {
        if ITEM_KEYS.contains(&key) {
            let item = self.items.get_mut(key).unwrap();

            if let Some(position) = item.content.iter().position(|x| x == value) {
                item.content.remove(position)
            } else {
                panic!("Item {value} could not be found within {key}");
            }
        } else {
            panic!("Attempted to add value with key {key} to JSON storage.");
        }
    }
}
