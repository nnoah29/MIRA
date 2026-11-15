#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outils des tests de la mémoire : service prêt sur une base en mémoire ou sur un fichier.

pub mod embedder;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};

use embedder::KeywordEmbedder;
use mira_core::ai::memory::MemoryStore;
use mira_core::ports::{ManualClock, SequentialIds};
use mira_core::storage::Database;
use mira_vocabulary::Timestamp;

/// Service, base et embedder partagés par un test
pub struct Setup {
    pub db: Arc<Database>,
    pub embedder: Arc<KeywordEmbedder>,
    pub store: MemoryStore,
}

/// Service neuf sur une base en mémoire, embedder du modèle `model`
pub fn setup(model: &str) -> Setup {
    let db = Arc::new(Database::open_in_memory().unwrap());
    let embedder = embedder(model);
    let store = store_over(&db, &embedder);
    Setup {
        db,
        embedder,
        store,
    }
}

/// Service neuf sur le fichier de base de ce dossier, pour observer le journal WAL
pub fn setup_in(dir: &Path, model: &str) -> Setup {
    let db = Arc::new(Database::open(&dir.join("mira.sqlite3")).unwrap());
    let embedder = embedder(model);
    let store = store_over(&db, &embedder);
    Setup {
        db,
        embedder,
        store,
    }
}

/// Embedder factice du modèle `model`
pub fn embedder(model: &str) -> Arc<KeywordEmbedder> {
    Arc::new(KeywordEmbedder {
        model: model.to_string(),
        fail: AtomicBool::new(false),
        short: AtomicBool::new(false),
        wide: AtomicBool::new(false),
        calls: AtomicUsize::new(0),
        last_len: AtomicUsize::new(0),
    })
}

/// Service branché sur cette base et cet embedder
pub fn store_over(db: &Arc<Database>, embedder: &Arc<KeywordEmbedder>) -> MemoryStore {
    MemoryStore::new(
        db.clone(),
        embedder.clone(),
        Arc::new(ManualClock::new(Timestamp::from_millis(5_000))),
        Arc::new(SequentialIds::default()),
    )
}
