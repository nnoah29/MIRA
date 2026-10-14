//! Fenêtre modale affichée au-dessus de l'écran principal.

use mira_assembly::mira_vocabulary::MessageSummary;

use super::move_picker::MovePicker;

/// Fenêtre ouverte, s'il y en a une.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Popup {
    #[default]
    Closed,
    ConfirmDelete {
        messages: Vec<MessageSummary>,
    },
    MovePicker(MovePicker),
    Help,
    ConfirmCloseDraft,
}

impl Popup {
    /// Vrai si une fenêtre est ouverte.
    pub fn is_open(&self) -> bool {
        !matches!(self, Popup::Closed)
    }
}
