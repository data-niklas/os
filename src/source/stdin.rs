use crate::model::{SearchItem, SelectAction};
use crate::source::{stdin, Source};
use atty;
use fuzzy_matcher::FuzzyMatcher;
use std::io::{stdin, Read};

pub struct StdinSource {
    items: Vec<String>,
}

impl StdinSource {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

impl Source for StdinSource {
    fn name(&self) -> &'static str {
        "stdin"
    }

    fn init(&mut self) {
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

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> Vec<SearchItem> {
        self.items
            .iter()
            .map(|s| (s, matcher.fuzzy_match(&s, query).unwrap_or(0)))
            .filter(|(_, score)| *score > 0 || query.is_empty())
            .map(|(s, score)| {
                let text = s.clone();
                SearchItem {
                    id: "stdin".to_string() + &text,
                    title: Some(s.clone()),
                    subtitle: None,
                    icon: None,
                    image: None,
                    score,
                    source: self.name(),
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
