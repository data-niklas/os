use crate::history::History;
use crate::model::SearchItem;
use crate::opts::Config;
use crate::plugin::Plugin;
use crate::source::{
    ApplicationsSource, CliphistSource, HstrSource, LinkdingSource, Source, StdinSource,
    SystemctlSource, ZoxideSource,
};
use crate::APP_NAME;
use shlex::{self, Shlex};
use std::collections::HashMap;
use std::process::Command;
use xdg::BaseDirectories;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use rayon::prelude::*;

pub struct Os {
    plugins: Vec<Plugin>,
    matcher: Box<dyn FuzzyMatcher + Send + Sync>,
    sources: HashMap<String, Box<dyn Source + Send + Sync>>,
    pub config: Config,
    history: History,
}

impl Os {
    fn load_plugins(plugin_config: &HashMap<String, HashMap<String, toml::Value>>) -> Vec<Plugin> {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        xdg_dirs
            .get_config_dirs()
            .into_iter()
            .map(|config_directory| {
                let plugins_directory = config_directory.join("plugins");
                if plugins_directory.exists() {
                    vec![]
                } else {
                    vec![]
                }
            })
            .flatten()
            .collect()
    }

    fn init_sources(&mut self) {
        self.sources.par_iter_mut().for_each(|(_, source)| {
            let source_name = source.name();
            if let Some(config) = self.config.source.get(source_name) {
                source.init(config);
            } else {
                source.init(&toml::Table::new());
            }
        });
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
                (item, history_score)
            })
            .collect::<Vec<(SearchItem, u32)>>();
        items_with_history_score.sort_by(|a, b| {
            let cmp = b.1.cmp(&a.1);
            if cmp == std::cmp::Ordering::Equal {
                b.0.cmp(&a.0)
            } else {
                cmp
            }
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

    pub fn run_select_action(&mut self, select_action: crate::model::SelectAction) {
        match select_action {
            crate::model::SelectAction::Print(text) => {
                println!("{}", text);
            }
            crate::model::SelectAction::Run(action) => {
                let args: Vec<_> = Shlex::new(&action).into_iter().collect();
                let mut command = Command::new(&args[0]);
                command.args(&args[1..]);
                command.spawn().expect("Failed to spawn command");
            }
            crate::model::SelectAction::RunInTerminal(action) => {
                let terminal_command = &self.config.terminal;

                let mut command = Command::new(terminal_command);
                command.arg("-e").arg(action);
                command.spawn().expect("Failed to spawn command");
            }
            crate::model::SelectAction::Exit => {}
            crate::model::SelectAction::Noop => {
                return;
            }
            crate::model::SelectAction::CopyToClipboard(content) => {
                content.copy();
            }
            crate::model::SelectAction::OpenUrl(url) => {
                let mut command = Command::new("xdg-open");
                command.arg(url);
                command.spawn().expect("Failed to spawn command");
            }
        };
        self.deinit();
    }

    pub fn select(&mut self, item: &crate::model::SearchItem) {
        self.history.add(item);
        let select_action = (item.action)();
        self.run_select_action(select_action);
    }

    fn load_sources(enabled_sources: &Vec<String>) -> Vec<Box<dyn Source + Send + Sync>> {
        let mut sources: Vec<Box<dyn Source + Send + Sync>> = vec![];
        for name in enabled_sources {
            // TODO: use registered name
            match name.as_str() {
                "stdin" => sources.push(Box::new(StdinSource::new())),
                "hstr" => sources.push(Box::new(HstrSource::new())),
                "cliphist" => sources.push(Box::new(CliphistSource::new())),
                "zoxide" => sources.push(Box::new(ZoxideSource::new())),
                "applications" => sources.push(Box::new(ApplicationsSource::new())),
                "systemctl" => sources.push(Box::new(SystemctlSource::new())),
                "linkding" => sources.push(Box::new(LinkdingSource::new())),
                _ => {}
            }
        }

        sources
    }

    pub fn new(config: Config) -> Self {
        let plugins = Os::load_plugins(&config.plugin);
        let matcher = Box::new(SkimMatcherV2::default());
        let sources: Vec<Box<dyn Source + Send + Sync>> = Self::load_sources(&config.sources);
        let sources = sources
            .into_iter()
            .map(|s| (s.name().to_string(), s))
            .collect();
        let mut config = Self {
            history: History::new(),
            plugins,
            matcher,
            sources,
            config,
        };
        config.init_sources();
        config
    }
}
