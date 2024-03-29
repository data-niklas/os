use super::Source;
use crate::model::ImmutablePixbuf;
use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter, PathSource};
use freedesktop_icon_lookup::Cache;
use rayon::prelude::*;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use serde::de::value::MapDeserializer;
use std::collections::HashMap;
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

const fn _default_icons() -> bool {
    true
}

#[derive(Deserialize)]
pub struct ApplicationsConfig {
    #[serde(default = "_default_icons")]
    pub icons: bool,
}

#[derive(Clone)]
pub struct ParsedDesktopEntry {
    pub name: String,
    pub description: String,
    pub icon: Option<ImmutablePixbuf>,
    pub exec: String,
    pub terminal: bool,
}

impl ParsedDesktopEntry {
    fn lookup_icon(icon: &str, cache: &Cache) -> Option<PathBuf> {
        let path = Path::new(icon);
        if path.exists() {
            return Some(path.to_path_buf());
        }
        cache.lookup(icon, None)
    }

    pub fn from_desktop_entry(entry: DesktopEntry, cache: &Cache, icons: bool) -> Option<Self> {
        if entry.exec().is_none() {
            return None;
        }
        let name = entry
            .name(None)
            .unwrap_or(Cow::Borrowed(""))
            .trim()
            .to_string();
        let description = entry
            .comment(None)
            .unwrap_or(Cow::Borrowed(""))
            .trim()
            .to_string();
        let exec = entry.exec().unwrap().to_string();
        let icon = if icons {
            entry
                .icon()
                .and_then(|icon| Self::lookup_icon(&icon, cache))
                .map(|icon_path| icon_path)
                .map(|icon_path| Pixbuf::from_file(icon_path).map(ImmutablePixbuf::new).ok())
                .unwrap_or(None)
        } else {
            None
        };
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

impl Source for ApplicationsSource {
    fn name(&self) -> &'static str {
        "applications"
    }

    fn init(&mut self, config: &toml::Table) {
        let config: ApplicationsConfig = config.clone().try_into().unwrap();
        let mut cache = Cache::new().unwrap();
        cache
            .load_default()
            .expect("Failed to load default icon cache");
        let paths = default_paths();
        let parsed_entries: Vec<ParsedDesktopEntry> = Iter::new(paths)
            .par_bridge()
            .filter_map(|path| {
                let path_src = PathSource::guess_from(&path);
                if let Ok(bytes) = fs::read_to_string(&path) {
                    let entry = DesktopEntry::decode(&path, &bytes);
                    if entry.is_err() {
                        return None;
                    }
                    let entry = entry.unwrap();
                    ParsedDesktopEntry::from_desktop_entry(entry, &cache, config.icons)
                } else {
                    None
                }
            })
            .collect();
        self.entries = parsed_entries;
    }

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn fuzzy_matcher::FuzzyMatcher + Send + Sync>,
    ) -> Vec<crate::model::SearchItem> {
        self.entries
            .iter()
            .map(|s| (s, matcher.fuzzy_match(&s.name, query).unwrap_or(0)))
            .filter(|(_, score)| *score > 0 || query.is_empty())
            .map(|(entry, score)| {
                let exec = entry.exec.clone();
                let terminal = entry.terminal.clone();
                crate::model::SearchItem {
                    id: "applications".to_string() + &entry.name,
                    title: Some(entry.name.clone()),
                    subtitle: Some(entry.description.clone()),
                    icon: entry.icon.clone(),
                    image: None,
                    score,
                    source: self.name(),
                    action: Box::new(move || {
                        let exec = exec.clone();
                        let exec = exec
                            .replace(" %U", "")
                            .replace(" %u", "")
                            .replace(" %F", "")
                            .replace(" %f", "");
                        if terminal {
                            crate::model::SelectAction::RunInTerminal(exec)
                        } else {
                            crate::model::SelectAction::Run(exec)
                        }
                    }),
                    layer: crate::model::ItemLayer::Middle,
                }
            })
            .collect()
    }
}
