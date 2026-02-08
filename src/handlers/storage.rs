use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{File, create_dir_all, read_to_string},
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::config::{SharedCoreConfig, get_data_dir};

static ITEM_KEYS: &[&str] = &["channels", "mentions", "chatters"];
const DEFAULT_STORAGE_FILE_NAME: &str = "storage.json";

pub type SharedStorage = Rc<RefCell<Storage>>;
type StorageMap = HashMap<String, StorageItem>;

#[derive(Debug)]
pub struct Storage {
    items: StorageMap,
    file_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageItem {
    content: Vec<String>,
    enabled: bool,
}

impl Storage {
    pub fn new(config: &SharedCoreConfig) -> Self {
        let twitch_channel = &config.twitch.channel;
        let storage_config = config.storage.clone();
        let storage_parent_path = get_data_dir();
        if !storage_parent_path.exists() {
            create_dir_all(&storage_parent_path).unwrap();
        }
        let storage_path = storage_parent_path.join(DEFAULT_STORAGE_FILE_NAME);

        if Path::new(&storage_path).exists() {
            let file_content = read_to_string(&storage_path).unwrap();

            let items: StorageMap = serde_json::from_str(&file_content).unwrap();

            return Self {
                items,
                file_path: storage_path,
            };
        }

        let mut items = StorageMap::new();

        for item_key in ITEM_KEYS {
            let enabled = match *item_key {
                "channels" => storage_config.channels,
                "mentions" => storage_config.mentions,
                "chatters" => storage_config.chatters,
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

        if let Some(channels) = items.get_mut("channels") {
            if !channels.content.contains(twitch_channel) {
                channels.content.push(twitch_channel.clone());
            }
        }

        let storage_str = serde_json::to_string(&items).unwrap();
        let mut file = File::create(&storage_path).unwrap();
        file.write_all(storage_str.as_bytes()).unwrap();

        Self {
            items,
            file_path: storage_path,
        }
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
