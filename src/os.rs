use crate::history::History;
use crate::opts::Config;
use crate::plugin::Plugin;
use crate::source::{ApplicationsSource, Source, StdinSource};
use crate::ui::{GtkUI, UI};
use crate::APP_NAME;
use shlex::{self, Shlex};
use std::process::Command;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use xdg::BaseDirectories;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use tokio::runtime::Runtime;

pub struct Os {
    plugins: Vec<Plugin>,
    runtime: Runtime,
    matcher: Box<dyn FuzzyMatcher>,
    sources: HashMap<String, Box<dyn Source>>,
    config: Config,
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

    async fn init_sources(sources: &mut HashMap<String, Box<dyn Source>>) {
        for (_, source) in sources.iter_mut() {
            source.init().await;
        }
    }

    fn wrap_init_sources(&mut self) {
        let sources = &mut self.sources;
        self.runtime.block_on(Os::init_sources(sources));
    }

    pub fn search(&self, query: &str) -> Vec<crate::model::SearchItem> {
        let sources = &self.sources;
        let matcher = &self.matcher;
        self.runtime.block_on(async {
            let mut items = vec![];
            for (_, source) in sources.iter() {
                let source_results = source.search(query, matcher).await;
                items.extend(source_results);
            }
            let mut items_with_history_score = items
                .into_iter()
                .map(|item| {
                    let history_score = self.history.get(&item);
                    (item, history_score)
                })
                .collect::<Vec<_>>();
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
        })
    }

    pub fn deinit(&mut self) {
        let sources = &mut self.sources;
        self.runtime.block_on(async {
            for (_, source) in sources.iter_mut() {
                source.deinit().await;
            }
        });
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
        };
        self.deinit();
        std::process::exit(0);
    }

    pub fn select(&mut self, item: &crate::model::SearchItem) {
        self.history.add(item);
        let select_action = (item.action)();
        self.run_select_action(select_action);
    }

    pub fn new(config: Config) -> Self {
        let plugins = Os::load_plugins(&config.plugin);
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        let matcher = Box::new(SkimMatcherV2::default());
        let sources: Vec<Box<dyn Source>> = vec![
            Box::new(StdinSource::new()),
            Box::new(ApplicationsSource::new()),
        ];
        let mut sources = sources
            .into_iter()
            .map(|s| (s.name().to_string(), s))
            .collect();
        let mut config = Self {
            history: History::new(),
            plugins,
            runtime,
            matcher,
            sources,
            config,
        };
        config.wrap_init_sources();
        config
    }
}
