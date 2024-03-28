use crate::APP_NAME;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
// use serde_json::{from_reader, from_str, to_string};
use std::{
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};
use toml::{from_str, to_string};
use xdg::BaseDirectories;

pub struct Helpers {
    cache_dir: PathBuf,
}

impl Helpers {
    pub fn read_cache<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let cache_file = self.cache_dir.join(key);
        if cache_file.exists() {
            // Use the from_reader
            // let file = std::fs::File::open(cache_file).unwrap();
            let content = std::fs::read_to_string(cache_file).unwrap();
            let data = from_str(&content).unwrap();
            Some(data)
        } else {
            None
        }
    }

    pub fn write_cache<T>(&self, key: &str, data: &T)
    where
        T: Serialize,
    {
        let cache_file = self.cache_dir.join(key);
        let content = to_string(data).unwrap();
        std::fs::write(cache_file, content).unwrap();
    }

    pub fn cache_expired(&self, key: &str, duration: Duration) -> bool {
        let cache_file = self.cache_dir.join(key);
        if cache_file.exists() {
            let metadata = std::fs::metadata(cache_file).unwrap();
            let modified = metadata.modified().unwrap();
            let now = SystemTime::now();
            let elapsed = now.duration_since(modified).unwrap();
            elapsed > duration
        } else {
            true
        }
    }
}

impl Default for Helpers {
    fn default() -> Self {
        let xdg = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let cache_dir = xdg.get_cache_home();
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).unwrap();
        }
        Self { cache_dir }
    }
}
