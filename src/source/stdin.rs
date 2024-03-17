use crate::model::{SearchItem, SelectAction};
use crate::source::{stdin, Source};
use async_trait::async_trait;
use fuzzy_matcher::FuzzyMatcher;
use std::io::{stdin, Read};
use atty;

pub struct StdinSource {
    items: Vec<String>,
}

impl StdinSource {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

#[async_trait]
impl Source for StdinSource {
    async fn name(&self) -> &str {
        "stdin"
    }

    async fn init(&mut self) {
        if atty::is(atty::Stream::Stdin) {
            return;
        }
        let mut buf = String::new();
        stdin()
            .lock()
            .read_to_string(&mut buf)
            .expect("Failed to read from stdin");
        self.items = buf.lines().map(|s| s.to_string()).collect();
    }

    async fn deinit(&mut self) {
    }

    async fn search(&self, query: &str, matcher: &Box<dyn FuzzyMatcher>) -> Vec<SearchItem> {
        self.items
            .iter()
            .map(|s|{
                (s, matcher.fuzzy_match(&s, query).unwrap_or(0))
            })
            .filter(|(_, score)| *score > 0 || query.is_empty())
            .map(|(s, score)| {
                let text = s.clone();
                SearchItem {
                    title: Some(s.clone()),
                    subtitle: None,
                    icon: None,
                    image: None,
                    score,
                    action: Box::new(move || {
                        let text = text.clone();
                        SelectAction::Print(text)
                    }),
                    layer: crate::model::ItemLayer::Middle,
                }
            })
            .collect()
    }
}
