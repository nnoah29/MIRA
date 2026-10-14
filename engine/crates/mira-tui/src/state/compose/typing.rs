//! Frappe et déplacement dans le champ qui a le focus.

use super::field::Field;
use super::state::ComposeState;
use super::text_buffer::TextBuffer;

impl ComposeState {
    /// Modifie le texte du champ qui a le focus ; son signalement d'erreur tombe.
    pub fn edit_text(&mut self, edit: impl FnOnce(&mut TextBuffer)) {
        let slot = self.focused_mut();
        slot.flagged = false;
        edit(&mut slot.buffer);
    }

    /// Déplace le curseur du champ qui a le focus.
    pub fn move_cursor(&mut self, step: impl FnOnce(&mut TextBuffer)) {
        step(&mut self.focused_mut().buffer);
    }

    /// Fait suivre la fenêtre du champ qui a le focus à son curseur.
    pub fn follow_cursor(&mut self, rows: usize) {
        self.focused_mut().buffer.follow(rows);
    }

    /// Passe au champ suivant.
    pub fn focus_next(&mut self) {
        self.focus = self.focus.next();
    }

    /// Monte d'un champ sans revenir au corps depuis le premier champ.
    pub fn focus_above(&mut self) {
        if self.focus != Field::To {
            self.focus = self.focus.previous();
        }
    }

    /// Passe au champ précédent.
    pub fn focus_previous(&mut self) {
        self.focus = self.focus.previous();
    }
}
