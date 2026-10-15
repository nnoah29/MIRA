//! Sélection ancrée sur l'action, pas sur l'indice : une relecture qui déplace les fiches ne change pas la cible.

use super::QueueState;
use crate::state::Selection;

impl QueueState {
    /// Après une relecture : retrouve l'action sélectionnée ; disparue, plus rien n'est sélectionné (aucune autre ne la remplace).
    pub(super) fn reanchor(&mut self) {
        match &self.anchor {
            Some(anchor) => {
                self.selection.selected = self.entries.iter().position(|e| &e.action == anchor);
            }
            None => {
                self.selection.clamp_to(self.entries.len());
                self.anchor = self.selected().map(|entry| entry.action.clone());
            }
        }
    }

    /// Déplace la sélection et ancre la nouvelle fiche choisie.
    pub fn move_selection(&mut self, step: fn(&mut Selection, usize)) {
        step(&mut self.selection, self.entries.len());
        self.anchor = self.selected().map(|entry| entry.action.clone());
    }
}
