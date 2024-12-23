use mlua::prelude::*;
use mlua::{UserData, Value};

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
pub struct RowScore {
    row: LuaRow,
    score: f32,
}

impl RowScore {
    pub fn new(row: LuaRow, score: f32) -> Self {
        RowScore { row, score }
    }

    pub fn to_ui(self, func: &mlua::Function) -> LuaResult<UIRowScore> {
        let row = func.call::<UIRow>(self.row)?;
        Ok(UIRowScore {
            row,
            score: self.score,
        })
    }
}

impl FromLua for RowScore {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let row = t.get("row")?;

                let score = t.get("score")?;

                Ok(RowScore { row, score })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "RowScore".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UIRow {
    title: String,
    description: String,
}

impl FromLua for UIRow {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let title = t.get("title")?;
                let description = t.get("description")?;
                Ok(UIRow { title, description })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "UIRow".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UIRowScore {
    row: UIRow,
    score: f32,
}

impl FromLua for UIRowScore {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let row = t.get("row")?;
                let score = t.get("score")?;
                Ok(UIRowScore { row, score })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "UIRowScore".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

#[derive(Clone)]
pub struct LuaBuffer {
    pub buffer: Vec<u8>,
}
impl UserData for LuaBuffer {}
