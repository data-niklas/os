use super::Source;
use crate::helpers::Helpers;
use crate::model::SearchItem;

use std::collections::HashMap;
use std::sync::Arc;

struct RunCommand {
    command: String,
    run_in_terminal: bool,
}

pub struct RunCommandsSource {
    pub commands: HashMap<String, RunCommand>,
}

impl RunCommandsSource {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
}

impl Source for RunCommandsSource {
    fn name(&self) -> &'static str {
        "run_commands"
    }

    fn init(&mut self, config: &toml::Table, _helpers: Arc<Helpers>) {
        let default_commands: HashMap<String, String> = config
            .get("default")
            .map_or(toml::Value::Table(toml::Table::new()), |value| {
                value.to_owned()
            })
            .try_into()
            .unwrap();
        let terminal_commands: HashMap<String, String> = config
            .get("terminal")
            .map_or(toml::Value::Table(toml::Table::new()), |value| {
                value.to_owned()
            })
            .try_into()
            .unwrap();
        let default_commands_iter = default_commands.into_iter().map(|(name, command)| {
            (
                name,
                RunCommand {
                    command: command.clone(),
                    run_in_terminal: false,
                },
            )
        });
        let terminal_commands_iter = terminal_commands.into_iter().map(|(name, command)| {
            (
                name,
                RunCommand {
                    command: command.clone(),
                    run_in_terminal: true,
                },
            )
        });
        let commands = default_commands_iter
            .chain(terminal_commands_iter)
            .collect();
        self.commands = commands;
    }

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        _matcher: &Box<dyn fuzzy_matcher::FuzzyMatcher + Send + Sync>,
    ) -> Vec<crate::model::SearchItem> {
        let (left, right) = match query.split_once(" ") {
            Some((left, right)) => (left, right),
            None => (query.trim_end(), ""),
        };
        if !self.commands.contains_key(left) {
            return vec![];
        }
        let run_command = self.commands.get(left).unwrap();
        let command_template = &run_command.command;
        let run_in_terminal = run_command.run_in_terminal;
        let command = command_template.replace("%s", right);
        let item = SearchItem {
            id: self.name().to_string() + &left,
            title: Some(format!("Run command {}", right)),
            subtitle: Some(command.clone()),
            icon: None,
            image: None,
            score: 100,
            source: self.name(),
            layer: crate::model::ItemLayer::Middle,
            action: Box::new(move |os| {
                if run_in_terminal {
                    os.run_in_terminal(&command);
                } else {
                    os.run(&command);
                }
                true
            }),
        };
        return vec![item];
    }
}
