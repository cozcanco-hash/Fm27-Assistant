pub mod analysis;
pub mod codex;
pub mod model;
pub mod scouting;
pub mod store;

pub use analysis::{analyze, player_detail, Analysis, Formation, PlayerDetail};
pub use model::{demo_snapshot, parse_snapshot, Player, Snapshot, Source};
pub use store::Store;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("Veri dosyası okunamadı: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Yerel veritabanı işlemi tamamlanamadı: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Yerel işlem tamamlanamadı: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
