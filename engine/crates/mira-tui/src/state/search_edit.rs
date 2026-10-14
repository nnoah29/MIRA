//! Modifications de la recherche : ouverture, frappe, envoi et sortie.

use mira_assembly::mira_vocabulary::ViewRef;

use super::search::SearchState;

const MAX_INPUT_CHARS: usize = 200;

impl SearchState {
    /// Ouvre une saisie vide ; la vue affichée est mémorisée sauf si une recherche est déjà active.
    pub fn open(&mut self, current: Option<ViewRef>) {
        self.editing = true;
        self.input.clear();
        if self.query.is_none() {
            self.previous = current;
        }
    }

    /// Ajoute un caractère imprimable, dans la limite de longueur de la saisie.
    pub fn push(&mut self, c: char) {
        if !c.is_control() && self.input.chars().count() < MAX_INPUT_CHARS {
            self.input.push(c);
        }
    }

    /// Retire le dernier caractère tapé.
    pub fn pop(&mut self) {
        self.input.pop();
    }

    /// Valide la saisie : rend la requête nettoyée, ou rien (saisie gardée ouverte) si elle est vide.
    pub fn submit(&mut self) -> Option<String> {
        let query = self.input.trim().to_string();
        if query.is_empty() {
            return None;
        }
        self.query = Some(query.clone());
        self.first_page_only = None;
        self.editing = false;
        Some(query)
    }

    /// Ferme la saisie et la recherche ; vrai si des résultats étaient affichés.
    pub fn leave(&mut self) -> bool {
        self.editing = false;
        self.input.clear();
        self.query.take().is_some()
    }
}
