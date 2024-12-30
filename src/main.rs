mod fts;
mod index;
mod lua;
mod modules;
mod plugin;

use mlua::prelude::*;
use mlua::{Function, Value};
use std::path::Path;

pub struct Config {
    pub plugins: Vec<plugin::Plugin>,
    pub enabled_plugins: Vec<usize>,
    pub item_map: Option<Function>, // receives the plugin name, the item score and content and may
                                    // return a different score and pinned state
}

impl Config {
    pub fn new() -> Self {
        Config {
            plugins: Vec::new(),
            enabled_plugins: Vec::new(),
            item_map: None,
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

                let item_map: Option<Function> = t.get("item_map").unwrap_or(None);

                Ok(Config {
                    plugins,
                    enabled_plugins,
                    item_map,
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

    pub fn search_unordered(&self, query: &str) -> Vec<lua::SearchResult> {
        let conn = fts::create_connection();
        let mut results = Vec::new();
        for plugin in self.config.enabled_plugins_iter() {
            let mut plugin_result = plugin.search(&self.lua, &conn, query).unwrap();

            if let Some(item_map) = &self.config.item_map {
                let plugin_name = plugin.name();
                plugin_result = plugin_result
                    .into_iter()
                    .map(|row| {
                        let result = item_map.call::<Value>((plugin_name, row)).unwrap();
                        lua::SearchResult::from_lua(result, &self.lua).unwrap()
                    })
                    .collect();
            }
            results.extend(plugin_result);
        }
        results
    }

    pub fn search(&self, query: &str) -> Vec<lua::SearchResult> {
        let mut results = self.search_unordered(query);
        results.sort_by(|a, b| {
            // first sort by field pinned (bool)
            // secondarily sort by field score (f32)
            // reverse sort order from pinned and highest score to unpinned and lowest score
            if a.pinned == b.pinned {
                b.score.partial_cmp(&a.score).unwrap()
            } else {
                a.pinned.cmp(&b.pinned)
            }
        });
        results
    }

    pub fn index(&mut self) {
        let conn = fts::create_connection();
        index::index(
            &self.lua,
            &conn,
            &mut self
                .config
                .enabled_plugins_iter_mut()
                .filter_map(|plugin| match plugin {
                    plugin::Plugin::PassivePlugin(plugin) => Some(plugin),
                    _ => None,
                }),
        )
        .unwrap();
    }
}

fn main() {
    let mut omni_search = OmniSearch::new();
    omni_search.load_scripts();
    omni_search.setup_plugins();
    let result = omni_search.search("test");
    println!("{:?}", result);
    omni_search.index();
}
