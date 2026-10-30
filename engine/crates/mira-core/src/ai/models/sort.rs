//! Ordre de présentation des modèles.

use std::cmp::Reverse;

use mira_vocabulary::AiModel;

/// Trie du plus récent au plus ancien, puis les modèles sans date par ordre alphabétique.
pub fn sort_models(mut models: Vec<AiModel>) -> Vec<AiModel> {
    models.sort_by(|left, right| {
        let dated = |model: &AiModel| model.created_at.map(Reverse);
        match (dated(left), dated(right)) {
            (Some(a), Some(b)) => a.cmp(&b).then_with(|| left.id.cmp(&right.id)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.id.cmp(&right.id),
        }
    });
    models
}
