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
use std::hash::{BuildHasher, Hasher};
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use xdg::BaseDirectories;

pub struct CliphistSource {
    items: Vec<(Option<String>, Option<ImmutablePixbuf>, u64)>,
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
        item: (Option<String>, Option<ImmutablePixbuf>, u64),
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
        let item_action = item.clone();

        SearchItem {
            id: "cliphist".to_string() + &item.2.to_string(),
            title: item.0,
            subtitle: Some("(cliphist)".to_string()),
            icon: None,
            image: item.1,
            score,
            source: self.name(),
            action: Box::new(move || {
                let item = &item_action;
                let content = if item.0.is_some() {
                    ClipboardContent::Text(item.0.as_ref().unwrap().to_string())
                } else {
                    ClipboardContent::Image(item.1.as_ref().unwrap().clone())
                };
                SelectAction::CopyToClipboard(content)
            }),
            layer: crate::model::ItemLayer::Bottom,
        }
    }
}

impl Source for CliphistSource {
    fn name(&self) -> &'static str {
        "cliphist"
    }

    fn init(&mut self) {
        let xdg = BaseDirectories::with_prefix("cliphist").unwrap();
        let db_path = xdg.place_cache_file("db").unwrap();
        let seed = 42;
        let random_state = RandomState::with_seed(seed);
        let mut hasher = random_state.build_hasher();
        let db = Self::db(db_path);
        let tx = db.begin_tx().unwrap();

        let bucket = tx.bucket(b"b").unwrap();
        let mut items = vec![];
        let _ = bucket.for_each::<nut::Error>(Box::new(|_key, value| {
            let value = value.unwrap();

            hasher.write(value);
            let owned_value = value.to_vec();
            let cursor = Cursor::new(owned_value);
            let hash = hasher.finish();
            let (title, image) = match Pixbuf::from_read(cursor) {
                Err(_) => (String::from_utf8(value.to_vec()).ok(), None),
                Ok(img) => (None, Some(ImmutablePixbuf::new(img))),
            };
            if title.is_some() || image.is_some() {
                items.push((title, image, hash));
            }
            Ok(())
        }));
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
