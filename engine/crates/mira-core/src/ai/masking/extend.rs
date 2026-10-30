//! Masquage sur plusieurs tours : nouvelles entrées du masqueur, restituteur étendu, texte complet restitué.

use super::apply::Masker;
use super::restore::Restorer;

impl Masker {
    /// Entrées ajoutées depuis le dernier appel, pour étendre le restituteur de la réponse.
    pub fn take_new(&mut self) -> Vec<(String, String)> {
        let fresh = self.table.get(self.taken..).unwrap_or_default().to_vec();
        self.taken = self.table.len();
        fresh
    }
}

impl Restorer {
    /// Restituteur sans repère connu : il laisse tout passer jusqu'à son extension.
    pub fn new_empty() -> Self {
        Self::new(Vec::new())
    }

    /// Ajoute des repères connus ; les suivants du flux en cours les restituent aussi.
    pub fn extend(&mut self, entries: Vec<(String, String)>) {
        for (placeholder, value) in entries {
            self.longest = self.longest.max(placeholder.len());
            self.table.insert(placeholder.to_ascii_uppercase(), value);
        }
    }

    /// Restitue un texte complet, sans toucher à l'attente du flux en cours.
    pub fn restore_text(&self, text: &str) -> String {
        let mut copy = Self {
            table: self.table.clone(),
            longest: self.longest,
            pending: String::new(),
        };
        let mut restored = copy.push(text);
        restored.push_str(&copy.finish());
        restored
    }
}
