use std::sync::Arc;

use crate::helpers::Helpers;
use crate::model::SearchItem;
use crate::source::Source;
use fuzzy_matcher::FuzzyMatcher;

pub struct SystemctlSource {
    items: Vec<(String, String, String)>,
}

impl SystemctlSource {
    pub fn new() -> Self {
        let items = vec![
            ("suspend", "Suspend the computer", "suspend"),
            ("hibernate", "Hibernate the computer", "hibernate"),
            ("reboot", "Reboot the computer", "reboot"),
            ("shutdown", "Shutdown the computer", "poweroff"),
        ];
        let items = items
            .into_iter()
            .map(|(title, subtitle, command)| {
                (title.to_string(), subtitle.to_string(), command.to_string())
            })
            .collect();
        SystemctlSource { items }
    }
}

impl Source for SystemctlSource {
    fn name(&self) -> &'static str {
        "systemctl"
    }

    fn init(&mut self, _config: &toml::Table, _helpers: Arc<Helpers>) {}

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> Vec<SearchItem> {
        self.items
            .iter()
            .map(|(title, subtitle, command)| {
                (
                    (title, subtitle, command),
                    matcher.fuzzy_match(&title, query).unwrap_or(0),
                )
            })
            .filter(|(_, score)| *score > 0 || query.is_empty())
            .map(|(item, score)| {
                let (title, subtitle, command) = item;
                let command = command.clone();
                SearchItem {
                    id: self.name().to_string() + &title,
                    title: Some(title.to_string()),
                    subtitle: Some(subtitle.to_string()),
                    icon: None,
                    image: None,
                    score,
                    source: self.name(),
                    action: Box::new(move |os| {
                        let command = command.clone();
                        os.run(&format!("systemctl {}", command));
                        true
                    }),
                    layer: crate::model::ItemLayer::Middle,
                }
            })
            .collect()
    }
}
