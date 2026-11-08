//! Ce que le démon demande à une icône de zone de notification, quel que soit le système.

use super::status::TrayStatus;

/// L'icône ne peut pas s'afficher (pas d'hôte de zone de notification, pas de bus).
#[derive(Debug, thiserror::Error)]
pub enum TrayError {
    #[error("pas d'hôte de zone de notification")]
    NoHost,
}

/// Ce que le démon demande à une icône : afficher son état.
pub trait TrayControl: Send + Sync + std::fmt::Debug {
    /// Change l'état affiché.
    fn set_status(&self, status: TrayStatus);
}
