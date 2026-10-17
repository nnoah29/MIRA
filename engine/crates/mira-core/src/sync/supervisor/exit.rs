//! Raisons pour lesquelles une session connectée se termine, et leur classement par catégorie d'erreur.

use mira_vocabulary::{ErrorCategory, MiraError};

/// Fin d'une session connectée de la tâche d'un compte.
#[derive(Debug)]
pub enum Exit {
    Shutdown,
    Network(MiraError),
    AuthRequired(MiraError),
    Failed(MiraError),
}

/// Classe une erreur : réseau (reconnexion), authentification (intervention), autre (nouvel essai plus tard).
pub fn classify(error: MiraError) -> Exit {
    match error.category {
        ErrorCategory::Network => Exit::Network(error),
        ErrorCategory::Authentication => Exit::AuthRequired(error),
        _ => Exit::Failed(error),
    }
}
