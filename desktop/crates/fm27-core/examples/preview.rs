use fm27_core::{analysis::player_detail, Formation, Store};
use serde_json::json;

fn main() {
    let mut store = Store::memory().unwrap();
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
        "page": store.players(Default::default()),
        "details": store.snapshot().players.iter().map(player_detail).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&fixture).unwrap());
}
