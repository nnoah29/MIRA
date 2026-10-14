//! Erreurs du client plein écran : celles du moteur et celles du terminal.

use mira_assembly::mira_vocabulary::MiraError;

/// Échec du client : erreur structurée du moteur ou erreur d'entrée-sortie du terminal.
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("moteur : {0}")]
    Engine(MiraError),
    #[error("moteur encore partagé : fermeture impossible")]
    EngineStillShared,
    #[error("moteur : arrêt des synchronisations hors délai")]
    ShutdownTimedOut,
    #[error("terminal : {0}")]
    Terminal(std::io::Error),
}

impl From<MiraError> for TuiError {
    /// Convertit une erreur du moteur.
    fn from(error: MiraError) -> Self {
        Self::Engine(error)
    }
}

impl From<std::io::Error> for TuiError {
    /// Convertit une erreur d'entrée-sortie du terminal.
    fn from(error: std::io::Error) -> Self {
        Self::Terminal(error)
    }
}
