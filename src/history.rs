use crate::model::SearchItem;
use crate::APP_NAME;
use chrono;
use rusqlite::Connection;
use xdg::BaseDirectories;

pub struct History {
    db: Connection,
}

impl History {
    pub fn new() -> Self {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let data_home = xdg_dirs.get_data_home();
        if !data_home.exists() {
            std::fs::create_dir_all(&data_home).unwrap();
        }
        let data_home = data_home.join("history.db");
        let db = Connection::open(data_home).unwrap();
        db.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();
        Self { db }
    }

    pub fn deinit(self) {
        let _ = self.db.close();
    }

    pub fn prune_old(&self) {
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - 60 * 60 * 24 * 30; // 30 days
        self.db
            .execute("DELETE FROM history WHERE timestamp < ?1", (cutoff,))
            .unwrap();
    }

    pub fn add(&self, item: &SearchItem) {
        self.db
            .execute(
                "INSERT INTO history (id, timestamp) VALUES (?1, ?2)",
                (&item.id, &chrono::Utc::now().timestamp()),
            )
            .unwrap();
        self.prune_old();
        self.db.cache_flush().unwrap();
    }

    pub fn get(&self, item: &SearchItem) -> u32 {
        self.db
            .query_row(
                "SELECT COUNT(1) FROM history WHERE id = ?1",
                (&item.id,),
                |row| Ok(row.get(0).unwrap()),
            )
            .unwrap_or(0)
    }
}
