//! Réouverture d'un brouillon déjà enregistré : il appartient à l'utilisateur, jamais supprimé à la fermeture.

use mira_assembly::mira_vocabulary::Draft;

use super::state::ComposeState;

impl ComposeState {
    /// Éditeur sur un brouillon enregistré : contenu chargé = dernière version enregistrée, aucune clé d'envoi.
    pub fn from_saved_draft(draft: &Draft) -> ComposeState {
        let mut compose = ComposeState::from_draft(draft);
        compose.saved_once = true;
        compose
    }
}
