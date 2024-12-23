use mlua::prelude::*;
use mlua::{Function, UserData, Value};
use rusqlite::types::Value as DBValue;
use std::path::Path;

pub fn create_connection() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open("test.db").unwrap();
    conn
}

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

// Creates the table in which the plugin stores its data
pub fn create_content_table_for_fts(conn: &rusqlite::Connection, name: &str, columns: &[String]) {
    let mut create_table_query = format!("CREATE TABLE IF NOT EXISTS {} (", name);
    for column in columns {
        create_table_query.push_str(&format!("{}, ", column));
    }
    create_table_query.push_str("PRIMARY KEY (");
    create_table_query.push_str(&columns[0]);
    create_table_query.push_str("));");
    conn.execute(&create_table_query, []).unwrap();
}

pub fn create_fts_table(
    conn: &rusqlite::Connection,
    name: &str,
    columns: &[String],
    search_columns: &[String],
) {
    let mut create_table_query = format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS {}_search USING fts5(",
        name
    );
    for column in search_columns {
        create_table_query.push_str(&format!("{}, ", column));
    }
    create_table_query.push_str(&format!(
        "tokenize=\"trigram\", content=\"{}\", content_rowid=\"{}\");",
        name, &columns[0]
    ));
    conn.execute(&create_table_query, []).unwrap();
}

pub fn create_after_insert_trigger(
    conn: &rusqlite::Connection,
    name: &str,
    columns: &[String],
    search_columns: &[String],
) {
    let mut create_ai_trigger_statement = format!(
        "CREATE TRIGGER IF NOT EXISTS {}_ai AFTER INSERT ON {} BEGIN ",
        name, name
    );
    create_ai_trigger_statement.push_str(&format!("INSERT INTO {}_search({}", name, &columns[0]));
    for column in search_columns {
        create_ai_trigger_statement.push_str(&format!(", {}", column));
    }
    create_ai_trigger_statement.push_str(") VALUES (");
    create_ai_trigger_statement.push_str("new.");
    create_ai_trigger_statement.push_str(&columns[0]);
    for column in search_columns {
        create_ai_trigger_statement.push_str(&format!(", new.{}", column));
    }

    create_ai_trigger_statement.push_str("); END;");
    conn.execute(&create_ai_trigger_statement, []).unwrap();
}

pub fn create_after_delete_trigger(
    conn: &rusqlite::Connection,
    name: &str,
    columns: &[String],
    search_columns: &[String],
) {
    let mut create_ad_trigger_statement = format!(
        "CREATE TRIGGER IF NOT EXISTS {}_ad AFTER DELETE ON {} BEGIN ",
        name, name
    );
    create_ad_trigger_statement.push_str(&format!(
        "INSERT INTO {}_search({}_search, {}",
        name, name, &columns[0]
    ));
    for column in search_columns {
        create_ad_trigger_statement.push_str(&format!(", {}", column));
    }
    create_ad_trigger_statement.push_str(") VALUES ('delete', ");
    create_ad_trigger_statement.push_str("old.");
    create_ad_trigger_statement.push_str(&columns[0]);
    for column in search_columns {
        create_ad_trigger_statement.push_str(&format!(", old.{}", column));
    }

    create_ad_trigger_statement.push_str("); END;");
    conn.execute(&create_ad_trigger_statement, []).unwrap();
}

pub fn create_after_update_trigger(
    conn: &rusqlite::Connection,
    name: &str,
    columns: &[String],
    search_columns: &[String],
) {
    let mut create_au_trigger_statement = format!(
        "CREATE TRIGGER IF NOT EXISTS {}_au AFTER UPDATE ON {} BEGIN ",
        name, name
    );
    create_au_trigger_statement.push_str(&format!(
        "INSERT INTO {}_search({}_search, {}",
        name, name, &columns[0]
    ));
    for column in search_columns {
        create_au_trigger_statement.push_str(&format!(", {}", column));
    }
    create_au_trigger_statement.push_str(") VALUES ('delete', ");
    create_au_trigger_statement.push_str("old.");
    create_au_trigger_statement.push_str(&columns[0]);
    for column in search_columns {
        create_au_trigger_statement.push_str(&format!(", old.{}", column));
    }

    create_au_trigger_statement.push_str("); ");
    create_au_trigger_statement.push_str(&format!("INSERT INTO {}_search({}", name, &columns[0]));
    for column in search_columns {
        create_au_trigger_statement.push_str(&format!(", {}", column));
    }
    create_au_trigger_statement.push_str(") VALUES (");
    create_au_trigger_statement.push_str("new.");
    create_au_trigger_statement.push_str(&columns[0]);
    for column in search_columns {
        create_au_trigger_statement.push_str(&format!(", new.{}", column));
    }

    create_au_trigger_statement.push_str("); END;");
    conn.execute(&create_au_trigger_statement, []).unwrap();
}

pub fn create_fts_triggers(
    conn: &rusqlite::Connection,
    name: &str,
    columns: &[String],
    search_columns: &[String],
) {
    create_after_insert_trigger(conn, name, columns, search_columns);
    create_after_delete_trigger(conn, name, columns, search_columns);
    create_after_update_trigger(conn, name, columns, search_columns);
}

// Create the content table, FTS5 table for search and necessary triggers
pub fn create_fts_objects(
    conn: &rusqlite::Connection,
    name: &str,
    columns: &[String],
    search_columns: &[String],
) {
    create_content_table_for_fts(conn, name, columns);
    create_fts_table(conn, name, columns, search_columns);
    create_fts_triggers(conn, name, columns, search_columns);
}
