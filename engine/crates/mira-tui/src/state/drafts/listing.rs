//! Lectures de la liste : une réponse ne compte que pour le jeton attendu, un seul tour de plus pendant une lecture.

use mira_assembly::mira_vocabulary::Draft;

use super::DraftsState;

impl DraftsState {
    /// Commence une lecture et rend son jeton ; vue fermée, rien ; pendant une lecture, note seulement qu'il en faudra une autre.
    pub fn start_list(&mut self) -> Option<u64> {
        if !self.is_open() {
            return None;
        }
        if self.is_loading() {
            self.again = true;
            return None;
        }
        let token = self.next_token();
        self.listing = Some(token);
        Some(token)
    }

    /// Vrai si cette lecture est celle qu'attend la vue ouverte.
    pub fn awaits_list(&self, token: u64) -> bool {
        self.listing == Some(token)
    }

    /// Range les brouillons lus (sélection gardée sur le même) ; vrai si une autre lecture a été demandée entre-temps.
    pub fn finish_list(&mut self, items: Option<Vec<Draft>>) -> bool {
        self.listing = None;
        if let Some(items) = items {
            self.items = items;
            self.reanchor();
        }
        self.loaded = true;
        std::mem::take(&mut self.again)
    }

    /// Vrai tant qu'une lecture est attendue.
    pub fn is_loading(&self) -> bool {
        self.listing.is_some()
    }

    /// Vrai une fois qu'une lecture est arrivée.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}
