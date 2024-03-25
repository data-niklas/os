// use clap::{Args as ClapArgs,Parser};
use crate::APP_NAME;
use clap_serde_derive::{
    clap,
    clap::{builder::OsStr, ArgAction, Parser},
    serde::{Deserialize, Serialize},
    ClapSerde,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use toml::Value;
use xdg::BaseDirectories;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Config file
    #[clap(short, long = "config", env, default_value = Config::default_path().into_os_string())]
    pub config_path: std::path::PathBuf,

    /// Rest of arguments
    #[command(flatten)]
    pub config: <Config as ClapSerde>::Opt,
}

impl Args {
    pub fn read_config() -> Config {
        let mut args = Args::parse();
        let config_path = args.config_path;
        let config = match std::fs::read_to_string(config_path) {
            Ok(config) => config,
            Err(_err) => {
                return Config::from(&mut args.config);
            }
        };
        match toml::from_str::<<Config as ClapSerde>::Opt>(&config) {
            Ok(config) => Config::from(config).merge(&mut args.config),
            Err(err) => panic!("Error in configuration file:\n{}", err),
        }
    }
}

fn default_terminal() -> String {
    std::env::var("TERMINAL").unwrap_or_else(|_| "xterm".to_string())
}

#[derive(ClapSerde, Deserialize, Debug)]
pub struct Config {
    /// String argument
    #[serde(default)]
    #[clap(long)]
    pub plugins: Vec<String>,

    #[default("gtk".to_string())]
    #[clap(short, long)]
    pub ui: String,

    #[serde(default)]
    #[clap(skip)]
    pub plugin: HashMap<String, HashMap<String, toml::Value>>,

    #[serde(default)]
    #[clap(skip)]
    pub source: HashMap<String, toml::Table>,

    #[default(vec!["stdin".to_string()])]
    #[clap(short, long, env, value_parser, value_delimiter = ' ', num_args = 1..)]
    pub sources: Vec<String>,

    #[default("Search".to_string())]
    #[clap(short, long)]
    pub prompt: String,

    #[default(default_terminal())]
    #[clap(short, long)]
    pub terminal: String,

    #[default(50)]
    #[clap(short, long)]
    pub maximum_list_item_count: usize
}

impl Config {
    pub fn default_path() -> PathBuf {
        let xdg = BaseDirectories::with_prefix(APP_NAME).unwrap();
        xdg.place_config_file("config.toml").unwrap()
    }
}
