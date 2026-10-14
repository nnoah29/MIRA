//! Lectures du brouillon ouvert : identifiants, texte et curseur de chaque champ.

use mira_assembly::mira_vocabulary::{AccountId, DraftId};

use super::field::Field;
use super::state::{ComposeState, FieldBuffer};
use super::text_buffer::TextBuffer;

impl ComposeState {
    /// Identifiant du brouillon dans le moteur.
    pub fn draft(&self) -> &DraftId {
        &self.draft
    }

    /// Compte du brouillon.
    pub fn account(&self) -> &AccountId {
        &self.account
    }

    /// Tampon de ce champ.
    pub fn buffer(&self, field: Field) -> &TextBuffer {
        &self.fields[field.index()].buffer
    }

    /// Texte de ce champ.
    pub fn text(&self, field: Field) -> String {
        self.buffer(field).text()
    }

    /// Curseur de ce champ : (ligne, colonne en caractères).
    pub fn cursor(&self, field: Field) -> (usize, usize) {
        self.buffer(field).cursor()
    }

    /// Champ qui a le focus, modifiable.
    pub(super) fn focused_mut(&mut self) -> &mut FieldBuffer {
        &mut self.fields[self.focus.index()]
    }
}
