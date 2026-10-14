//! Erreur rendue à la place de la réponse d'une requête dont la tâche a paniqué.

use mira_assembly::mira_vocabulary::error::codes::internal::BUG;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError};

/// Erreur interne générique, sans le texte de la panique (qui pourrait contenir des données).
pub fn task_failed() -> MiraError {
    MiraError::new(
        ErrorCategory::Internal,
        BUG,
        "la requête s'est interrompue de façon inattendue",
    )
}
