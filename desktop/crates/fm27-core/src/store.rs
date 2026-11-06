use crate::{
    analysis::{analyze, player_row, Analysis, Formation, PlayerRow},
    model::{demo_snapshot, parse_snapshot, Snapshot},
    Error, Result,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct Store {
    connection: Connection,
    snapshot: Snapshot,
    formation: Formation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub club: String,
    pub season: String,
    pub source: crate::model::Source,
    pub captured_at: String,
    pub player_count: usize,
    pub analysis: Analysis,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerQuery {
    pub search: Option<String>,
    pub position: Option<crate::model::Position>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerPage {
    pub players: Vec<PlayerRow>,
    pub total: usize,
    pub offset: usize,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }
    pub fn memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }
    fn from_connection(connection: Connection) -> Result<Self> {
        connection.busy_timeout(std::time::Duration::from_secs(3))?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS snapshots (id INTEGER PRIMARY KEY, body TEXT NOT NULL, imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')));
            CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);")?;
        // Length is checked in SQLite before allocating the stored payload.
        if let Some(true) = connection
            .query_row(
                "SELECT length(CAST(body AS BLOB)) > ? FROM snapshots ORDER BY id DESC LIMIT 1",
                [crate::model::MAX_IMPORT_BYTES as i64],
                |r| r.get::<_, bool>(0),
            )
            .optional()?
        {
            return Err(Error::Invalid(
                "Kayıtlı kadro izin verilen boyutu aşıyor.".into(),
            ));
        }
        let stored: Option<String> = connection
            .query_row(
                "SELECT body FROM snapshots ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let snapshot = match stored {
            Some(body) => parse_snapshot(body.as_bytes())?,
            None => demo_snapshot(),
        };
        let formation: Formation = connection
            .query_row(
                "SELECT value FROM settings WHERE key='formation'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default();
        let mut store = Self {
            connection,
            snapshot,
            formation,
        };
        if store
            .connection
            .query_row("SELECT count(*) FROM snapshots", [], |r| r.get::<_, i64>(0))?
            == 0
        {
            store.replace(store.snapshot.clone())?;
        }
        Ok(store)
    }
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub fn formation(&self) -> Formation {
        self.formation
    }
    pub fn dashboard(&self) -> Dashboard {
        Dashboard {
            club: self.snapshot.club.clone(),
            season: self.snapshot.season.clone(),
            source: self.snapshot.source,
            captured_at: self.snapshot.captured_at.clone(),
            player_count: self.snapshot.players.len(),
            analysis: analyze(&self.snapshot, self.formation),
        }
    }
    pub fn import(&mut self, content: &[u8]) -> Result<Dashboard> {
        let snapshot = parse_snapshot(content)?.imported();
        self.replace(snapshot)?;
        Ok(self.dashboard())
    }
    pub fn reset_demo(&mut self) -> Result<Dashboard> {
        self.replace(demo_snapshot())?;
        Ok(self.dashboard())
    }
    pub fn replace(&mut self, snapshot: Snapshot) -> Result<()> {
        snapshot.validate()?;
        let body = serde_json::to_string(&snapshot)?;
        if body.len() > crate::model::MAX_IMPORT_BYTES {
            return Err(Error::Invalid("Kayıt boyutu sınırı aşıldı.".into()));
        }
        let transaction = self.connection.transaction()?;
        transaction.execute("INSERT INTO snapshots(body) VALUES (?)", [body])?;
        // Retain the previous two valid snapshots. A failed import never clears one.
        transaction.execute("DELETE FROM snapshots WHERE id NOT IN (SELECT id FROM snapshots ORDER BY id DESC LIMIT 3)", [])?;
        transaction.commit()?;
        self.snapshot = snapshot;
        Ok(())
    }
    pub fn set_formation(&mut self, formation: Formation) -> Result<Dashboard> {
        self.connection.execute("INSERT INTO settings(key,value) VALUES ('formation',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![serde_json::to_string(&formation)?])?;
        self.formation = formation;
        Ok(self.dashboard())
    }
    pub fn players(&self, query: PlayerQuery) -> PlayerPage {
        let search = query.search.unwrap_or_default().to_lowercase();
        let offset = query.offset.unwrap_or(0);
        let matching: Vec<_> = self
            .snapshot
            .players
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&search)
                    && query.position.is_none_or(|pos| p.positions.contains(&pos))
            })
            .collect();
        PlayerPage {
            total: matching.len(),
            offset,
            players: matching
                .into_iter()
                .skip(offset)
                .take(query.limit.unwrap_or(30).clamp(1, 100))
                .map(player_row)
                .collect(),
        }
    }
}
