use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const MAX_IMPORT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_PLAYERS: usize = 5_000;
pub const HIDDEN_ATTRIBUTES: &[&str] = &[
    "consistency",
    "importantMatches",
    "injuryProneness",
    "versatility",
    "professionalism",
    "ambition",
    "pressure",
    "temperament",
    "loyalty",
    "adaptability",
    "sportsmanship",
    "controversy",
    "dirtiness",
];
pub const ATTRIBUTES: &[&str] = &[
    "acceleration",
    "pace",
    "stamina",
    "strength",
    "agility",
    "jumpingReach",
    "passing",
    "firstTouch",
    "technique",
    "dribbling",
    "finishing",
    "crossing",
    "tackling",
    "marking",
    "heading",
    "positioning",
    "anticipation",
    "decisions",
    "vision",
    "offTheBall",
    "teamwork",
    "workRate",
    "composure",
    "concentration",
    "reflexes",
    "handling",
    "aerialReach",
    "oneOnOnes",
    "kicking",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Demo,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    GK,
    DC,
    DL,
    DR,
    DM,
    MC,
    AML,
    AMR,
    AMC,
    ST,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Foot {
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub age: u8,
    pub positions: Vec<Position>,
    pub foot: Foot,
    pub condition: u8,
    pub morale: u8,
    pub form: Option<f64>,
    pub minutes: u32,
    pub injured: bool,
    pub attributes: BTreeMap<String, u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_ability: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub potential_ability: Option<u16>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hidden_attributes: BTreeMap<String, u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Snapshot {
    pub schema_version: u32,
    pub club: String,
    pub season: String,
    pub captured_at: String,
    pub source: Source,
    pub players: Vec<Player>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub career_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_date: Option<String>,
}

pub fn parse_snapshot(bytes: &[u8]) -> Result<Snapshot> {
    if bytes.len() > MAX_IMPORT_BYTES {
        return Err(Error::Invalid(
            "Kadro dosyası en fazla 4 MB olabilir.".into(),
        ));
    }
    let snapshot: Snapshot = serde_json::from_slice(bytes)?;
    snapshot.validate()?;
    Ok(snapshot)
}

fn valid_label(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= max && !value.chars().any(char::is_control)
}

impl Snapshot {
    pub fn validate(&self) -> Result<()> {
        let invalid = |message: &str| Error::Invalid(message.into());
        if self.schema_version != 1 {
            return Err(invalid("Desteklenmeyen kadro şeması. Beklenen sürüm: 1."));
        }
        if !valid_label(&self.club, 100) || !valid_label(&self.season, 30) {
            return Err(invalid("Kulüp ve sezon alanlarını kontrol edin."));
        }
        if self
            .career_id
            .as_ref()
            .is_some_and(|id| !valid_label(id, 100))
        {
            return Err(invalid("Kariyer kimliği 1–100 karakter olmalı."));
        }
        if self.game_date.as_ref().is_some_and(|date| {
            date.len() != 10 || chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err()
        }) {
            return Err(invalid("Oyun tarihi YYYY-AA-GG biçiminde olmalı."));
        }
        if self.captured_at.len() > 64
            || chrono::DateTime::parse_from_rfc3339(&self.captured_at).is_err()
        {
            return Err(invalid(
                "Veri tarihi saat dilimi içeren ISO 8601 biçiminde olmalı.",
            ));
        }
        if self.players.is_empty() || self.players.len() > MAX_PLAYERS {
            return Err(invalid("Kadro 1 ile 5.000 arasında oyuncu içermeli."));
        }
        let mut ids = HashSet::new();
        for player in &self.players {
            if player
                .current_ability
                .is_some_and(|v| !(1..=200).contains(&v))
                || player
                    .potential_ability
                    .is_some_and(|v| !(1..=200).contains(&v))
            {
                return Err(invalid("Mevcut ve potansiyel yetenek 1–200 aralığında olmalı. Bilinmeyen değerleri boş bırak."));
            }
            for (key, value) in &player.hidden_attributes {
                if !HIDDEN_ATTRIBUTES.contains(&key.as_str()) || !(1..=20).contains(value) {
                    return Err(invalid(
                        "Gizli özellikler tanımlı anahtarlarla ve 1–20 aralığında olmalı.",
                    ));
                }
            }
            if player.id == 0 || !ids.insert(player.id) {
                return Err(invalid("Oyuncu kimlikleri pozitif ve benzersiz olmalı."));
            }
            if !valid_label(&player.name, 100)
                || !(14..=60).contains(&player.age)
                || player.positions.is_empty()
                || player.positions.len() > 10
                || player.condition > 100
                || !(1..=20).contains(&player.morale)
                || player.minutes > 100_000
                || player
                    .form
                    .is_some_and(|v| !v.is_finite() || !(0.0..=10.0).contains(&v))
            {
                return Err(invalid(
                    "Oyuncu adı, yaş, mevki, kondisyon, moral veya form alanı geçersiz.",
                ));
            }
            for (key, value) in &player.attributes {
                if !ATTRIBUTES.contains(&key.as_str()) || !(1..=20).contains(value) {
                    return Err(invalid("Özellikler tanımlı anahtarlarla ve 1–20 aralığında girilmeli. Bilinmeyen özellikler boş bırakılmalı."));
                }
            }
        }
        Ok(())
    }

    /// Imported files never gain "live" provenance. A sample cannot silently
    /// become game data, and explicit demo provenance remains visible.
    pub fn imported(mut self) -> Self {
        if self.source != Source::Demo {
            self.source = Source::Manual;
        }
        self
    }
}

pub fn demo_snapshot() -> Snapshot {
    parse_snapshot(include_bytes!("../../../fixtures/demo-squad.json"))
        .expect("bundled fictional demo must validate")
}
