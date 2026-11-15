#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outils des tests de la mémoire : embedder factice par mots-clés et service prêt sur une base en mémoire.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use mira_core::ai::memory::{EmbedFuture, Embedder, MemoryStore};
use mira_core::ports::{ManualClock, SequentialIds};
use mira_core::storage::Database;
use mira_vocabulary::{MiraError, Timestamp};

/// Embedder déterministe : compte « chat », « chien » et « mail » dans chaque texte
pub struct KeywordEmbedder {
    pub model: String,
    pub fail: AtomicBool,
    pub calls: AtomicUsize,
}

impl Embedder for KeywordEmbedder {
    /// Identifiant du modèle factice
    fn model_id(&self) -> &str {
        &self.model
    }

    /// Un vecteur de quatre valeurs par texte ; échoue si `fail` est levé
    fn embed<'a>(&'a self, texts: &'a [String]) -> EmbedFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail.load(Ordering::SeqCst) {
                return Err(MiraError::new(
                    mira_vocabulary::ErrorCategory::Server,
                    mira_vocabulary::error::codes::server::AI_UNAVAILABLE,
                    "embedder en panne",
                ));
            }
            Ok(texts.iter().map(|t| keywords(t)).collect())
        })
    }
}

/// Vecteur d'un texte : occurrences des trois mots-clés puis une composante constante
fn keywords(text: &str) -> Vec<f32> {
    let count = |word: &str| text.matches(word).count() as f32;
    vec![count("chat"), count("chien"), count("mail"), 0.1]
}

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

/// Embedder factice du modèle `model`
pub fn embedder(model: &str) -> Arc<KeywordEmbedder> {
    Arc::new(KeywordEmbedder {
        model: model.to_string(),
        fail: AtomicBool::new(false),
        calls: AtomicUsize::new(0),
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
