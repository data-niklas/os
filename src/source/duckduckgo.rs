use super::Source;
use crate::helpers::Helpers;
use rayon::prelude::*;
use scraper::{Html, Selector};
use ureq::get;

use serde::{Deserialize, Serialize};

use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};
use urlencoding;

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
    #[serde(default = "_default_cache_duration")]
    pub cache_duration: Duration,
    #[serde(default = "_default_user_agent")]
    pub user_agent: String,
}

#[derive(Clone)]
struct SearchResult {
    title: String,
    url: String,
}

#[derive(Clone)]
enum DuckduckgoSearchState {
    None,
    Searched(Vec<SearchResult>),
}

pub struct DuckduckgoSource {
    state: Arc<RwLock<DuckduckgoSearchState>>,
    config: Option<DuckduckgoConfig>,
}

impl DuckduckgoSource {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(DuckduckgoSearchState::None)),
            config: None,
        }
    }
}

impl Source for DuckduckgoSource {
    fn name(&self) -> &'static str {
        "duckduckgo"
    }

    fn init(&mut self, config: &toml::Table, helpers: Arc<Helpers>) {
        let config: DuckduckgoConfig = config.clone().try_into().unwrap();
        let cache_duration = config.cache_duration;

        if !helpers.cache_expired(self.name(), cache_duration) {}
        self.config = Some(config);
    }

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn fuzzy_matcher::FuzzyMatcher + Send + Sync>,
    ) -> Vec<crate::model::SearchItem> {
        let state = self.state.clone();
        let current_state = { state.read().unwrap().clone() };
        match &current_state {
            DuckduckgoSearchState::None => {
                let user_agent = self.config.as_ref().unwrap().user_agent.clone();
                let query = query.to_string();
                let item = crate::model::SearchItem {
                    id: self.name().to_string(),
                    title: Some("Search DuckDuckGo".to_string()),
                    subtitle: Some(query.to_string()),
                    icon: None,
                    image: None,
                    score: 0,
                    source: self.name(),
                    action: Box::new(move |os| {
                        let user_agent = user_agent.clone();
                        let state = state.clone();
                        let url = format!("https://html.duckduckgo.com/html/?q={}", query);
                        let response = get(&url)
                            .set("User-Agent", &user_agent)
                            .call()
                            .unwrap()
                            .into_string()
                            .unwrap();
                        let html = Html::parse_document(&response);
                        let selector = Selector::parse(".result__body").unwrap();
                        let mut results = vec![];
                        for element in html.select(&selector) {
                            let title_link = element
                                .select(&Selector::parse(".result__title > a").unwrap())
                                .next()
                                .unwrap();
                            let title = title_link.text().next().unwrap().to_string();
                            let ddg_url = title_link.attr("href").unwrap().to_string();
                            let encoded_url = ddg_url
                                .split("uddg=")
                                .last()
                                .unwrap()
                                .split("&rut")
                                .next()
                                .unwrap()
                                .to_string();
                            let url = urlencoding::decode(&encoded_url)
                                .expect("UTF-8")
                                .to_string();
                            results.push(SearchResult { title, url });
                        }
                        *state.write().unwrap() = DuckduckgoSearchState::Searched(results);
                        false
                    }),
                    layer: crate::model::ItemLayer::Top,
                };
                vec![item]
            }
            DuckduckgoSearchState::Searched(items) => items
                .iter()
                .map(|item| {
                    let title = item.title.clone();
                    let url = item.url.clone();
                    let score = matcher.fuzzy_match(&title, &query).unwrap_or(0);
                    crate::model::SearchItem {
                        id: self.name().to_string(),
                        title: Some(title),
                        subtitle: Some(url.clone()),
                        icon: None,
                        image: None,
                        score,
                        source: self.name(),
                        action: Box::new(move |os| {
                            os.open_url(&url);
                            true
                        }),
                        layer: crate::model::ItemLayer::Top,
                    }
                })
                .collect(),
        }
    }
}
