use crate::model::{SearchItem, SelectAction};
use crate::source::Source;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use ureq::get;

fn _default_limit() -> u32 {
    100
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
struct Bookmarks {
    count: u32,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<Bookmark>,
}

#[derive(Deserialize)]
pub struct LinkdingConfig {
    pub api_key: String,
    pub host: String,
    #[serde(default = "_default_limit")]
    pub limit: u32,
}

pub struct LinkdingSource {
    bookmarks: Vec<Bookmark>,
}

impl LinkdingSource {
    pub fn new() -> LinkdingSource {
        LinkdingSource { bookmarks: vec![] }
    }
}

impl Source for LinkdingSource {
    fn name(&self) -> &'static str {
        "linkding"
    }

    fn init(&mut self, config: &toml::Table) {
        let config: LinkdingConfig = config.clone().try_into().unwrap();
        let limit = config.limit;
        let bookmarks_url = format!("{}/api/bookmarks/?limit={}", config.host, limit);
        let bookmarks: Bookmarks = get(&bookmarks_url)
            .set("Authorization", &format!("Token {}", config.api_key))
            .call()
            .expect("Failed to fetch bookmarks")
            .into_json()
            .expect("Failed to parse bookmarks");
        self.bookmarks = bookmarks.results;
    }

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> Vec<SearchItem> {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let query_tags: Vec<&str> = query_words
            .iter()
            .filter(|word| word.starts_with("#"))
            .map(|tag| &tag[1..])
            .collect();
        let query_without_tags: String = query_words
            .into_iter()
            .filter(|word| !word.starts_with("#"))
            .collect::<Vec<&str>>()
            .join(" ");
        let mut bookmarks: Vec<&Bookmark> = self.bookmarks.iter().collect();
        for tag in query_tags {
            bookmarks = bookmarks
                .into_iter()
                .filter(|bookmark| bookmark.tag_names.contains(&tag.to_string()))
                .collect();
        }
        let mut results: Vec<SearchItem> = vec![];
        for bookmark in bookmarks {
            let title_score = matcher
                .fuzzy_match(&bookmark.title, &query_without_tags)
                .unwrap_or(0);
            let description_score = matcher
                .fuzzy_match(&bookmark.description, &query_without_tags)
                .unwrap_or(0);
            let url_score = matcher
                .fuzzy_match(&bookmark.url, &query_without_tags)
                .unwrap_or(0);
            let score = title_score.max(description_score).max(url_score);
            if score == 0 && !query_without_tags.is_empty() {
                continue;
            }
            let url = bookmark.url.clone();
            let subtitle = format!(
                "{} ({})",
                url,
                &bookmark
                    .tag_names
                    .iter()
                    .map(|tag_name| "#".to_string() + tag_name)
                    .collect::<Vec<String>>()
                    .join(", "),
            );
            let title = if bookmark.title.is_empty() {
                None
            } else {
                Some(bookmark.title.clone())
            };
            results.push(SearchItem {
                id: self.name().to_string() + &bookmark.id.to_string(),
                title,
                subtitle: Some(subtitle),
                icon: None,
                image: None,
                score,
                source: self.name(),
                layer: crate::model::ItemLayer::Middle,
                action: Box::new(move || SelectAction::OpenUrl(url.clone())),
            });
        }

        results
    }
}
