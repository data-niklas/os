use crate::helpers::Helpers;
use crate::model::{SearchItem};
use crate::source::Source;
use fuzzy_matcher::FuzzyMatcher;
use std::process::Command;

pub struct ZoxideSource {}

impl ZoxideSource {
    pub fn new() -> ZoxideSource {
        ZoxideSource {}
    }
}

impl Source for ZoxideSource {
    fn name(&self) -> &'static str {
        "zoxide"
    }

    fn init(&mut self, _config: &toml::Table, _helpers: &Helpers) {}

    fn deinit(&mut self) {}

    fn search(
        &self,
        query: &str,
        _matcher: &Box<dyn FuzzyMatcher + Send + Sync>,
    ) -> Vec<SearchItem> {
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
                let (score, directory) = line.trim().split_once(' ').unwrap();
                (
                    score.parse::<f32>().unwrap().floor() as i64,
                    directory.to_string(),
                )
            })
            .filter(|(score, _)| score > &0)
            .map(|(score, directory)| {
                let action_directory = directory.clone();
                SearchItem {
                    id: "zoxide".to_string() + &directory,
                    title: Some(directory.to_string()),
                    subtitle: None,
                    icon: None,
                    image: None,
                    score,
                    source: self.name(),
                    action: Box::new(move |os| {
                        let shell = std::env::var("SHELL").unwrap_or("bash".to_string());
                        let command =
                            format!("{} -c 'cd \"{}\";exec $SHELL;'", shell, action_directory);
                        os.run_in_terminal(&command);
                        true
                    }),
                    layer: crate::model::ItemLayer::Middle,
                }
            })
            .collect()
    }
}
