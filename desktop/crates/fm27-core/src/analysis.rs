use crate::model::{Player, Position, Snapshot, Source};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Formation {
    #[default]
    #[serde(rename = "433dm")]
    FourThreeThree,
    #[serde(rename = "4231")]
    FourTwoThreeOne,
    #[serde(rename = "442")]
    FourFourTwo,
}

#[derive(Clone, Copy)]
pub struct Role {
    pub id: &'static str,
    pub name: &'static str,
    pub position: Position,
    pub weights: &'static [(&'static str, u8)],
}

pub const ROLES: &[Role] = &[
    Role {
        id: "keeper",
        name: "Kaleci",
        position: Position::GK,
        weights: &[
            ("reflexes", 3),
            ("handling", 3),
            ("oneOnOnes", 2),
            ("aerialReach", 2),
            ("decisions", 1),
            ("kicking", 1),
        ],
    },
    Role {
        id: "centreBack",
        name: "Stoper",
        position: Position::DC,
        weights: &[
            ("marking", 3),
            ("tackling", 3),
            ("positioning", 3),
            ("heading", 2),
            ("jumpingReach", 2),
            ("strength", 2),
            ("pace", 1),
        ],
    },
    Role {
        id: "leftBack",
        name: "Sol bek",
        position: Position::DL,
        weights: &[
            ("tackling", 2),
            ("positioning", 2),
            ("pace", 2),
            ("stamina", 2),
            ("crossing", 2),
            ("workRate", 2),
            ("passing", 1),
        ],
    },
    Role {
        id: "rightBack",
        name: "Sağ bek",
        position: Position::DR,
        weights: &[
            ("tackling", 2),
            ("positioning", 2),
            ("pace", 2),
            ("stamina", 2),
            ("crossing", 2),
            ("workRate", 2),
            ("passing", 1),
        ],
    },
    Role {
        id: "anchor",
        name: "Savunma önü",
        position: Position::DM,
        weights: &[
            ("positioning", 3),
            ("tackling", 3),
            ("anticipation", 2),
            ("decisions", 2),
            ("teamwork", 2),
            ("passing", 1),
        ],
    },
    Role {
        id: "playmaker",
        name: "Oyun kurucu",
        position: Position::MC,
        weights: &[
            ("passing", 3),
            ("vision", 3),
            ("decisions", 2),
            ("firstTouch", 2),
            ("technique", 2),
            ("composure", 1),
        ],
    },
    Role {
        id: "runner",
        name: "İki yönlü orta saha",
        position: Position::MC,
        weights: &[
            ("stamina", 3),
            ("workRate", 3),
            ("teamwork", 2),
            ("offTheBall", 2),
            ("passing", 1),
            ("tackling", 1),
        ],
    },
    Role {
        id: "leftWing",
        name: "Sol kanat",
        position: Position::AML,
        weights: &[
            ("dribbling", 3),
            ("acceleration", 3),
            ("pace", 2),
            ("crossing", 2),
            ("offTheBall", 2),
            ("technique", 1),
        ],
    },
    Role {
        id: "rightWing",
        name: "Sağ kanat",
        position: Position::AMR,
        weights: &[
            ("dribbling", 3),
            ("acceleration", 3),
            ("pace", 2),
            ("crossing", 2),
            ("offTheBall", 2),
            ("technique", 1),
        ],
    },
    Role {
        id: "creator",
        name: "Hücum oyun kurucu",
        position: Position::AMC,
        weights: &[
            ("vision", 3),
            ("passing", 3),
            ("firstTouch", 2),
            ("technique", 2),
            ("decisions", 2),
            ("offTheBall", 1),
        ],
    },
    Role {
        id: "striker",
        name: "Bitirici forvet",
        position: Position::ST,
        weights: &[
            ("finishing", 3),
            ("offTheBall", 3),
            ("composure", 2),
            ("anticipation", 2),
            ("acceleration", 2),
            ("firstTouch", 1),
        ],
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleFit {
    pub role_id: String,
    pub role_name: String,
    pub score: Option<f64>,
    pub coverage: f64,
    pub missing: Vec<String>,
    pub strengths: Vec<String>,
}

pub fn role_fit(player: &Player, role: &Role) -> RoleFit {
    let total: f64 = role.weights.iter().map(|(_, w)| f64::from(*w)).sum();
    let mut known = 0.0;
    let mut sum = 0.0;
    let mut missing = Vec::new();
    let mut strengths = Vec::new();
    for (name, weight) in role.weights {
        if let Some(value) = player.attributes.get(*name) {
            known += f64::from(*weight);
            sum += f64::from(*value) * f64::from(*weight);
            if *value >= 14 {
                strengths.push((*name).to_string());
            }
        } else {
            missing.push((*name).to_string());
        }
    }
    let coverage = known / total;
    RoleFit {
        role_id: role.id.into(),
        role_name: role.name.into(),
        score: (coverage >= 0.70).then(|| round(sum / known * 5.0)),
        coverage: round(coverage * 100.0),
        missing,
        strengths,
    }
}

fn round(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

pub fn fits(player: &Player) -> Vec<RoleFit> {
    let mut results: Vec<_> = ROLES
        .iter()
        .filter(|r| player.positions.contains(&r.position))
        .map(|r| role_fit(player, r))
        .collect();
    results.sort_by(|a, b| {
        b.score
            .unwrap_or(-1.0)
            .total_cmp(&a.score.unwrap_or(-1.0))
            .then_with(|| a.role_id.cmp(&b.role_id))
    });
    results
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDetail {
    pub player: Player,
    pub roles: Vec<RoleFit>,
}

pub fn player_detail(player: &Player) -> PlayerDetail {
    PlayerDetail {
        player: player.clone(),
        roles: fits(player),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRow {
    pub id: u32,
    pub name: String,
    pub age: u8,
    pub positions: Vec<Position>,
    pub condition: u8,
    pub form: Option<f64>,
    pub injured: bool,
    pub role: Option<RoleFit>,
}

pub fn player_row(player: &Player) -> PlayerRow {
    PlayerRow {
        id: player.id,
        name: player.name.clone(),
        age: player.age,
        positions: player.positions.clone(),
        condition: player.condition,
        form: player.form,
        injured: player.injured,
        role: fits(player).into_iter().next(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub key: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub role_id: String,
    pub player: Option<PlayerRow>,
    pub fit: Option<RoleFit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Insight {
    pub kind: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub formation: Formation,
    pub slots: Vec<Slot>,
    pub score: Option<f64>,
    pub filled: usize,
    pub available: usize,
    pub insights: Vec<Insight>,
}

fn slots(formation: Formation) -> Vec<Slot> {
    let mut definitions = vec![
        ("gk", "KL", 50.0, 89.0, "keeper"),
        ("lb", "SLB", 14.0, 71.0, "leftBack"),
        ("lcb", "STP", 37.0, 74.0, "centreBack"),
        ("rcb", "STP", 63.0, 74.0, "centreBack"),
        ("rb", "SĞB", 86.0, 71.0, "rightBack"),
    ];
    match formation {
        Formation::FourThreeThree => definitions.extend([
            ("dm", "DM", 50.0, 56.0, "anchor"),
            ("lcm", "MO", 32.0, 39.0, "playmaker"),
            ("rcm", "MO", 68.0, 39.0, "runner"),
            ("lw", "SLK", 16.0, 21.0, "leftWing"),
            ("rw", "SĞK", 84.0, 21.0, "rightWing"),
            ("st", "FV", 50.0, 12.0, "striker"),
        ]),
        Formation::FourTwoThreeOne => definitions.extend([
            ("lcm", "MO", 33.0, 52.0, "playmaker"),
            ("rcm", "MO", 67.0, 52.0, "runner"),
            ("am", "OMO", 50.0, 31.0, "creator"),
            ("lw", "SLK", 15.0, 30.0, "leftWing"),
            ("rw", "SĞK", 85.0, 30.0, "rightWing"),
            ("st", "FV", 50.0, 12.0, "striker"),
        ]),
        Formation::FourFourTwo => definitions.extend([
            ("lcm", "MO", 37.0, 46.0, "playmaker"),
            ("rcm", "MO", 63.0, 46.0, "runner"),
            ("lw", "SLK", 13.0, 44.0, "leftWing"),
            ("rw", "SĞK", 87.0, 44.0, "rightWing"),
            ("lst", "FV", 35.0, 17.0, "striker"),
            ("rst", "FV", 65.0, 17.0, "striker"),
        ]),
    }
    definitions
        .into_iter()
        .map(|(key, label, x, y, role_id)| Slot {
            key: key.into(),
            label: label.into(),
            x,
            y,
            role_id: role_id.into(),
            player: None,
            fit: None,
        })
        .collect()
}

/// Rectangular Hungarian assignment: each player can occupy at most one slot.
/// Extra dummy columns allow honest gaps instead of out-of-position selections.
fn assignment(costs: &[Vec<f64>]) -> Vec<usize> {
    let n = costs.len();
    let m = costs[0].len();
    let (mut u, mut v) = (vec![0.0; n + 1], vec![0.0; m + 1]);
    let (mut p, mut way) = (vec![0; m + 1], vec![0; m + 1]);
    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0;
        let mut minv = vec![f64::INFINITY; m + 1];
        let mut used = vec![false; m + 1];
        loop {
            used[j0] = true;
            let i0 = p[j0];
            let mut delta = f64::INFINITY;
            let mut j1 = 0;
            for j in 1..=m {
                if !used[j] {
                    let cur = costs[i0 - 1][j - 1] - u[i0] - v[j];
                    if cur < minv[j] {
                        minv[j] = cur;
                        way[j] = j0;
                    }
                    if minv[j] < delta {
                        delta = minv[j];
                        j1 = j;
                    }
                }
            }
            for j in 0..=m {
                if used[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    minv[j] -= delta;
                }
            }
            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 {
                break;
            }
        }
    }
    let mut result = vec![m; n];
    for j in 1..=m {
        if p[j] > 0 {
            result[p[j] - 1] = j - 1;
        }
    }
    result
}

pub fn analyze(snapshot: &Snapshot, formation: Formation) -> Analysis {
    let mut result_slots = slots(formation);
    let mut players: Vec<_> = snapshot.players.iter().filter(|p| !p.injured).collect();
    players.sort_by_key(|p| p.id);
    let costs: Vec<Vec<f64>> = result_slots
        .iter()
        .map(|slot| {
            let role = ROLES
                .iter()
                .find(|r| r.id == slot.role_id)
                .expect("defined role");
            let mut row: Vec<_> = players
                .iter()
                .map(|p| {
                    if !p.positions.contains(&role.position) {
                        return 1_000_000.0;
                    }
                    match role_fit(p, role).score {
                        // Condition is an explicit selection adjustment, not part of role skill.
                        Some(score) => 100.0 - score + f64::from(100 - p.condition) * 0.3,
                        None => 1_000_000.0,
                    }
                })
                .collect();
            row.extend(std::iter::repeat_n(140.0, result_slots.len()));
            row
        })
        .collect();
    for (slot, column) in result_slots.iter_mut().zip(assignment(&costs)) {
        if let Some(player) = players.get(column) {
            let role = ROLES
                .iter()
                .find(|r| r.id == slot.role_id)
                .expect("defined role");
            if player.positions.contains(&role.position) && role_fit(player, role).score.is_some() {
                slot.player = Some(player_row(player));
                slot.fit = Some(role_fit(player, role));
            }
        }
    }
    let selected: Vec<_> = result_slots
        .iter()
        .filter_map(|s| s.fit.as_ref()?.score)
        .collect();
    let filled = selected.len();
    let score = (filled == 11).then(|| round(selected.iter().sum::<f64>() / 11.0));
    let available = snapshot
        .players
        .iter()
        .filter(|p| !p.injured && p.condition >= 80)
        .count();
    let mut insights = Vec::new();
    if snapshot.source == Source::Demo {
        insights.push(Insight { kind: "info".into(), title: "Oyun olmadan keşfet".into(), detail: "Bu kadro tamamen kurgusal. Puanlar örnek özelliklerden hesaplanır; gerçek FM27 verisi içermez.".into() });
    }
    if filled < 11 {
        insights.push(Insight { kind: "warning".into(), title: format!("{} pozisyonda veri veya oyuncu eksik", 11 - filled), detail: "Mevkisi uymayan veya özellik verisi yetersiz oyuncular otomatik olarak ilk 11'e alınmadı.".into() });
    }
    if let Some(tired) = result_slots
        .iter()
        .filter_map(|s| s.player.as_ref())
        .filter(|p| p.condition < 80)
        .min_by_key(|p| p.condition)
    {
        insights.push(Insight { kind: "warning".into(), title: format!("{} için yük kontrolü", tired.name), detail: format!("Seçilen oyuncunun kondisyonu %{}. Alternatifini değerlendir; bu, maç motorunun sakatlık tahmini değildir.", tired.condition) });
    }
    if let Some(best) = result_slots
        .iter()
        .filter(|s| s.fit.is_some())
        .max_by(|a, b| {
            a.fit
                .as_ref()
                .unwrap()
                .score
                .unwrap()
                .total_cmp(&b.fit.as_ref().unwrap().score.unwrap())
        })
    {
        insights.push(Insight { kind: "positive".into(), title: format!("{} güçlü bir eşleşme", best.player.as_ref().unwrap().name), detail: format!("{} rolünde {:.0}/100 uyum. Özellik ağırlıklarını oyuncu kartından inceleyebilirsin.", best.fit.as_ref().unwrap().role_name, best.fit.as_ref().unwrap().score.unwrap()) });
    }
    let unknown_form = snapshot.players.iter().filter(|p| p.form.is_none()).count();
    if unknown_form > 0 {
        insights.push(Insight {
            kind: "info".into(),
            title: format!("{unknown_form} oyuncunun formu bilinmiyor"),
            detail: "Eksik form verileri sıfır puan olarak değerlendirilmez.".into(),
        });
    }
    Analysis {
        formation,
        slots: result_slots,
        score,
        filled,
        available,
        insights,
    }
}

pub fn prompt_context(snapshot: &Snapshot, formation: Formation) -> String {
    let result = analyze(snapshot, formation);
    let mut context = format!("VERİ KAYNAĞI: {:?}. Kulüp: {}. Sezon: {}. Veri tarihi: {}. Canlı bağlantı YOK.\nPuanlar uygulamanın deneysel ağırlıklı özellik hesabıdır; FM maç motorunun sonucu değildir.\nDiziliş: {:?}.\n", snapshot.source, snapshot.club, snapshot.season, snapshot.captured_at, formation);
    for slot in &result.slots {
        if let (Some(player), Some(fit)) = (&slot.player, &slot.fit) {
            context.push_str(&format!(
                "{}: {} (#{}), rol uyumu {:?}/100, veri kapsamı {}%, kondisyon {}%, form {:?}.\n",
                fit.role_name,
                player.name,
                player.id,
                fit.score,
                fit.coverage,
                player.condition,
                player.form
            ));
            if let Some(original) = snapshot.players.iter().find(|p| p.id == player.id) {
                // Include the actual known attributes, not only the derived fit,
                // so tactical questions can be grounded in the source values.
                context.push_str(&format!(
                    "Oyuncu verisi: {}\n",
                    serde_json::to_string(original).expect("validated player")
                ));
            }
        } else {
            context.push_str(&format!(
                "{}: uygun ve yeterli verili oyuncu yok.\n",
                slot.label
            ));
        }
    }
    let selected: std::collections::HashSet<_> = result
        .slots
        .iter()
        .filter_map(|s| s.player.as_ref().map(|p| p.id))
        .collect();
    for player in snapshot
        .players
        .iter()
        .filter(|p| !selected.contains(&p.id))
        .take(20)
    {
        let role = fits(player).into_iter().next();
        context.push_str(&format!(
            "Alternatif: {} (#{}), {:?}, sakat {}, kondisyon {}%, en iyi rol {:?}.\n",
            player.name, player.id, player.positions, player.injured, player.condition, role
        ));
        context.push_str(&format!(
            "Oyuncu verisi: {}\n",
            serde_json::to_string(player).expect("validated player")
        ));
    }
    context
}
