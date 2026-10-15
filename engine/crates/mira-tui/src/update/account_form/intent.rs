//! Intentions du formulaire d'ajout de compte.

use crate::update::compose::EditKey;

/// Ce que l'utilisateur fait dans le formulaire d'ajout de compte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountFormIntent {
    Open,
    Edit(EditKey),
    Enter,
    NextField,
    PrevField,
    CycleSecurity,
    Close,
    ConfirmClose,
    KeepEditing,
}
