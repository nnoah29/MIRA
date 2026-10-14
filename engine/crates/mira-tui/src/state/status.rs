//! Barre d'état : dernier message à montrer à l'utilisateur.

/// Contenu de la barre d'état.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatusState {
    pub message: Option<String>,
}
