use crate::helpers::Helpers;
use crate::model::SearchItem;
use crate::source::Source;
use fuzzy_matcher::FuzzyMatcher;

use std::process::Command;


pub struct HstrSource {}

impl HstrSource {
    pub fn new() -> HstrSource {
        HstrSource {}
    }
}

impl Source for HstrSource {
    fn name(&self) -> &'static str {
        "hstr"
    }

    fn init(&mut self, _config: &toml::Table, _helpers: &Helpers) {}
    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> Vec<SearchItem> {
        let output = Command::new("hstr")
            .arg("-n")
            .args(query.split_whitespace())
            .output()
            .expect("failed to execute process");
        let stdout = String::from_utf8(output.stdout).unwrap();
        stdout
            .lines()
            .map(|line| {
                (
                    matcher.fuzzy_match(&line, &query).unwrap_or(0),
                    line.to_string(),
                )
            })
            .filter(|(score, _)| score > &0)
            .map(|(score, command)| {
                let action_command = command.clone();
                SearchItem {
                    id: "hstr".to_string() + &command,
                    title: Some(command.to_string()),
                    subtitle: None,
                    icon: None,
                    image: None,
                    score,
                    source: self.name(),
                    action: Box::new(move |os| {
                        let shell = std::env::var("SHELL").unwrap_or("bash".to_string());
                        let command = format!("{} -c '{};exec $SHELL;'", shell, action_command);
                        os.run_in_terminal(&command);
                        true
                    }),
                    layer: crate::model::ItemLayer::Middle,
                }
            })
            .collect()
    }
}
