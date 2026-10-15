//! Sélection ancrée sur le brouillon, pas sur l'indice : une relecture qui déplace les brouillons ne change pas la cible.

use mira_assembly::mira_vocabulary::Draft;

use super::DraftsState;
use crate::state::Selection;

impl DraftsState {
    /// Brouillons affichés, le plus récemment modifié d'abord (ordre du moteur).
    pub fn items(&self) -> &[Draft] {
        &self.items
    }

    /// Brouillon sélectionné ; aucun si celui qui l'était a disparu.
    pub fn selected(&self) -> Option<&Draft> {
        self.selection
            .selected
            .and_then(|index| self.items.get(index))
    }

    /// Déplace la sélection et ancre le brouillon choisi.
    pub fn move_selection(&mut self, step: fn(&mut Selection, usize)) {
        step(&mut self.selection, self.items.len());
        self.anchor = self.selected().map(|draft| draft.id.clone());
    }

    /// Après une relecture : retrouve le brouillon sélectionné ; disparu, plus rien n'est sélectionné (aucun autre ne le remplace).
    pub(super) fn reanchor(&mut self) {
        match &self.anchor {
            Some(anchor) => {
                self.selection.selected = self.items.iter().position(|d| &d.id == anchor);
            }
            None => {
                self.selection.clamp_to(self.items.len());
                self.anchor = self.selected().map(|draft| draft.id.clone());
            }
        }
    }
}
