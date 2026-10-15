//! Saisie dans un formulaire : frappe, effacement et changement de champ.

use super::dialog::Form;

const MAX_FIELD_CHARS: usize = 200;

impl Form {
    /// Ajoute un caractère imprimable au champ qui a le focus, dans la limite de longueur.
    pub fn push(&mut self, c: char) {
        if c.is_control() {
            return;
        }
        if let Some(field) = self.fields.get_mut(self.focus)
            && field.value.chars().count() < MAX_FIELD_CHARS
        {
            field.value.push(c);
        }
    }

    /// Retire le dernier caractère du champ qui a le focus.
    pub fn pop(&mut self) {
        if let Some(field) = self.fields.get_mut(self.focus) {
            field.value.pop();
        }
    }

    /// Passe au champ suivant, en boucle.
    pub fn focus_next(&mut self) {
        let count = self.fields.len().max(1);
        self.focus = (self.focus + 1) % count;
    }

    /// Passe au champ précédent, en boucle.
    pub fn focus_previous(&mut self) {
        let count = self.fields.len().max(1);
        self.focus = (self.focus + count - 1) % count;
    }
}
