//! Libellés de départ, suppression en bloc et sort « refusé » pour les tests d'import.

use mira_core::api::{Engine, LocalLabelRequest};
use mira_vocabulary::{LabelImportOutcome, LabelImportRefusal};

use super::calls::{call, labels};
use super::files::create_colored;

/// Trois libellés de couleurs différentes
pub async fn seed_three(engine: &Engine) {
    let colors = [
        ("Banque", 0x80FF_AA00),
        ("Compta", 0xFF1E_90FF),
        ("Voyages", 0xFF2E_8B57),
    ];
    for (n, (name, color)) in colors.iter().enumerate() {
        create_colored(engine, name, *color, &format!("seed-{n}")).await;
    }
}

/// Supprime tous les libellés
pub async fn delete_all(engine: &Engine) {
    for label in labels(engine).await {
        let request = LocalLabelRequest::DeleteLocalLabel { label: label.id };
        call(engine, request).await.unwrap();
    }
}

/// Sort « refusé » pour cette raison
pub fn refused(reason: LabelImportRefusal) -> LabelImportOutcome {
    LabelImportOutcome::Refused { reason }
}

/// Nom et couleur de chaque libellé, dans l'ordre de la liste
pub async fn pairs(engine: &Engine) -> Vec<(String, u32)> {
    labels(engine)
        .await
        .into_iter()
        .map(|l| (l.name, l.color))
        .collect()
}
