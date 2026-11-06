use crate::{
    analysis::{fits, player_row, PlayerRow, ROLES},
    model::{Foot, Player, Position, Snapshot, ATTRIBUTES, HIDDEN_ATTRIBUTES},
    Error, Result,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleOption {
    pub id: String,
    pub name: String,
    pub position: Position,
}

pub fn role_catalog() -> Vec<RoleOption> {
    ROLES
        .iter()
        .map(|r| RoleOption {
            id: r.id.into(),
            name: r.name.into(),
            position: r.position,
        })
        .collect()
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttributeFilter {
    pub key: String,
    pub min: Option<u8>,
    pub max: Option<u8>,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerSort {
    #[default]
    Name,
    Age,
    Ability,
    Potential,
    Condition,
    Form,
    Role,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerQuery {
    pub search: Option<String>,
    pub position: Option<Position>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub min_age: Option<u8>,
    pub max_age: Option<u8>,
    pub min_condition: Option<u8>,
    pub min_form: Option<f64>,
    pub min_ca: Option<u16>,
    pub min_pa: Option<u16>,
    pub foot: Option<Foot>,
    pub role_id: Option<String>,
    pub min_role_score: Option<f64>,
    pub list_id: Option<i64>,
    #[serde(default)]
    pub available_only: bool,
    #[serde(default)]
    pub attributes: Vec<AttributeFilter>,
    #[serde(default)]
    pub sort: PlayerSort,
    #[serde(default)]
    pub descending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerPage {
    pub players: Vec<PlayerRow>,
    pub total: usize,
    pub offset: usize,
}

impl PlayerQuery {
    pub fn validate(&self) -> Result<()> {
        let bad = || {
            Error::Invalid("Filtre aralıklarını kontrol et. Alt sınır üst sınırı aşmamalı.".into())
        };
        if self.min_age.is_some_and(|v| !(14..=60).contains(&v))
            || self.max_age.is_some_and(|v| !(14..=60).contains(&v))
            || self.min_age.zip(self.max_age).is_some_and(|(a, b)| a > b)
            || self.min_condition.is_some_and(|v| v > 100)
            || self.min_ca.is_some_and(|v| !(1..=200).contains(&v))
            || self.min_pa.is_some_and(|v| !(1..=200).contains(&v))
            || self
                .min_form
                .is_some_and(|v| !v.is_finite() || !(0.0..=10.0).contains(&v))
            || self
                .min_role_score
                .is_some_and(|v| !v.is_finite() || !(0.0..=100.0).contains(&v))
            || self.attributes.len() > 32
            || self
                .search
                .as_ref()
                .is_some_and(|s| s.chars().count() > 100)
            || self
                .role_id
                .as_ref()
                .is_some_and(|id| !ROLES.iter().any(|r| r.id == id))
        {
            return Err(bad());
        }
        for filter in &self.attributes {
            let keys = if filter.hidden {
                HIDDEN_ATTRIBUTES
            } else {
                ATTRIBUTES
            };
            if !keys.contains(&filter.key.as_str())
                || filter.min.is_some_and(|v| !(1..=20).contains(&v))
                || filter.max.is_some_and(|v| !(1..=20).contains(&v))
                || filter.min.zip(filter.max).is_some_and(|(a, b)| a > b)
            {
                return Err(bad());
            }
        }
        Ok(())
    }
}

// Turkish dotted/dotless I share a search form, including uppercase imports.
fn search_key(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| match c {
            'I' | 'İ' | 'ı' => vec!['i'],
            c => c.to_lowercase().collect(),
        })
        .collect()
}

fn optional_order<T: PartialOrd>(a: Option<T>, b: Option<T>, descending: bool) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => {
            let order = a.partial_cmp(&b).unwrap_or(Ordering::Equal);
            if descending {
                order.reverse()
            } else {
                order
            }
        }
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}

pub fn query_players(
    snapshot: &Snapshot,
    query: PlayerQuery,
    members: Option<&HashSet<u32>>,
) -> Result<PlayerPage> {
    query.validate()?;
    let search = search_key(query.search.as_deref().unwrap_or(""));
    let mut rows: Vec<_> = snapshot
        .players
        .iter()
        .filter(|p| {
            search_key(&p.name).contains(&search)
                && query.position.is_none_or(|pos| p.positions.contains(&pos))
                && query.min_age.is_none_or(|v| p.age >= v)
                && query.max_age.is_none_or(|v| p.age <= v)
                && query.min_condition.is_none_or(|v| p.condition >= v)
                && query
                    .min_form
                    .is_none_or(|v| p.form.is_some_and(|n| n >= v))
                && query
                    .min_ca
                    .is_none_or(|v| p.current_ability.is_some_and(|n| n >= v))
                && query
                    .min_pa
                    .is_none_or(|v| p.potential_ability.is_some_and(|n| n >= v))
                && query.foot.is_none_or(|v| p.foot == v)
                && (!query.available_only || !p.injured)
                && members.is_none_or(|ids| ids.contains(&p.id))
                && query.attributes.iter().all(|f| {
                    let attrs = if f.hidden {
                        &p.hidden_attributes
                    } else {
                        &p.attributes
                    };
                    attrs.get(&f.key).is_some_and(|n| {
                        f.min.is_none_or(|v| *n >= v) && f.max.is_none_or(|v| *n <= v)
                    })
                })
        })
        .filter_map(|p| {
            let mut row = player_row(p);
            if let Some(id) = &query.role_id {
                row.role = fits(p).into_iter().find(|r| &r.role_id == id);
                row.role.as_ref()?;
            }
            if query.min_role_score.is_some_and(|v| {
                !row.role
                    .as_ref()
                    .and_then(|r| r.score)
                    .is_some_and(|n| n >= v)
            }) {
                return None;
            }
            Some(row)
        })
        .collect();
    rows.sort_by(|a, b| {
        let order = match query.sort {
            PlayerSort::Name => optional_order(
                Some(search_key(&a.name)),
                Some(search_key(&b.name)),
                query.descending,
            ),
            PlayerSort::Age => optional_order(Some(a.age), Some(b.age), query.descending),
            PlayerSort::Ability => {
                optional_order(a.current_ability, b.current_ability, query.descending)
            }
            PlayerSort::Potential => {
                optional_order(a.potential_ability, b.potential_ability, query.descending)
            }
            PlayerSort::Condition => {
                optional_order(Some(a.condition), Some(b.condition), query.descending)
            }
            PlayerSort::Form => optional_order(a.form, b.form, query.descending),
            PlayerSort::Role => optional_order(
                a.role.as_ref().and_then(|r| r.score),
                b.role.as_ref().and_then(|r| r.score),
                query.descending,
            ),
        };
        order.then_with(|| a.id.cmp(&b.id))
    });
    let offset = query.offset.unwrap_or(0);
    Ok(PlayerPage {
        total: rows.len(),
        offset,
        players: rows
            .into_iter()
            .skip(offset)
            .take(query.limit.unwrap_or(30).clamp(1, 100))
            .collect(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortlistEntry {
    pub player_id: u32,
    pub name: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shortlist {
    pub id: i64,
    pub name: String,
    pub entries: Vec<ShortlistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryPoint {
    pub game_date: String,
    pub captured_at: String,
    pub current_ability: Option<u16>,
    pub potential_ability: Option<u16>,
    pub attributes: std::collections::BTreeMap<String, u8>,
}

impl HistoryPoint {
    pub fn validate(&self) -> Result<()> {
        if self.game_date.len() != 10
            || chrono::NaiveDate::parse_from_str(&self.game_date, "%Y-%m-%d").is_err()
            || chrono::DateTime::parse_from_rfc3339(&self.captured_at).is_err()
            || self
                .current_ability
                .is_some_and(|v| !(1..=200).contains(&v))
            || self
                .potential_ability
                .is_some_and(|v| !(1..=200).contains(&v))
            || self
                .attributes
                .iter()
                .any(|(k, v)| !ATTRIBUTES.contains(&k.as_str()) || !(1..=20).contains(v))
        {
            return Err(Error::Invalid("Kayıtlı gelişim verisi geçersiz.".into()));
        }
        Ok(())
    }
    pub fn from_player(p: &Player, snapshot: &Snapshot) -> Self {
        Self {
            game_date: snapshot.game_date.clone().unwrap_or_default(),
            captured_at: snapshot.captured_at.clone(),
            current_ability: p.current_ability,
            potential_ability: p.potential_ability,
            attributes: p.attributes.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerHistory {
    pub points: Vec<HistoryPoint>,
    pub message: String,
}
