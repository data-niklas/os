use std::{borrow::Cow, fs};

use async_trait::async_trait;
use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter, PathSource};
use freedesktop_icon_lookup::Cache;
use image::DynamicImage;

use super::Source;

pub struct ParsedDesktopEntry {
    pub name: String,
    pub description: String,
    pub icon: Option<DynamicImage>,
    pub exec: String,
    pub terminal: bool,
}

impl ParsedDesktopEntry {
    pub fn from_desktop_entry(entry: DesktopEntry) -> Option<Self> {
        if entry.exec().is_none() {
            return None;
        }
        let name = entry.name(None).unwrap_or(Cow::Borrowed("")).trim().to_string();
        let description = entry.comment(None).unwrap_or(Cow::Borrowed("")).trim().to_string();
        let exec = entry.exec().unwrap().to_string();
        let icon = entry
            .icon()
            .and_then(|icon| Cache::new().unwrap().lookup(icon, None))
            .map(|icon_path| image::open(icon_path).unwrap());
        let terminal = entry.terminal();
        Some(ParsedDesktopEntry {
            name,
            description,
            icon,
            exec,
            terminal,
        })
    }
}

pub struct ApplicationsSource {
    pub entries: Vec<ParsedDesktopEntry>,
}

impl ApplicationsSource {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }
}

#[async_trait]
impl Source for ApplicationsSource {
    async fn name(&self) -> &str {
        "applications"
    }

    async fn init(&mut self) {
        let cache = Cache::new().unwrap();
        let paths = default_paths();
        let parsed_entries: Vec<ParsedDesktopEntry> = Iter::new(paths)
            .into_iter()
            .map(|path| {
                let path_src = PathSource::guess_from(&path);
                if let Ok(bytes) = fs::read_to_string(&path) {
                    let entry = DesktopEntry::decode(&path, &bytes);
                    if entry.is_err() {
                        return None;
                    }
                    let entry = entry.unwrap();
                    ParsedDesktopEntry::from_desktop_entry(entry)
                } else {
                    None
                }
            })
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();
        self.entries = parsed_entries;
    }

    async fn deinit(&mut self) {}

    async fn search(
        &self,
        query: &str,
        matcher: &Box<dyn fuzzy_matcher::FuzzyMatcher>,
    ) -> Vec<crate::model::SearchItem> {
        self.entries
            .iter()
            .map(|s| (s, matcher.fuzzy_match(&s.name, query).unwrap_or(0)))
            .filter(|(_, score)| *score > 0 || query.is_empty())
            .map(|(entry, score)| {
                let exec = entry.exec.clone();
                let terminal = entry.terminal.clone();
                crate::model::SearchItem {
                    title: Some(entry.name.clone()),
                    subtitle: Some(entry.description.clone()),
                    icon: entry.icon.clone(),
                    image: None,
                    score,
                    action: Box::new(move || {
                        if terminal {
                            crate::model::SelectAction::RunInTerminal(exec.clone())
                        } else {
                            crate::model::SelectAction::Run(exec.clone())
                        }
                    }),
                    layer: crate::model::ItemLayer::Middle,
                }
            })
            .filter(|item| item.score > 0 || query.is_empty())
            .collect()
    }
}
