use crate::model::{SearchItem, SelectAction};
use crate::source::Source;
use async_trait::async_trait;
use fuzzy_matcher::FuzzyMatcher;
use std::process::Command;

pub struct ZoxideSource {}

impl ZoxideSource {
    pub fn new() -> ZoxideSource {
        ZoxideSource {}
    }
}

#[async_trait]
impl Source for ZoxideSource {
    fn name(&self) -> &'static str {
        "zoxide"
    }

    async fn init(&mut self) {}

    async fn deinit(&mut self) {}

    async fn search(&self, query: &str, _matcher: &Box<dyn FuzzyMatcher>) -> Vec<SearchItem> {
        let output = Command::new("zoxide")
            .arg("query")
            .arg("-ls")
            .arg(query)
            .output()
            .expect("failed to execute process");
        let stdout = String::from_utf8(output.stdout).unwrap();
        stdout
            .lines()
            .map(|line| {
                let (scores, directory) = line.trim().split_once(' ').unwrap();
                let directory = directory.to_string();
                let score = scores.parse::<f32>().unwrap();
                let score: i64 = score.round() as i64;
                let action_directory = directory.clone();
                SearchItem {
                    id: "zoxide".to_string() + &directory,
                    title: Some(directory.to_string()),
                    subtitle: None,
                    icon: None,
                    image: None,
                    score,
                    source: self.name(),
                    action: Box::new(move || {
                        let shell = std::env::var("SHELL").unwrap_or("bash".to_string());
                        let command = format!("{} -c 'cd \"{}\";exec $SHELL;'", shell, action_directory);
                        SelectAction::RunInTerminal(command)
                    }),
                    layer: crate::model::ItemLayer::Middle,
                }
            })
            .collect()
    }
}
