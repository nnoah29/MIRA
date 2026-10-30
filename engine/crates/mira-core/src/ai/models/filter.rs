//! Retrait des modèles que le fournisseur déclare inutiles à la discussion.

use mira_vocabulary::AiModel;

/// Écarte les modèles dont l'identifiant contient un mot exclu, sans tenir compte de la casse.
pub fn filter_models(models: Vec<AiModel>, exclusions: &[String]) -> Vec<AiModel> {
    let words: Vec<String> = exclusions.iter().map(|word| word.to_lowercase()).collect();
    models
        .into_iter()
        .filter(|model| {
            let id = model.id.to_lowercase();
            !words.iter().any(|word| id.contains(word.as_str()))
        })
        .collect()
}
