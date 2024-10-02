use super::Source;
use crate::helpers::Helpers;
use crate::model::SearchItem;

use std::collections::HashMap;
use std::sync::Arc;

pub struct SearchSitesSource {
    pub sites: HashMap<String, String>,
}

impl SearchSitesSource {
    pub fn new() -> Self {
        Self {
            sites: HashMap::new(),
        }
    }
}

impl Source for SearchSitesSource {
    fn name(&self) -> &'static str {
        "search_sites"
    }

    fn init(&mut self, config: &toml::Table, _helpers: Arc<Helpers>) {
        self.sites = config.clone().try_into().unwrap();
    }

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        _matcher: &Box<dyn fuzzy_matcher::FuzzyMatcher + Send + Sync>,
    ) -> Vec<crate::model::SearchItem> {
        if let Some((left, right)) = query.split_once(" ") {
            if !self.sites.contains_key(left) {
                return vec![];
            }
            let url = self.sites.get(left).unwrap();
            let search_url = url.replace("%s", right);
            let item = SearchItem {
                id: self.name().to_string() + &left,
                title: Some(format!("Search for {}", right)),
                subtitle: Some(search_url.clone()),
                icon: None,
                image: None,
                score: 100,
                source: self.name(),
                layer: crate::model::ItemLayer::Middle,
                action: Box::new(move |os| {
                    os.open_url(&search_url);
                    true
                }),
            };
            return vec![item];
        }
        return vec![];
    }
}
