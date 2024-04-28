use crate::helpers::Helpers;
use crate::model::SearchItem;
use crate::source::Source;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

fn _default_limit() -> u32 {
    100
}

fn _default_cache_duration() -> Duration {
    // 24 hours
    Duration::from_secs(60 * 60 * 24)
}

#[derive(Deserialize, Serialize)]
struct Bookmark {
    id: u32,
    url: String,
    title: String,
    description: String,
    notes: String,
    website_title: Option<String>,
    website_description: Option<String>,
    web_archive_snapshot_url: Option<String>,
    is_archived: bool,
    unread: bool,
    shared: bool,
    tag_names: Vec<String>,
    date_added: String,
    date_modified: String,
}

#[derive(Serialize, Deserialize)]
struct Bookmarks {
    count: u32,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<Bookmark>,
}

#[derive(Deserialize)]
pub struct HistoryConfig {
    #[serde(default = "_default_cache_duration")]
    pub cache_duration: Duration,
}

pub struct HistorySource {
    bookmarks: Vec<Bookmark>,
}

impl HistorySource {
    pub fn new() -> HistorySource {
        HistorySource { bookmarks: vec![] }
    }
}

impl Source for HistorySource {
    fn name(&self) -> &'static str {
        "history"
    }

    fn init(&mut self, config: &toml::Table, helpers: &Helpers) {
        let config: HistoryConfig = config.clone().try_into().unwrap();
        // let cache_duration = config.cache_duration;
        // if !helpers.cache_expired(self.name(), cache_duration) {
        //     let bookmarks: Bookmarks = helpers.read_cache(self.name()).unwrap();
        //     self.bookmarks = bookmarks.results;
        //     return;
        // }
        // let bookmarks: Bookmarks = get(&bookmarks_url)
        //     .set("Authorization", &format!("Token {}", config.api_key))
        //     .call()
        //     .expect("Failed to fetch bookmarks")
        //     .into_json()
        //     .expect("Failed to parse bookmarks");
        // helpers.write_cache(self.name(), &bookmarks);
        // self.bookmarks = bookmarks.results;
    }

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> Vec<SearchItem> {
        // let query_words: Vec<&str> = query.split_whitespace().collect();
        // let query_tags: Vec<&str> = query_words
        //     .iter()
        //     .filter(|word| word.starts_with("#"))
        //     .map(|tag| &tag[1..])
        //     .collect();
        // let query_without_tags: String = query_words
        //     .into_iter()
        //     .filter(|word| !word.starts_with("#"))
        //     .collect::<Vec<&str>>()
        //     .join(" ");
        // let mut bookmarks: Vec<&Bookmark> = self.bookmarks.iter().collect();
        // for tag in query_tags {
        //     bookmarks = bookmarks
        //         .into_iter()
        //         .filter(|bookmark| bookmark.tag_names.contains(&tag.to_string()))
        //         .collect();
        // }
        // let mut results: Vec<SearchItem> = vec![];
        // for bookmark in bookmarks {
        //     let title_score = matcher
        //         .fuzzy_match(&bookmark.title, &query_without_tags)
        //         .unwrap_or(0);
        //     let description_score = matcher
        //         .fuzzy_match(&bookmark.description, &query_without_tags)
        //         .unwrap_or(0);
        //     let url_score = matcher
        //         .fuzzy_match(&bookmark.url, &query_without_tags)
        //         .unwrap_or(0);
        //     let score = title_score.max(description_score).max(url_score);
        //     if score == 0 && !query_without_tags.is_empty() {
        //         continue;
        //     }
        //     let url = bookmark.url.clone();
        //     let formatted_tags = bookmark
        //         .tag_names
        //         .iter()
        //         .map(|tag_name| "#".to_string() + tag_name)
        //         .collect::<Vec<String>>()
        //         .join(", ");
        //
        //     let (title, subtitle) = if bookmark.title != "" {
        //         (
        //             Some(bookmark.title.clone()),
        //             Some(format!("{} ({})", url, formatted_tags)),
        //         )
        //     } else {
        //         (Some(url.clone()), Some(formatted_tags))
        //     };
        //     results.push(SearchItem {
        //         id: self.name().to_string() + &bookmark.id.to_string(),
        //         title,
        //         subtitle,
        //         icon: None,
        //         image: None,
        //         score,
        //         source: self.name(),
        //         layer: crate::model::ItemLayer::Middle,
        //         action: Box::new(move |os| {
        //             os.open_url(&url);
        //             true
        //         }),
        //     });
        // }
        //
        // results
        vec![]
    }
}
