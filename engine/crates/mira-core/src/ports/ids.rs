//! Générateurs d'identifiants opaques : UUID v7 en production, séquence en test.

use std::sync::atomic::{AtomicU64, Ordering};

/// Source d'identifiants uniques du moteur.
pub trait IdGenerator: Send + Sync + std::fmt::Debug {
    /// Fabrique un nouvel identifiant
    fn next_id(&self) -> String;
}

/// Identifiants UUID v7 (triables par date de création).
#[derive(Debug, Clone, Copy, Default)]
pub struct UuidV7Ids;

impl IdGenerator for UuidV7Ids {
    /// Fabrique un UUID v7 au format texte
    fn next_id(&self) -> String {
        uuid::Uuid::now_v7().to_string()
    }
}

/// Identifiants déterministes `id-000001`, `id-000002`… pour les tests.
#[derive(Debug, Default)]
pub struct SequentialIds {
    counter: AtomicU64,
}

impl SequentialIds {
    /// Crée une séquence qui commence à `id-000001`
    pub fn new() -> Self {
        Self::default()
    }
}

impl IdGenerator for SequentialIds {
    /// Donne l'identifiant suivant de la séquence
    fn next_id(&self) -> String {
        let number = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        format!("id-{number:06}")
    }
}
