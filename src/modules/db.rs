use mlua::prelude::LuaResult;
use mlua::{FromLua, IntoLua, Lua, UserData, UserDataMethods, Value};
use rusqlite::types::Value as DBValue;
use rusqlite::ToSql;

pub fn sqlite_to_lua_value(lua: &Lua, value: &DBValue) -> Value {
    match value {
        DBValue::Null => Value::Nil,
        DBValue::Integer(i) => Value::Integer(*i),
        DBValue::Real(f) => Value::Number(*f),
        DBValue::Text(s) => Value::String(lua.create_string(s).unwrap()),
        DBValue::Blob(b) => crate::lua::LuaBuffer { buffer: b.to_vec() }
            .into_lua(lua)
            .unwrap(),
    }
}

pub fn lua_to_sqlite_value(lua: Value) -> DBValue {
    match lua {
        Value::Nil => DBValue::Null,
        Value::Integer(i) => DBValue::Integer(i),
        Value::Number(f) => DBValue::Real(f),
        Value::String(s) => DBValue::Text(s.to_str().unwrap().to_string()),
        Value::UserData(ud) => {
            if ud.is::<crate::lua::LuaBuffer>() {
                let buffer = ud.borrow::<crate::lua::LuaBuffer>().unwrap();
                DBValue::Blob(buffer.buffer.clone().into())
            } else {
                panic!("Unsupported Lua user data type")
            }
        }
        _ => panic!("Unsupported Lua value type"),
    }
}

struct InsertParameters {
    columns: Vec<String>,
    values: Vec<Vec<Value>>,
}

impl InsertParameters {
    fn execute(&self, conn: &rusqlite::Connection, table: &str) -> LuaResult<()> {
        let mut query = format!("INSERT INTO {} (", table);
        let mut params = Vec::new();
        for (i, column) in self.columns.iter().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str(column);
        }
        query.push_str(") VALUES ");
        for (i, row) in self.values.iter().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str("(");
            for (j, value) in row.iter().enumerate() {
                if j > 0 {
                    query.push_str(", ");
                }
                query.push_str("?");
                params.push(lua_to_sqlite_value(value.clone()));
            }
            query.push_str(")");
        }
        let mut stmt = conn.prepare(&query).unwrap();
        stmt.execute(rusqlite::params_from_iter(params.iter()))
            .unwrap();

        Ok(())
    }
}

impl FromLua for InsertParameters {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(t) => {
                let columns = t.get("columns")?;
                let values = t.get("values")?;
                Ok(InsertParameters { columns, values })
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "InsertParameters".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

struct DeleteParameters {
    where_clause: Option<WhereClause>,
}

impl DeleteParameters {
    fn execute(&self, conn: &rusqlite::Connection, table: &str) -> LuaResult<()> {
        let (query, params) = self.build_query(table);
        let mut stmt = conn.prepare(&query).unwrap();
        stmt.execute(rusqlite::params_from_iter(params.iter()))
            .unwrap();
        Ok(())
    }

    fn build_query(&self, table: &str) -> (String, Vec<DBValue>) {
        let mut params = Vec::new();
        let mut query = format!("DELETE FROM {}", table);
        if let Some(where_clause) = &self.where_clause {
            query.push_str(" WHERE ");
            where_clause.build_where(&mut query, &mut params);
        }
        (query, params)
    }
}

impl FromLua for DeleteParameters {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(t) => {
                let where_clause = if t.contains_key("where")? {
                    Some(t.get("where")?)
                } else {
                    None
                };
                Ok(DeleteParameters { where_clause })
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "DeleteParameters".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

struct QueryParameters {
    columns: Vec<String>,
    where_clause: Option<WhereClause>,
}
impl QueryParameters {
    fn execute(
        &self,
        lua: &Lua,
        conn: &rusqlite::Connection,
        table: &str,
    ) -> LuaResult<Vec<Value>> {
        let (query, params) = self.build_query(table);
        let mut stmt = conn.prepare(&query).unwrap();
        let mut rows = Vec::new();
        let mut rows_iter = stmt
            .query(rusqlite::params_from_iter(params.iter()))
            .unwrap();
        while let Some(row) = rows_iter.next().unwrap() {
            let mut row_values = Vec::new();
            for i in 0..self.columns.len() {
                row_values.push(sqlite_to_lua_value(lua, &row.get(i).unwrap()));
            }
            rows.push(Value::Table(lua.create_sequence_from(row_values).unwrap()));
        }
        Ok(rows)
    }

    fn build_query(&self, table: &str) -> (String, Vec<DBValue>) {
        let mut query = format!("SELECT ");
        for (i, column) in self.columns.iter().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str(column);
        }
        query.push_str(&format!(" FROM {}", table));
        let mut params = Vec::new();
        if let Some(where_clause) = &self.where_clause {
            where_clause.build_where(&mut query, &mut params);
        }
        (query, params)
    }
}

impl FromLua for QueryParameters {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(t) => {
                let columns = t.get("columns")?;
                let where_clause = if t.contains_key("where")? {
                    Some(t.get("where")?)
                } else {
                    None
                };
                Ok(QueryParameters {
                    columns,
                    where_clause,
                })
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "QueryParameters".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

enum WhereClause {
    And(Box<WhereClause>, Box<WhereClause>),
    Or(Box<WhereClause>, Box<WhereClause>),
    BinaryOp(String, WhereOperator, Value),
}

impl WhereClause {
    fn build_where(&self, query: &mut String, params: &mut Vec<DBValue>) {
        match self {
            WhereClause::And(left, right) => {
                query.push_str("(");
                left.build_where(query, params);
                query.push_str(") AND (");
                right.build_where(query, params);
                query.push_str(")");
            }
            WhereClause::Or(left, right) => {
                query.push_str("(");
                left.build_where(query, params);
                query.push_str(") OR (");
                right.build_where(query, params);
                query.push_str(")");
            }
            WhereClause::BinaryOp(column, operator, value) => {
                query.push_str(column);
                match operator {
                    WhereOperator::Equal => query.push_str("= ?"),
                    WhereOperator::NotEqual => query.push_str("!= ?"),
                    WhereOperator::GreaterThan => query.push_str("> ?"),
                    WhereOperator::GreaterThanOrEqual => query.push_str(">= ?"),
                    WhereOperator::LessThan => query.push_str("< ?"),
                    WhereOperator::LessThanOrEqual => query.push_str("<= ?"),
                    WhereOperator::Like => query.push_str("LIKE ?"),
                }
                params.push(lua_to_sqlite_value(value.clone()));
            }
        }
    }
}

impl FromLua for WhereClause {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(t) => {
                // Lisp-like tree structure of nested tables, which are used like lists
                // First element is either and, or, or an operator

                let first = t.get(1)?;
                match first {
                    Value::String(s) => match &*s.to_str()? {
                        "and" => {
                            let left = t.get(2)?;
                            let right = t.get(3)?;
                            Ok(WhereClause::And(
                                Box::new(WhereClause::from_lua(left, lua)?),
                                Box::new(WhereClause::from_lua(right, lua)?),
                            ))
                        }
                        "or" => {
                            let left = t.get(2)?;
                            let right = t.get(3)?;
                            Ok(WhereClause::Or(
                                Box::new(WhereClause::from_lua(left, lua)?),
                                Box::new(WhereClause::from_lua(right, lua)?),
                            ))
                        }
                        _ => {
                            let column = s.to_str()?.to_string();
                            let operator = t.get(2)?;
                            let value = t.get(3)?;
                            Ok(WhereClause::BinaryOp(column, operator, value))
                        }
                    },
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: "Value",
                        to: "WhereClause".to_string(),
                        message: Some("expected string".to_string()),
                    }),
                }
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "WhereClause".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

enum WhereOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
}

impl FromLua for WhereOperator {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::String(s) => match &*s.to_str()? {
                "eq" => Ok(WhereOperator::Equal),
                "ne" => Ok(WhereOperator::NotEqual),
                "gt" => Ok(WhereOperator::GreaterThan),
                "ge" => Ok(WhereOperator::GreaterThanOrEqual),
                "lt" => Ok(WhereOperator::LessThan),
                "le" => Ok(WhereOperator::LessThanOrEqual),
                "like" => Ok(WhereOperator::Like),
                _ => Err(mlua::Error::FromLuaConversionError {
                    from: "Value",
                    to: "WhereOperator".to_string(),
                    message: Some("invalid operator".to_string()),
                }),
            },
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "WhereOperator".to_string(),
                message: Some("expected string".to_string()),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TableLease<'t> {
    conn: &'t rusqlite::Connection,
    table: &'t str,
}

impl<'t> TableLease<'t> {
    pub fn new(conn: &'t rusqlite::Connection, table: &'t str) -> TableLease<'t> {
        TableLease { conn, table }
    }

    pub fn count(&self) -> LuaResult<Value> {
        let query = format!("SELECT COUNT(*) FROM {}", self.table);
        let result: i64 = self.conn.query_row(&query, (), |row| row.get(0)).unwrap();
        Ok(Value::Integer(result))
    }
}

impl<'t> UserData for TableLease<'t> {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("query", |scope, this, query: QueryParameters| {
            query.execute(scope, this.conn, this.table)
        });
        methods.add_method("insert", |_, this, insert: InsertParameters| {
            insert.execute(this.conn, this.table)
        });
        methods.add_method("delete", |_, this, delete: DeleteParameters| {
            delete.execute(this.conn, this.table)
        });
        // count
        methods.add_method("count", |_, this, _empty: ()| this.count());
    }
}
