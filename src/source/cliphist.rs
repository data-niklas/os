use crate::model::ImmutablePixbuf;
use crate::model::{ClipboardContent, SearchItem, SelectAction};
use crate::source::Source;
use ahash::{AHasher, RandomState};
use fuzzy_matcher::FuzzyMatcher;
use nut::{DBBuilder, DB};
use rayon::prelude::*;
use relm4::gtk::gdk::Texture;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use relm4::gtk::glib::Bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use xdg::BaseDirectories;

const fn _default_icons() -> bool {
    true
}

#[derive(Deserialize)]
pub struct CliphistConfig {
    #[serde(default = "_default_icons")]
    pub icons: bool,
}

pub struct CliphistSource {
    items: Vec<(
        Option<String>,
        Option<ImmutablePixbuf>,
        ClipboardContent,
        u64,
    )>,
}

impl CliphistSource {
    pub fn new() -> CliphistSource {
        CliphistSource { items: vec![] }
    }

    fn db(db_path: PathBuf) -> nut::DB {
        let db = DBBuilder::new(db_path).read_only(true).build().unwrap();
        db
    }
    fn build_item(
        &self,
        item: (
            Option<String>,
            Option<ImmutablePixbuf>,
            ClipboardContent,
            u64,
        ),
        query: &str,
        matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> SearchItem {
        let score = if query.is_empty() {
            1
        } else if item.0.is_some() {
            let match_text = item.0.as_ref().unwrap();
            let match_text = match_text.chars().take(500).collect::<String>();
            matcher.fuzzy_match(&match_text, &query).unwrap_or(0)
        } else {
            // TODO: better default score for images
            1
        };

        SearchItem {
            id: "cliphist".to_string() + &item.3.to_string(),
            title: item.0,
            subtitle: Some("(cliphist)".to_string()),
            icon: None,
            image: item.1,
            score,
            source: self.name(),
            action: Box::new(move || {
                let clipboard_content = item.2.clone();

                SelectAction::CopyToClipboard(clipboard_content)
            }),
            layer: crate::model::ItemLayer::Bottom,
        }
    }
}

impl Source for CliphistSource {
    fn name(&self) -> &'static str {
        "cliphist"
    }

    fn init(&mut self, config: &toml::Table) {
        let config: CliphistConfig = config.clone().try_into().unwrap();
        let xdg = BaseDirectories::with_prefix("cliphist").unwrap();
        let db_path = xdg.place_cache_file("db").unwrap();
        let seed = 42;
        let random_state = RandomState::with_seed(seed);
        let db = Self::db(db_path);
        let tx = db.begin_tx().unwrap();

        let bucket = tx.bucket(b"b").unwrap();
        let mut values = vec![];
        let _ = bucket.for_each::<nut::Error>(Box::new(|_key, value| {
            let value = value.unwrap();
            values.push(value.to_vec());
            Ok(())
        }));
        let items = values
            .into_par_iter()
            .filter_map(|value| {
                let mut hasher = random_state.build_hasher();
                hasher.write(&value);
                let cursor_value = value.to_vec();
                let clipboard_content = ClipboardContent(cursor_value.to_vec());
                let cursor = Cursor::new(cursor_value);
                let hash = hasher.finish();
                let (title, image) = match Pixbuf::from_read(cursor) {
                    Err(_) => (String::from_utf8(value).ok(), None),
                    Ok(img) => (
                        None,
                        if config.icons {
                            Some(ImmutablePixbuf::new(img))
                        } else {
                            None
                        },
                    ),
                };
                if title.is_none() && image.is_none() {
                    return None;
                }
                Some((title, image, clipboard_content, hash))
            })
            .collect();
        self.items = items;
    }
    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> Vec<SearchItem> {
        self.items
            .par_iter()
            .map(|item| self.build_item(item.clone(), query, matcher))
            .collect()
    }
}
