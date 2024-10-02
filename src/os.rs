use crate::helpers::Helpers;
use crate::history::History;
use crate::model::{ClipboardContent, SearchItem};
use crate::opts::Config;

#[cfg(feature = "cliphist")]
use crate::source::CliphistSource;
#[cfg(feature = "linkding")]
use crate::source::LinkdingSource;

#[cfg(feature = "duckduckgo")]
use crate::source::DuckduckgoSource;

use crate::source::{
    ApplicationsSource, EvalSource, HistorySource, HstrSource, RunCommandsSource,
    SearchSitesSource, Source, StdinSource, SystemctlSource, ZoxideSource,
};

use shlex::{self, Shlex};
use std::process::Command;
use std::sync::Arc;
use std::{collections::HashMap, process::Stdio};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use rayon::prelude::*;

use log::warn;
pub struct Os {
    matcher: Box<dyn FuzzyMatcher + Send + Sync>,
    sources: HashMap<String, Box<dyn Source + Send + Sync>>,
    pub config: Config,
    history: History,
    helpers: Arc<Helpers>,
}

impl Os {
    fn init_sources(&mut self) {
        self.sources.par_iter_mut().for_each(|(_, source)| {
            let source_name = source.name();
            if let Some(config) = self.config.source.get(source_name) {
                source.init(config, self.helpers.clone());
            } else {
                source.init(&toml::Table::new(), self.helpers.clone());
            }
        });
    }

    fn item_score(item: &SearchItem, history_score: u32) -> f32 {
        let start_score = match item.layer {
            crate::model::ItemLayer::Top => 2.,
            crate::model::ItemLayer::Middle => 1.,
            crate::model::ItemLayer::Bottom => 0.,
        };
        // Maps score to between 0 and 1
        let relative_score = 1. - 1. / ((item.score as f32) / 10. + 1.);
        let absolute_score = start_score + relative_score;
        // Lets history grow slower towards 1
        let history_score = 1. - 1. / ((history_score as f32) / 50. + 1.);
        let combined_score = (absolute_score + history_score).min(3.0);

        combined_score / 3.0
    }

    pub fn search(&self, query: &str) -> Vec<crate::model::SearchItem> {
        let sources = &self.sources;
        let matcher = &self.matcher;
        let items = sources
            .iter()
            .flat_map(|(_, source)| source.search(query, matcher))
            .collect::<Vec<SearchItem>>();
        let mut items_with_history_score = items
            .into_iter()
            .map(|item| {
                let history_score = self.history.get(&item);
                let item_score = Self::item_score(&item, history_score);
                (item, item_score)
            })
            .collect::<Vec<(SearchItem, f32)>>();
        items_with_history_score.sort_by(|(_a_item, a_item_score), (_b_item, b_item_score)| {
            b_item_score.total_cmp(a_item_score)
        });
        items_with_history_score
            .into_iter()
            .map(|(item, _)| item)
            .collect()
    }

    pub fn deinit(&mut self) {
        let sources = &mut self.sources;
        for (_, source) in sources.iter_mut() {
            source.deinit();
        }
        // self.history.deinit();
    }

    pub fn print(&self, text: &str) {
        println!("{}", text);
    }

    pub fn run(&self, action: &str) {
        let args: Vec<_> = Shlex::new(&action).into_iter().collect();
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command.spawn().expect("Failed to spawn command");
    }

    pub fn run_in_terminal(&self, action: &str) {
        let terminal_command = &self.config.terminal;

        let mut command = Command::new(terminal_command);
        command.arg("-e").arg(action);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command.spawn().expect("Failed to spawn command");
    }

    pub fn copy_to_clipboard(&self, content: ClipboardContent) {
        content.copy();
    }

    pub fn open_url(&self, url: &str) {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command.spawn().expect("Failed to spawn command");
    }

    pub fn select(&mut self, item: &crate::model::SearchItem) -> bool {
        self.history.add(item);
        (item.action)(self)
    }

    fn load_sources(enabled_sources: &Vec<String>) -> Vec<Box<dyn Source + Send + Sync>> {
        let mut sources: Vec<Box<dyn Source + Send + Sync>> = vec![];
        for name in enabled_sources {
            match name.as_str() {
                "stdin" => sources.push(Box::new(StdinSource::new())),
                "eval" => sources.push(Box::new(EvalSource::new())),
                "hstr" => sources.push(Box::new(HstrSource::new())),
                "search_sites" => sources.push(Box::new(SearchSitesSource::new())),
                "history" => sources.push(Box::new(HistorySource::new())),
                #[cfg(feature = "cliphist")]
                "cliphist" => sources.push(Box::new(CliphistSource::new())),
                "zoxide" => sources.push(Box::new(ZoxideSource::new())),
                "applications" => sources.push(Box::new(ApplicationsSource::new())),
                "systemctl" => sources.push(Box::new(SystemctlSource::new())),
                "run_commands" => sources.push(Box::new(RunCommandsSource::new())),
                #[cfg(feature = "linkding")]
                "linkding" => sources.push(Box::new(LinkdingSource::new())),
                #[cfg(feature = "duckduckgo")]
                "duckduckgo" => sources.push(Box::new(DuckduckgoSource::new())),
                _ => {
                    warn!("No source '{name}'");
                }
            }
        }

        sources
    }

    pub fn new(config: Config) -> Self {
        let matcher = Box::new(SkimMatcherV2::default());
        let sources: Vec<Box<dyn Source + Send + Sync>> = Self::load_sources(&config.sources);
        let sources = sources
            .into_iter()
            .map(|s| (s.name().to_string(), s))
            .collect();
        let mut config = Self {
            history: History::new(),
            matcher,
            sources,
            config,
            helpers: Arc::new(Helpers::default()),
        };
        config.init_sources();
        config
    }
}
