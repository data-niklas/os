use super::Source;
use crate::helpers::Helpers;
use rayon::prelude::*;
use ureq::get;

use serde::{Deserialize, Serialize};

use std::time::Duration;
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

const fn _default_icons() -> bool {
    true
}
fn _default_cache_duration() -> Duration {
    // 24 hours
    Duration::from_secs(60 * 60 * 4)
}

fn _default_user_agent() -> String {
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0".to_string()
}

#[derive(Deserialize)]
pub struct DuckduckgoConfig {
    #[serde(default = "_default_icons")]
    pub icons: bool,
    #[serde(default = "_default_cache_duration")]
    pub cache_duration: Duration,
    #[serde(default = "_default_user_agent")]
    pub user_agent: String,
}

pub struct DuckduckgoSource {}

impl DuckduckgoSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl Source for DuckduckgoSource {
    fn name(&self) -> &'static str {
        "duckduckgo"
    }

    fn init(&mut self, config: &toml::Table, helpers: &Helpers) {
        let config: DuckduckgoConfig = config.clone().try_into().unwrap();
        let cache_duration = config.cache_duration;

        if !helpers.cache_expired(self.name(), cache_duration) {}
    }

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn fuzzy_matcher::FuzzyMatcher + Send + Sync>,
    ) -> Vec<crate::model::SearchItem> {
        vec![]
    }
}
