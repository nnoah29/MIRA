#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Outils des tests de propriétés sur les données : modèle, opérations aléatoires, monde réel, vérifications.

pub mod checks;
pub mod model;
pub mod operations;
pub mod store_world;

use proptest::prelude::ProptestConfig;

/// Réglage commun : peu de cas par propriété pour tenir dans le temps de l'intégration continue
pub fn quick_config() -> ProptestConfig {
    ProptestConfig {
        cases: 200,
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}
