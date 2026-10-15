//! Réouverture et suppression en cours : la réponse ne compte que pour la demande attendue de la vue ouverte.

use mira_assembly::mira_vocabulary::DraftId;

use super::DraftsState;

impl DraftsState {
    /// Note la relecture d'un brouillon à rouvrir et rend son jeton.
    pub fn begin_open(&mut self) -> u64 {
        let token = self.next_token();
        self.opening = Some(token);
        token
    }

    /// Vrai (et oublié) si cette relecture est celle qu'attend la vue ouverte.
    pub fn take_open(&mut self, token: u64) -> bool {
        let awaited = self.opening == Some(token);
        if awaited {
            self.opening = None;
        }
        awaited
    }

    /// Vrai tant qu'une réouverture attend la réponse du moteur.
    pub fn is_opening(&self) -> bool {
        self.opening.is_some()
    }

    /// Retient le brouillon dont la suppression vient d'être demandée, et son objet.
    pub fn note_delete(&mut self, draft: DraftId, subject: String) {
        self.deleting = Some((draft, subject));
    }

    /// Rend l'objet si cette réponse concerne la suppression demandée par la vue, et l'oublie.
    pub fn take_delete(&mut self, draft: &DraftId) -> Option<String> {
        match self.deleting.take() {
            Some((pending, subject)) if &pending == draft => Some(subject),
            other => {
                self.deleting = other;
                None
            }
        }
    }
}
