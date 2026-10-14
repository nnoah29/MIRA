//! Fenêtre modale affichée au-dessus de l'écran principal.

/// Fenêtre ouverte, s'il y en a une.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Popup {
    #[default]
    Closed,
}
