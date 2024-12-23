use mlua::prelude::*;
use mlua::{Function, UserData, Value};
use rusqlite::types::Value as DBValue;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct LuaRow(Vec<Value>);

impl LuaRow {
    pub fn new(row: Vec<Value>) -> Self {
        LuaRow(row)
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
}

impl FromLua for RowScore {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::Table(t) => {
                let row = t.get(1)?;
                let score = t.get(2)?;
                Ok(RowScore {
                    row: row,
                    score: score,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "RowScore".to_string(),
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
