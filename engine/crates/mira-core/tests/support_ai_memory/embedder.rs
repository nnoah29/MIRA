#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Embedder factice par mots-clés, avec pannes et défauts réglables pour les tests de la mémoire.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use mira_core::ai::memory::{EmbedFuture, Embedder};
use mira_vocabulary::error::codes::server::AI_UNAVAILABLE;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Embedder déterministe : compte « chat », « chien » et « mail » dans chaque texte
pub struct KeywordEmbedder {
    pub model: String,
    pub fail: AtomicBool,
    pub short: AtomicBool,
    pub wide: AtomicBool,
    pub calls: AtomicUsize,
    pub last_len: AtomicUsize,
}

impl Embedder for KeywordEmbedder {
    /// Identifiant du modèle factice
    fn model_id(&self) -> &str {
        &self.model
    }

    /// Un vecteur par texte ; échoue si `fail` est levé ou si un texte contient « poison », ne rend rien si `short`
    fn embed<'a>(&'a self, texts: &'a [String]) -> EmbedFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let longest = texts.iter().map(|t| t.chars().count()).max().unwrap_or(0);
            self.last_len.store(longest, Ordering::SeqCst);
            let poisoned = texts.iter().any(|t| t.contains("poison"));
            if self.fail.load(Ordering::SeqCst) || poisoned {
                return Err(MiraError::new(
                    ErrorCategory::Server,
                    AI_UNAVAILABLE,
                    "embedder en panne",
                ));
            }
            if self.short.load(Ordering::SeqCst) {
                return Ok(Vec::new());
            }
            let wide = self.wide.load(Ordering::SeqCst);
            Ok(texts.iter().map(|t| keywords(t, wide)).collect())
        })
    }
}

/// Vecteur d'un texte : occurrences des trois mots-clés, une constante, et une valeur de plus si `wide`
fn keywords(text: &str, wide: bool) -> Vec<f32> {
    let count = |word: &str| text.matches(word).count() as f32;
    let mut vector = vec![count("chat"), count("chien"), count("mail"), 0.1];
    if wide {
        vector.push(0.1);
    }
    vector
}
