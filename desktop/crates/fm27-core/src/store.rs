pub use crate::scouting::{PlayerPage, PlayerQuery};
use crate::{
    analysis::{analyze, Analysis, Formation},
    model::{demo_snapshot, parse_snapshot, Snapshot},
    scouting::{query_players, HistoryPoint, PlayerHistory, Shortlist, ShortlistEntry},
    Error, Result,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct Store {
    connection: Connection,
    snapshot: Snapshot,
    formation: Formation,
    snapshot_id: i64,
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
    pub game_date: Option<String>,
    pub roles: Vec<crate::scouting::RoleOption>,
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
            CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS scout_lists (id INTEGER PRIMARY KEY, scope TEXT NOT NULL, name TEXT NOT NULL, UNIQUE(scope,name));
            CREATE TABLE IF NOT EXISTS scout_entries (list_id INTEGER NOT NULL REFERENCES scout_lists(id) ON DELETE CASCADE, player_id INTEGER NOT NULL, name TEXT NOT NULL, PRIMARY KEY(list_id,player_id));
            CREATE TABLE IF NOT EXISTS player_history (scope TEXT NOT NULL, player_id INTEGER NOT NULL, name TEXT NOT NULL, game_date TEXT NOT NULL, body TEXT NOT NULL, PRIMARY KEY(scope,player_id,game_date));")?;
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
        let stored: Option<(i64, String)> = connection
            .query_row(
                "SELECT id,body FROM snapshots ORDER BY id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let (snapshot_id, snapshot) = match stored {
            Some((id, body)) => (id, parse_snapshot(body.as_bytes())?),
            None => (0, demo_snapshot()),
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
            snapshot_id,
        };
        if store
            .connection
            .query_row("SELECT count(*) FROM snapshots", [], |r| r.get::<_, i64>(0))?
            == 0
        {
            store.replace(store.snapshot.clone())?;
        }
        store.connection.execute("INSERT INTO scout_lists(scope,name) SELECT ?, 'Takip listem' WHERE NOT EXISTS (SELECT 1 FROM scout_lists WHERE scope=?)", params![store.scope(), store.scope()])?;
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
            game_date: self.snapshot.game_date.clone(),
            roles: crate::scouting::role_catalog(),
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
        let snapshot_id = transaction.last_insert_rowid();
        let scope = snapshot_scope(&snapshot, snapshot_id);
        transaction.execute("INSERT INTO scout_lists(scope,name) SELECT ?, 'Takip listem' WHERE NOT EXISTS (SELECT 1 FROM scout_lists WHERE scope=?)", params![scope, scope])?;
        // Game time, never the machine clock, defines development. Missing
        // career identity isolates manual imports rather than mixing careers.
        if snapshot.game_date.is_some()
            && (snapshot.source == crate::Source::Demo || snapshot.career_id.is_some())
        {
            for player in &snapshot.players {
                let point = HistoryPoint::from_player(player, &snapshot);
                transaction.execute("INSERT INTO player_history(scope,player_id,name,game_date,body) VALUES (?,?,?,?,?) ON CONFLICT(scope,player_id,game_date) DO UPDATE SET name=excluded.name,body=excluded.body", params![scope, player.id, player.name, point.game_date, serde_json::to_string(&point)?])?;
                transaction.execute("DELETE FROM player_history WHERE scope=? AND player_id=? AND game_date NOT IN (SELECT game_date FROM player_history WHERE scope=? AND player_id=? ORDER BY game_date DESC LIMIT 24)", params![scope, player.id, scope, player.id])?;
            }
        }
        // Retain the previous two valid snapshots. A failed import never clears one.
        transaction.execute("DELETE FROM snapshots WHERE id NOT IN (SELECT id FROM snapshots ORDER BY id DESC LIMIT 3)", [])?;
        transaction.commit()?;
        self.snapshot = snapshot;
        self.snapshot_id = snapshot_id;
        Ok(())
    }
    pub fn set_formation(&mut self, formation: Formation) -> Result<Dashboard> {
        self.connection.execute("INSERT INTO settings(key,value) VALUES ('formation',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![serde_json::to_string(&formation)?])?;
        self.formation = formation;
        Ok(self.dashboard())
    }
    pub fn players(&self, query: PlayerQuery) -> Result<PlayerPage> {
        let members = query
            .list_id
            .map(|id| {
                self.shortlists()?
                    .into_iter()
                    .find(|l| l.id == id)
                    .map(|l| {
                        l.entries
                            .into_iter()
                            .filter(|e| e.available)
                            .map(|e| e.player_id)
                            .collect::<std::collections::HashSet<_>>()
                    })
                    .ok_or_else(|| Error::Invalid("Takip listesi bulunamadı.".into()))
            })
            .transpose()?;
        query_players(&self.snapshot, query, members.as_ref())
    }

    fn scope(&self) -> String {
        snapshot_scope(&self.snapshot, self.snapshot_id)
    }

    pub fn shortlists(&self) -> Result<Vec<Shortlist>> {
        let current: std::collections::HashMap<_, _> = self
            .snapshot
            .players
            .iter()
            .map(|p| (p.id, p.name.as_str()))
            .collect();
        let mut statement = self
            .connection
            .prepare("SELECT id,name FROM scout_lists WHERE scope=? ORDER BY id LIMIT 20")?;
        let rows = statement.query_map([self.scope()], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut lists = Vec::new();
        for row in rows {
            let (id, name) = row?;
            let mut entries = self.connection.prepare("SELECT player_id,name FROM scout_entries WHERE list_id=? ORDER BY name,player_id LIMIT 5000")?;
            let entries = entries
                .query_map([id], |r| {
                    let player_id = r.get::<_, u32>(0)?;
                    let name = r.get::<_, String>(1)?;
                    let available = current.get(&player_id).is_some_and(|n| *n == name);
                    Ok(ShortlistEntry {
                        player_id,
                        name,
                        available,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            lists.push(Shortlist { id, name, entries });
        }
        Ok(lists)
    }

    pub fn create_shortlist(&mut self, name: &str) -> Result<Vec<Shortlist>> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 60 || name.chars().any(char::is_control) {
            return Err(Error::Invalid("Liste adı 1–60 karakter olmalı.".into()));
        }
        let lists = self.shortlists()?;
        if lists.len() >= 20
            || lists
                .iter()
                .any(|l| l.name.to_lowercase() == name.to_lowercase())
        {
            return Err(Error::Invalid(
                "Farklı bir liste adı seç. En fazla 20 liste oluşturabilirsin.".into(),
            ));
        }
        self.connection.execute(
            "INSERT INTO scout_lists(scope,name) VALUES (?,?)",
            params![self.scope(), name],
        )?;
        self.shortlists()
    }

    pub fn delete_empty_shortlist(&mut self, id: i64) -> Result<Vec<Shortlist>> {
        let lists = self.shortlists()?;
        if lists.len() <= 1 || !lists.iter().any(|l| l.id == id && l.entries.is_empty()) {
            return Err(Error::Invalid(
                "Yalnızca boş bir ek liste silinebilir.".into(),
            ));
        }
        self.connection.execute(
            "DELETE FROM scout_lists WHERE id=? AND scope=?",
            params![id, self.scope()],
        )?;
        self.shortlists()
    }

    pub fn set_shortlisted(
        &mut self,
        list_id: i64,
        player_id: u32,
        selected: bool,
    ) -> Result<Vec<Shortlist>> {
        if !self.shortlists()?.iter().any(|l| l.id == list_id) {
            return Err(Error::Invalid("Takip listesi bulunamadı.".into()));
        }
        if selected {
            let count: i64 = self.connection.query_row(
                "SELECT count(*) FROM scout_entries WHERE list_id=?",
                [list_id],
                |r| r.get(0),
            )?;
            let exists: bool = self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM scout_entries WHERE list_id=? AND player_id=?)",
                params![list_id, player_id],
                |r| r.get(0),
            )?;
            if count >= 5000 && !exists {
                return Err(Error::Invalid(
                    "Bir takip listesi en fazla 5.000 oyuncu içerebilir.".into(),
                ));
            }
            let p = self
                .snapshot
                .players
                .iter()
                .find(|p| p.id == player_id)
                .ok_or_else(|| Error::Invalid("Oyuncu bulunamadı.".into()))?;
            self.connection.execute("INSERT INTO scout_entries(list_id,player_id,name) VALUES (?,?,?) ON CONFLICT(list_id,player_id) DO UPDATE SET name=excluded.name", params![list_id, player_id, p.name])?;
        } else {
            self.connection.execute(
                "DELETE FROM scout_entries WHERE list_id=? AND player_id=?",
                params![list_id, player_id],
            )?;
        }
        self.shortlists()
    }

    pub fn player_history(&self, id: u32) -> Result<PlayerHistory> {
        let p = self
            .snapshot
            .players
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| Error::Invalid("Oyuncu bulunamadı.".into()))?;
        if self.snapshot.source != crate::Source::Demo && self.snapshot.career_id.is_none() {
            return Ok(PlayerHistory {
                points: vec![],
                message: "Gelişim için aynı kariyer kimliğiyle hazırlanmış kayıtlar gerekli."
                    .into(),
            });
        }
        let Some(date) = &self.snapshot.game_date else {
            return Ok(PlayerHistory {
                points: vec![],
                message: "Gelişim için dosyada oyun tarihi bulunmalı.".into(),
            });
        };
        let mut stmt = self.connection.prepare("SELECT CASE WHEN length(CAST(body AS BLOB))<=8192 THEN body ELSE NULL END FROM player_history WHERE scope=? AND player_id=? AND name=? AND game_date<=? ORDER BY game_date DESC LIMIT 24")?;
        let bodies = stmt.query_map(params![self.scope(), id, p.name, date], |r| {
            r.get::<_, Option<String>>(0)
        })?;
        let mut points = Vec::new();
        for body in bodies {
            let body = body?.ok_or_else(|| {
                Error::Invalid("Kayıtlı gelişim verisi izin verilen boyutu aşıyor.".into())
            })?;
            let point: HistoryPoint = serde_json::from_str(&body)?;
            point.validate()?;
            points.push(point);
        }
        points.reverse();
        Ok(PlayerHistory {
            message: if points.len() < 2 {
                "Değişimi görmek için ileri bir oyun tarihinden yeni kadro ekle."
            } else {
                "Aynı kariyerin son 24 oyun tarihi. Yalnızca kaydedilmiş değerler gösterilir."
            }
            .into(),
            points,
        })
    }
}

fn snapshot_scope(snapshot: &Snapshot, id: i64) -> String {
    if snapshot.source == crate::Source::Demo {
        format!("demo:{}", snapshot.career_id.as_deref().unwrap_or("legacy"))
    } else if let Some(career) = &snapshot.career_id {
        format!("career:{career}")
    } else {
        format!("snapshot:{id}")
    }
}
