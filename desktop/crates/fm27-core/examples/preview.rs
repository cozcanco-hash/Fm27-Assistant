use fm27_core::{analysis::player_detail, Formation, Store};
use serde_json::json;

fn main() {
    let mut store = Store::memory().unwrap();
    let current = store.snapshot().clone();
    let mut earlier = current.clone();
    earlier.game_date = Some("2026-07-01".into());
    earlier.captured_at = "2026-11-01T12:00:00Z".into();
    for p in &mut earlier.players {
        p.current_ability = p.current_ability.map(|v| v.saturating_sub(3).max(1));
        if let Some(v) = p.attributes.get_mut("passing") {
            *v = v.saturating_sub(1).max(1);
        }
    }
    store.replace(earlier).unwrap();
    store.replace(current).unwrap();
    let list_id = store.shortlists().unwrap()[0].id;
    store.set_shortlisted(list_id, 10, true).unwrap();
    let dashboards: Vec<_> = [
        Formation::FourThreeThree,
        Formation::FourTwoThreeOne,
        Formation::FourFourTwo,
    ]
    .into_iter()
    .map(|f| store.set_formation(f).unwrap())
    .collect();
    let fixture = json!({
        "dashboards": dashboards,
        "page": store.players(Default::default()).unwrap(),
        "details": store.snapshot().players.iter().map(player_detail).collect::<Vec<_>>(),
        "shortlists": store.shortlists().unwrap(),
        "histories": store.snapshot().players.iter().map(|p| json!({"id": p.id, "history": store.player_history(p.id).unwrap()})).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&fixture).unwrap());
}
