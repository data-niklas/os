mod fts;
mod lua;
mod plugin;

use mlua::prelude::*;
use mlua::{Function, UserData, Value};
use rusqlite::types::Value as DBValue;
use std::path::Path;

pub struct Config {
    pub plugins: Vec<plugin::Plugin>,
    pub enabled_plugins: Vec<usize>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            plugins: Vec::new(),
            enabled_plugins: Vec::new(),
        }
    }

    pub fn enabled_plugins_iter(&self) -> impl Iterator<Item = &plugin::Plugin> {
        self.plugins.iter().enumerate().filter_map(|(i, plugin)| {
            if self.enabled_plugins.contains(&i) {
                Some(plugin)
            } else {
                None
            }
        })
    }

    pub fn enabled_plugins_iter_mut(&mut self) -> impl Iterator<Item = &mut plugin::Plugin> {
        self.plugins
            .iter_mut()
            .enumerate()
            .filter_map(|(i, plugin)| {
                if self.enabled_plugins.contains(&i) {
                    Some(plugin)
                } else {
                    None
                }
            })
    }
}

impl FromLua for Config {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let plugins: Vec<plugin::Plugin> = t.get("plugins")?;
                let enabled_plugins: Vec<String> = t.get("enabled_plugins")?;
                let enabled_plugins = enabled_plugins
                    .iter()
                    .map(|name| {
                        plugins
                            .iter()
                            .position(|plugin| plugin.name() == *name)
                            .unwrap()
                    })
                    .collect();
                Ok(Config {
                    plugins,
                    enabled_plugins,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "Config".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

struct OmniSearch {
    lua: Lua,
    config: Config,
}

impl OmniSearch {
    pub fn new() -> Self {
        OmniSearch {
            lua: Lua::new(),
            config: Config::new(),
        }
    }

    pub fn load_scripts(&mut self) {
        let script = Path::new("test.lua");
        self.config = self.lua.load(script).eval().unwrap();
    }

    pub fn setup_plugins(&mut self) {
        for plugin in self.config.enabled_plugins_iter_mut() {
            match plugin {
                plugin::Plugin::ActivePlugin(plugin) => {
                    // Do nothing
                }
                plugin::Plugin::PassivePlugin(plugin) => {
                    let conn = fts::create_connection();
                    plugin.initialize(&conn);
                }
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<lua::RowScore> {
        let conn = fts::create_connection();
        let mut results = Vec::new();
        for plugin in self.config.enabled_plugins_iter() {
            let plugin_result = plugin.search(&self.lua, &conn, query);
            results.extend(plugin_result);
        }
        results
    }
}

fn main() {
    let mut omni_search = OmniSearch::new();
    omni_search.load_scripts();
    omni_search.setup_plugins();
    let result = omni_search.search("test");
    println!("{:?}", result);
}
