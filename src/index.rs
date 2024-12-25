use crate::plugin::PassivePlugin;

pub fn index<'a>(
    lua: &mlua::Lua,
    conn: &rusqlite::Connection,
    plugins: &mut impl Iterator<Item = &'a mut PassivePlugin>,
) -> Result<(), Box<dyn std::error::Error>> {
    for plugin in plugins {
        plugin.refresh_rows(lua, conn);
    }
    Ok(())
}
