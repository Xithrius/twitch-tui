use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{File, create_dir_all, read_to_string},
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
};

use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, bail},
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
    pub fn new(config: &SharedCoreConfig) -> Result<Self> {
        let twitch_channel = &config.twitch.channel;
        let storage_config = config.storage.clone();
        let storage_parent_path = get_data_dir();

        if !storage_parent_path.exists() {
            create_dir_all(&storage_parent_path).context("Failed to create storage directory")?;
        }

        let storage_path = storage_parent_path.join(DEFAULT_STORAGE_FILE_NAME);

        if Path::new(&storage_path).exists() {
            let file_content =
                read_to_string(&storage_path).context("Failed to read storage file")?;

            let items: StorageMap =
                serde_json::from_str(&file_content).context("Failed to parse storage file")?;

            return Ok(Self {
                items,
                file_path: storage_path,
            });
        }

        let mut items = StorageMap::new();

        for item_key in ITEM_KEYS {
            let enabled = match *item_key {
                "channels" => storage_config.channels,
                "mentions" => storage_config.mentions,
                "chatters" => storage_config.chatters,
                _ => bail!("Invalid storage key {item_key}."),
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

        let storage_str =
            serde_json::to_string(&items).context("Failed to serialize storage items")?;
        let mut file = File::create(&storage_path).context("Failed to create storage file")?;
        file.write_all(storage_str.as_bytes())
            .context("Failed to write storage to file")?;

        Ok(Self {
            items,
            file_path: storage_path,
        })
    }

    pub fn dump_data(&self) -> Result<()> {
        let storage_str =
            serde_json::to_string(&self.items).context("Failed to serialize storage items")?;

        let mut file = File::create(&self.file_path).context("Failed to create storage file")?;

        file.write_all(storage_str.as_bytes())
            .context("Failed to write storage to file")?;

        Ok(())
    }

    pub fn add(&mut self, key: &str, value: String) -> Result<()> {
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
            bail!("Attempted to add value with key {key} to JSON storage.");
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Vec<String>> {
        if ITEM_KEYS.contains(&key) {
            let items = self
                .items
                .get(key)
                .map_or_else(Vec::default, |item| item.content.clone());
            Ok(items)
        } else {
            bail!("Attempted to get key {key} from JSON storage.");
        }
    }

    pub fn get_last_n(&self, key: &str, n: usize, reverse: bool) -> Result<Vec<String>> {
        let items = self.get(key)?;

        let mut out = if items.len() <= n {
            items
        } else {
            items[items.len() - n..].to_vec()
        };

        if reverse {
            out.reverse();
        }

        Ok(out)
    }

    pub fn remove_inner_with(&mut self, key: &str, value: &str) -> Result<Option<String>> {
        if !ITEM_KEYS.contains(&key) {
            bail!("Key {key} is not a valid storage key.");
        }

        let item = self
            .items
            .get_mut(key)
            .context(format!("Item {value} could not be found within key {key}"))?;

        let Some(position) = item.content.iter().position(|x| x == value) else {
            return Ok(None);
        };

        Ok(Some(item.content.remove(position)))
    }
}
