use crate::modules::db::sqlite_to_lua_value;
use mlua::prelude::*;
use mlua::{Function, UserData, Value};
use rusqlite::types::Value as DBValue;

#[derive(Clone, Debug)]
pub struct ActivePlugin {
    name: String,
    // First column is the primary key
    query_results: Function,
}

impl UserData for ActivePlugin {}

impl FromLua for ActivePlugin {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let name = t.get("name")?;
                let query_results = t.get("query_results")?;
                Ok(ActivePlugin {
                    name,
                    query_results,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "ActivePlugin".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl ActivePlugin {
    pub fn new(name: String, query_results: Function) -> Self {
        ActivePlugin {
            name,
            query_results,
        }
    }

    pub fn search(&self, lua: &Lua, query: String) -> Vec<crate::lua::SearchResult> {
        let rows = self
            .query_results
            .call::<Vec<crate::lua::SearchResult>>(query)
            .unwrap();
        rows
    }
}

#[derive(Clone, Debug)]
pub struct PassivePlugin {
    name: String,
    // First column is the primary key
    columns: Vec<String>,
    search_columns: Vec<String>,
    build_search_result: Function,
    refresh_rows: Function,
}

impl UserData for PassivePlugin {}

impl FromLua for PassivePlugin {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let name = t.get("name")?;
                let columns = t.get("columns")?;
                let search_columns = t.get("search_columns")?;
                let build_search_result = t.get("build_search_result")?;
                let refresh_rows = t.get("refresh_rows")?;
                Ok(PassivePlugin {
                    name,
                    columns,
                    search_columns,
                    build_search_result,
                    refresh_rows,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "PassivePlugin".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl PassivePlugin {
    fn new(
        name: String,
        columns: Vec<String>,
        search_columns: Vec<String>,
        build_search_result: Function,
        refresh_rows: Function,
    ) -> Self {
        PassivePlugin {
            name,
            columns,
            search_columns,
            build_search_result,
            refresh_rows,
        }
    }

    fn column_count(&self) -> usize {
        self.columns.len()
    }

    fn primary_key(&self) -> &str {
        &self.columns[0]
    }

    pub fn refresh_rows(&self, lua: &Lua, conn: &rusqlite::Connection) -> LuaResult<()> {
        lua.scope(|scope| {
            let table_lease = crate::modules::db::TableLease::new(conn, &self.name);
            let lua_table_lease = scope.create_userdata(table_lease)?;
            self.refresh_rows.call::<()>(lua_table_lease).unwrap();
            Ok(())
        })
    }

    fn search(
        &self,
        lua: &Lua,
        conn: &rusqlite::Connection,
        query: &str,
    ) -> Vec<crate::lua::DBSearchResult> {
        let mut stmt = conn.prepare(&format!("SELECT b.*, bm25({}_search) FROM {}_search AS a INNER JOIN {} AS b ON a.rowid = b.{} WHERE {}_search MATCH ? ORDER BY bm25({}_search) DESC", self.name, self.name, self.name, &self.columns[0], self.name, self.name)).unwrap();
        let rows = stmt
            .query_map(&[&query], |row| {
                let mut fields: Vec<DBValue> = Vec::new();
                for i in 0..self.column_count() {
                    fields.push(row.get(i)?);
                }
                let score: f32 = row.get(self.column_count())?;
                Ok((fields, score))
            })
            .unwrap();
        let rows: Vec<(Vec<DBValue>, f32)> = rows.map(|row| row.unwrap()).collect();
        rows.into_iter()
            .map(|(fields, score)| {
                crate::lua::DBSearchResult::new(
                    crate::lua::LuaRow::new(
                        fields
                            .into_iter()
                            .map(|field| sqlite_to_lua_value(lua, &field))
                            .collect(),
                    ),
                    score,
                )
            })
            .collect()
    }

    pub fn initialize(&mut self, conn: &rusqlite::Connection) {
        crate::fts::create_fts_objects(
            conn,
            &self.name,
            &self.columns[..],
            &self.search_columns[..],
        );
    }
}

pub enum Plugin {
    ActivePlugin(ActivePlugin),
    PassivePlugin(PassivePlugin),
}

impl Plugin {
    pub fn search(
        &self,
        lua: &Lua,
        conn: &rusqlite::Connection,
        query: &str,
    ) -> LuaResult<Vec<crate::lua::SearchResult>> {
        match self {
            Plugin::ActivePlugin(plugin) => Ok(plugin.search(lua, query.to_string())),
            Plugin::PassivePlugin(plugin) => {
                let db_search_results = plugin.search(lua, conn, query);
                db_search_results
                    .into_iter()
                    .map(|row| row.to_search_result(&plugin.build_search_result))
                    .collect()
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Plugin::ActivePlugin(plugin) => &plugin.name,
            Plugin::PassivePlugin(plugin) => &plugin.name,
        }
    }
}

// If columns in the table, it is an active plugin
impl FromLua for Plugin {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                if t.contains_key("columns")? {
                    let plugin = PassivePlugin::from_lua(Value::Table(t), lua)?;
                    Ok(Plugin::PassivePlugin(plugin))
                } else {
                    let plugin = ActivePlugin::from_lua(Value::Table(t), lua)?;
                    Ok(Plugin::ActivePlugin(plugin))
                }
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "Plugin".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}
