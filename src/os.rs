use crate::opts::Config;
use crate::plugin::Plugin;
use crate::APP_NAME;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use crate::ui::{GtkUI, UI};
use xdg::BaseDirectories;

pub struct Os {
    plugins: Vec<Plugin>,
}

impl Os {
    fn load_plugins(
        plugin_config: HashMap<String, HashMap<String, toml::Value>>,
    ) -> Vec<Plugin> {
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
            .flatten().collect()
    }


    pub fn new(config: Config) -> Self {
        let plugins = Os::load_plugins(config.plugin);
        Self { plugins }
    }
}
