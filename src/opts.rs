use clap::{Args as ClapArgs,Parser};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use toml::Value;
use xdg::BaseDirectories;

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Input files
    input: Vec<std::path::PathBuf>,

    /// Config file
    #[clap(short, long = "config", env)]
    pub config_path: Option<std::path::PathBuf>,

    /// Rest of arguments
    #[clap(flatten)]
    // pub config: <Config as ClapSerde>::Opt,
    pub config: Config
}

impl Args {
    pub fn read_config() -> Config {
        let args = Args::parse();
        let config_path = args.config_path;
        let config_path = if config_path.is_none() {
            let default_path = Config::default_path();
            if default_path.exists() {
                default_path
            } else {
                return Config::default();
            }
        } else {
            config_path.unwrap()
        };
        let config = std::fs::read_to_string(config_path).unwrap();
        toml::from_str(&config).unwrap()
    }
}

fn default_ui() -> String {
    "gtk".to_string()
}

#[derive(ClapArgs, Serialize, Deserialize, Debug, Default)]
pub struct Config {
    /// String argument
    #[clap(short, long)]
    #[serde(default)]
    plugins: Vec<String>,

    #[clap(short, long, default_value = "gtk")]
    #[serde(default = "default_ui")]
    ui: String,

    #[clap(skip)]
    #[serde(default)]
    plugin: HashMap<String, HashMap<String, toml::Value>>,
}

impl Config {
    pub fn default_path() -> PathBuf {
        let xdg = BaseDirectories::with_prefix(APP_NAME).unwrap();
        xdg.place_config_file("config.toml").unwrap()
    }
}
