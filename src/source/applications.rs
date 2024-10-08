use super::Source;
use crate::helpers::Helpers;
use crate::model::OSImage;
use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter, PathSource};
use freedesktop_icon_lookup::Cache;

use rayon::prelude::*;

use image::io::Reader as ImageReader;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use std::io::Cursor;
use std::time::Duration;
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

const fn _default_icons() -> bool {
    true
}
fn _default_cache_duration() -> Duration {
    // 24 hours
    Duration::from_secs(60 * 60 * 4)
}

#[derive(Deserialize)]
pub struct ApplicationsConfig {
    #[serde(default = "_default_icons")]
    pub icons: bool,
    #[serde(default = "_default_cache_duration")]
    pub cache_duration: Duration,
}

#[derive(Serialize, Deserialize)]
struct LoadedDesktopEntry {
    name: String,
    description: String,
    icon: Option<PathBuf>,
    exec: String,
    terminal: bool,
}

#[derive(Serialize, Deserialize)]
struct LoadedDesktopEntries {
    entries: Vec<LoadedDesktopEntry>,
}

impl LoadedDesktopEntry {
    pub fn from_desktop_entry(entry: DesktopEntry, cache: &Cache, _icons: bool) -> Option<Self> {
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
        let icon = entry
            .icon()
            .and_then(|icon| LoadedDesktopEntry::lookup_icon(&icon, cache));

        let terminal = entry.terminal();
        Some(LoadedDesktopEntry {
            name,
            description,
            icon,
            exec,
            terminal,
        })
    }

    fn lookup_icon(icon: &str, cache: &Cache) -> Option<PathBuf> {
        let path = Path::new(icon);
        if path.exists() {
            return Some(path.to_path_buf());
        }
        cache.lookup(icon, None)
    }
}

#[derive(Clone)]
pub struct ParsedDesktopEntry {
    pub name: String,
    pub description: String,
    pub icon: Option<OSImage>,
    pub exec: String,
    pub terminal: bool,
}

impl ParsedDesktopEntry {
    fn from_loaded(entry: LoadedDesktopEntry, icons: bool) -> Self {
        let icon = if icons {
            entry.icon.and_then(|icon_path| {
                let bytes = fs::read(&icon_path).ok()?;
                let arc_bytes: Arc<[u8]> = bytes.into();
                let cursor = Cursor::new(arc_bytes);
                let rgba_image = ImageReader::new(cursor)
                    .with_guessed_format()
                    .ok()?
                    .decode()
                    .ok()?
                    .into_rgba8();
                let rgba_size = rgba_image.dimensions();
                let raw_bytes = rgba_image.into_raw();
                let arc_raw_bytes: Arc<[u8]> = raw_bytes.into();
                OSImage::from_raw(rgba_size.0, rgba_size.1, arc_raw_bytes)
            })
        } else {
            None
        };
        Self {
            name: entry.name,
            description: entry.description,
            icon,
            exec: entry.exec,
            terminal: entry.terminal,
        }
    }
}

struct ApplicationsSourceInner {
    pub entries: Mutex<Vec<ParsedDesktopEntry>>,
}

pub struct ApplicationsSource {
    inner: Arc<ApplicationsSourceInner>,
}

impl ApplicationsSource {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ApplicationsSourceInner {
                entries: Mutex::new(vec![]),
            }),
        }
    }
}

impl Source for ApplicationsSource {
    fn name(&self) -> &'static str {
        "applications"
    }

    fn init(&mut self, config: &toml::Table, helpers: Arc<Helpers>) {
        let config: ApplicationsConfig = config.clone().try_into().unwrap();
        let cache_duration = config.cache_duration;

        if !helpers.cache_expired(self.name(), cache_duration) {
            let entries: LoadedDesktopEntries = helpers.read_cache(self.name()).unwrap();
            *self.inner.entries.lock().unwrap() = entries
                .entries
                .into_iter()
                .map(|entry| ParsedDesktopEntry::from_loaded(entry, config.icons))
                .collect();
            return;
        }
        let source = self.inner.clone();
        let name = self.name();
        // run in a separate thread to avoid blocking the main thread
        std::thread::spawn(move || {
            let mut cache = Cache::new().unwrap();
            cache
                .load_default()
                .expect("Failed to load default icon cache");
            let paths = default_paths();
            let loaded_entries: Vec<LoadedDesktopEntry> = Iter::new(paths)
                .par_bridge()
                .filter_map(|path| {
                    let _path_src = PathSource::guess_from(&path);
                    if let Ok(bytes) = fs::read_to_string(&path) {
                        let entry = DesktopEntry::decode(&path, &bytes);
                        if entry.is_err() {
                            return None;
                        }
                        let entry = entry.unwrap();
                        LoadedDesktopEntry::from_desktop_entry(entry, &cache, config.icons)
                    } else {
                        None
                    }
                })
                .collect();
            let loaded_entries = LoadedDesktopEntries {
                entries: loaded_entries,
            };
            helpers.write_cache(name, &loaded_entries);
            let parsed_entries: Vec<ParsedDesktopEntry> = loaded_entries
                .entries
                .into_iter()
                .map(|entry| ParsedDesktopEntry::from_loaded(entry, config.icons))
                .collect();
            *source.entries.lock().unwrap() = parsed_entries;
        });
    }

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn fuzzy_matcher::FuzzyMatcher + Send + Sync>,
    ) -> Vec<crate::model::SearchItem> {
        self.inner
            .entries
            .lock()
            .unwrap()
            .iter()
            .map(|s| (s, matcher.fuzzy_match(&s.name, query).unwrap_or(0)))
            .filter(|(_, score)| *score > 0 || query.is_empty())
            .map(|(entry, score)| {
                let exec = entry.exec.clone();
                let terminal = entry.terminal.clone();
                crate::model::SearchItem {
                    id: self.name().to_string() + &entry.name,
                    title: Some(entry.name.clone()),
                    subtitle: Some(entry.description.clone()),
                    icon: entry.icon.clone(),
                    image: None,
                    score,
                    source: self.name(),
                    action: Box::new(move |os| {
                        let exec = exec.clone();
                        let exec = exec
                            .replace(" %U", "")
                            .replace(" %u", "")
                            .replace(" %F", "")
                            .replace(" %f", "");
                        if terminal {
                            os.run_in_terminal(&exec);
                        } else {
                            os.run(&exec);
                        }
                        true
                    }),
                    layer: crate::model::ItemLayer::Middle,
                }
            })
            .collect()
    }
}
