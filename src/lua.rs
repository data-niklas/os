use mlua::prelude::*;
use mlua::{Function, UserData, Value};

#[derive(Clone, Debug)]
pub struct LuaRow(Vec<Value>);

impl LuaRow {
    pub fn new(row: Vec<Value>) -> Self {
        LuaRow(row)
    }
}

impl IntoLua for LuaRow {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        let table = lua.create_table()?;
        for (i, value) in self.0.into_iter().enumerate() {
            table.set(i + 1, value)?;
        }
        Ok(Value::Table(table))
    }
}

impl FromLua for LuaRow {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let mut row = Vec::new();
                for i in 1..=t.len()? {
                    row.push(t.get(i)?);
                }
                Ok(LuaRow(row))
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "LuaRow".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DBSearchResult {
    row: LuaRow,
    score: f32,
}

impl DBSearchResult {
    pub fn new(row: LuaRow, score: f32) -> Self {
        DBSearchResult { row, score }
    }

    pub fn to_search_result(self, map_fn: &Function) -> LuaResult<SearchResult> {
        let row = self.row;
        let score = self.score;
        let result = map_fn.call::<SearchResult>((row, score))?;
        Ok(result)
    }
}

impl FromLua for DBSearchResult {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let row = t.get("row")?;
                let score = t.get("score")?;
                Ok(DBSearchResult { row, score })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "DBSearchResult".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl IntoLua for DBSearchResult {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        let table = lua.create_table()?;
        table.set("row", self.row)?;
        table.set("score", self.score)?;
        Ok(Value::Table(table))
    }
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub action: Function,
    pub score: f32,
    pub pinned: bool,
}

impl SearchResult {
    pub fn new(
        title: String,
        description: String,
        action: Function,
        score: f32,
        pinned: bool,
    ) -> Self {
        SearchResult {
            title,
            description,
            action,
            score,
            pinned,
        }
    }
}

impl FromLua for SearchResult {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let title = t.get("title")?;
                let description = t.get("description")?;
                let action = t.get("action")?;
                let score = t.get("score")?;
                let pinned = t.get("pinned").unwrap_or(false);
                Ok(SearchResult {
                    title,
                    description,
                    action,
                    score,
                    pinned,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "SearchResult".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl IntoLua for SearchResult {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        let table = lua.create_table()?;
        table.set("title", self.title)?;
        table.set("description", self.description)?;
        table.set("action", self.action)?;
        table.set("score", self.score)?;
        table.set("pinned", self.pinned)?;
        Ok(Value::Table(table))
    }
}

#[derive(Clone)]
pub struct LuaBuffer {
    pub buffer: Vec<u8>,
}
impl UserData for LuaBuffer {}
