//! Reproducible synthetic core measurement, not an end-to-end game benchmark.
use fm27_core::{analyze, demo_snapshot, Formation};
use std::time::Instant;

fn main() {
    let sample = demo_snapshot();
    let mut snapshot = sample.clone();
    snapshot.players = (0..5_000)
        .map(|index| {
            let mut player = sample.players[index % sample.players.len()].clone();
            player.id = index as u32 + 1;
            player.name = format!("Örnek Oyuncu {}", index + 1);
            player
        })
        .collect();
    snapshot.validate().unwrap();
    std::hint::black_box(analyze(&snapshot, Formation::FourThreeThree));
    let mut durations = Vec::new();
    for _ in 0..25 {
        let start = Instant::now();
        std::hint::black_box(analyze(&snapshot, Formation::FourThreeThree));
        durations.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    durations.sort_by(f64::total_cmp);
    println!(
        "{}",
        serde_json::json!({"kind":"synthetic-core-only","players":5000,"iterations":25,"medianMs":durations[12],"p95Ms":durations[23],"architecture":std::env::consts::ARCH})
    );
}
