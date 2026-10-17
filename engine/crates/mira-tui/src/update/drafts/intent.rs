//! Intentions de la vue « Brouillons ».

/// Ce que l'utilisateur fait de ses brouillons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftsIntent {
    Open,
    AskDelete,
    ConfirmDelete,
}
